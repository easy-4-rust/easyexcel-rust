/// 每个工作表的 `(row, col) -> numeric cell` 映射。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type Biff8NumericSheets = Vec<HashMap<(u32, usize), Biff8NumericCell>>;

