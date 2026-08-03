//! Calamine `Data` / `DataRef` → [`crate::core::CellValue`] conversion helpers.

use crate::core::CellValue;
#[cfg(test)]
use calamine::DataRef;
use calamine::{Data, ExcelDateTime, ExcelDateTimeType};

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
