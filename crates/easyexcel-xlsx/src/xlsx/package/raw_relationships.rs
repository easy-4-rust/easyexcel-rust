/// 包含外部标记的关系映射：`Id -> (Target, Type, External)`。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type RawRelationships = HashMap<String, (String, String, bool)>;

