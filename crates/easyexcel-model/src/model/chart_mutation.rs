use super::{ChartSeries, ChartType};

/// 在指定工作表创建图表的后端中立声明式请求。
///
/// 对应 Java：无直接对应对象；Rust 后端中立模型扩展。XLS 与 XLSX 引擎共同消费
/// 该对象，facade Handler 只负责提交请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartMutation {
    /// 图表所在工作表。
    pub sheet_name: String,
    /// 图表类型。
    pub chart_type: ChartType,
    /// 可选图表标题。
    pub title: Option<String>,
    /// 锚点起始行。
    pub first_row: u32,
    /// 锚点起始列。
    pub first_column: u16,
    /// 锚点结束行。
    pub last_row: u32,
    /// 锚点结束列。
    pub last_column: u16,
    /// 图表数据系列。
    pub series: Vec<ChartSeries>,
}

impl ChartMutation {
    /// 创建图表修改请求。
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        chart_type: ChartType,
        first_row: u32,
        first_column: u16,
        last_row: u32,
        last_column: u16,
    ) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            chart_type,
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

    /// 追加一个数据系列。
    #[must_use]
    pub fn with_series(mut self, series: ChartSeries) -> Self {
        self.series.push(series);
        self
    }
}
