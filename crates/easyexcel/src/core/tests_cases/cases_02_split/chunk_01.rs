#[test]
fn row_data_convert_context() {
    let headers = Arc::new(HashMap::new());
    let row = RowData::new("Users", 5, vec![], headers);
    let col = ExcelColumn::new("email", "Email", Some(2), 0, Some("%Y-%m-%d"));
    let ctx = row.convert_context(&col);
    assert_eq!(ctx.sheet_name, "Users");
    assert_eq!(ctx.row_index, 5);
    assert_eq!(ctx.column_index, Some(2));
    assert_eq!(ctx.field, "email");
    assert_eq!(ctx.format, Some("%Y-%m-%d"));
}

#[test]
fn read_listener_invoke_works() {
    let mut listener = TestListener::new();
    let ctx = AnalysisContext::new("S1", 0, 0);
    listener.invoke("row1".to_owned(), &ctx).unwrap();
    assert_eq!(listener.rows, vec!["row1"]);
}

#[test]
fn read_listener_can_stop() {
    struct StopAfterOne {
        count: usize,
    }
    impl ReadListener<String> for StopAfterOne {
        fn invoke(&mut self, _data: String, _context: &AnalysisContext) -> Result<()> {
            self.count += 1;
            if self.count >= 2 {
                return Err(ExcelError::Format("stop".to_owned()));
            }
            Ok(())
        }
        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }
    let mut listener = StopAfterOne { count: 0 };
    let ctx = AnalysisContext::new("S", 0, 0);
    listener.invoke("a".to_owned(), &ctx).unwrap();
    let err = listener.invoke("b".to_owned(), &ctx);
    assert!(err.is_err());
}

#[test]
fn read_listener_has_next_can_stop() {
    struct StopAfterTwo {
        count: usize,
    }
    impl ReadListener<String> for StopAfterTwo {
        fn invoke(&mut self, _data: String, _context: &AnalysisContext) -> Result<()> {
            self.count += 1;
            Ok(())
        }
        fn has_next(&mut self, _context: &AnalysisContext) -> bool {
            self.count < 2
        }
        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }
    let mut listener = StopAfterTwo { count: 0 };
    let ctx = AnalysisContext::new("S", 0, 0);
    assert!(listener.has_next(&ctx));
    listener.invoke("a".to_owned(), &ctx).unwrap();
    assert!(listener.has_next(&ctx));
    listener.invoke("b".to_owned(), &ctx).unwrap();
    assert!(!listener.has_next(&ctx));
}

#[test]
fn page_read_listener_batches() {
    let batch_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let bc = batch_count.clone();
    let mut listener = PageReadListener::new(2, move |data, _ctx| {
        bc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(data.len() <= 2);
        Ok(())
    });

    let ctx = AnalysisContext::new("S", 0, 0);
    listener.invoke("a".to_owned(), &ctx).unwrap();
    assert_eq!(batch_count.load(std::sync::atomic::Ordering::Relaxed), 0);
    listener.invoke("b".to_owned(), &ctx).unwrap();
    assert_eq!(batch_count.load(std::sync::atomic::Ordering::Relaxed), 1);
    listener.invoke("c".to_owned(), &ctx).unwrap();
    // partial batch: no callback yet
    assert_eq!(batch_count.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn page_read_listener_flush_on_end() {
    let batch_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let bc = batch_count.clone();
    let mut listener = PageReadListener::new(5, move |_data, _ctx| {
        bc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    });
    let ctx = AnalysisContext::new("S", 0, 0);
    listener.invoke("a".to_owned(), &ctx).unwrap();
    listener.invoke("b".to_owned(), &ctx).unwrap();
    assert_eq!(batch_count.load(std::sync::atomic::Ordering::Relaxed), 0);
    listener.do_after_all_analysed(&ctx).unwrap();
    assert_eq!(batch_count.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn excel_data_error_contains_row_column() {
    let err = ExcelError::Data {
        sheet: "Users".to_owned(),
        row: 10,
        column: Some(5),
        field: "amount",
        value: "abc".to_owned(),
        message: "not a number".to_owned(),
    };
    let msg = err.to_string();
    assert!(msg.contains("sheet=Users"));
    assert!(msg.contains("row=10"));
    assert!(msg.contains("field=amount"));
    assert!(msg.contains("not a number"));
}

#[test]
fn excel_error_is_cloneable() {
    let err = ExcelError::Format("test".to_owned());
    let err2 = err.clone();
    assert_eq!(err.to_string(), err2.to_string());
}

#[test]
fn analysis_context_construction() {
    let ctx = AnalysisContext::new("Sheet1", 0, 100);
    assert_eq!(ctx.sheet_name(), "Sheet1");
    assert_eq!(ctx.sheet_no(), 0);
    assert_eq!(ctx.row_index(), 100);
}

#[test]
fn analysis_context_with_custom_object() {
    let ctx =
        AnalysisContext::new("S", 0, 0).with_custom_object(Some(CustomReadObject::new(42u32)));
    let val = ctx.custom::<u32>();
    assert_eq!(val, Some(&42u32));
}

#[test]
fn analysis_context_with_batch_index() {
    let ctx = AnalysisContext::new("S", 0, 0).with_batch_index(7);
    assert_eq!(ctx.batch_index(), 7);
}

#[test]
fn write_handler_order_and_before_workbook() {
    let mut h = TestWriteHandler {
        order: -10,
        before_workbook_called: std::sync::atomic::AtomicBool::new(false),
        before_cell_value: None,
    };
    assert_eq!(h.order(), -10);
    let ctx = WriteWorkbookContext::new("out.xlsx");
    h.before_workbook(&ctx).unwrap();
    assert!(
        h.before_workbook_called
            .load(std::sync::atomic::Ordering::Relaxed)
    );
}

#[test]
fn write_handler_before_cell_receives_value() {
    let mut h = TestWriteHandler {
        order: 0,
        before_workbook_called: std::sync::atomic::AtomicBool::new(false),
        before_cell_value: None,
    };
    let mut ctx = WriteCellContext::new("S", 0, 0, CellValue::String("hello".to_owned()));
    h.before_cell(&mut ctx).unwrap();
    assert_eq!(
        h.before_cell_value,
        Some(CellValue::String("hello".to_owned()))
    );
}

#[test]
fn converter_registry_register_and_read() {
    let mut registry = ConverterRegistry::default();
    registry.register::<String, _>(PrefixConverter);
    assert!(!registry.is_empty());

    let ctx = ConvertContext {
        sheet_name: "S".to_owned(),
        row_index: 0,
        column_index: Some(0),
        field: "f",
        format: None,
        date_time_format: None,
        number_format: None,
        use_1904_windowing: false,
    };
    let cell = CellValue::String("abc".to_owned());
    let col = ExcelColumn::new("f", "F", Some(0), 0, None);
    let rctx = ReadConverterContext::new(Some(&cell), &col, &ctx);
    let result = registry
        .convert_to_rust_data::<String>(&rctx)
        .unwrap()
        .unwrap();
    assert_eq!(result, "custom:abc");
}

#[test]
fn converter_registry_write() {
    let mut registry = ConverterRegistry::default();
    registry.register::<String, _>(PrefixConverter);
    let ctx = ConvertContext {
        sheet_name: "S".to_owned(),
        row_index: 0,
        column_index: Some(0),
        field: "f",
        format: None,
        date_time_format: None,
        number_format: None,
        use_1904_windowing: false,
    };
    let col = ExcelColumn::new("f", "F", Some(0), 0, None);
    let cell = registry
        .convert_to_excel_data(&"test".to_owned(), &col, &ctx)
        .unwrap()
        .unwrap();
    assert_eq!(
        cell.effective_value(),
        CellValue::String("custom:test".to_owned())
    );
}

#[test]
fn converter_registry_merged_with_takes_priority() {
    struct AConverter;
    impl Converter<String> for AConverter {
        fn convert_to_rust_data(&self, _: &ReadConverterContext<'_>) -> Result<String> {
            Ok("A".to_owned())
        }
        fn convert_to_excel_data(
            &self,
            _: &WriteConverterContext<'_, String>,
        ) -> Result<WriteCellData> {
            Ok(WriteCellData::from_string("A"))
        }
    }
    struct BConverter;
    impl Converter<String> for BConverter {
        fn convert_to_rust_data(&self, _: &ReadConverterContext<'_>) -> Result<String> {
            Ok("B".to_owned())
        }
        fn convert_to_excel_data(
            &self,
            _: &WriteConverterContext<'_, String>,
        ) -> Result<WriteCellData> {
            Ok(WriteCellData::from_string("B"))
        }
    }

    let mut base = ConverterRegistry::default();
    base.register::<String, _>(AConverter);
    let mut overrides = ConverterRegistry::default();
    overrides.register::<String, _>(BConverter);

    let merged = base.merged_with(&overrides);
    let ctx = ConvertContext {
        sheet_name: "S".to_owned(),
        row_index: 0,
        column_index: Some(0),
        field: "f",
        format: None,
        date_time_format: None,
        number_format: None,
        use_1904_windowing: false,
    };
    let col = ExcelColumn::new("f", "F", Some(0), 0, None);
    let empty_cell = CellValue::String(String::new());
    let rctx = ReadConverterContext::new(Some(&empty_cell), &col, &ctx);
    let result = merged
        .convert_to_rust_data::<String>(&rctx)
        .unwrap()
        .unwrap();
    assert_eq!(result, "B"); // overrides take priority
}

#[test]
fn string_image_converter_read_error() {
    let converter = StringImageConverter;
    let ctx = ConvertContext {
        sheet_name: "S".to_owned(),
        row_index: 0,
        column_index: Some(0),
        field: "img",
        format: None,
        date_time_format: None,
        number_format: None,
        use_1904_windowing: false,
    };
    let cell = CellValue::String("nonexistent.png".to_owned());
    let col = ExcelColumn::new("img", "Img", Some(0), 0, None);
    let rctx = ReadConverterContext::new(Some(&cell), &col, &ctx);
    // convert_to_rust_data should return Unsupported error
    let err = Converter::<String>::convert_to_rust_data(&converter, &rctx);
    assert!(err.is_err());
}

#[test]
fn url_image_converter_timeouts() {
    let c = UrlImageConverter::default();
    assert_eq!(c.connect_timeout(), Duration::from_secs(1));
    assert_eq!(c.read_timeout(), Duration::from_secs(5));

    let c2 = UrlImageConverter::new(Duration::from_secs(2), Duration::from_secs(10));
    assert_eq!(c2.connect_timeout(), Duration::from_secs(2));
    assert_eq!(c2.read_timeout(), Duration::from_secs(10));
}

#[test]
fn excel_type_enum_value() {
    assert_eq!(ExcelTypeEnum::Csv.value(), ".csv");
    assert_eq!(ExcelTypeEnum::Xls.value(), ".xls");
    assert_eq!(ExcelTypeEnum::Xlsx.value(), ".xlsx");
}

#[test]
fn excel_type_enum_from_extension() {
    assert_eq!(
        ExcelTypeEnum::from_extension("csv"),
        Some(ExcelTypeEnum::Csv)
    );
    assert_eq!(
        ExcelTypeEnum::from_extension("xls"),
        Some(ExcelTypeEnum::Xls)
    );
    assert_eq!(
        ExcelTypeEnum::from_extension("xlsx"),
        Some(ExcelTypeEnum::Xlsx)
    );
    assert_eq!(ExcelTypeEnum::from_extension("unknown"), None);
}

#[test]
fn builtin_formats_has_all_indices() {
    assert!(!get_builtin_format(0, "").is_empty());
    assert!(!get_builtin_format(1, "").is_empty());
    assert!(!get_builtin_format(49, "").is_empty());
}

#[test]
fn builtin_format_14_is_date() {
    let fmt = get_builtin_format(14, "");
    assert!(fmt.contains("yyyy") || fmt.contains("m/d"));
}

#[test]
fn xml_constants_are_nonempty() {
    assert!(!ROW_TAG.is_empty());
    assert!(!CELL_TAG.is_empty());
    assert!(!CELL_VALUE_TAG.is_empty());
    assert!(!CELL_FORMULA_TAG.is_empty());
}

#[test]
fn easy_excel_constants_math_context() {
    assert_eq!(EXCEL_MATH_CONTEXT_PRECISION, 15);
}

