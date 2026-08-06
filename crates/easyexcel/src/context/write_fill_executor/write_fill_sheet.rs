/// Worksheet metadata passed into template fill execution.
///
/// 对应 Java：`WriteSheet` selection inside `ExcelBuilderImpl.fill`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteFillSheet {
    /// Selected worksheet name.
    pub sheet_name: String,
    /// Optional zero-based sheet index.
    pub sheet_index: Option<usize>,
}

impl Default for WriteFillSheet {
    fn default() -> Self {
        Self {
            sheet_name: "Sheet1".to_owned(),
            sheet_index: None,
        }
    }
}

