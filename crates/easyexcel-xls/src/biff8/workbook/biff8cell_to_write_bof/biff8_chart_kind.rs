/// BIFF8 内嵌图表类型。
///
/// 对应 Java：POI `HSSFChart.HSSFChartType`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8ChartKind {
    /// 柱状图，对应 BIFF8 `Bar` 记录。
    Bar,
    /// 折线图，对应 BIFF8 `Line` 记录。
    Line,
    /// 饼图，对应 BIFF8 `Pie` 记录。
    Pie,
}
