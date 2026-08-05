//! 基础引擎单元格到 EasyExcel 门面单元格的适配。

use crate::core::{CellValue, FormulaData};
#[cfg(test)]
use calamine::{Data, DataRef, ExcelDateTime, ExcelDateTimeType};

#[cfg(test)]
pub(crate) fn excel_datetime_cell(value: &ExcelDateTime, use_1904_windowing: bool) -> CellValue {
    if !value.is_datetime() {
        return CellValue::Float(value.as_f64());
    }
    ExcelDateTime::new(
        value.as_f64(),
        ExcelDateTimeType::DateTime,
        use_1904_windowing,
    )
    .as_datetime()
    .map_or(CellValue::Float(value.as_f64()), CellValue::DateTime)
}

/// 将 Excel 日期序列转换为门面日期时间值，无法表示时保留原始数字。
pub(crate) fn excel_serial_datetime_cell(value: f64, use_1904_windowing: bool) -> CellValue {
    let system = easyexcel_model::DateSystem::from_1904_windowing(use_1904_windowing);
    system
        .serial_to_datetime(value)
        .map_or(CellValue::Float(value), CellValue::DateTime)
}

/// 将中立模型单元格转换为门面值及公式元数据。
pub(crate) fn from_model_cell(
    cell: &easyexcel_model::Cell,
) -> (CellValue, Option<FormulaData>) {
    use easyexcel_model::{Cell, value::CellValue as ModelCellValue};

    let value = match cell.value() {
        ModelCellValue::Empty => CellValue::Empty,
        ModelCellValue::Number(value) => CellValue::Float(value),
        ModelCellValue::Text(value) => CellValue::String(value),
        ModelCellValue::Bool(value) => CellValue::Bool(value),
        ModelCellValue::Error(value) => CellValue::Error(value.to_string()),
    };
    let formula = match cell {
        Cell::Formula { expr, .. } => Some(FormulaData::new(expr.clone())),
        _ => None,
    };
    (value, formula)
}

#[cfg(test)]
pub(crate) fn from_data(value: &Data, use_1904_windowing: bool) -> CellValue {
    match value {
        Data::Empty => CellValue::Empty,
        Data::String(value) | Data::DateTimeIso(value) | Data::DurationIso(value) => {
            CellValue::String(value.clone())
        }
        Data::Bool(value) => CellValue::Bool(*value),
        Data::Int(value) => CellValue::Int(*value),
        Data::Float(value) => CellValue::Float(*value),
        Data::DateTime(value) => excel_datetime_cell(value, use_1904_windowing),
        Data::Error(value) => CellValue::Error(value.to_string()),
    }
}

#[cfg(test)]
pub(crate) fn from_calamine(value: &DataRef<'_>, use_1904_windowing: bool) -> CellValue {
    match value {
        DataRef::Empty => CellValue::Empty,
        DataRef::String(value) | DataRef::DateTimeIso(value) | DataRef::DurationIso(value) => {
            CellValue::String(value.clone())
        }
        DataRef::SharedString(value) => CellValue::String((*value).to_owned()),
        DataRef::Bool(value) => CellValue::Bool(*value),
        DataRef::Int(value) => CellValue::Int(*value),
        DataRef::Float(value) => CellValue::Float(*value),
        DataRef::DateTime(value) => excel_datetime_cell(value, use_1904_windowing),
        DataRef::Error(value) => CellValue::Error(value.to_string()),
    }
}
