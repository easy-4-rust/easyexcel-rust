# RFC: XLS Streaming Read Mode (xls-streaming)

> 状态：**预研 / 设计稿** -- 本文件不包含任何代码实现。
> 作者：ZCode Agent
> 日期：2026-08-11

---

## 1. 背景与动机

### 1.1 现状

当前 `easyexcel-xls` 的读取路径是 **DOM 全量解析**：

| 步骤 | 代码位置 | 行为 |
|------|---------|------|
| 打开 OLE2 容器 | `reader.rs:36` | `cfb::CompoundFile::open(reader)` |
| 读取 Workbook 流 | `reader.rs:53-59` | `s.read_to_end(&mut wb_bytes)` -- **整个流一次性读入 `Vec<u8>`** |
| 解析全局子流 | `reader.rs:161-225` | `parse_workbook_stream(&buf, ...)` 遍历 BOF..EOF 提取 SST、XF、BOUNDSHEET |
| 解析每张 sheet | `reader.rs:373-482` | `parse_worksheet(buf, bs.pos, ...)` -- **buf 是整个 workbook 的切片** |

这意味着：
- **内存**：整个 Workbook 流（可达数百 MB）常驻 `Vec<u8>`
- **吞吐**：实测约 12K rows/s，而 xlsx-event-read 达 594K rows/s（50x 差距）
- **延迟**：即使只读一张 sheet，也要先解析全部 SST + 所有 sheet 数据

### 1.2 目标

在不破坏现有 API 的前提下，新增"流式模式"作为可选实现：

- **保留**现有 `read()` / `read_with_password()` 签名和行为不变
- **新增** `read_streaming()` 入口，返回 `StreamingWorkbook`
- 用 **cargo feature flag** 切换：`features = ["xls-streaming"]`
- 目标吞吐：100K+ rows/s（接近 xlsx-event-read 的 60%）
- 目标内存：峰值 RSS 常数级（不随 sheet 数 / 行数增长）

---

## 2. 代码现状深度分析

### 2.1 核心数据流

```
cfb::CompoundFile::open(reader)
  │
  ├── cf.open_stream("/Workbook") -> stream.read_to_end(&mut wb_bytes)
  │     └── wb_bytes: Vec<u8>   ← 整个 Workbook 流，常驻内存
  │
  ├── contains_filepass(&wb_bytes)  → 解密 → workbook_stream: Vec<u8>
  │
  └── parse_workbook_stream(&workbook_stream, decrypted)
        │
        ├── Records::new(buf) → 遍历全局子流 (BOF → EOF)
        │     ├── SST record → sst::parse_sst(&data, &continue_breaks) → Vec<String>
        │     ├── FORMAT / XF → 构建样式表
        │     └── BOUNDSHEET → 记录每张 sheet 的 (name, byte_offset, visibility)
        │
        └── for bs in globals.boundsheets:
              parse_worksheet(buf, bs.pos, &globals, &xf_to_style, &mut sheet)
                    └── Records::new(&buf[start..]) → 遍历 sheet 子流
```

### 2.2 SST 解析细节

SST（Shared String Table）是 BIFF8 的核心：

- **位置**：在全局子流中，紧跟 BOF/FILEPASS/DATEMODE 等记录之后
- **格式**：SST 记录头 (8 bytes) + N 个 `XLUnicodeRichExtendedString`
- **CONTINUE 问题**：SST 的字符数据可能跨越多个 CONTINUE 记录边界，每个边界处有一个**新的 grbit 字节**重新声明压缩模式
- **当前实现**：`sst.rs:104` `parse_sst(data, breaks)` 一次性解析全部字符串为 `Vec<String>`
- **SST 大小**：典型 100K 行文件的 SST 约 5-20MB

### 2.3 cfb crate 现状

项目使用 `cfb = "0.14.0"`（已锁定在 workspace Cargo.toml）。

**关键 API**：

```rust
// CompoundFile::open_stream 返回 Stream<F>
// Stream<F> 实现了 Read + Seek + BufRead
pub fn open_stream<P: AsRef<Path>>(&mut self, path: P) -> io::Result<Stream<F>>;
```

`Stream<F>` 的内部机制：
- 持有 `Weak<RwLock<MiniAllocator<F>>>` 引用（指向 CompoundFile 的内部状态）
- 使用 `StreamBuffer` 做缓冲读取
- **支持 Seek**：可随机跳转到流内任意偏移
- **支持多个流同时打开**：因为每个 Stream 只持有 Weak 引用

**这意味着**：我们可以在不读取整个 Workbook 流的情况下，用 Seek 跳转到任意 sheet 子流的起始位置。

### 2.4 加密层现状

`biff8/encrypt.rs` 实现 CryptoAPI RC4 加密：

- `decrypt_crypto_api_workbook_stream(&wb_bytes, password)` 对整个 Workbook 流解密
- RC4 是流密码，理论上支持逐块解密
- 但当前 API 要求传入完整的 `&[u8]`，返回完整的 `Vec<u8>`
- **限制**：加密文件的流式读取需要先解密整个流（或实现逐块解密流包装器）

### 2.5 event_record 模块

`biff8/event_record.rs` 已有独立的记录解码器：
- `Biff8NumberRecord` / `Biff8LabelSstRecord` / `Biff8FormulaRecord` 等
- 这些是无状态的纯函数，可直接复用于流式模式

---

## 3. Crate 调研：OLE2/CFB 读取方案

### 3.1 cfb (v0.14.0) -- 当前使用

| 维度 | 评估 |
|------|------|
| **API 成熟度** | 高 -- 已在项目中广泛使用，行为已知 |
| **流式支持** | `Stream<F>: Read + Seek + BufRead`，内部带缓冲，支持按需读取 |
| **多流并发** | 支持 -- 多个 Stream 可同时打开（Weak 引用共享 allocator） |
| **Seek 支持** | 完整 -- 可跳转到流内任意偏移 |
| **维护状态** | 活跃 -- 0.14.0 于 2025 年发布 |
| **问题** | `CompoundFile::open` 需要 `Read + Seek`，不支持纯 `Read` 流；目录/FAT 解析在 `open` 时完成（一次性开销小） |

**结论**：cfb 完全满足需求，无需引入新 crate。关键能力是 `Stream::seek()` 可跳转到任意 sheet 子流偏移。

### 3.2 ole (crates.io)

| 维度 | 评估 |
|------|------|
| **API 成熟度** | 低 -- 文档简陋，仅提供基本 parser/reader |
| **流式支持** | 未明确 -- `Reader::new(file)` 接受 `File`，但未文档化 Seek/流式 API |
| **维护状态** | 不确定 -- WTFPL 许可证，社区活跃度低 |
| **与 cfb 对比** | 功能子集，无明显优势 |

**结论**：不推荐。cfb 功能更完整，且已是项目依赖。

### 3.3 compound-file (crates.io)

| 维度 | 评估 |
|------|------|
| **可发现性** | crates.io 文档页面 404，docs.rs 无记录 |
| **维护状态** | 不可用 |

**结论**：排除。

### 3.4 最终选择

**继续使用 cfb 0.14.0**，不引入新依赖。理由：
1. 已有依赖，零迁移成本
2. `Stream<F>` 的 `Read + Seek + BufRead` 能力完全满足按需读取需求
3. 无许可证或维护风险

---

## 4. 架构设计

### 4.1 Feature Flag

```toml
# crates/easyexcel-xls/Cargo.toml
[features]
default = ["xls-streaming"]       # 开箱即用流式模式
xls-streaming = []                # 仅控制代码条件编译
```

用户可通过以下方式关闭流式模式、回退到旧 DOM 实现：

```toml
easyexcel-xls = { version = "0.1.3", default-features = false }
```

### 4.2 模块结构

```
crates/easyexcel-xls/src/
├── xls/
│   ├── mod.rs                      # 新增 pub mod streaming_reader (条件编译)
│   ├── reader.rs                   # 保持不变（DOM 模式）
│   ├── streaming_reader.rs         # 【新建】流式模式入口
│   ├── streaming_workbook.rs       # 【新建】StreamingWorkbook 类型
│   ├── streaming_sheet.rs          # 【新建】StreamingSheetReader 类型
│   ├── sst.rs                      # 保持不变（被两个模式共用）
│   └── biff.rs                     # 保持不变
└── biff8/
    ├── mod.rs                      # 保持不变
    └── record_stream.rs            # 保持不变（event 层使用）
```

### 4.3 核心类型

#### 4.3.1 StreamingWorkbook

```rust
/// 流式工作簿 -- 按需打开 sheet，不一次性加载全部数据。
///
/// 对应 Java：无直接对应；Rust 架构扩展。
pub struct StreamingWorkbook {
    /// 全局元数据（日期系统、样式表）
    date_system: DateSystem,
    styles: StyleTable,
    xf_to_style: Vec<u32>,
    /// 共享字符串表 -- 延迟解码（见 4.4.4）
    sst: LazySst,
    /// sheet 元数据（名称、偏移、可见性）
    sheets_meta: Vec<SheetMeta>,
    /// OLE2 容器引用（持有文件句柄）
    compound: CompoundFile<Box<dyn ReadSeek>>,
}

struct SheetMeta {
    name: String,
    pos: usize,           // Workbook 流内的字节偏移
    visibility: Visibility,
    is_worksheet: bool,
}
```

#### 4.3.2 StreamingSheetReader

```rust
/// 单张 sheet 的流式读取器 -- 每次只持有当前 sheet 的数据。
///
/// 对应 Java：无直接对应；Rust 架构扩展。
pub struct StreamingSheetReader<'a> {
    sheet_idx: usize,
    name: String,
    visibility: Visibility,
    /// 从 cfb Stream 读取的 BIFF 记录迭代器
    records: StreamingRecordIter<'a>,
    /// 全局 SST 引用（延迟解码）
    sst: &'a LazySst,
    /// 全局样式映射
    xf_to_style: &'a [u32],
    /// 当前解析状态
    state: SheetParseState,
}
```

#### 4.3.3 StreamingRecordIter -- 记录流迭代器

```rust
/// 从 cfb Stream 按需读取 BIFF 记录的迭代器。
///
/// 不一次性加载整个子流；每次 next() 读取一个记录头 + payload。
pub struct StreamingRecordIter<'a> {
    stream: &'a mut Stream<Box<dyn ReadSeek>>,
    buf: Vec<u8>,          // 复用的读取缓冲区
    pos: u64,
    ended: bool,
}

impl<'a> Iterator for StreamingRecordIter<'a> {
    type Item = Result<(u16, Vec<u8>)>;  // (sid, payload)

    fn next(&mut self) -> Option<Self::Item> {
        // 读 4 字节头 → 解析 sid + length → 读 payload
        // 处理 CONTINUE 合并
    }
}
```

#### 4.3.4 LazySst -- SST 延迟解码

```rust
/// 延迟解码的共享字符串表。
///
/// SST 的原始字节保留在内存中，按需解码具体字符串。
/// 对于 100K 行文件，SST 原始字节约 5-20MB，但实际访问的
/// 字符串通常远少于总数（例如只读前 1000 行）。
pub struct LazySst {
    /// SST 原始字节（已合并 CONTINUE）
    raw_data: Vec<u8>,
    /// CONTINUE 边界偏移
    continue_breaks: Vec<usize>,
    /// 每个 string 在 raw_data 中的字节偏移
    offsets: Vec<usize>,
    /// 已解码的字符串缓存（按需填充）
    cache: RefCell<HashMap<u32, String>>,
}
```

### 4.4 核心优化详解

#### 4.4.1 优化 1：按需读取 Workbook 流（不 read_to_end）

**当前问题**：`reader.rs:53-59` 调用 `s.read_to_end(&mut wb_bytes)` 把整个 Workbook 流读入内存。

**解决方案**：保留 `CompoundFile` 引用，按需通过 `Stream::seek()` + `Stream::read()` 访问。

```rust
// 伪代码
let mut cf = CompoundFile::open(reader)?;
let mut wb_stream = cf.open_stream("/Workbook")?;

// 第一遍：只读全局子流（BOF → EOF），提取 SST + BOUNDSHEET
let globals = parse_globals_streaming(&mut wb_stream)?;

// 第二遍：按需打开每张 sheet
for meta in &globals.sheets_meta {
    wb_stream.seek(SeekFrom::Start(meta.pos as u64))?;
    // 逐记录读取该 sheet 的数据
}
```

**内存收益**：不再持有整个 Workbook 流的 `Vec<u8>`，只持有当前 sheet 的数据。

#### 4.4.2 优化 2：每张 sheet 独立 reader

**当前问题**：`parse_worksheet(buf, start, ...)` 接收整个 workbook 的切片，意味着所有 sheet 共享同一块内存。

**解决方案**：每张 sheet 用独立的 `StreamingRecordIter`，通过 `Stream::seek()` 定位到 sheet 起始偏移。

```rust
impl StreamingWorkbook {
    /// 按索引打开一张 sheet 的流式读取器。
    pub fn sheet(&mut self, idx: usize) -> Result<StreamingSheetReader<'_>> {
        let meta = &self.sheets_meta[idx];
        self.wb_stream.seek(SeekFrom::Start(meta.pos as u64))?;
        Ok(StreamingSheetReader {
            records: StreamingRecordIter::new(&mut self.wb_stream),
            sst: &self.sst,
            xf_to_style: &self.xf_to_style,
            // ...
        })
    }
}
```

**注意**：`StreamingSheetReader` 持有 `&mut Stream` 的可变借用，因此同一时刻只能有一个活跃的 sheet reader。这符合实际使用模式（逐 sheet 处理）。

#### 4.4.3 优化 3：RecordStream 迭代器模式

**当前问题**：`biff/records.rs` 的 `Records<'a>` 要求 `&'a [u8]`（整个子流的切片）。

**解决方案**：新增 `StreamingRecordIter` 从 `cfb::Stream` 按需读取：

```rust
impl StreamingRecordIter<'_> {
    fn next_record(&mut self) -> Result<Option<(u16, Vec<u8>)>> {
        // 1. 读 4 字节 BIFF 头
        let mut header = [0u8; 4];
        if self.stream.read(&mut header)? < 4 {
            return Ok(None);
        }
        let sid = u16::from_le_bytes([header[0], header[1]]);
        let length = u16::from_le_bytes([header[2], header[3]]) as usize;

        // 2. 读 payload
        let mut payload = vec![0u8; length];
        self.stream.read_exact(&mut payload)?;

        // 3. 合并后续 CONTINUE 记录
        while self.peek_sid()? == CONTINUE {
            // 读 CONTINUE 头 + payload，追加到 current payload
        }

        Ok(Some((sid, payload)))
    }
}
```

**与现有 `Records` 的关系**：`StreamingRecordIter` 是 `Records` 的流式等价物。两者可以共用 CONTINUE 合并逻辑（提取为共用函数）。

#### 4.4.4 优化 4：SST 延迟解码

**当前问题**：`sst::parse_sst()` 一次性解码全部字符串为 `Vec<String>`。对于 100K 行文件，SST 可能包含数十万个字符串，但实际访问的可能只有前几千行涉及的字符串。

**解决方案**：`LazySst` 保留原始字节，按需解码：

```rust
impl LazySst {
    /// 解析 SST 头部，记录每个 string 的字节偏移（不解码内容）。
    pub fn from_raw(data: Vec<u8>, breaks: Vec<usize>) -> Self {
        let offsets = Self::scan_offsets(&data, &breaks);
        LazySst {
            raw_data: data,
            continue_breaks: breaks,
            offsets,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// 按索引获取字符串（首次访问时解码）。
    pub fn get(&self, idx: u32) -> Result<String> {
        if let Some(s) = self.cache.borrow().get(&idx) {
            return Ok(s.clone());
        }
        let s = self.decode_one(idx)?;
        self.cache.borrow_mut().insert(idx, s.clone());
        Ok(s)
    }
}
```

**内存收益**：SST 原始字节（压缩态）比解码后的 `Vec<String>` 小 2-4x。加上按需解码，实际内存占用更小。

---

## 5. API 设计

### 5.1 新增公共 API

```rust
// crates/easyexcel-xls/src/xls/streaming_reader.rs

/// 流式读取 XLS 工作簿。
///
/// 返回 [`StreamingWorkbook`]，提供按需打开 sheet 的能力。
/// 不一次性加载全部数据到内存。
///
/// # Errors
///
/// 输入不是有效 OLE2 容器、缺少 Workbook 流或 BIFF8 记录损坏时返回错误。
#[cfg(feature = "xls-streaming")]
pub fn read_streaming<R: Read + Seek + 'static>(reader: R) -> Result<StreamingWorkbook>;

/// 流式读取加密 XLS 工作簿。
///
/// # Errors
///
/// 输入无效、加密类型不支持、未提供密码或密码错误时返回错误。
#[cfg(feature = "xls-streaming")]
pub fn read_streaming_with_password<R: Read + Seek + 'static>(
    reader: R,
    password: Option<&str>,
) -> Result<StreamingWorkbook>;
```

### 5.2 StreamingWorkbook API

```rust
impl StreamingWorkbook {
    /// 返回 sheet 数量。
    pub fn sheet_count(&self) -> usize;

    /// 返回第 idx 张 sheet 的元数据（名称、可见性）。
    pub fn sheet_meta(&self, idx: usize) -> Option<&SheetMeta>;

    /// 按索引打开一张 sheet 的流式读取器。
    ///
    /// 同一时刻只能有一个活跃的 StreamingSheetReader。
    pub fn sheet(&mut self, idx: usize) -> Result<StreamingSheetReader<'_>>;

    /// 返回工作簿的日期系统。
    pub fn date_system(&self) -> DateSystem;

    /// 返回样式表引用。
    pub fn styles(&self) -> &StyleTable;
}
```

### 5.3 StreamingSheetReader API

```rust
impl StreamingSheetReader<'_> {
    /// 返回 sheet 名称。
    pub fn name(&self) -> &str;

    /// 返回 sheet 可见性。
    pub fn visibility(&self) -> Visibility;

    /// 逐行遍历 sheet 数据。
    ///
    /// 每次迭代返回一行的单元格数据。
    pub fn rows(&mut self) -> RowIterator<'_>;
}

/// 行迭代器。
pub struct RowIterator<'a> { /* ... */ }

/// 一行的单元格数据。
pub struct RowData {
    pub row_idx: u32,
    pub cells: Vec<CellData>,
}

pub enum CellData {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(CellError),
    Formula { expr: String, cached: CellValue },
    Empty,
}
```

### 5.4 与现有 API 的兼容性

| API | 签名变化 | 行为变化 |
|-----|---------|---------|
| `read()` | 无 | 无 |
| `read_with_password()` | 无 | 无 |
| `read_path()` | 无 | 无 |
| `read_streaming()` | 新增 | -- |
| `read_streaming_with_password()` | 新增 | -- |

现有测试不受影响。新测试使用 `default = ["xls-streaming"]` 运行。

---

## 6. 实现路线图

### Phase 1：基础设施（约 8h）

1. 在 `crates/easyexcel-xls/Cargo.toml` 添加 `xls-streaming` feature
2. 创建 `streaming_reader.rs`，实现 `read_streaming()` 入口
3. 实现 `StreamingRecordIter`（从 cfb Stream 按需读取 BIFF 记录）
4. 实现全局子流的流式解析（只读 BOF→EOF 段）

### Phase 2：SST 延迟解码（约 8h）

1. 实现 `LazySst::from_raw()` -- 扫描 SST 原始字节，记录每个 string 的偏移
2. 实现 `LazySst::get(idx)` -- 按需解码单个字符串
3. 实现 `LazySst::decode_one()` -- 复用 `sst.rs` 的 CONTINUE/grbit 逻辑
4. 单元测试：与 `parse_sst()` 的结果对比

### Phase 3：Sheet 流式读取（约 12h）

1. 实现 `StreamingWorkbook::sheet(idx)` -- seek 到 sheet 偏移
2. 实现 `StreamingSheetReader` -- 复用 `reader.rs` 的 cell 解析逻辑
3. 实现 `RowIterator` -- 按行产出单元格数据
4. 集成 `LazySst` 的 `LABELSST` 解析

### Phase 4：加密兼容（约 8h）

1. 实现 `read_streaming_with_password()` -- 先解密 Workbook 流，再流式解析
2. 注意：加密文件需要先完整解密（RC4 流密码的限制），但解密后的流可以流式处理
3. 验证加密文件的 checksum 一致性

### Phase 5：集成与测试（约 12h）

1. 在 `xls/mod.rs` 添加条件编译的 `pub mod streaming_reader`
2. 在 `xls_sax_analyser.rs` 添加 `use_streaming` 选项
3. 准备 100K rows xls fixture
4. 对比 old vs streaming 模式的：
   - throughput (rows/s)
   - peak RSS
   - checksum 一致性
5. 确保旧测试用 `default-features = false` 跑（旧实现不退化）
6. 确保新测试用 `default = ["xls-streaming"]` 跑

---

## 7. 风险评估

### 7.1 OLE2 格式复杂度（中等风险）

**风险**：OLE2/CFB 的 FAT/MiniFAT/DIFAT chains 和 DirEntries 结构复杂。

**缓解**：cfb 0.14.0 已稳定处理这些结构（项目已在生产中使用）。流式模式不改变 OLE2 解析层，只改变 Workbook 流的读取方式。

### 7.2 SST 延迟解码的偏移计算（中等风险）

**风险**：SST 的 CONTINUE 边界处有 grbit 字节重置，计算每个 string 的字节偏移需要正确处理这些边界。

**缓解**：
- `sst.rs` 已有完整的 CONTINUE/grbit 处理逻辑
- `LazySst::scan_offsets()` 可复用 `SstCursor` 的遍历逻辑
- 与 `parse_sst()` 的结果做 checksum 对比验证

### 7.3 加密 XLS 的处理（低风险）

**风险**：CryptoAPI RC4 加密要求对整个 Workbook 流解密。

**缓解**：加密文件走"先解密再流式解析"路径，解密后的行为与非加密文件一致。内存峰值 = 解密后的 Workbook 流大小（与当前相同），但后续解析是流式的。

### 7.4 Stream 的可变借用限制（低风险）

**风险**：`StreamingSheetReader` 持有 `&mut Stream`，同一时刻只能有一个活跃的 sheet reader。

**缓解**：这是有意设计 -- 逐 sheet 处理是标准模式。如果用户需要随机访问多个 sheet，应使用旧的 DOM 模式。在文档中明确说明此限制。

### 7.5 与现有 `parse_worksheet` 的接口差异（低风险）

**风险**：`parse_worksheet(buf, start, ...)` 接收整个 workbook 的切片；流式模式需要逐记录读取。

**缓解**：`StreamingSheetReader` 的 cell 解析逻辑从 `reader.rs` 提取为共用函数，两种模式复用同一套解析代码。

---

## 8. 验证计划

### 8.1 功能验证

| 测试 | 描述 |
|------|------|
| `streaming_roundtrip_basic` | 基本读写往返：write → read_streaming → 断言值一致 |
| `streaming_roundtrip_multisheet` | 多 sheet 往返：6 张 sheet，逐 sheet 验证 |
| `streaming_sst_checksum` | SST 延迟解码结果与 `parse_sst()` 完全一致 |
| `streaming_encrypted` | 加密文件的流式读取 |
| `streaming_empty_workbook` | 空工作簿的流式读取 |
| `streaming_hidden_sheets` | 隐藏 sheet 的可见性正确传递 |

### 8.2 性能验证

| 指标 | 目标 | 测量方法 |
|------|------|---------|
| throughput | >= 100K rows/s | 100K rows fixture，`std::time::Instant` 计时 |
| peak RSS | <= 50MB（100K rows） | `/usr/bin/time -l` 或 `jemalloc` 统计 |
| 与 old 模式 checksum | 完全一致 | 逐 cell 比较 `CellValue` |

### 8.3 回归验证

```bash
# 旧模式测试（确保不退化）
cargo test -p easyexcel-xls --no-default-features

# 新模式测试
cargo test -p easyexcel-xls --features xls-streaming

# 全量测试
cargo test --workspace
```

---

## 9. 回滚策略

Feature flag 关掉即可回旧模式，零迁移成本：

```toml
# 用户在自己的 Cargo.toml 中
easyexcel-xls = { version = "0.1.3", default-features = false }
```

所有旧 API 签名和行为完全不变。新代码通过 `#[cfg(feature = "xls-streaming")]` 隔离。

---

## 10. 预计工作量

| 阶段 | 工时 | 依赖 |
|------|------|------|
| Phase 1：基础设施 + StreamingRecordIter | 8h | 无 |
| Phase 2：SST 延迟解码 | 8h | Phase 1 |
| Phase 3：Sheet 流式读取 | 12h | Phase 2 |
| Phase 4：加密兼容 | 8h | Phase 3 |
| Phase 5：集成 + 测试 + 性能验证 | 12h | Phase 4 |
| **合计** | **48h** | |

---

## 11. 附录

### A. 关键文件索引

| 文件 | 作用 |
|------|------|
| `crates/easyexcel-xls/src/xls/reader.rs` | DOM 模式读取（保持不变） |
| `crates/easyexcel-xls/src/xls/sst.rs` | SST 解析（两个模式共用） |
| `crates/easyexcel-xls/src/xls/biff.rs` | BIFF 记录原语（两个模式共用） |
| `crates/easyexcel-xls/src/biff8/record_stream.rs` | event 层 record 遍历（保持不变） |
| `crates/easyexcel-xls/src/biff8/event_record.rs` | 事件记录解码器（可复用） |
| `crates/easyexcel/src/analysis/v03/xls_sax_analyser.rs` | XLS 事件分析器（需添加 streaming 选项） |

### B. cfb Stream 能力确认

cfb 0.14.0 的 `Stream<F>` 实现了：
- `std::io::Read` -- 按需读取
- `std::io::Seek` -- 随机跳转
- `std::io::BufRead` -- 缓冲读取
- `stream.len()` -- 获取流总长度
- 内部使用 `StreamBuffer` 做自适应缓冲

这些能力完全支持"先 seek 到 sheet 偏移，再逐记录读取"的流式模式。
