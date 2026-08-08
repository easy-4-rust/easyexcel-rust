# XLSX 性能热点记录

采样环境：macOS arm64、release binary、`benchmark-suite-v1.json`、1,000,000 行、单 worker。这里的三次短测用于定位优化方向，不替代 Linux 固定环境的 7 样本发布门禁。

## Streaming write

初始 Rust 历史基线约 105,346 rows/s。采样显示 Handler 调用链反复克隆/释放 `WriteContextHolderState`，并通过 `Arc<Mutex<Box<dyn WriteHandler>>>` 串行加锁；即使内置 Handler 不使用 row/cell callback，也会为每个对象构造完整上下文。

实施项：

- 不可变 Holder 状态改为 `Arc` 共享，路径改为 `Arc<PathBuf>`。
- 单线程 Handler 链改为 `Rc<RefCell<_>>`，自定义 Handler 仍走保守完整生命周期。
- 内置 Handler 声明 row/cell context 能力；无上下文需求时进入直接 cell emission 快路径。
- 注解样式 Handler 始终保留 Java 对齐的注册数量与顺序，仅用能力标记跳过空回调。

优化后三次为 266,575、257,799、253,368 rows/s，中位数 257,799 rows/s；文件均为 24,918,576 bytes，checksum 均为 `df7966ddec70e23c9df5f8890d6c512c6ea1883d30f5283ac8d09d483f876c95`。

## Event read

初始采样中 `XlsxCellEventReader::finish_cell → format_with_code → ssfmt::parser::parse` 是主要热点：格式字符串按单元格重复解析并克隆 AST。格式预编译并绑定到 `XlsxCellEventReader` 生命周期后，三次读取中位数从约 130,605 提升至 181,052 rows/s。

第二次采样显示热点转移到 `excel_display_number` 的 `format!("{value:.14e}").parse::<f64>()`。对小于 `10^14` 且二进制精确的整数/半整数增加等价快路径后，三次为 204,877、208,307、205,551 rows/s，中位数 205,551 rows/s；checksum 保持一致。

读取仍未达到 Java 历史约 307K–343K rows/s，下一轮应检查：

- 每个 numeric cell 的 `BigDecimal` 构造是否能按读取模式惰性化。
- 每行 `HashMap`/`HashSet` 元数据容器能否改为复用的稠密 scratch。
- typed scalar/no-extra/no-formula dispatch 能否跳过通用 metadata 所有权转换。
- 显式并发 Listener 的有界“解析—转换—有序提交”管线；普通 Listener 不允许并发回调。

## 证据文件

- `current-before-optimization/xlsx-stream-write.sample.txt`
- `current-after-holder-arc/xlsx-stream-write.sample.txt`
- `current-before-read-optimization/xlsx-event-read.sample.txt`
- `current-after-format-cache/xlsx-event-read.sample.txt`

