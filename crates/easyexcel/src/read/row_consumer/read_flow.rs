/// 对应 Java：无直接对应对象；Rust 架构扩展。 Controls whether the read loop continues or stops after a row event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadFlow {
    Continue,
    Stop,
}

