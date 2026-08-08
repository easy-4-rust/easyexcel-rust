//! Handler 图表数据区域。

/// 图表系列引用的矩形单元格区域，坐标均从零开始且包含末端。
///
/// 对应 Java：`org.apache.poi.ss.util.CellRangeAddress`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartRange {
    /// 数据所在工作表名称。
    pub sheet_name: String,
    /// 起始行，从零开始。
    pub first_row: u32,
    /// 起始列，从零开始。
    pub first_column: u16,
    /// 结束行，从零开始且包含该行。
    pub last_row: u32,
    /// 结束列，从零开始且包含该列。
    pub last_column: u16,
}

impl ChartRange {
    /// 创建图表数据区域。
    ///
    /// 对应 Java：`CellRangeAddress(firstRow, lastRow, firstCol, lastCol)`。
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        first_row: u32,
        first_column: u16,
        last_row: u32,
        last_column: u16,
    ) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            first_row,
            first_column,
            last_row,
            last_column,
        }
    }
}
