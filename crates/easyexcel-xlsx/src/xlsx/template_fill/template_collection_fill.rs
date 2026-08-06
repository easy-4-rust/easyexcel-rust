/// 对应 Java：无直接对应对象；Rust 架构扩展。 一次集合填充请求。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateCollectionFill {
    /// 可选集合前缀；`None` 对应 `{.field}`。
    pub name: Option<String>,
    /// 需要填充的数据行。
    pub rows: Vec<TemplateFillData>,
    /// 填充方向。
    pub direction: TemplateFillDirection,
    /// 是否在垂直填充时平移模板尾部行。
    pub force_new_row: bool,
    /// 是否保留模板单元格样式。
    pub auto_style: bool,
    /// 同一工作表中的调用顺序。
    pub order: usize,
    /// 物理列下标到目标 `cellXfs` 样式下标的映射。
    pub column_styles: std::collections::BTreeMap<usize, u32>,
}
