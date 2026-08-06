/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 OBJ 记录中的公共对象数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8CommonObjectData {
    /// 对象类型码，例如批注对象为 `0x0019`。
    pub object_type: u16,
    /// 工作表范围内的对象编号。
    pub object_id: u32,
}

