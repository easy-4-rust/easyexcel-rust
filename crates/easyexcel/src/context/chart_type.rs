//! Handler 图表修改支持的图表类型。

/// Handler 可跨 XLSX 与 XLS 后端创建的基础图表类型。
///
/// 对应 Java：`org.apache.poi.ss.usermodel.charts.ChartData` 的常用实现。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// 柱状图。
    Bar,
    /// 折线图。
    Line,
    /// 饼图。
    Pie,
}
