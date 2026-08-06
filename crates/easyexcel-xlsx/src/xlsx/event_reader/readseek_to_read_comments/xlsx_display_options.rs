/// 对应 Java：无直接对应对象；Rust 架构扩展。 单元格显示配置。
#[derive(Debug, Clone, Default)]
pub struct XlsxDisplayOptions {
    /// 是否按 1904 日期系统解释序列值。
    pub date_1904: bool,
    /// General 极值是否使用科学计数法。
    pub use_scientific_format: bool,
    /// 数字和日期显示区域设置。
    pub locale: SpreadsheetLocale,
}

