/// 格式引擎可共同创建的基础图表类型。
///
/// 对应 Java：无直接对应对象；Rust 后端中立模型扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// 柱状图。
    Bar,
    /// 折线图。
    Line,
    /// 饼图。
    Pie,
}
