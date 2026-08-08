/// BIFF8 图表的一个数据系列。
///
/// 对应 Java：POI `HSSFChart.HSSFSeries`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8ChartSeries {
    /// 可选系列名称，序列化为 BIFF8 `SeriesText` 记录。
    pub name: Option<String>,
    /// 可选分类标签区域。
    pub categories: Option<Biff8ChartRange>,
    /// 必填数值区域。
    pub values: Biff8ChartRange,
}

impl Biff8ChartSeries {
    /// 创建仅包含数值区域的数据系列。
    #[must_use]
    pub const fn new(values: Biff8ChartRange) -> Self {
        Self {
            name: None,
            categories: None,
            values,
        }
    }

    /// 设置系列名称。
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置分类标签区域。
    #[must_use]
    pub fn with_categories(mut self, categories: Biff8ChartRange) -> Self {
        self.categories = Some(categories);
        self
    }
}
