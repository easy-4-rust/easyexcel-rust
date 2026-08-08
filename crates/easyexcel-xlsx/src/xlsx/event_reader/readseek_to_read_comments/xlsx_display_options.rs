/// 对应 Java：无直接对应对象；Rust 架构扩展。 单元格显示配置。
#[derive(Debug, Clone)]
pub struct XlsxDisplayOptions {
    /// 是否按 1904 日期系统解释序列值。
    pub date_1904: bool,
    /// General 极值是否使用科学计数法。
    pub use_scientific_format: bool,
    /// 数字和日期显示区域设置。
    pub locale: SpreadsheetLocale,
    /// 是否保留每个数字单元格的高精度十进制值。
    ///
    /// 动态读取和自定义 Converter 需要该元数据；只读取不含 `BigDecimal` 的静态
    /// scalar model 时可以关闭，避免每个数字单元格都构造 `BigDecimal`。
    pub retain_decimal_values: bool,
    /// 需要保留数字显示文本的物理列；`None` 表示全部列。
    ///
    /// typed schema 可只为 String 目标列执行 DataFormatter；动态读取、自定义
    /// Converter 或名称尚未解析的 schema 保持 `None`，确保完整 Java 语义。
    pub retain_display_columns: Option<HashSet<usize>>,
}

impl Default for XlsxDisplayOptions {
    fn default() -> Self {
        Self {
            date_1904: false,
            use_scientific_format: false,
            locale: SpreadsheetLocale::default(),
            retain_decimal_values: true,
            retain_display_columns: None,
        }
    }
}
