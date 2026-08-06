/// Metadata + implementation for one worksheet function.
#[derive(Clone)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct FnEntry {
    pub name: &'static str,
    pub min_args: usize,
    pub max_args: usize,
    pub volatile: bool,
    pub func: FnImpl,
}

