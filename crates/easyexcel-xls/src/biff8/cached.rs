//! 公式缓存值求值桥接。
//!
//! 使用工作区内的 `easyexcel-formula` 公式求值引擎在写入时当场计算
//! FORMULA 记录缓存值，
//! 等价于 Java POI `FormulaEvaluator` 写缓存结果的路径：
//! `Excel` / `LibreOffice` 打开 `.xls` 即显示正确数值，不依赖打开时重算。
//!
//! 求值失败（循环引用、未知函数、语法不支持）时静默跳过——写入端
//! 回退为全零缓存值（`CALCMODE` 自动重算兜底），永不返回错误。

use std::collections::HashMap;

use super::workbook::{Biff8Sheet, Biff8Value};
use easyexcel_formula::Engine;
use easyexcel_model::{Cell, CellError, CellValue, Workbook};

/// FORMULA 记录缓存结果（8 字节结果字段，字符串另走 STRING 记录）。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Biff8Cached {
    /// 数值结果（0.0 全零字节 = 触发打开时重算）。
    Number(f64),
    /// 布尔结果。
    Bool(bool),
    /// 错误结果（BIFF8 错误码）。
    Error(u8),
    /// 字符串结果（写入 STRING 记录，0x0207）。
    Text(String),
}

/// 对每个工作表求值公式，返回 `(row, col) → 缓存值` 的映射表。
/// 与 `write_worksheet` 的 sheet 顺序一一对应。
pub(crate) fn recalc_cached_values(sheets: &[Biff8Sheet]) -> Vec<HashMap<(u16, u8), Biff8Cached>> {
    let mut all = Vec::with_capacity(sheets.len());
    for sheet in sheets {
        all.push(recalc_sheet(sheet));
    }
    all
}

fn recalc_sheet(sheet: &Biff8Sheet) -> HashMap<(u16, u8), Biff8Cached> {
    let mut workbook = Workbook::new();
    {
        let ws = workbook.sheet_mut(0).expect("default sheet");
        for (&(row, col), cell) in &sheet.cells {
            let xcell = match &cell.value {
                Biff8Value::Text(text) => Cell::Text(text.clone()),
                Biff8Value::Number(number) => Cell::Number(*number),
                Biff8Value::Bool(flag) => Cell::Bool(*flag),
                Biff8Value::Formula(expr) => Cell::Formula {
                    expr: expr.clone(),
                    cached: CellValue::Empty,
                },
                Biff8Value::Blank => continue,
            };
            ws.set(u32::from(row), u32::from(col), xcell);
        }
    }
    Engine::new().recalc(&mut workbook);
    let mut cache = HashMap::new();
    let ws = workbook.sheet_mut(0).expect("default sheet");
    for (&(row, col), cell) in &sheet.cells {
        if matches!(cell.value, Biff8Value::Formula(_))
            && let Some(Cell::Formula { cached, .. }) = ws.get(u32::from(row), u32::from(col))
            && let Some(value) = to_biff8_cached(cached)
        {
            cache.insert((row, col), value);
        }
    }
    cache
}

fn to_biff8_cached(value: &CellValue) -> Option<Biff8Cached> {
    Some(match value {
        CellValue::Number(number) => Biff8Cached::Number(*number),
        CellValue::Text(text) => Biff8Cached::Text(text.clone()),
        CellValue::Bool(flag) => Biff8Cached::Bool(*flag),
        CellValue::Error(error) => Biff8Cached::Error(error_code(*error)),
        CellValue::Empty => return None,
    })
}

/// `xls` 错误枚举 → BIFF8 错误码（[MS-XLS] 2.5.24）。
fn error_code(error: CellError) -> u8 {
    match error {
        CellError::Null => 0x00,
        CellError::Div0 => 0x07,
        CellError::Ref => 0x17,
        CellError::Name => 0x1d,
        CellError::Num => 0x24,
        CellError::NA => 0x2a,
        CellError::GettingData => 0x2b,
        // Value / Spill / Calc 等错误在 BIFF8 无专属码 → #VALUE!
        _ => 0x0f,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biff8::workbook::{Biff8Cell, Biff8Sheet};

    fn sheet_with(values: &[(u16, u8, Biff8Value)]) -> Biff8Sheet {
        let mut sheet = Biff8Sheet::new("Sheet1");
        for &(row, col, ref value) in values {
            sheet
                .set(
                    u32::from(row),
                    usize::from(col),
                    Biff8Cell::general(value.clone()),
                )
                .unwrap();
        }
        sheet
    }

    #[test]
    fn arithmetic_and_function_cached_values() {
        // 对应 Java：POI FormulaEvaluator 求值缓存结果
        let sheet = sheet_with(&[
            (0, 0, Biff8Value::Number(2.0)),
            (0, 1, Biff8Value::Number(3.0)),
            (0, 2, Biff8Value::Formula("A1+B1".to_owned())),
            (0, 3, Biff8Value::Formula("SUM(A1:B1)".to_owned())),
        ]);
        let cache = recalc_sheet(&sheet);
        assert_eq!(cache.get(&(0, 2)), Some(&Biff8Cached::Number(5.0)));
        assert_eq!(cache.get(&(0, 3)), Some(&Biff8Cached::Number(5.0)));
    }

    #[test]
    fn if_and_comparison_and_text() {
        let sheet = sheet_with(&[
            (0, 0, Biff8Value::Number(2.0)),
            (0, 1, Biff8Value::Number(3.0)),
            (
                0,
                2,
                Biff8Value::Formula("IF(A1>B1,\"Y\",\"N\")".to_owned()),
            ),
            (0, 3, Biff8Value::Formula("A1&\"x\"".to_owned())),
        ]);
        let cache = recalc_sheet(&sheet);
        assert_eq!(cache.get(&(0, 2)), Some(&Biff8Cached::Text("N".to_owned())));
        assert_eq!(
            cache.get(&(0, 3)),
            Some(&Biff8Cached::Text("2x".to_owned()))
        );
    }

    #[test]
    fn errors_map_to_biff8_codes() {
        let sheet = sheet_with(&[
            (0, 0, Biff8Value::Number(1.0)),
            (0, 1, Biff8Value::Number(0.0)),
            (0, 2, Biff8Value::Formula("A1/B1".to_owned())),
        ]);
        let cache = recalc_sheet(&sheet);
        assert_eq!(cache.get(&(0, 2)), Some(&Biff8Cached::Error(0x07))); // #DIV/0!
    }

    #[test]
    fn unknown_function_and_missing_sheet_map_to_errors() {
        // 对应 Java：POI 求值未知函数 → #NAME?、不存在的工作表 → #REF!
        let sheet = sheet_with(&[
            (0, 0, Biff8Value::Number(1.0)),
            (0, 1, Biff8Value::Formula("UNKNOWN_FN(A1)".to_owned())),
            (0, 2, Biff8Value::Formula("Sheet2!A1".to_owned())),
        ]);
        let cache = recalc_sheet(&sheet);
        assert_eq!(cache.get(&(0, 1)), Some(&Biff8Cached::Error(0x1d))); // #NAME?
        assert_eq!(cache.get(&(0, 2)), Some(&Biff8Cached::Error(0x17))); // #REF!
    }

    #[test]
    fn text_operand_in_arithmetic_yields_value_error() {
        let sheet = sheet_with(&[
            (0, 0, Biff8Value::Text("abc".to_owned())),
            (0, 1, Biff8Value::Number(1.0)),
            (0, 2, Biff8Value::Formula("A1+B1".to_owned())),
        ]);
        let cache = recalc_sheet(&sheet);
        assert_eq!(cache.get(&(0, 2)), Some(&Biff8Cached::Error(0x0f))); // #VALUE!
    }
}
