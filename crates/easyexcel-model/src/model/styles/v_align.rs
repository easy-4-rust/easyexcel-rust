/// 对应 Java：无直接对应对象；Rust 架构扩展。 Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VAlign {
    Top,
    #[default]
    Bottom,
    Center,
    Justify,
    Distributed,
}

