/// 对应 Java：无直接对应对象；Rust 架构扩展。 与具体门面元数据解耦的 XLSX 字体格式描述。
#[derive(Debug, Clone, Default)]
pub struct FontFormatSpec {
    /// 字体名称。
    pub name: Option<String>,
    /// 字号（磅）。
    pub size: Option<f64>,
    /// 是否斜体。
    pub italic: Option<bool>,
    /// 是否使用删除线。
    pub strikeout: Option<bool>,
    /// 字体颜色。
    pub color: Option<Color>,
    /// 上标或下标格式。
    pub script: Option<FormatScript>,
    /// 下划线格式。
    pub underline: Option<FormatUnderline>,
    /// 字符集编号。
    pub charset: Option<u8>,
    /// 是否粗体。
    pub bold: Option<bool>,
}

