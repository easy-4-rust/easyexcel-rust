/// 对应 Java：无直接对应对象；Rust 架构扩展。 Frozen-pane configuration: number of frozen rows/columns at the top-left.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrozenPanes {
    pub rows: u32,
    pub cols: u32,
}

