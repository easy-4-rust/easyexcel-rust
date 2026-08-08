//! Handler 提交的后端中立图表修改。

use super::chart_series::ChartSeries;
use super::chart_type::ChartType;

/// 在指定工作表创建图表的声明式请求。
///
/// 锚点坐标均从零开始。结束坐标用于 BIFF8 的双单元格锚点；XLSX 后端
/// 使用起始坐标插入图表，并以该矩形计算宽高。
///
/// 对应 Java：Handler 中通过 `Drawing#createChart(ClientAnchor)` 创建图表。
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
