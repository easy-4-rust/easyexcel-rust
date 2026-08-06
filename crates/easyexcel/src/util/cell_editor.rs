//! Cell value editor trait — mirrors hutool `cn.hutool.poi.excel.cell.CellEditor`.
//!
//! In hutool, `CellEditor` transforms cell values during reading.
//! In easyexcel-rust, this can be registered on the reader builder and
//! applied before the value reaches the `ReadListener`.

use crate::CellValue;

/// Transforms a cell value during reading.
///
/// Mirrors hutool `CellEditor` interface:
/// ```java
/// public interface CellEditor {
///     Object edit(Cell cell, Object value);
/// }
/// ```
///
/// In Rust, the `Cell` object is replaced by the `CellValue` since
/// we don't have a POI cell handle.
pub trait CellEditor: Send + Sync {
    /// Transforms a cell value before it reaches the listener.
    fn edit(&self, original: &CellValue, sheet_name: &str, row: u32, col: u32) -> CellValue;
}

/// Trims whitespace from string cell values.
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

/// Converts numeric (Int/Float/Decimal) cell values to integers by truncation.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_editor_strips_whitespace() {
        let editor = TrimEditor;
        let result = editor.edit(&CellValue::String("  hello  ".into()), "", 0, 0);
        assert_eq!(result, CellValue::String("hello".into()));
    }

    #[test]
    fn trim_editor_preserves_non_string() {
        let editor = TrimEditor;
        let result = editor.edit(&CellValue::Int(42), "", 0, 0);
        assert_eq!(result, CellValue::Int(42));
    }

    #[test]
    fn numeric_editor_converts_float_to_int() {
        let editor = NumericToIntEditor;
        assert_eq!(
            editor.edit(&CellValue::Float(3.7), "", 0, 0),
            CellValue::Int(3)
        );
        assert_eq!(
            editor.edit(&CellValue::Int(42), "", 0, 0),
            CellValue::Int(42)
        );
        assert_eq!(
            editor.edit(&CellValue::Bool(true), "", 0, 0),
            CellValue::Int(1)
        );
        assert_eq!(
            editor.edit(&CellValue::String("99".into()), "", 0, 0),
            CellValue::Int(99)
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn trim_editor_trims_strings_and_preserves_others() {
        // 对应 Java（hutool）：TrimEditor 选择性去空白
        let editor = TrimEditor;
        assert_eq!(
            editor.edit(&CellValue::String("\t x \n".to_owned()), "S", 0, 0),
            CellValue::String("x".to_owned())
        );
        assert_eq!(
            editor.edit(&CellValue::Bool(true), "S", 0, 0),
            CellValue::Bool(true)
        );
    }

    #[test]
    fn numeric_editor_handles_decimal_string_and_other_values() {
        // 对应 Java（hutool）：NumericToIntEditor 其余分支
        let editor = NumericToIntEditor;
        // Decimal 可解析为整数
        assert_eq!(
            editor.edit(&CellValue::Decimal("12".parse().unwrap()), "S", 0, 0),
            CellValue::Int(12)
        );
        // Decimal 带小数部分，字符串解析失败回退 0
        assert_eq!(
            editor.edit(&CellValue::Decimal("12.5".parse().unwrap()), "S", 0, 0),
            CellValue::Int(0)
        );
        // 非数字字符串原样保留
        assert_eq!(
            editor.edit(&CellValue::String("abc".to_owned()), "S", 0, 0),
            CellValue::String("abc".to_owned())
        );
        // 其他类型原样保留
        assert_eq!(
            editor.edit(
                &CellValue::DateTime(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                ),
                "S",
                0,
                0
            ),
            CellValue::DateTime(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            )
        );
    }
}
