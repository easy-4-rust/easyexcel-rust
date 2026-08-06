/// A function implementation pointer.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type FnImpl = fn(&mut dyn Context, &[Value]) -> Value;

