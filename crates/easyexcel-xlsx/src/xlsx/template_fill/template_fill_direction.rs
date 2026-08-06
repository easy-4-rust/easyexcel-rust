/// 对应 Java：无直接对应对象；Rust 架构扩展。 集合填充方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TemplateFillDirection {
    /// 逐行向下填充。
    #[default]
    Vertical,
    /// 逐列向右填充。
    Horizontal,
}

