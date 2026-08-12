# 写侧恒定内存与写优化 - 可执行任务清单

> 工作流：恒定内存与写优化
>
> 范围：覆盖写侧样式去重热点、WriteBackendSelection 状态机文档化、Moka 缓存审计、compress × constant_memory 交互矩阵测试，并产出读侧 spill 与公式结果缓存两份 RFC。
>
> 硬约束：所有写优化必须保持输出文件 checksum 不变（基准 `df7966ddec70e23c9df5f8890d6c512c6ea1883d30f5283ac8d09d483f876c95`）。本清单只产出文档与测试，不改写优化相关的生产代码（任务 1 的代码改动单独立项）。
>
> 关联 RFC：
> - [READ-CONSTANT-MEMORY-RFC.md](READ-CONSTANT-MEMORY-RFC.md)（任务 3.1 产出）
> - [FORMULA-RESULT-CACHE-RFC.md](FORMULA-RESULT-CACHE-RFC.md)（任务 6.x 产出）

## 事实速查（已确认，带证据）

| 事实 | 文件:行号 |
|---|---|
| 写侧样式去重 O(n) 热点 | `crates/easyexcel/src/write/gzip_spill/gzip_sheet_data_writer.rs:62` |
| `write_journal_row()` 全貌 | `crates/easyexcel/src/write/gzip_spill/gzip_sheet_data_writer.rs:52-94` |
| `JournalCellStyle` 定义（`#[derive(PartialEq)]`） | `crates/easyexcel/src/write/gzip_spill/journal_cell_style.rs:8-14` |
| `WriteBackendSelection` 7 态枚举 | `crates/easyexcel/src/write/write_backend_selection.rs:7-23` |
| 状态机主入口 `ensure_backend_for_write` | `crates/easyexcel/src/excel_writer/new_to_output_path.rs:400-468` |
| 晋升回放 `promote_auto_streaming_to_memory` | `crates/easyexcel/src/excel_writer/new_to_output_path.rs:470-514` |
| `record_streaming_write_result` → Failed | `crates/easyexcel/src/excel_writer/new_to_output_path.rs:518-529` |
| `compress_temp_files` 默认 false | `crates/easyexcel/src/write/write_options.rs:42` |
| `constant_memory` 默认 false | `crates/easyexcel/src/write/write_options.rs:40` |
| `default_constant_memory` 字段 | `crates/easyexcel/src/excel_writer.rs:87` |
| `uses_constant_memory_spill()` | `crates/easyexcel/src/write/excel_writer_core/state_and_conversion.rs:27-29` |
| Moka 无 capacity/TTL/TTI | `crates/easyexcel-cache/src/cache/shared_string_cache.rs:124-136` |
| Moka builder 调用点 `Cache::builder().build()` | `crates/easyexcel-cache/src/cache/shared_string_cache.rs:132` |
| `select_mode` 5MB 阈值 | `crates/easyexcel-cache/src/cache/shared_string_cache_policy.rs:50-56` |
| 公式引擎无结果缓存，仅 AST 缓存 | `crates/easyexcel-formula/src/formula/engine.rs:41-44`、`64-72` |
| `recalc()` 全量重算 | `crates/easyexcel-formula/src/formula/engine.rs:89-281` |
| spill 收敛 `MAX_PASSES=12` | `crates/easyexcel-formula/src/formula/engine.rs:197-263` |
| `ENUM_THRESHOLD=4096` | `crates/easyexcel-formula/src/formula/engine.rs:126` |
| 读侧 XLSX reader 全 DOM | `crates/easyexcel-xlsx/src/xlsx/reader.rs:80-99`（`parts: HashMap<String, Vec<u8>>`） |
| 测试用例 include 入口 | `crates/easyexcel/src/write/tests.rs:662-672` |

---

## 子任务 1：写侧样式去重哈希化

### 任务 1.1 - 为 JournalCellStyle 增加 Eq + Hash 派生并建立 HashMap 索引

- **涉及文件**
  - `crates/easyexcel/src/write/gzip_spill/journal_cell_style.rs`（改派生）
  - `crates/easyexcel/src/write/gzip_spill/gzip_sheet_data_writer.rs`（结构体加字段、查重逻辑）
  - `crates/easyexcel/src/write/gzip_spill/gzip_spill_reader.rs`（reader 侧仍按 `styles: Vec` 下标读取，无需改）
- **当前状态**
  - `JournalCellStyle` 派生了 `Debug, Clone, PartialEq`（`journal_cell_style.rs:8`），未派生 `Eq`/`Hash`。
  - 查重在 `gzip_sheet_data_writer.rs:62` 用 `self.styles.iter().position(|item| item == style)`，O(n) 线性扫描；`write_journal_row()` 每行每个带样式 cell 都调用一次（`:52-94`）。
  - `GzipSheetDataWriter` 结构体当前只持有 `inner: EngineSpillWriter` + `styles: Vec<JournalCellStyle>`（`:3-6`）。
  - `finish()` 把 `styles` 整体移交给 `GzipSpillReader`（`:122-127`），reader 按 `style_id` 下标回查（`gzip_spill_reader.rs:61-74`）。索引语义必须保持「下标 == 插入顺序」。
- **具体动作步骤**
  1. 在 `journal_cell_style.rs:8` 的派生链上加 `Eq, Hash`。字段类型：`bool`、`Option<ExcelCellStyle>`、`Option<WriteFont>`、`Option<WriteCellStyle>` —— 确认 `ExcelCellStyle`、`WriteFont`、`WriteCellStyle` 是否已派生 `Eq + Hash`；若未派生，需向上补齐派生（用 `mcp__code-review-graph__query_graph_tool` pattern=`children_of` target=`ExcelCellStyle` 定位其字段，确认全部为 `Eq+Hash` 友好类型）。
  2. 在 `GzipSheetDataWriter` 增加 `style_index: HashMap<JournalCellStyle, u32>`（键为样式，值为 styles 下标），与 `styles: Vec<JournalCellStyle>` 并存，保持「Vec 下标即 style_id」对外不变。
  3. 把 `gzip_sheet_data_writer.rs:62-73` 的 `position` 查重替换为：
     ```rust
     let style_id = if let Some(&id) = self.style_index.get(style) {
         id
     } else {
         let id = u32::try_from(self.styles.len()).map_err(|_| {
             ExcelError::Format("stateful journal style count exceeds u32".to_owned())
         })?;
         self.styles.push(style.clone());
         self.style_index.insert(style.clone(), id);
         id
     };
     ```
  4. `create()`（`:15-17`）与 `create_owned()`（`:29-35`）初始化 `style_index: HashMap::new()`。
  5. `finish()`（`:122-128`）用 `self.style_index`（或 `mem::forget`/drop 掉，只把 `styles` 交给 reader）—— reader 不需要 index，只按 `style_id` 下标读 Vec。
- **验收标准（可机器校验）**
  - `cargo test -p easyexcel gzip_spill` 全绿（含 `gzip_spill_round_trips_every_cell_value_variant`）。
  - `cargo test -p easyexcel stateful` 全绿（`cases_11.rs` 中 `auto_streaming_promotes_without_replaying_handler_callbacks` 等）。
  - 新增 bench/微测试：写 N=10000 行 × 1 cell、相同样式，断言 `style_index.len() == 1` 且 `styles.len() == 1`。
  - checksum 校验：对同一批输入，改前/改后产物 SHA-256 相等（见任务 5.x 的 checksum harness）。
  - `grep -n "iter().position" crates/easyexcel/src/write/gzip_spill/` 无命中。
- **估算工作量**：3.0 小时
- **依赖**：无（独立任务，优先做）
- **优先级**：**P0**

### 任务 1.2 - 派生传导验证：确认上游类型支持 Hash

- **涉及文件**
  - `crates/easyexcel/src/core/excel_cell_style.rs`（`ExcelCellStyle` 定义处）
  - `crates/easyexcel/src/core/write_cell_style.rs`（`WriteCellStyle`、`WriteFont`）
- **当前状态**：`JournalCellStyle` 持有三个 `Option<T>`，T 的 Hash 派生状态未在本工作流确认过。
- **具体动作步骤**
  1. 用 `mcp__code-review-graph__semantic_search_nodes_tool` 搜 `ExcelCellStyle`/`WriteFont`/`WriteCellStyle`，读其 `#[derive(...)]`。
  2. 若任一未派生 `Hash`：评估字段（颜色枚举、border style 枚举、`Option<...>`、`String`）是否都是 `Hash` 友好；若友好则补 `Eq, Hash`，否则在 `JournalCellStyle` 上实现一个「归一化哈希键」中间结构（如序列化后的 `String` 做键），避免污染上游类型。
  3. 若采用归一化键方案：`style_index: HashMap<String, u32>`，键为 `serde_json::to_string(style)` 或手写归一化，记下「不直接 Hash JournalCellStyle」的理由。
- **验收标准**
  - `cargo build -p easyexcel` 无错。
  - `cargo clippy -p easyexcel` 无新 warning。
  - 在 PR 描述里贴出三种上游类型的派生链证据（文件:行号）。
- **估算工作量**：1.5 小时
- **依赖**：任务 1.1
- **优先级**：**P0**

---

## 子任务 2：读链路恒定内存 spill 可行性 RFC

> 完整 RFC 见独立文件 [READ-CONSTANT-MEMORY-RFC.md](READ-CONSTANT-MEMORY-RFC.md)。

### 任务 2.1 - 起草 READ-CONSTANT-MEMORY-RFC.md

- **涉及文件**：新建 `docs/superpowers/specs/2026-08-12-read-spill-decision-design.md`
- **当前状态**：读侧无 spill-to-disk。`sharedStrings` 走 File 缓存（`shared_string_cache.rs:147-168` `FileSharedStringCache`），worksheet body 是 SAX 流式（`crates/easyexcel-xlsx/src/xlsx/event_reader.rs`）。但 XLSX workbook reader（`crates/easyexcel-xlsx/src/xlsx/reader.rs:80-99`）把所有 zip entry 读进 `HashMap<String, Vec<u8>>`（全 DOM），`f.read_to_end(&mut data)?`（`:97`）。
- **具体动作步骤**：按 RFC 模板（背景、决策选项≥2、推荐、风险、回滚）撰写，结论必须给出明确推荐。详见独立 RFC 文件。
- **验收标准**：RFC 文件存在；含「推荐方案」小节且不模棱两可；引用 `reader.rs:80-99` 作为全 DOM 证据。
- **估算工作量**：2.0 小时
- **依赖**：无
- **优先级**：**P1**

---

## 子任务 3：WriteBackendSelection 状态机文档化

### 任务 3.1 - 在 ARCHITECTURE.md 增补状态机 Mermaid 图与迁移条件表

- **涉及文件**：`docs/ARCHITECTURE.md`（在「Data Flow」与「Core Traits」之间插入新章节「## WriteBackendSelection 状态机」）
- **当前状态**：`write_backend_selection.rs:7-23` 定义 7 态，但 ARCHITECTURE.md 无任何状态机说明；迁移逻辑分散在 `new_to_output_path.rs:400-468`、`:470-514`、`:518-529`、`write_raw_bytes_to_write_xls_batch_onto_template.rs:172-191`、`:380-406`。
- **具体动作步骤**
  1. 在 ARCHITECTURE.md「Data Flow」章节后插入如下 Mermaid `stateDiagram-v2`（已按代码核对迁移条件，可直接合入）：

     ```mermaid
     stateDiagram-v2
         [*] --> AutoUndecided : build() 初始<br/>(builder.rs:414-421 / new_to_output_path.rs:39-41)
         [*] --> ExplicitStreaming : constant_memory(true)<br/>(builder.rs:418 / new_to_output_path.rs:39)
         [*] --> ExplicitInMemory : in_memory(true)<br/>(builder.rs:420 / excel_writer_builder.rs:258)
         [*] --> InMemory : CSV / XLS 路径<br/>(new_to_output_path.rs:408-412)

         AutoUndecided --> AutoStreaming : 首写且 schema+handler 全 StreamingSafe<br/>(new_to_output_path.rs:434-440)
         AutoUndecided --> InMemory : 首写但能力不满足<br/>(new_to_output_path.rs:441-444)

         AutoStreaming --> InMemory : 冲突且尚无已落盘 sheet<br/>(new_to_output_path.rs:451-456)
         AutoStreaming --> Promoting : 冲突且有已落盘 sheet（journal 回放）<br/>(new_to_output_path.rs:457-459 / :470-471)
         AutoStreaming --> Promoting : add_deferred_merge 触发晋升<br/>(new_to_output_path.rs:376-378)

         Promoting --> InMemory : journal 重放成功<br/>(new_to_output_path.rs:512)
         Promoting --> Failed : finish() 已消费 journal，无法回滚<br/>(new_to_output_path.rs:501)

         ExplicitStreaming --> Failed : 遇到能力冲突（显式禁止晋升）<br/>(new_to_output_path.rs:445-450 / write_raw_bytes...:179-180)
         ExplicitStreaming --> Failed : 写入中途出错<br/>(new_to_output_path.rs:518-528)

         AutoStreaming --> Failed : 写入中途出错<br/>(new_to_output_path.rs:518-528)
         InMemory --> InMemory : 后续写直接复用<br/>(write_raw_bytes...:405-406)
         ExplicitInMemory --> ExplicitInMemory : 后续写直接复用<br/>(write_raw_bytes...:405-406)
         Failed --> [*] : 终止态，fail-closed
     ```

  2. 在图后追加「迁移条件表」，每行：迁移 | 触发文件:行号 | 触发条件 | 副作用（`default_constant_memory`/`compress_temp_files` 是否翻转）。重点标注：
     - `AutoUndecided → AutoStreaming` 时强制 `default_constant_memory=true; compress_temp_files=true`（`new_to_output_path.rs:436-438`）—— 即 AutoStreaming 始终带磁盘 journal。
     - `AutoStreaming → InMemory`（空 sheet 回退）时 `default_constant_memory=false; compress_temp_files=false`（`:453-454`）。
     - `Promoting → InMemory` 时清空各 sheet state 的 `constant_memory`/`compress_temp_files`（`:506-509`）。
  3. 说明 `is_streaming()` 只对 `AutoStreaming | ExplicitStreaming` 为真（`write_backend_selection.rs:28-30`），`InMemory`/`Promoting`/`Failed` 都不是 streaming。
- **验收标准**
  - `docs/ARCHITECTURE.md` 含 `stateDiagram-v2` 代码块，mermaid 语法可通过 `mmdc` 或 GitHub 预览渲染（无语法错误）。
  - 每个 `-->` 标注的行号在代码中真实存在（可用 `grep -n` 抽检 3 条）。
  - 7 个状态全部在图中出现。
- **估算工作量**：2.0 小时
- **依赖**：无
- **优先级**：**P1**

---

## 子任务 4：Moka 缓存策略审计

### 任务 4.1 - 评估 Moka capacity 上限 vs 当前「预分配/不淘汰」语义

- **涉及文件**
  - `crates/easyexcel-cache/src/cache/shared_string_cache.rs:83-89`（`create_moka_cache` 文档）、`:124-136`（`MokaSharedStringCache`）、`:129-135`（`Cache::builder().build()` 无配置）
  - `docs/ARCHITECTURE.md:160`（边界约束：「Moka 不得配置容量/时间淘汰」）
- **当前状态**
  - `Cache::builder().build()` 不设 `max_capacity`/`time_to_live`/`time_to_idle`（`:132`）。文档注释 `:83-85` 明确「不设置容量、TTL 或 TTI；条目只在缓存对象销毁时整体释放」。
  - `facade-boundary-audit`（ARCHITECTURE.md:160）硬性要求 Moka 不得配置淘汰。
  - 测试 `moka_object_cache_keeps_every_value_before_and_after_finish`（`:198-213`）断言 128 条全保留、finish 后仍可读 —— 依赖「不淘汰」语义。
- **具体动作步骤**
  1. 在本任务清单下方「审计结论」一节，给出推荐：**保持现状（不加 capacity）**，理由：
     - Moka 后端的语义是「一次性载入全部 shared strings，读完即销毁」，加 capacity 会引入静默淘汰，导致 `get(index)` 返回 Err 并误判为「索引越界」。
     - 大文件保护由 `select_mode` 在 5MB 阈值切到 `FileSharedStringCache`（`shared_string_cache_policy.rs:50-56`），不依赖 Moka 自身淘汰。
     - 若要加 capacity，必须同时把 `SharedStringCacheReader::get` 的错误语义从「越界」改为「可能被淘汰，需重读源」，破坏现有契约。
  2. 在 `shared_string_cache.rs:83-89` 的 doc comment 上补一行「设计意图：禁止 capacity/TTL/TTI，见 ARCHITECTURE.md 边界约束与本任务审计」。
  3. 在 ARCHITECTURE.md:160 那条边界约束后补一句交叉引用，指向本审计文件。
- **验收标准**
  - 本文档「审计结论」一节存在且结论为「保持现状」。
  - `shared_string_cache.rs` doc comment 含交叉引用。
  - `cargo test -p easyexcel-cache` 全绿（不改动行为）。
- **估算工作量**：1.0 小时
- **依赖**：无
- **优先级**：**P2**

---

## 子任务 5：compress_temp_files × constant_memory 4 态交互矩阵测试

### 任务 5.1 - 新增 cases_12_spill_matrix.rs 测试文件

- **涉及文件**
  - 新建 `crates/easyexcel/src/write/tests_cases/cases_12_spill_matrix.rs`
  - 改 `crates/easyexcel/src/write/tests.rs`（在 `:672` 后加 `include!("tests_cases/cases_12_spill_matrix.rs");`）
- **当前状态**
  - 现有测试只覆盖 AutoStreaming 自动路径（`cases_11.rs:309-329` `stateful_build_auto_selects_streaming_for_scalar_batches`，断言 `AutoStreaming` + `compress_temp_files_enabled()==true`）和显式冲突（`:350-369` `explicit_streaming_rejects_unknown_handler_before_writing`）。
  - 4 态矩阵（`constant_memory` × `compress_temp_files` 各 true/false）从未被系统化覆盖。`uses_constant_memory_spill()`（`state_and_conversion.rs:27-29`）= `constant_memory || compress_temp_files`，即 3/4 态都会走 spill。
- **具体动作步骤**
  1. 新建 `cases_12_spill_matrix.rs`，复用 `cases_11.rs` 的 import（`tempdir`、`WriteOptions`、`WriteSheet`、`Xlsx`、`open_workbook`、`zip_entry`、`ExcelError`、`Result` 等，因为 tests.rs 用 `include!` 展开在同一作用域）。
  2. 写 4 个 `#[test]`，按矩阵：
     | constant_memory | compress_temp_files | 期望 backend | 期望 spill | 期望 checksum |
     |---|---|---|---|---|
     | false | false | AutoStreaming（首写后）→ 但 default_constant_memory=false | 否（纯内存） | 基准 |
     | false | true  | AutoStreaming | 是（gzip） | 等于基准 |
     | true  | false | ExplicitStreaming | 是（gzip，AutoStreaming 始终 compress） | 等于基准 |
     | true  | true  | ExplicitStreaming | 是（gzip） | 等于基准 |
     > 注：`constant_memory=true` 走 `ExplicitStreaming`（`builder.rs:418`）；首写时 `ensure_backend_for_write` 不再翻 `AutoUndecided`。`compress_temp_files=true` 但 `constant_memory=false` 时，`uses_constant_memory_spill()` 为 true，但 stateful backend 仍按 AutoUndecided→AutoStreaming 走（`new_to_output_path.rs:434-440` 强制 `compress_temp_files=true`）。
  3. 每个测试：
     - 用 `EasyExcel::write::<AutoStateRow>(&path).constant_memory(X).compress_temp_files(Y)?` —— 确认 builder 是否暴露 `compress_temp_files` setter；若无，则用 `WriteOptions{ compress_temp_files: Y, .. }` 经 `write_xlsx` 路径（待确认 builder API）。
     - 写同一批确定性数据（如 `vec![AutoStateRow{value:1}, AutoStateRow{value:2}]`，固定 sheet 名）。
     - `writer.finish()?` 后 `sha256` 文件，断言 4 个产物 checksum 两两相等。
     - 断言 `backend_selection()` 符合上表。
  4. 额外加 2 个冲突测试：
     - `explicit_streaming_with_compress_rejects_random_access_handler`：`constant_memory(true).compress_temp_files(true)` + 注册 `NoOpHandler`（未知能力）→ `write()` 返回 `Err(Unsupported)`，状态变 `ExplicitStreaming`，文件不存在（对齐 `cases_11.rs:350-369`）。
     - `auto_promotion_clears_compress_flag_after_replay`：首 sheet AutoStreaming，第二 sheet 带冲突能力 → 晋升后 `compress_temp_files_enabled()==false`（对齐 `new_to_output_path.rs:510-511`）。
- **验收标准**
  - `cargo test -p easyexcel cases_12` 全绿。
  - 4 态 checksum 断言通过（同一数据集 → 同一 SHA-256）。
  - `grep -n "include!(\"tests_cases/cases_12" crates/easyexcel/src/write/tests.rs` 有命中。
- **估算工作量**：3.0 小时
- **依赖**：任务 1.1（若先改样式去重，需保证 checksum 不变才能跑通矩阵）；可并行先写测试骨架
- **优先级**：**P0**

### 任务 5.2 - 抽取 checksum 比对辅助函数

- **涉及文件**：`crates/easyexcel/src/write/tests.rs`（顶部 test helpers 区）或新建 `crates/easyexcel/src/write/tests_cases/_helpers.rs`（用 include!）
- **当前状态**：`cases_11.rs` 用 `zip_entry(&path, "xl/styles.xml")` 做字符串包含断言，无 SHA-256 工具。
- **具体动作步骤**
  1. 在 tests.rs helper 区加 `fn sha256_of_file(path: &Path) -> String`（用 `sha2` crate；确认 `easyexcel` dev-deps 是否已有，若无需在 `Cargo.toml [dev-dependencies]` 加 `sha2 = "0.10"`）。
  2. 加 `fn assert_same_checksum(paths: &[&Path])`，两两比对。
- **验收标准**
  - `cargo test -p easyexcel` 编译通过。
  - `sha256_of_file` 被 cases_12 至少调用 4 次。
- **估算工作量**：0.5 小时
- **依赖**：任务 5.1
- **优先级**：**P0**

---

## 子任务 6：公式引擎结果缓存 RFC

> 完整 RFC 见独立文件 [FORMULA-RESULT-CACHE-RFC.md](FORMULA-RESULT-CACHE-RFC.md)。

### 任务 6.1 - 起草 FORMULA-RESULT-CACHE-RFC.md（dirty-cell 增量重算）

- **涉及文件**：新建 `docs/superpowers/specs/2026-08-12-formula-cache-decision-design.md`
- **当前状态**
  - `Engine` 仅缓存 AST（`engine.rs:41-44` `ast_cache: HashMap<String, Rc<Expr>>`），`parse_cached()`（`:64-72`）只省解析。
  - `recalc()`（`:89-281`）每次全量重算所有公式 cell，`report.evaluated` 每趟重置计数（`:201`）。
  - 依赖图已拓扑排序（Kahn，`:159-171`），`ENUM_THRESHOLD=4096`（`:126`）控制 range 枚举 vs 全集扫描。
  - spill 收敛 `MAX_PASSES=12`（`:197-263`），每趟全量重算。
  - 模型层 `Cell::Formula { expr, cached }`（见 `:283-289` `write_cached`）已有 `cached` 字段，但无 dirty 标记。
- **具体动作步骤**：按 RFC 模板撰写，重点评估「dirty-cell 增量重算」vs「全量重算+结果缓存」vs「维持现状」三选项。详见独立 RFC。
- **验收标准**：RFC 存在；含「推荐方案」且明确；含风险与回滚；引用 `engine.rs:89-281` 与 `:197-263`。
- **估算工作量**：2.5 小时
- **依赖**：无
- **优先级**：**P1**

---

## 审计结论（子任务 4 产出）

### Moka 缓存策略：保持现状，不加 capacity/TTL/TTI

- **结论**：**不修改** `MokaSharedStringCache`（`shared_string_cache.rs:124-136`）。
- **理由**：
  1. Moka 后端语义 = 「加载期一次性载入全部 shared strings，读侧按 index 随机访问，读完好销毁」。`get(index)` 当前对越界返回 `Err`（`value_at` / `read_file_entry`，`:117-122`/`:177-186`），若加淘汰会让合法 index 偶发 Err，破坏 `SharedStringCacheReader` 契约。
  2. 大文件内存保护由 `SharedStringCachePolicy::select_mode` 在 5MB 阈值切到 `FileSharedStringCache`（`shared_string_cache_policy.rs:50-56`），不依赖 Moka 淘汰。
  3. `facade-boundary-audit`（ARCHITECTURE.md:160）明文禁止 Moka 配置淘汰，是项目级不变量。
  4. 测试 `moka_object_cache_keeps_every_value_before_and_after_finish`（`:198-213`）与 `moka_object_cache_accepts_multibyte_values_without_eviction`（`:216-222`）显式依赖「不淘汰」。
- **唯一允许的改动**：在 `create_moka_cache` 的 doc comment（`:83-89`）补交叉引用，记录此结论的依据文件路径，便于后续审计追溯。

---

## 汇总：任务清单总览

| 编号 | 标题 | 优先级 | 工时(h) | 依赖 | 产出类型 |
|---|---|---|---|---|---|
| 1.1 | JournalCellStyle 哈希化索引 | P0 | 3.0 | - | 代码 |
| 1.2 | 上游类型 Hash 派生传导验证 | P0 | 1.5 | 1.1 | 代码 |
| 2.1 | READ-CONSTANT-MEMORY-RFC.md | P1 | 2.0 | - | RFC 文档 |
| 3.1 | WriteBackendSelection 状态机入 ARCHITECTURE.md | P1 | 2.0 | - | 文档 |
| 4.1 | Moka 缓存策略审计（结论：保持现状） | P2 | 1.0 | - | 文档+注释 |
| 5.1 | cases_12_spill_matrix.rs 4 态矩阵 | P0 | 3.0 | 1.1,5.2 | 测试 |
| 5.2 | checksum 比对辅助函数 | P0 | 0.5 | - | 测试 |
| 6.1 | FORMULA-RESULT-CACHE-RFC.md | P1 | 2.5 | - | RFC 文档 |

**合计：8 个任务项，总工时约 15.5 小时。**

> P0（5 项）：1.1, 1.2, 5.1, 5.2 — 写性能热点与回归保护网。
> P1（3 项）：2.1, 3.1, 6.1 — 决策与文档化。
> P2（1 项）：4.1 — 审计性记录。
