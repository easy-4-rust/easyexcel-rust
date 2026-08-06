/// 对应 Java：无直接对应对象；Rust 架构扩展。 POI `BOFRecord` type codes used by `EasyExcel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BofType {
    /// Workbook-level BOF.
    Workbook,
    /// Worksheet-level BOF.
    Worksheet,
    /// Other (chart, macro, …) — ignored by Java.
    Other,
}

