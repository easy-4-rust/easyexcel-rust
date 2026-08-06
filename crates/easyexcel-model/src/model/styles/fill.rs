/// 对应 Java：无直接对应对象；Rust 架构扩展。 Fill pattern. We model the common solid fill plus "none"; other patterns are
/// preserved opaquely by index in the readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Fill {
    pub pattern: FillPattern,
    pub fg: Color,
    pub bg: Color,
}

