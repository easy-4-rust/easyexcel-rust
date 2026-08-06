/// 对应 Java：无直接对应对象；Rust 架构扩展。 An ARGB color, e.g. `FFFF0000` for opaque red. `None` means "automatic".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color(pub Option<u32>);

impl Color {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn rgb(argb: u32) -> Self {
        Color(Some(argb))
    }
    pub const AUTO: Color = Color(None);
}

