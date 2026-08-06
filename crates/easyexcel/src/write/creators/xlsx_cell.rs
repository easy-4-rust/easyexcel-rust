/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct XlsxCell<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
    pub(crate) row_index: u32,
    pub(crate) column_index: u16,
}

