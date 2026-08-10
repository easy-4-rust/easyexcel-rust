#[test]
fn csv_writer_to_owned_stream_validates_options() {
    assert!(
        write_csv_to_writer::<EveryCell, _, _>(
            Path::new("stream.csv"),
            Cursor::new(Vec::new()),
            &WriteOptions::default(),
            [every_cell()],
            &mut [],
        )
        .is_ok()
    );
    assert!(
        write_csv_to_writer::<EveryCell, _, _>(
            Path::new("stream.csv"),
            Cursor::new(Vec::new()),
            &WriteOptions {
                charset: CsvCharset::new("not-a-charset"),
                ..WriteOptions::default()
            },
            [every_cell()],
            &mut [],
        )
        .is_err()
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn csv_writer_propagates_io_faults_and_column_overflow() {
    let write_errors = (0..64)
        .filter(|fail_at| {
            write_csv_to::<EveryCell, _>(
                Path::new("fault.csv"),
                Box::new(FaultyWrite::writing(*fail_at)),
                &WriteOptions::default(),
                [every_cell()],
                &mut [],
            )
            .is_err()
        })
        .count();
    assert!(write_errors > 0);
    assert!(
        write_csv_to::<EveryCell, _>(
            Path::new("fault.csv"),
            Box::new(FaultyWrite::flushing()),
            &WriteOptions::default(),
            [every_cell()],
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_to::<EveryCell, _>(
            Path::new("finish-fault.csv"),
            Box::new(FailThirdFlush::default()),
            &WriteOptions::default(),
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_to::<EveryCell, _>(
            Path::new("into-inner-fault.csv"),
            Box::new(FailSecondFlush::default()),
            &WriteOptions::default(),
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    assert!(
        write_csv_to::<EveryCell, _>(
            Path::new("charset-fault.csv"),
            Box::new(Vec::<u8>::new()),
            &WriteOptions {
                charset: CsvCharset::new("not-a-charset"),
                ..WriteOptions::default()
            },
            Vec::new(),
            &mut []
        )
        .is_err()
    );
    for (options, rows) in [
        (WriteOptions::default(), Vec::<SparseRow>::new()),
        (
            WriteOptions {
                dynamic_head: Some(vec![vec!["Dynamic".to_owned()]]),
                ..WriteOptions::default()
            },
            Vec::<SparseRow>::new(),
        ),
        (
            WriteOptions {
                need_head: false,
                ..WriteOptions::default()
            },
            vec![SparseRow],
        ),
    ] {
        assert!(
            write_csv_to::<SparseRow, _>(
                Path::new("record-fault.csv"),
                Box::new(FaultyWrite::writing(1)),
                &options,
                rows,
                &mut [],
            )
            .is_err()
        );
    }

    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    let wide_result = write_csv_to::<EveryCell, _>(
        Path::new("wide.csv"),
        Box::new(Vec::<u8>::new()),
        &WriteOptions::default(),
        [every_cell()],
        &mut [],
    );
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));
    assert!(wide_result.is_err());
    USE_WIDE_SCHEMA.with(|wide| wide.set(true));
    let wide_data_result = write_csv_to::<EveryCell, _>(
        Path::new("wide-data.csv"),
        Box::new(Vec::<u8>::new()),
        &WriteOptions {
            need_head: false,
            ..WriteOptions::default()
        },
        [every_cell()],
        &mut [],
    );
    USE_WIDE_SCHEMA.with(|wide| wide.set(false));
    assert!(wide_data_result.is_err());
    assert!(csv_record(&[]).is_empty());
}

/// Registered style strategies rewrite the workbook without annotation styles.
///
/// 对应 Java：`StyleDataTest.readAndWrite` handler-only path for
/// `SimpleColumnWidthStyleStrategy` / `SimpleRowHeightStyleStrategy` /
/// `HorizontalCellStyleStrategy`.
#[test]
fn registered_style_strategies_rewrite_cell_style_width_and_height() -> Result<()> {
    #[derive(Debug, Clone)]
    struct PlainRow {
        name: String,
        value: String,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("name", "name", Some(0), 0, None),
                ExcelColumn::new("value", "value", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self {
                name: String::new(),
                value: String::new(),
            })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String(self.name.clone()),
                CellValue::String(self.value.clone()),
            ])
        }
    }

    let directory = tempdir()?;
    let path = directory.path().join("strategy-only.xlsx");
    let mut head = ExcelCellStyle::new();
    head.fill_pattern = Some(ExcelFillPattern::Solid);
    head.fill_foreground_color = Some(ExcelColor::Rgb(0x00FF_FF00));
    let mut content = ExcelCellStyle::new();
    content.fill_pattern = Some(ExcelFillPattern::Solid);
    content.fill_foreground_color = Some(ExcelColor::Rgb(0x0000_8080));

    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![
        Box::new(SimpleColumnWidthStyleStrategy::uniform(40)),
        Box::new(SimpleRowHeightStyleStrategy::new(Some(30), Some(45))),
        Box::new(HorizontalCellStyleStrategy::with_head_and_content(
            head.into(), content.into(),
        )),
        Box::new(LongestMatchColumnWidthStyleStrategy::new()),
    ];
    write_xlsx_with_handlers::<PlainRow, _>(
        &path,
        &WriteOptions {
            // Neutralise default bold-only head CellStyle so fills come from strategy.
            head_style: CellStyle::new(),
            ..WriteOptions::default()
        },
        vec![PlainRow {
            name: "a".to_owned(),
            value: "bbbbbbbbbb".to_owned(),
        }],
        &mut handlers,
    )?;

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(test_error)?
        .read_to_string(&mut sheet)
        .map_err(test_error)?;
    assert!(
        sheet.contains("customHeight=\"1\"") || sheet.contains("ht=\""),
        "expected row height from SimpleRowHeightStyleStrategy"
    );
    assert!(
        sheet.contains("customWidth=\"1\"") || sheet.contains("width=\""),
        "expected column width from strategies"
    );

    let mut styles = String::new();
    archive
        .by_name("xl/styles.xml")
        .map_err(test_error)?
        .read_to_string(&mut styles)
        .map_err(test_error)?;
    assert!(
        styles.contains("rgb=\"FFFFFF00\"") || styles.contains("FFFF00"),
        "expected yellow head fill from HorizontalCellStyleStrategy"
    );
    assert!(
        styles.contains("rgb=\"FF008080\"")
            || styles.contains("rgb=\"00008080\"")
            || styles.contains("008080"),
        "expected teal content fill from HorizontalCellStyleStrategy"
    );
    Ok(())
}

/// `VerticalCellStyleStrategy` applies per-column fills without field annotations.
#[test]
fn vertical_cell_style_strategy_rewrites_per_column_fills() -> Result<()> {
    #[derive(Debug, Clone)]
    struct PlainRow {
        left: String,
        right: String,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("left", "left", Some(0), 0, None),
                ExcelColumn::new("right", "right", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self {
                left: String::new(),
                right: String::new(),
            })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String(self.left.clone()),
                CellValue::String(self.right.clone()),
            ])
        }
    }

    let directory = tempdir()?;
    let path = directory.path().join("vertical-strategy.xlsx");
    let strategy = VerticalCellStyleStrategy::new(
        |column| {
            let mut style = ExcelCellStyle::new();
            style.fill_pattern = Some(ExcelFillPattern::Solid);
            style.fill_foreground_color = Some(if column == 0 {
                ExcelColor::Indexed(13)
            } else {
                ExcelColor::Indexed(12)
            });
            style
        },
        |column| {
            let mut style = ExcelCellStyle::new();
            style.fill_pattern = Some(ExcelFillPattern::Solid);
            style.fill_foreground_color = Some(if column == 0 {
                ExcelColor::Indexed(58)
            } else {
                ExcelColor::Indexed(14)
            });
            style
        },
    );
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(strategy)];
    write_xlsx_with_handlers::<PlainRow, _>(
        &path,
        &WriteOptions {
            head_style: CellStyle::new(),
            ..WriteOptions::default()
        },
        vec![PlainRow {
            left: "L".to_owned(),
            right: "R".to_owned(),
        }],
        &mut handlers,
    )?;

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut styles = String::new();
    archive
        .by_name("xl/styles.xml")
        .map_err(test_error)?
        .read_to_string(&mut styles)
        .map_err(test_error)?;
    assert!(styles.contains("rgb=\"FFFFFF00\""));
    assert!(styles.contains("rgb=\"FF0000FF\""));
    assert!(styles.contains("rgb=\"FF003300\""));
    assert!(styles.contains("rgb=\"FFFF00FF\""));
    Ok(())
}

