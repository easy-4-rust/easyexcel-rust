//! 对应 Java：`com.alibaba.excel.converters.date` 下的日期转换器与 `com.alibaba.excel.util.DateUtils`
//!
//! 日期/时间与 Excel 数字序列号、字符串之间的转换辅助函数，
//! 使用 Java 兼容的 `yyyy-MM-dd` / `yyyy-MM-dd HH:mm:ss` 默认格式。

use chrono::{NaiveDate, NaiveDateTime};

use crate::util::work_book_util::fill_data_format;
use crate::{
    CellValue, ExcelError, FromExcelCell, ReadConverterContext, WriteCellData,
    WriteConverterContext,
};

/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) const DEFAULT_DATE_FORMAT: &str = "yyyy-MM-dd";
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) const DEFAULT_DATETIME_FORMAT: &str = "yyyy-MM-dd HH:mm:ss";
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn read_date(context: &ReadConverterContext<'_>) -> Result<NaiveDate, ExcelError> {
    if let Some(CellValue::String(value)) = context.cell() {
        let patterns = context
            .convert_context()
            .effective_date_time_format()
            .map_or_else(|| vec!["%Y-%m-%d", "%Y/%m/%d"], |pattern| vec![pattern]);
        return patterns
            .into_iter()
            .find_map(|pattern| {
                let pattern = easyexcel_model::chrono_date_format(pattern);
                NaiveDate::parse_from_str(value, &pattern).ok()
            })
            .ok_or_else(|| {
                context
                    .convert_context()
                    .invalid(context.cell().expect("string cell exists"), "NaiveDate")
            });
    }
    NaiveDate::from_excel_cell(context.cell(), context.convert_context())
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn read_datetime(
    context: &ReadConverterContext<'_>,
) -> Result<NaiveDateTime, ExcelError> {
    if let Some(CellValue::String(value)) = context.cell() {
        let patterns = context
            .convert_context()
            .effective_date_time_format()
            .map_or_else(
                || {
                    vec![
                        "%Y%m%d%H%M%S",
                        "%Y-%m-%d %H:%M",
                        "%Y/%m/%d %H:%M",
                        "%Y%m%d %H:%M:%S",
                        "%Y-%m-%d %H:%M:%S",
                        "%Y/%m/%d %H:%M:%S",
                    ]
                },
                |pattern| vec![pattern],
            );
        return patterns
            .into_iter()
            .find_map(|pattern| {
                let pattern = easyexcel_model::chrono_date_format(pattern);
                NaiveDateTime::parse_from_str(value, &pattern).ok()
            })
            .ok_or_else(|| {
                context
                    .convert_context()
                    .invalid(context.cell().expect("string cell exists"), "NaiveDateTime")
            });
    }
    NaiveDateTime::from_excel_cell(context.cell(), context.convert_context())
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn write_date_value(
    value: NaiveDate,
    context: &WriteConverterContext<'_, NaiveDate>,
) -> WriteCellData {
    let mut cell = WriteCellData::new(CellValue::Date(value));
    fill_data_format(
        &mut cell,
        context.convert_context().effective_date_time_format(),
        DEFAULT_DATE_FORMAT,
    );
    cell
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn write_datetime_value<T>(
    value: NaiveDateTime,
    context: &WriteConverterContext<'_, T>,
) -> WriteCellData {
    let mut cell = WriteCellData::new(CellValue::DateTime(value));
    fill_data_format(
        &mut cell,
        context.convert_context().effective_date_time_format(),
        DEFAULT_DATETIME_FORMAT,
    );
    cell
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn write_date_string(
    value: NaiveDate,
    context: &WriteConverterContext<'_, NaiveDate>,
) -> WriteCellData {
    let pattern = context
        .convert_context()
        .effective_date_time_format()
        .unwrap_or("%Y-%m-%d");
    WriteCellData::from_string(
        value
            .format(&easyexcel_model::chrono_date_format(pattern))
            .to_string(),
    )
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn write_datetime_string<T>(
    value: NaiveDateTime,
    context: &WriteConverterContext<'_, T>,
) -> WriteCellData {
    let pattern = context
        .convert_context()
        .effective_date_time_format()
        .unwrap_or("%Y-%m-%d %H:%M:%S");
    WriteCellData::from_string(
        value
            .format(&easyexcel_model::chrono_date_format(pattern))
            .to_string(),
    )
}
/// 对应 Java：com.alibaba.excel.converters.date。
pub(crate) fn format_number_as_datetime_string(
    context: &ReadConverterContext<'_>,
    pattern: &str,
) -> Result<String, ExcelError> {
    let value = NaiveDateTime::from_excel_cell(context.cell(), context.convert_context())?;
    Ok(value
        .format(&easyexcel_model::chrono_date_format(pattern))
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::date::date_date_converter::DateDateConverter;
    use crate::converters::date::date_number_converter::DateNumberConverter;
    use crate::converters::date::date_string_converter::DateStringConverter;
    use crate::converters::localdate::local_date_date_converter::LocalDateDateConverter;
    use crate::converters::localdate::local_date_number_converter::LocalDateNumberConverter;
    use crate::converters::localdate::local_date_string_converter::LocalDateStringConverter;
    use crate::converters::localdatetime::local_date_time_date_converter::LocalDateTimeDateConverter;
    use crate::converters::localdatetime::local_date_time_number_converter::LocalDateTimeNumberConverter;
    use crate::converters::localdatetime::local_date_time_string_converter::LocalDateTimeStringConverter;
    use crate::{CellDataType, ConvertContext, Converter, ExcelColumn, JavaDate};

    const COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);

    fn context(format: Option<&'static str>, use_1904_windowing: bool) -> ConvertContext {
        ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format,
            date_time_format: None,
            number_format: None,
            use_1904_windowing,
        }
    }

    #[test]
    fn number_converters_are_real_bidirectional_java_equivalents() {
        let context_1900 = context(None, false);
        let serial = CellValue::Float(1.5);
        let read = ReadConverterContext::new(Some(&serial), &COLUMN, &context_1900);

        let date = LocalDateNumberConverter
            .convert_to_rust_data(&read)
            .expect("local date from number");
        assert_eq!(date, NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());

        let date_time = LocalDateTimeNumberConverter
            .convert_to_rust_data(&read)
            .expect("local datetime from number");
        assert_eq!(
            date_time,
            NaiveDate::from_ymd_opt(1900, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        );
        assert_eq!(
            DateNumberConverter
                .convert_to_rust_data(&read)
                .expect("java date equivalent")
                .naive_local(),
            date_time
        );
        let datetime_write = WriteConverterContext::new(&date_time, &COLUMN, &context_1900);
        let java_date = JavaDate::from(date_time);
        let java_date_write = WriteConverterContext::new(&java_date, &COLUMN, &context_1900);
        assert_eq!(
            DateNumberConverter
                .convert_to_excel_data(&java_date_write)
                .expect("java date to number")
                .value(),
            &CellValue::Float(1.5)
        );
        assert_eq!(
            LocalDateTimeNumberConverter
                .convert_to_excel_data(&datetime_write)
                .expect("local datetime to number")
                .value(),
            &CellValue::Float(1.5)
        );

        let date_write = WriteConverterContext::new(&date, &COLUMN, &context_1900);
        assert_eq!(
            LocalDateNumberConverter
                .convert_to_excel_data(&date_write)
                .unwrap()
                .value(),
            &CellValue::Float(1.0)
        );

        let context_1904 = context(None, true);
        let epoch_1904 = NaiveDate::from_ymd_opt(1904, 1, 1).unwrap();
        let write_1904 = WriteConverterContext::new(&epoch_1904, &COLUMN, &context_1904);
        assert_eq!(
            LocalDateNumberConverter
                .convert_to_excel_data(&write_1904)
                .unwrap()
                .value(),
            &CellValue::Float(0.0)
        );
        assert_eq!(
            <LocalDateNumberConverter as Converter<NaiveDate>>::support_excel_type(
                &LocalDateNumberConverter
            ),
            CellDataType::Number
        );
    }

    #[test]
    fn string_converters_honor_field_format_and_reject_invalid_input() {
        let date_context = context(Some("dd/MM/yyyy"), false);
        let date_cell = CellValue::String("31/12/2025".to_owned());
        let date_read = ReadConverterContext::new(Some(&date_cell), &COLUMN, &date_context);
        let date = LocalDateStringConverter
            .convert_to_rust_data(&date_read)
            .expect("formatted local date");
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());
        let date_write = WriteConverterContext::new(&date, &COLUMN, &date_context);
        assert_eq!(
            LocalDateStringConverter
                .convert_to_excel_data(&date_write)
                .unwrap()
                .value(),
            &CellValue::String("31/12/2025".to_owned())
        );

        let datetime_context = context(Some("yyyy-MM-dd HH:mm"), false);
        let datetime_cell = CellValue::String("2025-12-31 23:45".to_owned());
        let datetime_read =
            ReadConverterContext::new(Some(&datetime_cell), &COLUMN, &datetime_context);
        let datetime = LocalDateTimeStringConverter
            .convert_to_rust_data(&datetime_read)
            .expect("formatted local datetime");
        assert_eq!(
            DateStringConverter
                .convert_to_rust_data(&datetime_read)
                .expect("java date equivalent")
                .naive_local(),
            datetime
        );
        let datetime_write = WriteConverterContext::new(&datetime, &COLUMN, &datetime_context);
        let java_date = JavaDate::from(datetime);
        let java_date_write = WriteConverterContext::new(&java_date, &COLUMN, &datetime_context);
        assert_eq!(
            LocalDateTimeStringConverter
                .convert_to_excel_data(&datetime_write)
                .unwrap()
                .value(),
            &CellValue::String("2025-12-31 23:45".to_owned())
        );
        assert_eq!(
            DateStringConverter
                .convert_to_excel_data(&java_date_write)
                .unwrap()
                .value(),
            &CellValue::String("2025-12-31 23:45".to_owned())
        );

        let automatic_context = context(None, false);
        let automatic_cell = CellValue::String("2025/12/31 23:45:01".to_owned());
        let automatic_read =
            ReadConverterContext::new(Some(&automatic_cell), &COLUMN, &automatic_context);
        assert_eq!(
            LocalDateTimeStringConverter
                .convert_to_rust_data(&automatic_read)
                .expect("Java switchDateFormat equivalent"),
            NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 45, 1)
                .unwrap()
        );

        let invalid = CellValue::String("not-a-date".to_owned());
        let invalid_read = ReadConverterContext::new(Some(&invalid), &COLUMN, &date_context);
        assert!(
            LocalDateStringConverter
                .convert_to_rust_data(&invalid_read)
                .is_err()
        );
    }

    #[test]
    fn date_cell_converters_attach_java_default_data_formats() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let date_context = context(None, false);
        let date_write = WriteConverterContext::new(&date, &COLUMN, &date_context);
        let date_cell = LocalDateDateConverter
            .convert_to_excel_data(&date_write)
            .expect("date cell");
        assert_eq!(date_cell.value(), &CellValue::Date(date));
        assert_eq!(
            date_cell.data_format_data().and_then(|data| data.format()),
            Some(DEFAULT_DATE_FORMAT)
        );

        let datetime = date.and_hms_opt(3, 4, 5).unwrap();
        let datetime_write = WriteConverterContext::new(&datetime, &COLUMN, &date_context);
        let java_date = JavaDate::from(datetime);
        let java_date_write = WriteConverterContext::new(&java_date, &COLUMN, &date_context);
        let java_cell = DateDateConverter
            .convert_to_excel_data(&java_date_write)
            .expect("java date cell");
        let local_cell = LocalDateTimeDateConverter
            .convert_to_excel_data(&datetime_write)
            .expect("local datetime cell");
        for cell in [java_cell, local_cell] {
            assert_eq!(cell.value(), &CellValue::DateTime(datetime));
            assert_eq!(
                cell.data_format_data().and_then(|data| data.format()),
                Some(DEFAULT_DATETIME_FORMAT)
            );
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::{CellValue, ConvertContext, ExcelColumn, ReadConverterContext};

    const COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);

    fn context(format: Option<&'static str>) -> ConvertContext {
        ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn read_datetime_rejects_unparseable_strings() {
        // 对应 Java：`switchDateFormat` 全部失败时报转换错误
        let cell = CellValue::String("not-a-datetime".to_owned());
        let convert_context = context(None);
        let read = ReadConverterContext::new(Some(&cell), &COLUMN, &convert_context);
        assert!(read_datetime(&read).is_err());
        let bad_value = CellValue::String("2025-13-45 99:99".to_owned());
        let convert_context = context(Some("%Y-%m-%d %H:%M"));
        let read = ReadConverterContext::new(Some(&bad_value), &COLUMN, &convert_context);
        assert!(read_datetime(&read).is_err());
    }

    #[test]
    fn format_number_as_datetime_string_formats_and_rejects_negative_serials() {
        // 对应 Java：`StringNumberConverter` 内部日期格式分支
        let serial = CellValue::Float(1.5);
        let convert_context = context(None);
        let read = ReadConverterContext::new(Some(&serial), &COLUMN, &convert_context);
        assert_eq!(
            format_number_as_datetime_string(&read, "yyyy-MM-dd HH:mm:ss").unwrap(),
            "1900-01-01 12:00:00"
        );
        assert_eq!(
            format_number_as_datetime_string(&read, "yyyy/MM/dd HH:mm").unwrap(),
            "1900/01/01 12:00"
        );
        let negative = CellValue::Int(-1);
        let read = ReadConverterContext::new(Some(&negative), &COLUMN, &convert_context);
        assert!(format_number_as_datetime_string(&read, "yyyy-MM-dd HH:mm:ss").is_err());
    }

    #[test]
    // 日期序列号由整数天数精确转换而来，比较结果必然精确，精确断言正是测试意图
    #[allow(clippy::float_cmp)]
    fn date_to_excel_serial_after_1900_boundary_uses_1899_12_30_epoch() {
        // 对应 Java：1900-03-01 之后使用 1899-12-30 纪元（含虚拟闰日）
        let date = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        assert_eq!(easyexcel_model::date_to_excel_serial(date, false), 45659.0);
        assert_eq!(easyexcel_model::date_to_excel_serial(date, true), 44197.0);
    }
}
