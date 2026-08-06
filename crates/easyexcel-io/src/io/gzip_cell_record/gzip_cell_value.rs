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
    /// 带批注的嵌套值。
    Comment {
        /// 被批注修饰的原始单元格值。
        value: Box<Self>,
        /// 批注正文。
        text: String,
    },
    /// 图片字节。
    Image(Vec<u8>),
    /// 已展平的富文本。
    RichText(String),
    /// 单元格值及多张图片。
    Images {
        /// 被图片修饰的原始单元格值。
        value: Box<Self>,
        /// 图片二进制内容。
        images: Vec<Vec<u8>>,
    },
}

