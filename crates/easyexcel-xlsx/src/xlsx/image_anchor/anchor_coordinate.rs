/// 对应 Java：无直接对应对象；Rust 架构扩展。 相对当前单元格或绝对指定的锚点坐标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnchorCoordinate {
    /// 非零绝对坐标；零值按未指定处理，以匹配 EasyExcel/POI 语义。
    pub absolute: Option<u32>,
    /// 相对当前坐标的有符号偏移。
    pub relative: Option<i32>,
}

