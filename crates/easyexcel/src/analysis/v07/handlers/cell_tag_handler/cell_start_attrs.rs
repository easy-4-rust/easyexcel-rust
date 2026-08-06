/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parsed `<c>` start attributes — used by both the handler and `xlsx_rows`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellStartAttrs {
    /// Zero-based `(row, column)` from `r` or the fallback cursor.
    pub position: (u32, usize),
    /// Zero-based style index from `s` (default 0).
    pub style_index: usize,
    /// Raw OOXML `t` attribute (`s` / `n` / `b` / …).
    pub cell_type: Option<String>,
    /// Logical type from Java `CellDataTypeEnum.buildFromCellType`.
    pub data_type: CellDataType,
}

