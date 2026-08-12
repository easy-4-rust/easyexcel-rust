# RFC: 读链路恒定内存 spill 可行性

> 状态：**草案** | 关联：[WRITE-CONSTANT-MEMORY-OPTIMIZATION.md](WRITE-CONSTANT-MEMORY-OPTIMIZATION.md) 子任务 2.1
>
> 决策问题：读侧是否需要引入类似写侧 `GzipSheetDataWriter` 的 spill-to-disk 机制，以在超大 XLSX 读取时保持恒定内存？

## 1. 背景

写侧已具备完整的恒定内存路径：

- `GzipSheetDataWriter`（`crates/easyexcel/src/write/gzip_spill/gzip_sheet_data_writer.rs`）在 `compress_temp_files` 启用时把行 XML gzip 落盘，`finish()` 返回 `GzipSpillReader` 流式读回。
- `WriteBackendSelection` 7 态状态机（`write_backend_selection.rs:7-23`）管理 Auto/Explicit 流式与内存后端的晋升/回退。

读侧现状（已确认证据）：

| 环节 | 实现 | 内存特征 | 证据 |
|---|---|---|---|
| sharedStrings 缓存 | `FileSharedStringCache` / `MokaSharedStringCache` / `MemorySharedStringCache` 三后端 | Auto 模式按 5MB 阈值选 File/Memory | `easyexcel-cache/src/cache/shared_string_cache.rs:124-168`、`shared_string_cache_policy.rs:50-56` |
| worksheet body 解析 | SAX 事件流（`XlsxCellEventReader`） | 流式，单行驻留 | `crates/easyexcel-xlsx/src/xlsx/event_reader.rs` |
| **XLSX workbook 整体读取** | **全 DOM：所有 zip entry 一次性 `read_to_end` 进 `HashMap<String, Vec<u8>>`** | **峰值 = 整个解压后包大小** | `crates/easyexcel-xlsx/src/xlsx/reader.rs:80-99` |

关键热点：`reader.rs:86-99`

```rust
let mut parts: HashMap<String, Vec<u8>> = HashMap::new();
for i in 0..archive.len() {
    let mut f = archive.by_index(i)?;
    ...
    f.read_to_end(&mut data)?;   // :97 —— 每个 entry 全量进内存
    parts.insert(name, data);
}
```

之后 `parts.get("xl/workbook.xml")`、`parts.get("xl/sharedStrings.xml")`、`parts.get("xl/styles.xml")` 等按需取出解析。即：**不论用户读哪张表、哪几行，整个 XLSX 的全部部件（含所有 sheet、嵌入图片、charts、主题、主题色等）都被解压进内存**。

## 2. 触发本 RFC 的场景

- 超大 XLSX（数百 MB ~ GB 级），用户只想读某一张 sheet 的一个行范围。
- Web 导入场景（`easyexcel-web`）按 `ReadOptions` 限制行范围，但底层仍把整个包物化，峰值内存与文件大小线性相关，背压/限制失效。
- 多 sheet 工作簿，sheet0 很大，用户只想读 sheet5。

当前读侧的「内存边界」实际只覆盖 sharedStrings（5MB 阈值切文件缓存），worksheet body 是 SAX 流（OK），**但包级 DOM 物化是漏网的内存放大点**。

## 3. 决策选项

### 选项 A：维持现状（不做读侧 spill）

- 包级 DOM 物化保持不变。
- 仅依赖 sharedStrings 文件缓存 + SAX body。
- 内存峰值 ≈ 解压后全包大小。

### 选项 B：包级惰性加载（lazy parts）—— 只按需 `read_to_end` 实际用到的 entry

- 改造 `reader.rs:80-99`：不预先读全部 entry，改为持有 `ZipArchive`，按 sheet/rels/styles 实际访问时才 `by_name(path)?.read_to_end()`。
- 不引入新的磁盘 spill，只是「不把不读的 entry 读进来」。
- `parts: HashMap<String, Vec<u8>>` 改为 `parts: HashMap<String, OnceCell<Vec<u8>>>` 或直接每次按需从 archive 取（需 archive 可复用 borrow）。

### 选项 C：完整读侧 spill（仿写侧 GzipSheetDataWriter）

- 对超大包，把各 entry 解压后落盘到临时目录（NamedTempFile per entry），内存只留索引 `(path → temp file handle)`。
- 读 sheet 时从对应 temp file SAX 流式读。

## 4. 推荐方案：选项 B（包级惰性加载）

**推荐选项 B，不做选项 C 的读侧 spill。**

### 理由

1. **痛点定位精确**。内存放大点不是 worksheet body（已是 SAX），而是 `reader.rs:86-99` 的全包 DOM 物化。选项 B 直接消灭这个放大点，收益与选项 C 几乎相同，但复杂度低一个数量级。

2. **选项 C 与写侧不对称**。写侧 spill 的存在理由是「SXSSF 兼容 + 行 XML 必须先于 ZIP 生成」。读侧不存在这种顺序约束——`ZipArchive` 天然支持随机按 entry 访问，spill 一层临时文件纯属多此一举。

3. **选项 B 是纯收益、低风险**：
   - 不新增磁盘 I/O（选项 C 的 NamedTempFile 在大包时反而增加写盘）。
   - 不新增生命周期管理（选项 C 要管 temp dir 清理）。
   - `zip` crate 的 `ZipArchive::by_name` 已支持按需读取，无需新依赖。
   - 唯一约束：`ZipArchive` 需要 `Read + Seek` 的源，当前 reader 已满足（`reader.rs:80` `R: Read + Seek`）。

4. **与现有 sharedStrings 文件缓存互补**：选项 B 让包级访问变惰性，sharedStrings 缓存在 5MB 阈值仍切文件（`shared_string_cache_policy.rs:50-56`），两者正交。

### 不推荐选项 C 的理由

- 写侧 spill 的代码量与复杂度（`gzip_spill.rs` 模块、journal、style 去重、reader 生命周期）远高于读侧需要的程度。
- 读侧的 zip 容器已经提供「按 entry 随机访问」，不需要再用临时文件模拟一遍。
- 维护两套 spill（写 gzip + 读 temp-file）会显著增加长期维护成本。

### 不推荐选项 A 的理由

- 包级 DOM 物化是真实内存放大点，Web 导入场景下会让 `easyexcel-web` 的资源限制形同虚设。
- 修复成本（选项 B）低，没有理由保留。

## 5. 选项 B 的实施轮廓（不在本 RFC 落地，仅描述）

> 本 RFC 只决策「要不要做读侧 spill」，答案是「不要，改做惰性加载」。具体实施任务应单独立项。轮廓如下，供后续 WBS 拆解：

1. `reader.rs:read_zip` 把 `HashMap<String, Vec<u8>>` 改为持有 `ZipArchive<R>`（需把 archive 从函数局部提升为结构体字段，或在一个作用域内完成全部解析）。
2. 解析顺序：先 `by_name("xl/workbook.xml")` → 得到 sheets 与 rels → 只对目标 sheet（按 `SheetSelection`）对应的 worksheet entry `read_to_end`/SAX 解析；非目标 sheet 不读。
3. `sharedStrings`/`styles` 仍按现有缓存策略载入。
4. 加 benchmark：构造 100MB 级 XLSX（多 sheet + 大 sharedStrings），对比改前/改后峰值内存（用 `jemalloc` 或 `/usr/bin/time -v` maxrss）。

## 6. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| `ZipArchive` 的 borrow 冲突（持有 archive 同时 by_name 取 entry） | 中 | 用 `read_zip` 内单作用域完成，或封装 `LazyParts` 结构持有 archive + `HashMap<String, OnceCell<Vec<u8>>>` |
| 加密包（CFB 包裹）路径（`reader.rs:65-74`）先解密进 `Cursor<Vec<u8>>` 再走 read_zip，惰性化收益在该路径打折 | 低 | 加密包通常体量小；惰性化主要面向明文大包 |
| 现有测试假设 `parts` 已全量载入（若有依赖全包遍历的逻辑） | 低 | 需先 `grep` `parts.iter()`/`parts.keys()` 全集遍历点，确认无遍历依赖 |

## 7. 回滚策略

- 选项 B 改造完全限定在 `reader.rs:read_zip` 内部，回滚 = 恢复 `HashMap<String, Vec<u8>>` 预读循环。
- 不引入新依赖、不改公开 API，回滚成本 ≈ 一次 revert。

## 8. 结论

- **读侧不需要写侧风格的 spill-to-disk 机制**。
- **应改为选项 B（包级惰性加载）**，消灭 `reader.rs:86-99` 的全包 DOM 物化。
- 本 RFC 的产出是「决策」，具体惰性加载实施不包含在本工作流，应作为独立的读性能优化任务单独立项。

## 9. 待确认项

- `easyexcel-web` 导入路径是否也走 `reader.rs:read_zip`，还是另有 Web 专用入口（待 `mcp__code-review-graph__query_graph_tool` pattern=`callers_of` target=`read_zip` 确认）。
- `parts` 是否被全集遍历（`parts.iter()`）—— 若有，惰性化需保留「能枚举 entry 名」的能力（archive 本身支持 `by_index(i).name()`）。
