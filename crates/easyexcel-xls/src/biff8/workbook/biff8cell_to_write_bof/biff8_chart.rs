/// BIFF8 工作表中的内嵌图表。
///
/// 对应 Java：`HSSFChart` 与 `HSSFPatriarch#createChart` 的组合状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Chart {
    /// 图表类型。
    pub kind: Biff8ChartKind,
    /// 可选标题。
    pub title: Option<String>,
    /// 双单元格锚点起始行。
    pub first_row: u16,
    /// 双单元格锚点起始列。
    pub first_column: u8,
    /// 双单元格锚点结束行。
    pub last_row: u16,
    /// 双单元格锚点结束列。
    pub last_column: u8,
    /// 数据系列。
    pub series: Vec<Biff8ChartSeries>,
}

impl Biff8Chart {
    /// 创建不含标题和数据系列的图表。
    #[must_use]
    pub const fn new(
        kind: Biff8ChartKind,
        first_row: u16,
        first_column: u8,
        last_row: u16,
        last_column: u8,
    ) -> Self {
        Self {
            kind,
            title: None,
            first_row,
            first_column,
            last_row,
            last_column,
            series: Vec::new(),
        }
    }

    /// 设置图表标题。
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 追加数据系列。
    #[must_use]
    pub fn with_series(mut self, series: Biff8ChartSeries) -> Self {
        self.series.push(series);
        self
    }
}
