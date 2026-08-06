/// 每个工作表的 `(row, col) -> Excel 格式化显示文本` 映射。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type Biff8SheetDisplays = Vec<HashMap<(u32, usize), String>>;

