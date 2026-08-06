#[test]
fn facade_applies_1904_windowing_to_numeric_date_converters() -> Result<()> {
    #[derive(Debug, PartialEq, ExcelRow)]
    struct NumericDates {
        #[excel(index = 0, use_1904_windowing = true)]
        date: NaiveDate,
        #[excel(index = 1, use_1904_windowing = false)]
        datetime: chrono::NaiveDateTime,
    }

    let directory = tempdir()?;
    let path = directory.path().join("numeric-date-1904.xlsx");
    let source = DynamicRow::new(BTreeMap::from([
        (0, DynamicValue::ActualData(CellValue::Int(0))),
        (1, DynamicValue::ActualData(CellValue::Float(1.5))),
    ]));
    EasyExcel::write::<DynamicRow>(&path).do_write([source])?;

    let rows = EasyExcel::read_sync::<NumericDates>(&path)
        .head_row_number(0)
        .use_1904_windowing(true)
        .do_read_sync()?;
    assert_eq!(
        rows,
        vec![NumericDates {
            date: NaiveDate::from_ymd_opt(1904, 1, 1).expect("date"),
            datetime: NaiveDate::from_ymd_opt(1900, 1, 1)
                .expect("date")
                .and_hms_opt(12, 0, 0)
                .expect("time"),
        }]
    );
    Ok(())
}

