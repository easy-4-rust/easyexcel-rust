/// BIFF8 图表系列引用的单元格矩形区域。
///
/// 坐标均从零开始并包含末端。对应 Java：POI
/// `HSSFChart.HSSFSeries#setValuesCellRange`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8ChartRange {
    /// 数据所在工作表名称。
    pub sheet_name: String,
    /// 起始行。
    pub first_row: u16,
    /// 起始列。
    pub first_column: u8,
    /// 结束行。
    pub last_row: u16,
    /// 结束列。
    pub last_column: u8,
}

impl Biff8ChartRange {
    /// 创建已经过 BIFF8 坐标边界校验的数据区域。
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        first_row: u16,
        first_column: u8,
        last_row: u16,
        last_column: u8,
    ) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            first_row,
            first_column,
            last_row,
            last_column,
        }
    }

    pub(crate) fn cell_count(&self) -> u16 {
        let rows = u32::from(self.last_row - self.first_row) + 1;
        let columns = u32::from(self.last_column - self.first_column) + 1;
        u16::try_from(rows.saturating_mul(columns)).unwrap_or(u16::MAX)
    }
}
