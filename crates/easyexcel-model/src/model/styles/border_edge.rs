#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct BorderEdge {
    pub style: BorderStyle,
    pub color: Color,
}

