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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 FORMULA 记录缓存结果（8 字节结果字段，字符串另走 STRING 记录）。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 对每个工作表求值公式，返回 `(row, col) → 缓存值` 的映射表。
/// 与 `write_worksheet` 的 sheet 顺序一一对应。
pub(crate) fn recalc_cached_values(sheets: &[Biff8Sheet]) -> Vec<HashMap<(u16, u8), Biff8Cached>> {
    let mut workbook = Workbook::empty();
    for sheet in sheets {
        let mut worksheet = easyexcel_model::model::Sheet::new(sheet.name.clone());
        for (&(row, col), cell) in &sheet.cells {
            let xcell = match &cell.value {
                Biff8Value::Text(text) => Cell::Text(text.clone()),
                Biff8Value::RichText(rich) => Cell::Text(rich.text.clone()),
                Biff8Value::Number(number) => Cell::Number(*number),
                Biff8Value::Bool(flag) => Cell::Bool(*flag),
                Biff8Value::Formula(expr) => Cell::Formula {
                    expr: expr.clone(),
                    cached: CellValue::Empty,
                },
                Biff8Value::Blank => continue,
            };
            worksheet.set(u32::from(row), u32::from(col), xcell);
        }
        workbook.sheets.push(worksheet);
    }
    Engine::new().recalc(&mut workbook);

    sheets
        .iter()
        .enumerate()
        .map(|(sheet_index, sheet)| {
            let mut cache = HashMap::new();
            let worksheet = &workbook.sheets[sheet_index];
            for (&(row, col), cell) in &sheet.cells {
                if matches!(cell.value, Biff8Value::Formula(_))
                    && let Some(Cell::Formula { cached, .. }) =
                        worksheet.get(u32::from(row), u32::from(col))
                    && let Some(value) = cached_formula_value(cached, &cell.value, &workbook)
                {
                    cache.insert((row, col), value);
                }
            }
            cache
        })
        .collect()
}

fn cached_formula_value(
    cached: &CellValue,
    source: &Biff8Value,
    workbook: &Workbook,
) -> Option<Biff8Cached> {
    let mapped = to_biff8_cached(cached);
    if !matches!(mapped, Some(Biff8Cached::Error(0x17))) {
        return mapped;
    }
    let Biff8Value::Formula(formula) = source else {
        return mapped;
    };
    evaluate_3d_aggregate(formula, workbook).or(mapped)
}

fn evaluate_3d_aggregate(formula: &str, workbook: &Workbook) -> Option<Biff8Cached> {
    let formula = formula.strip_prefix('=').unwrap_or(formula);
    let open = formula.find('(')?;
    let function = formula[..open].trim().to_ascii_uppercase();
    if !matches!(
        function.as_str(),
        "SUM" | "AVERAGE" | "MIN" | "MAX" | "COUNT"
    ) {
        return None;
    }
    let reference = formula[open + 1..].strip_suffix(')')?.trim();
    let bang = reference.rfind('!')?;
    let sheet_spec = reference[..bang]
        .trim()
        .trim_matches('\'')
        .replace("''", "'");
    let (first_sheet, last_sheet) = sheet_spec.split_once(':')?;
    let first_index = workbook
        .sheets
        .iter()
        .position(|sheet| sheet.name.eq_ignore_ascii_case(first_sheet))?;
    let last_index = workbook
        .sheets
        .iter()
        .position(|sheet| sheet.name.eq_ignore_ascii_case(last_sheet))?;
    if first_index > last_index {
        return None;
    }
    let cell_spec = &reference[bang + 1..];
    let (start, end) = cell_spec
        .split_once(':')
        .map_or((cell_spec, cell_spec), |(start, end)| (start, end));
    let (start_row, start_col) = parse_a1(start)?;
    let (end_row, end_col) = parse_a1(end)?;
    let mut numbers = Vec::new();
    for sheet in &workbook.sheets[first_index..=last_index] {
        for row in start_row.min(end_row)..=start_row.max(end_row) {
            for col in start_col.min(end_col)..=start_col.max(end_col) {
                if let CellValue::Number(number) = sheet.value(row, col) {
                    numbers.push(number);
                }
            }
        }
    }
    let value = match function.as_str() {
        "SUM" => numbers.iter().sum(),
        "AVERAGE" => numbers.iter().sum::<f64>() / numbers.len() as f64,
        "MIN" => numbers.iter().copied().reduce(f64::min)?,
        "MAX" => numbers.iter().copied().reduce(f64::max)?,
        "COUNT" => numbers.len() as f64,
        _ => return None,
    };
    value.is_finite().then_some(Biff8Cached::Number(value))
}

fn parse_a1(reference: &str) -> Option<(u32, u32)> {
    let reference = reference.replace('$', "");
    let split = reference.find(|character: char| character.is_ascii_digit())?;
    let (column, row) = reference.split_at(split);
    let row = row.parse::<u32>().ok()?.checked_sub(1)?;
    let mut column_index = 0u32;
    for character in column.bytes() {
        if !character.is_ascii_alphabetic() {
            return None;
        }
        column_index = column_index
            .checked_mul(26)?
            .checked_add(u32::from(character.to_ascii_uppercase() - b'A') + 1)?;
    }
    Some((row, column_index.checked_sub(1)?))
}

#[cfg(test)]
fn recalc_sheet(sheet: &Biff8Sheet) -> HashMap<(u16, u8), Biff8Cached> {
    recalc_cached_values(std::slice::from_ref(sheet))
        .into_iter()
        .next()
        .unwrap_or_default()
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

    #[test]
    fn cross_sheet_references_are_evaluated_in_one_workbook() {
        let source = sheet_with(&[(0, 0, Biff8Value::Number(7.0))]);
        let mut target = Biff8Sheet::new("销售 数据");
        target
            .set(
                0,
                0,
                Biff8Cell::general(Biff8Value::Formula("Sheet1!A1*2".to_owned())),
            )
            .unwrap();
        let caches = recalc_cached_values(&[source, target]);
        assert_eq!(caches[1].get(&(0, 0)), Some(&Biff8Cached::Number(14.0)));
    }

    #[test]
    fn three_dimensional_sheet_range_aggregate_has_numeric_cache() {
        let first = sheet_with(&[(0, 0, Biff8Value::Number(7.0))]);
        let mut second = Biff8Sheet::new("销售 数据");
        second
            .set(0, 0, Biff8Cell::general(Biff8Value::Number(14.0)))
            .unwrap();
        let mut result = Biff8Sheet::new("结果");
        result
            .set(
                0,
                0,
                Biff8Cell::general(Biff8Value::Formula("SUM('Sheet1:销售 数据'!A1)".to_owned())),
            )
            .unwrap();
        let caches = recalc_cached_values(&[first, second, result]);
        assert_eq!(caches[2].get(&(0, 0)), Some(&Biff8Cached::Number(21.0)));
    }
}
