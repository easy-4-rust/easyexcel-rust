/// 对应 Java：无直接对应对象；Rust 架构扩展。 Sheet visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    VeryHidden,
}

