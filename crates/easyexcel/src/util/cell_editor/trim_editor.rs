/// 对应 Java：无直接对应对象；Rust 架构扩展。 Trims whitespace from string cell values.
///
/// Mirrors hutool `TrimEditor`.
/// Note: easyexcel-rust has `auto_trim(true)` which does this globally
/// without needing a `CellEditor`. This editor is for selective trimming.
#[derive(Debug, Default, Clone)]
pub struct TrimEditor;

impl CellEditor for TrimEditor {
    fn edit(&self, original: &CellValue, _sheet_name: &str, _row: u32, _col: u32) -> CellValue {
        match original {
            CellValue::String(s) => {
                CellValue::String(easyexcel_utils::string_utils::java_trim(s).to_owned())
            }
            other => other.clone(),
        }
    }
}

