//! Java annotation semantics exercised through Rust derive metadata and real XLSX I/O.

use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use easyexcel::{EasyExcel, ExcelRow, NumberRoundingMode, Result};
use tempfile::tempdir;
use zip::ZipArchive;

#[derive(Debug, PartialEq, ExcelRow)]
#[excel(ignore_unannotated)]
struct AnnotationModel {
    unannotated: String,
    #[excel(name = "Amount", number_format = "0.0", rounding_mode = "HALF_DOWN")]
    amount: BigDecimal,
    #[excel(
        name = "Date",
        date_time_format = "%Y-%m-%d",
        use_1904_windowing = true
    )]
    date: NaiveDate,
    #[excel(ignore, name = "Ignored")]
    ignored: String,
}

#[derive(Debug, PartialEq, ExcelRow)]
struct MultiLevelHeadModel {
    #[excel(value = ["订单", "编号"])]
    id: String,
    #[excel(value = ["订单", "金额"])]
    amount: i32,
}

#[derive(Debug, PartialEq, ExcelRow)]
struct IndependentFormatModel {
    #[excel(name = "Date", date_time_format = "%d/%m/%Y", number_format = "0.000")]
    date: NaiveDate,
    #[excel(
        name = "Amount",
        date_time_format = "%d/%m/%Y",
        number_format = "0.000"
    )]
    amount: BigDecimal,
}

#[test]
fn derive_applies_ignore_and_format_annotations_to_real_model_mapping() -> Result<()> {
    let schema = AnnotationModel::schema();
    assert_eq!(schema.len(), 2);
    assert_eq!(schema[0].field, "amount");
    assert_eq!(schema[0].name, "Amount");
    assert_eq!(schema[0].format, None);
    assert_eq!(schema[0].number_format, Some("0.0"));
    assert_eq!(
        schema[0].number_rounding_mode,
        Some(NumberRoundingMode::HalfDown)
    );
    assert_eq!(schema[1].field, "date");
    assert_eq!(schema[1].format, None);
    assert_eq!(schema[1].date_time_format, Some("%Y-%m-%d"));
    assert_eq!(schema[1].use_1904_windowing, Some(true));

    let directory = tempdir()?;
    let path = directory.path().join("annotation-mapping.xlsx");
    let expected = AnnotationModel {
        unannotated: String::new(),
        amount: BigDecimal::from_str("12.5")
            .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?,
        date: NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date"),
        ignored: String::new(),
    };
    EasyExcel::write::<AnnotationModel>(&path).do_write([expected])?;

    let rows = EasyExcel::read_sync::<AnnotationModel>(&path).do_read_sync()?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].unannotated, "");
    assert_eq!(rows[0].ignored, "");
    assert_eq!(
        rows[0].amount,
        BigDecimal::from(125_i32) / BigDecimal::from(10_i32)
    );
    assert_eq!(
        rows[0].date,
        NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date")
    );
    Ok(())
}

#[test]
fn date_and_number_formats_do_not_overwrite_each_other() -> Result<()> {
    let expected = IndependentFormatModel {
        date: NaiveDate::from_ymd_opt(2026, 7, 24).expect("valid date"),
        amount: BigDecimal::from_str("12.500")
            .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?,
    };
    let converters =
        easyexcel::converters::default_converter_loader::load_default_write_converter();
    let (_, cells) = expected.to_excel_write_row(&converters)?;
    assert_eq!(
        cells[0]
            .data_format_data()
            .and_then(|format| format.format()),
        Some("%d/%m/%Y")
    );
    assert_eq!(
        cells[1]
            .data_format_data()
            .and_then(|format| format.format()),
        Some("0.000")
    );

    let source = easyexcel::RowData::new(
        "Sheet1",
        1,
        vec![
            easyexcel::CellValue::String("24/07/2026".to_owned()),
            easyexcel::CellValue::String("12.500".to_owned()),
        ],
        std::sync::Arc::new(std::collections::HashMap::from([
            ("Date".to_owned(), 0),
            ("Amount".to_owned(), 1),
        ])),
    );
    assert_eq!(IndependentFormatModel::from_row(&source)?, expected);
    Ok(())
}

#[test]
fn excel_property_value_writes_merges_and_reads_multi_level_head() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("multi-level-head.xlsx");
    EasyExcel::write::<MultiLevelHeadModel>(&path).do_write([MultiLevelHeadModel {
        id: "A-1".to_owned(),
        amount: 42,
    }])?;

    let rows = EasyExcel::read_sync::<MultiLevelHeadModel>(&path)
        .head_row_number(2)
        .do_read_sync()?;
    assert_eq!(
        rows,
        vec![MultiLevelHeadModel {
            id: "A-1".to_owned(),
            amount: 42,
        }]
    );

    let file = File::open(&path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?;
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?
        .read_to_string(&mut sheet)?;
    assert!(sheet.contains("mergeCell ref=\"A1:B1\""));
    assert!(
        sheet.contains("r=\"A3\""),
        "data must start after two head rows"
    );
    Ok(())
}
