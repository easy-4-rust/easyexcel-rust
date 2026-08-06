#[test]
fn csv_uses_java_target_string_converter_before_cell_handlers() -> Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("target-string.csv");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(ConvertedTypeProbe(Arc::clone(&observed)))];
    write_csv_with_handlers::<NumericConverterContextRow, _>(
        &output,
        &WriteOptions::default(),
        [NumericConverterContextRow(42)],
        &mut handlers,
    )?;
    assert_eq!(
        *observed
            .lock()
            .map_err(|_| test_error("converted type probe poisoned"))?,
        [crate::core::CellDataType::String]
    );
    assert!(std::fs::read_to_string(output)?.contains("42"));
    Ok(())
}

#[test]
// 语义敏感：xlsx/xls 双后端并行断言（对应 Java 双引擎测试），命名刻意
// 只差一个字母以对照两端结果，故豁免 similar_names。
#[allow(clippy::similar_names)]
fn stateless_xlsx_and_xls_materialize_default_write_converters() -> Result<()> {
    let directory = tempdir()?;
    let xlsx = directory.path().join("default-converters.xlsx");
    let xls = directory.path().join("default-converters.xls");
    let options = WriteOptions {
        need_head: false,
        ..WriteOptions::default()
    };

    write_xlsx::<DefaultRegistryRequiredRow, _>(&xlsx, &options, [DefaultRegistryRequiredRow])?;
    write_xls::<DefaultRegistryRequiredRow, _>(&xls, &options, [DefaultRegistryRequiredRow])?;

    let mut xlsx_book: Xlsx<_> = open_workbook(&xlsx).map_err(test_error)?;
    let xlsx_range = xlsx_book.worksheet_range("Sheet1").map_err(test_error)?;
    assert!(matches!(
        xlsx_range.get((0, 0)),
        Some(Data::Int(7) | Data::Float(7.0))
    ));

    let mut xls_book: Xls<_> = open_workbook(&xls).map_err(test_error)?;
    let xls_range = xls_book.worksheet_range("Sheet1").map_err(test_error)?;
    assert!(matches!(
        xls_range.get((0, 0)),
        Some(Data::Int(7) | Data::Float(7.0))
    ));
    Ok(())
}

#[test]
fn annotation_config_loads_real_ordered_handlers_for_every_java_strategy() -> Result<()> {
    let options = WriteOptions {
        sheet_name: "Data".to_owned(),
        ..WriteOptions::default()
    };
    let mut handlers = load_annotation_handlers::<AnnotationHandlerRow>(&options)?;
    assert_eq!(handlers.len(), 5);
    sort_handlers(&mut handlers);
    assert_eq!(
        handlers
            .iter()
            .map(|handler| handler.order())
            .collect::<Vec<_>>(),
        vec![-60_000, -60_000, -50_000, -50_000, 0]
    );
    assert!(handlers.iter().any(|handler| {
        handler.style_once_absolute_merge() == Some(OnceAbsoluteMergeProperty::new(10, 10, 0, 1))
    }));
    assert!(handlers.iter().any(|handler| {
        handler.style_loop_merge() == Some((crate::core::LoopMergeProperty::new(2, 1), 0))
    }));
    assert!(
        handlers
            .iter()
            .any(|handler| handler.style_column_width(0) == Some(18))
    );
    assert!(
        handlers
            .iter()
            .any(|handler| handler.style_head_row_height() == Some(31))
    );
    assert!(
        handlers
            .iter()
            .any(|handler| handler.style_content_row_height() == Some(24))
    );
    Ok(())
}

#[test]
// 语义敏感：断言 XML 解析出的行高/列宽必须精确等于写入值（浮点往返
// 无损），严格比较即测试意图，故豁免 float_cmp。
#[allow(clippy::float_cmp)]
fn stateful_sheet_persists_annotation_handlers_and_deduplicates_merges() -> Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("annotation-handler-scope.xlsx");
    let sheet = WriteSheet::<AnnotationHandlerRow>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        ..WriteOptions::default()
    });
    let mut writer = ExcelWriter::new(&output);
    writer.write(
        vec![
            AnnotationHandlerRow("first"),
            AnnotationHandlerRow("second"),
        ],
        &sheet,
    )?;

    let annotation_handlers = writer
        .sheet_annotation_handlers
        .get("Data")
        .expect("sheet annotation handlers");
    assert_eq!(annotation_handlers.len(), 5);
    assert_eq!(
        writer
            .sheet_handler_scope("Data")
            .own
            .iter()
            .map(SharedWriteHandler::order)
            .collect::<Vec<_>>(),
        vec![-60_000, -60_000, -50_000, -50_000, 0]
    );
    writer.finish()?;

    let sheet_xml = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!(sheet_xml.contains("ref=\"A2:A3\""), "{sheet_xml}");
    assert!(sheet_xml.contains("ref=\"A11:B11\""), "{sheet_xml}");
    assert_eq!(sheet_xml.matches("ref=\"A2:A3\"").count(), 1);
    assert_eq!(sheet_xml.matches("ref=\"A11:B11\"").count(), 1);
    assert!((sheet_row_height(&sheet_xml, 1)? - 31.0).abs() <= 0.25);
    assert_eq!(sheet_row_height(&sheet_xml, 2)?, 24.0);
    assert_eq!(sheet_row_height(&sheet_xml, 3)?, 24.0);
    Ok(())
}

#[test]
fn parent_custom_dimension_handler_overrides_annotation_defaults() -> Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("annotation-handler-override.xlsx");
    let sheet = WriteSheet::<AnnotationHandlerRow>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        ..WriteOptions::default()
    });
    let mut writer = ExcelWriter::new(&output);
    writer.register_write_handler(Box::new(OverrideAnnotationDimensions))?;
    writer.write(vec![AnnotationHandlerRow("first")], &sheet)?;

    let effective = writer.sheet_handler_scope("Data").effective;
    let orders = effective
        .iter()
        .map(SharedWriteHandler::order)
        .collect::<Vec<_>>();
    assert_eq!(&orders[..5], &[-60_000, -60_000, -50_000, -50_000, -50_000]);
    assert!(orders[5..].iter().all(|order| *order == 0), "{orders:?}");
    writer.finish()?;

    let sheet_xml = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!(
        sheet_xml.contains("width=\"27\"") || sheet_xml.contains("width=\"27.0\""),
        "{sheet_xml}"
    );
    assert!((sheet_row_height(&sheet_xml, 1)? - 40.0).abs() <= 0.25);
    assert!((sheet_row_height(&sheet_xml, 2)? - 36.0).abs() <= 0.25);
    Ok(())
}

#[test]
fn live_holder_converter_map_matches_sheet_and_table_write_precedence() -> Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("holder-converter-map.xlsx");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut workbook_options = WriteOptions::default();
    workbook_options
        .converters
        .register::<i32, _>(ContextI32Converter("workbook"));
    let mut writer = ExcelWriter::with_handlers_and_options(
        &output,
        vec![Box::new(ConverterMapProbe(Arc::clone(&observed)))],
        workbook_options,
    );
    let sheet = WriteSheet::<ConverterContextRow>::new("Data")
        .register_converter::<String, _>(ContextStringConverter("sheet"));
    writer.write(vec![ConverterContextRow("sheet-row".to_owned())], &sheet)?;

    let mut table = crate::write::metadata::WriteTable::with_table_no(2);
    table
        .options
        .converters
        .register::<String, _>(ContextStringConverter("table"));
    writer.write_with_table(
        vec![ConverterContextRow("table-row".to_owned())],
        &sheet,
        &table,
    )?;
    writer.finish()?;

    assert_eq!(
        observed
            .lock()
            .map_err(|_| ExcelError::Format("converter map probe poisoned".to_owned()))?
            .as_slice(),
        [
            ("sheet:probe".to_owned(), "workbook:7".to_owned()),
            ("table:probe".to_owned(), "workbook:7".to_owned()),
        ]
    );

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Data").map_err(test_error)?;
    let values = range
        .cells()
        .filter_map(|cell| cell.2.get_string())
        .collect::<Vec<_>>();
    assert!(values.contains(&"sheet:sheet-row"), "{values:?}");
    assert!(values.contains(&"table:table-row"), "{values:?}");
    Ok(())
}

#[test]
fn sheet_then_table_deduplicates_the_same_absolute_merge() -> Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("sheet-table-absolute-merge.xlsx");
    let sheet = WriteSheet::<AnnotationHandlerRow>::new("Data");
    let mut writer = ExcelWriter::new(&output);
    writer.write(vec![AnnotationHandlerRow("sheet-row")], &sheet)?;
    writer.write_with_table(
        vec![AnnotationHandlerRow("table-row")],
        &sheet,
        &crate::write::metadata::WriteTable::with_table_no(2),
    )?;
    writer.finish()?;

    let sheet_xml = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert_eq!(sheet_xml.matches("ref=\"A11:B11\"").count(), 1);
    Ok(())
}

