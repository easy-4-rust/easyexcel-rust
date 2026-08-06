/// 对应 Java：无直接对应对象；Rust 架构扩展。 一个按文档顺序产生的 XLSX 单元格事件。
#[derive(Debug, Clone, PartialEq)]
pub struct XlsxCellEvent {
    /// 零基 `(row, column)` 坐标。
    pub position: (u32, usize),
    /// 缓存值。
    pub value: XlsxCellValue,
    /// 公式文本，不含强制添加的等号。
    pub formula: Option<String>,
    /// 按 Excel 数字格式渲染的显示值。
    pub display_value: Option<String>,
    /// 数字的十进制表示。
    pub decimal_value: Option<BigDecimal>,
    /// 当前样式是否是日期格式。
    pub date_formatted: bool,
}

