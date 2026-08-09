use super::ChartRange;

/// 一个后端中立图表数据系列。
///
/// 对应 Java：无直接对应对象；Rust 后端中立模型扩展。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartSeries {
    /// 可选系列名称。
    pub name: Option<String>,
    /// 可选分类轴区域。
    pub categories: Option<ChartRange>,
    /// 必填数值区域。
    pub values: ChartRange,
}

impl ChartSeries {
    /// 创建仅包含数值区域的系列。
    #[must_use]
    pub const fn new(values: ChartRange) -> Self {
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

    /// 设置分类轴区域。
    #[must_use]
    pub fn with_categories(mut self, categories: ChartRange) -> Self {
        self.categories = Some(categories);
        self
    }
}
