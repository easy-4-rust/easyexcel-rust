//! Cell value editor trait — mirrors hutool `cn.hutool.poi.excel.cell.CellEditor`.
//!
//! In hutool, `CellEditor` transforms cell values during reading.
//! In easyexcel-rust, this can be registered on the reader builder and
//! applied before the value reaches the `ReadListener`.

use crate::CellValue;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Transforms a cell value during reading.
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

include!("cell_editor/trim_editor.rs");

include!("cell_editor/numeric_to_int_editor.rs");

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
