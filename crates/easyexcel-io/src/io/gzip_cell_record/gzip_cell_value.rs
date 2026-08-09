/// 对应 Java：无直接对应对象；Rust 架构扩展。 可写入 gzip spill 的中立单元格值。
#[derive(Debug, Clone, PartialEq)]
pub enum GzipCellValue {
    /// 空值。
    Empty,
    /// 文本。
    Text(String),
    /// 布尔值。
    Bool(bool),
    /// 有符号整数。
    Int(i64),
    /// IEEE 754 双精度数。
    Float(f64),
    /// 十进制数字符串。
    Decimal(String),
    /// ISO 日期字符串。
    Date(String),
    /// ISO 日期时间字符串。
    DateTime(String),
    /// Excel 错误文本。
    Error(String),
    /// 公式表达式。
    Formula(String),
    /// 超链接显示值。
    Hyperlink {
        /// 目标地址。
        url: String,
        /// 显示文本。
        text: String,
    },
    /// Java `HyperlinkData` 的完整类型和绝对/相对范围。
    TypedHyperlink {
        /// 目标地址。
        address: String,
        /// 显示文本。
        text: String,
        /// 0=NONE、1=URL、2=DOCUMENT、3=EMAIL、4=FILE。
        kind: u8,
        /// 绝对首行。
        first_row: Option<u32>,
        /// 绝对首列。
        first_col: Option<u16>,
        /// 绝对末行。
        last_row: Option<u32>,
        /// 绝对末列。
        last_col: Option<u16>,
        /// 相对首行。
        relative_first_row: Option<i32>,
        /// 相对首列。
        relative_first_col: Option<i32>,
        /// 相对末行。
        relative_last_row: Option<i32>,
        /// 相对末列。
        relative_last_col: Option<i32>,
    },
    /// 带批注的嵌套值。
    Comment {
        /// 被批注修饰的原始单元格值。
        value: Box<Self>,
        /// 批注正文。
        text: String,
    },
    /// 带完整 Java `CommentData` JSON 元数据的嵌套值。
    CommentMetadata {
        /// 被批注修饰的原始单元格值。
        value: Box<Self>,
        /// 由上层 easyexcel crate 编解码的版本化 JSON 元数据。
        metadata: Vec<u8>,
    },
    /// 图片字节。
    Image(Vec<u8>),
    /// 已展平的富文本。
    RichText(String),
    /// 带完整字体区间元数据的富文本 JSON。
    RichTextMetadata(Vec<u8>),
    /// 单元格值及多张图片。
    Images {
        /// 被图片修饰的原始单元格值。
        value: Box<Self>,
        /// 图片二进制内容。
        images: Vec<Vec<u8>>,
    },
    /// 单元格值、多张图片及其 Java 锚点/类型元数据。
    ImagesMetadata {
        /// 被图片修饰的原始单元格值。
        value: Box<Self>,
        /// 图片二进制内容。
        images: Vec<Vec<u8>>,
        /// 由上层 easyexcel crate 编解码的图片元数据 JSON。
        metadata: Vec<u8>,
    },
    /// Stateful writer journal cell decorated with a deduplicated style id.
    Styled {
        /// Underlying neutral cell value.
        value: Box<Self>,
        /// Index into the writer-owned style registry.
        style_id: u32,
    },
    /// Stateful writer row metadata appended after the physical cells.
    JournalMetadata {
        /// Final row height after handler processing, when explicitly set.
        row_height: Option<u16>,
    },
    /// Stateful writer 中已经实际应用到 worksheet 的绝对合并范围。
    ///
    /// 该值是运行结果而不是待重新执行的策略，AutoStreaming 晋升时可据此
    /// 重建合并区域，同时保证用户 Handler 不会被二次调用。
    JournalMergeRange {
        /// 起始行（零基）。
        first_row: u32,
        /// 结束行（零基，包含）。
        last_row: u32,
        /// 起始列（零基）。
        first_col: u16,
        /// 结束列（零基，包含）。
        last_col: u16,
    },
}
