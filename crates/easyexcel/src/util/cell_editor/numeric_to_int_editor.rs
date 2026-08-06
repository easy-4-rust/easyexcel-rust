/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts numeric (Int/Float/Decimal) cell values to integers by truncation.
///
/// Mirrors hutool `NumericToIntEditor`.
#[derive(Debug, Default, Clone)]
pub struct NumericToIntEditor;

impl CellEditor for NumericToIntEditor {
    // 对应 Java（hutool）：NumericToIntEditor 对浮点做截断取整，截断正是本转换器的语义
    #[allow(clippy::cast_possible_truncation)]
    fn edit(&self, original: &CellValue, _sheet_name: &str, _row: u32, _col: u32) -> CellValue {
        match original {
            CellValue::Int(n) => CellValue::Int(*n),
            CellValue::Float(f) => CellValue::Int(*f as i64),
            CellValue::Decimal(d) => {
                let s = d.to_string();
                if let Ok(n) = s.parse::<i64>() {
                    CellValue::Int(n)
                } else {
                    CellValue::Int(0)
                }
            }
            CellValue::Bool(b) => CellValue::Int(i64::from(*b)),
            CellValue::String(s) => {
                if let Ok(n) = easyexcel_utils::string_utils::java_trim(s).parse::<i64>() {
                    CellValue::Int(n)
                } else {
                    original.clone()
                }
            }
            other => other.clone(),
        }
    }
}

