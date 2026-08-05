//! Ensures callers never need to name `easyexcel_derive` directly.

use std::collections::HashMap;
use std::sync::Arc;

use easyexcel::{CellValue, ExcelRow, RowData};

#[derive(Debug, PartialEq, ExcelRow)]
struct FacadeRow {
    #[excel(value = ["用户", "姓名"])]
    name: String,
    #[excel(property, order = 1)]
    age: i32,
}

struct NoDefault(&'static str);

#[derive(ExcelRow)]
#[excel(once_absolute_merge())]
struct IndependentFormatsRow {
    #[excel(
        property,
        order = -10,
        date_time_format = "%Y/%m/%d",
        number_format = "0.000"
    )]
    value: String,
    #[excel(ignore, default = NoDefault("derived"))]
    ignored: NoDefault,
}

#[test]
fn derives_read_and_write_through_easyexcel() -> easyexcel::Result<()> {
    let row = FacadeRow {
        name: "Ada".to_owned(),
        age: 36,
    };
    assert_eq!(
        row.to_row()?,
        vec![CellValue::String("Ada".to_owned()), CellValue::Int(36)]
    );

    let headers = Arc::new(HashMap::from([
        ("姓名".to_owned(), 0),
        ("age".to_owned(), 1),
    ]));
    let source = RowData::new(
        "Sheet1",
        1,
        vec![CellValue::String("Ada".to_owned()), CellValue::Int(36)],
        headers,
    );
    assert_eq!(FacadeRow::from_row(&source)?, row);
    Ok(())
}

#[test]
fn keeps_java_formats_and_signed_defaults_independent() {
    let column = &IndependentFormatsRow::schema()[0];
    assert_eq!(column.order, -10);
    assert_eq!(column.date_time_format, Some("%Y/%m/%d"));
    assert_eq!(column.number_format, Some("0.000"));

    let metadata = IndependentFormatsRow::write_metadata();
    let merge = metadata.once_absolute_merge.expect("merge metadata");
    assert_eq!(merge.first_row_index, -1);
    assert_eq!(merge.last_row_index, -1);
    assert_eq!(merge.first_column_index, -1);
    assert_eq!(merge.last_column_index, -1);

    let ignored = IndependentFormatsRow {
        value: String::new(),
        ignored: NoDefault("manual"),
    };
    assert_eq!(ignored.ignored.0, "manual");
}
