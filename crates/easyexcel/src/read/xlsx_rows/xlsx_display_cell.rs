/// 对应 Java：无直接对应对象；Rust 架构扩展。 `EasyExcel` listener 消费的显示单元格。
pub(crate) struct XlsxDisplayCell {
    pub(crate) position: (u32, usize),
    pub(crate) value: CellValue,
    pub(crate) formula: Option<FormulaData>,
    pub(crate) display_value: Option<String>,
    pub(crate) decimal_value: Option<BigDecimal>,
}

