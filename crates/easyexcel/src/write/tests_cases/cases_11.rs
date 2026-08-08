/// Annotation column width / row height / content style write to `.xls`.
#[test]
fn write_xls_annotation_dimensions_and_style() -> Result<()> {
    let directory = tempdir().map_err(test_error)?;
    let path = directory.path().join("annotation_style03.xls");
    write_xls::<StyledAnnotationRow, _>(
        &path,
        &WriteOptions {
            sheet_name: "styled".to_owned(),
            ..WriteOptions::default()
        },
        vec![StyledAnnotationRow],
    )?;
    let mut book: Xls<_> = open_workbook(&path).map_err(test_error)?;
    let range = book.worksheet_range("styled").map_err(test_error)?;
    assert!(range.get((1, 0)).map(|c| format!("{c:?}")).is_some());
    // DimensionRow-like widths via StyledAnnotationRow schema — file must be real BIFF8
    let magic = std::fs::read(&path).map_err(test_error)?;
    assert_eq!(&magic[..4], &[0xD0, 0xCF, 0x11, 0xE0], "OLE compound magic");
    Ok(())
}

/// BIFF8 password output emits POI-compatible `CryptoAPI` `FILEPASS` encryption.
#[test]
fn write_xls_encrypts_with_password() {
    let directory = tempdir().expect("tempdir");
    write_xls::<DimensionRow, _>(
        &directory.path().join("protected03.xls"),
        &WriteOptions {
            password: Some("secret".to_owned()),
            ..WriteOptions::default()
        },
        Vec::new(),
    )
    .expect("write encrypted XLS");
    let workbook = easyexcel_xls::read_path_with_password(
        &directory.path().join("protected03.xls"),
        Some("secret"),
    )
    .expect("read encrypted XLS");
    assert_eq!(workbook.sheets.len(), 1);
    assert!(matches!(
        easyexcel_xls::read_path_with_password(
            &directory.path().join("protected03.xls"),
            Some("wrong")
        ),
        Err(easyexcel_io::Error::WrongPassword)
    ));
    // Image writing also works — see core_phase5_xls_features test.
}

/// Java `WorkBookUtil.createWorkBook/createSheet/createRow/createCell` delegates
/// to real POI objects. The Rust adapter must likewise produce a readable XLSX,
/// rather than returning parity-only placeholders.
#[test]
fn workbook_util_creator_chain_materializes_real_xlsx_cells() -> Result<()> {
    let mut workbook = create_work_book(XlsxWorkBookCreator)?;
    {
        let mut sheet_creator = XlsxSheetCreator {
            workbook: &mut workbook,
            constant_memory: false,
        };
        let worksheet = create_sheet(&mut sheet_creator, "用户")?;
        let mut row_creator = XlsxRowCreator { worksheet };
        let mut row = create_row(&mut row_creator, 2)?;
        let cell = create_cell(&mut row, 4)?;
        let XlsxCell {
            worksheet,
            row_index,
            column_index,
        } = cell;
        worksheet
            .write_string(row_index, column_index, "真实单元格")
            .map_err(format_error)?;
    }

    let bytes = workbook.save_to_buffer().map_err(format_error)?;
    let mut parsed = Xlsx::new(Cursor::new(bytes)).map_err(test_error)?;
    let range = parsed.worksheet_range("用户").map_err(test_error)?;
    assert_eq!(
        range.get_value((2, 4)),
        Some(&Data::String("真实单元格".to_owned()))
    );

    let mut workbook = create_work_book(XlsxWorkBookCreator)?;
    let mut sheet_creator = XlsxSheetCreator {
        workbook: &mut workbook,
        constant_memory: false,
    };
    let worksheet = create_sheet(&mut sheet_creator, "limit")?;
    let mut row_creator = XlsxRowCreator { worksheet };
    assert!(matches!(
        create_row(&mut row_creator, 1_048_576),
        Err(ExcelError::Format(message)) if message.contains("1048575")
    ));
    Ok(())
}

#[test]
fn is_scientific_magnitude_large_positive() {
    assert!(easyexcel_format::is_scientific_magnitude(1e11));
}

#[test]
fn is_scientific_magnitude_large_negative() {
    assert!(easyexcel_format::is_scientific_magnitude(-1e12));
}

#[test]
fn is_scientific_magnitude_small_positive() {
    assert!(easyexcel_format::is_scientific_magnitude(1e-11));
}

#[test]
fn is_scientific_magnitude_small_negative() {
    assert!(easyexcel_format::is_scientific_magnitude(-1e-11));
}

#[test]
fn is_scientific_magnitude_zero_is_false() {
    assert!(!easyexcel_format::is_scientific_magnitude(0.0));
}

#[test]
fn is_scientific_magnitude_normal_positive() {
    assert!(!easyexcel_format::is_scientific_magnitude(1.0));
}

#[test]
fn is_scientific_magnitude_normal_negative() {
    assert!(!easyexcel_format::is_scientific_magnitude(-100.0));
}

#[test]
fn is_scientific_magnitude_boundary_1e11() {
    assert!(easyexcel_format::is_scientific_magnitude(1e11));
}

#[test]
fn is_scientific_magnitude_boundary_1e_10() {
    assert!(easyexcel_format::is_scientific_magnitude(1e-10));
}

#[test]
fn biff8_sheet_creator_duplicate_name_fails() {
    let mut book = Biff8Book::default();
    let _ = book.create_sheet("Sheet1");
    let result = book.create_sheet("Sheet1");
    assert!(result.is_err());
}

#[test]
fn excel_writer_output_path_returns_correct_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("output.xlsx");
    let writer = ExcelWriter::new(&path);
    assert_eq!(writer.output_path(), path);
}

#[test]
fn excel_writer_template_file_returns_none_by_default() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.xlsx");
    let writer = ExcelWriter::new(&path);
    assert!(writer.template_file().is_none());
}

#[test]
fn excel_writer_template_bytes_returns_none_by_default() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.xlsx");
    let writer = ExcelWriter::new(&path);
    assert!(writer.template_bytes().is_none());
}

#[test]
fn excel_writer_has_template_configured_false_by_default() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.xlsx");
    let writer = ExcelWriter::new(&path);
    assert!(!writer.has_template_configured());
}

#[test]
fn excel_writer_set_compress_temp_files_true() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("compress.xlsx");
    let mut writer = ExcelWriter::new(&path);
    writer.set_compress_temp_files(true);
    assert!(writer.compress_temp_files_enabled());
}

#[test]
fn excel_writer_set_compress_temp_files_false() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("no_compress.xlsx");
    let mut writer = ExcelWriter::new(&path);
    writer.set_compress_temp_files(false);
    assert!(!writer.compress_temp_files_enabled());
}

#[allow(dead_code)]
struct NoOpHandler;

impl WriteHandler for NoOpHandler {}

struct GeneratedChartHandler(crate::ChartType);

impl WriteHandler for GeneratedChartHandler {
    fn after_workbook_dispose(&mut self, context: &WriteWorkbookContext) -> Result<()> {
        let mut chart = crate::ChartMutation::new("Data", self.0, 4, 3, 20, 12)
            .with_title("Sales")
            .with_series(
                crate::ChartSeries::new(crate::ChartRange::new("Data", 0, 1, 2, 1))
                    .with_name("Amount")
                    .with_categories(crate::ChartRange::new("Data", 0, 0, 2, 0)),
            );
        if self.0 != crate::ChartType::Pie {
            chart = chart.with_series(
                crate::ChartSeries::new(crate::ChartRange::new("Data", 0, 2, 2, 2))
                    .with_name("Amount2")
                    .with_categories(crate::ChartRange::new("Data", 0, 0, 2, 0)),
            );
        }
        context.add_chart(chart)
    }
}

#[test]
fn handler_generates_native_biff8_bar_line_and_pie_charts() -> Result<()> {
    for (kind, sid, name) in [
        (crate::ChartType::Bar, 0x1017u16, "bar"),
        (crate::ChartType::Line, 0x1018u16, "line"),
        (crate::ChartType::Pie, 0x1019u16, "pie"),
    ] {
        let directory = tempdir().map_err(test_error)?;
        let path = directory.path().join(format!("generated-{name}.xls"));
        let rows = (0..3)
            .map(|row| {
                let mut values = std::collections::BTreeMap::new();
                values.insert(0, DynamicValue::String(format!("C{row}")));
                values.insert(
                    1,
                    DynamicValue::ActualData(CellValue::Float(f64::from(row + 1))),
                );
                values.insert(
                    2,
                    DynamicValue::ActualData(CellValue::Float(f64::from((row + 1) * 10))),
                );
                DynamicRow::new(values)
            })
            .collect::<Vec<_>>();
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(GeneratedChartHandler(kind))];
        write_xls_with_handlers::<DynamicRow, _>(
            &path,
            &WriteOptions {
                sheet_name: "Data".to_owned(),
                need_head: false,
                ..WriteOptions::default()
            },
            rows,
            &mut handlers,
        )?;
        let workbook = easyexcel_xls::biff8::record_stream::read_workbook_stream(&path)
            .map_err(test_error)?;
        assert!(workbook_records(&workbook).any(|(actual, _)| actual == sid));
        assert_eq!(
            workbook_records(&workbook)
                .filter(|(actual, _)| *actual == 0x1003)
                .count(),
            if kind == crate::ChartType::Pie { 1 } else { 2 }
        );
        assert!(workbook_records(&workbook).any(|(actual, payload)| {
            actual == 0x1051
                && payload == [
                    0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x3B, 0x00, 0x00, 0x00,
                    0x00, 0x02, 0x00, 0x01, 0x00, 0x01, 0x00,
                ]
        }));
    }
    Ok(())
}

fn workbook_records(bytes: &[u8]) -> impl Iterator<Item = (u16, &[u8])> {
    let mut offset = 0usize;
    std::iter::from_fn(move || {
        if offset + 4 > bytes.len() {
            return None;
        }
        let sid = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        let length = usize::from(u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]));
        if offset + 4 + length > bytes.len() {
            return None;
        }
        let payload = &bytes[offset + 4..offset + 4 + length];
        offset += 4 + length;
        Some((sid, payload))
    })
}

#[derive(Debug, Clone, easyexcel_derive::ExcelRow)]
struct AutoStateRow {
    #[excel(index = 0, name = "Value")]
    value: i64,
}

#[test]
fn stateful_build_auto_selects_streaming_for_scalar_batches() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("stateful_auto.xlsx");
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path).build();
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoUndecided
    );
    writer.write(
        vec![AutoStateRow { value: 1 }],
        &WriteSheet::new("Data"),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoStreaming
    );
    assert!(writer.compress_temp_files_enabled());
    writer.finish()?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn stateful_build_auto_uses_memory_for_unknown_handler() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("stateful_auto_unknown.xlsx");
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path)
        .register_write_handler(NoOpHandler)
        .build();
    writer.write(
        vec![AutoStateRow { value: 1 }],
        &WriteSheet::new("Data"),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::InMemory
    );
    writer.finish()
}

#[test]
fn explicit_streaming_rejects_unknown_handler_before_writing() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("stateful_explicit_conflict.xlsx");
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path)
        .constant_memory(true)
        .register_write_handler(NoOpHandler)
        .build();
    let Err(error) = writer.write(
        vec![AutoStateRow { value: 1 }],
        &WriteSheet::new("Data"),
    ) else {
        panic!("unknown handler requires random access");
    };
    assert!(matches!(error, ExcelError::Unsupported(message) if message.contains("explicit constant-memory")));
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::ExplicitStreaming
    );
    assert!(!path.exists());
}

struct StreamingCounterHandler(Rc<Cell<usize>>);

impl WriteHandler for StreamingCounterHandler {
    fn backend_capability(&self) -> crate::WriteHandlerCapability {
        crate::WriteHandlerCapability::StreamingSafe
    }

    fn after_cell(&mut self, context: &WriteCellContext) -> Result<()> {
        self.0.set(self.0.get().saturating_add(1));
        if !context.is_head {
            context.cell().set_style(crate::ExcelCellStyle {
                border_bottom: Some(crate::ExcelBorderStyle::Double),
                ..crate::ExcelCellStyle::default()
            });
        }
        Ok(())
    }

    fn after_row(&mut self, context: &WriteRowContext) -> Result<()> {
        if !context.is_head {
            context.row().set_height(37);
        }
        Ok(())
    }

    fn before_cell(&mut self, context: &mut WriteCellContext) -> Result<()> {
        if context.is_head {
            context.value = CellValue::String("Renamed".to_owned());
        }
        Ok(())
    }
}

#[test]
fn auto_streaming_promotes_without_replaying_handler_callbacks() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("stateful_promote.xlsx");
    let callbacks = Rc::new(Cell::new(0));
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path)
        .register_write_handler(StreamingCounterHandler(Rc::clone(&callbacks)))
        .build();
    writer.write(
        vec![AutoStateRow { value: 11 }],
        &WriteSheet::new("First"),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoStreaming
    );
    let first_callbacks = callbacks.get();

    writer.write(
        vec![AutoStateRow { value: 22 }],
        &WriteSheet::new("Advanced").auto_width(true),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::InMemory
    );
    assert_eq!(
        callbacks.get(),
        first_callbacks.saturating_add(2),
        "promotion must not replay the first sheet's head/data callbacks"
    );
    writer.finish()?;

    let mut book: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    assert_eq!(
        book.worksheet_range("First")
            .map_err(test_error)?
            .get_value((0, 0)),
        Some(&Data::String("Renamed".to_owned()))
    );
    assert_eq!(
        book.worksheet_range("First")
            .map_err(test_error)?
            .get_value((1, 0)),
        Some(&Data::Float(11.0))
    );
    assert_eq!(
        book.worksheet_range("Advanced")
            .map_err(test_error)?
            .get_value((1, 0)),
        Some(&Data::Float(22.0))
    );
    let styles_xml = zip_entry(&path, "xl/styles.xml")?;
    assert!(
        styles_xml.contains("style=\"double\""),
        "promotion must retain the first batch's handler cell style"
    );
    let first_sheet_xml = zip_entry(&path, "xl/worksheets/sheet1.xml")?;
    assert!(
        first_sheet_xml.contains("r=\"2\" spans=\"1:1\" ht=\"36.75\"")
            && first_sheet_xml.contains("customHeight=\"1\""),
        "promotion must retain the first batch's handler row height: {first_sheet_xml}"
    );
    Ok(())
}

#[test]
fn register_write_handler_after_first_write_fails() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("handler_after.xlsx");
    let mut writer = ExcelWriter::new(&path);
    let data = vec![DynamicRow::default()];
    let sheet = WriteSheet::new("Sheet1");
    writer.write(data, &sheet)?;
    let handler: Box<dyn WriteHandler> = Box::new(NoOpHandler);
    let result = writer.register_write_handler(handler);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn prepend_write_handlers_after_first_write_fails() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("prepend_after.xlsx");
    let mut writer = ExcelWriter::new(&path);
    let data = vec![DynamicRow::default()];
    let sheet = WriteSheet::new("Sheet1");
    writer.write(data, &sheet)?;
    let handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoOpHandler)];
    let result = writer.prepend_write_handlers(handlers);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn register_write_handler_before_first_write_succeeds() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("handler_before.xlsx");
    let mut writer = ExcelWriter::new(&path);
    let handler: Box<dyn WriteHandler> = Box::new(NoOpHandler);
    let result = writer.register_write_handler(handler);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn prepend_write_handlers_before_first_write_succeeds() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("prepend_before.xlsx");
    let mut writer = ExcelWriter::new(&path);
    let handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoOpHandler)];
    let result = writer.prepend_write_handlers(handlers);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn write_xlsx_with_handlers_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("with_handlers.xlsx");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoOpHandler)];
    let options = WriteOptions::default();
    write_xlsx_with_handlers::<DynamicRow, _>(path.as_path(), &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn write_xls_with_handlers_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("with_handlers.xls");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoOpHandler)];
    let options = WriteOptions::default();
    write_xls_with_handlers::<DynamicRow, _>(path.as_path(), &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn write_csv_with_handlers_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("with_handlers.csv");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(NoOpHandler)];
    let options = WriteOptions::default();
    write_csv_with_handlers::<DynamicRow, _>(path.as_path(), &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn write_xlsx_to_writer_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("writer.xlsx");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![];
    let options = WriteOptions::default();
    let file = std::fs::File::create(&path).map_err(test_error)?;
    write_xlsx_to_writer::<DynamicRow, _, _>(path.as_path(), file, &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn write_xls_to_writer_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("writer.xls");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![];
    let options = WriteOptions::default();
    let file = std::fs::File::create(&path).map_err(test_error)?;
    write_xls_to_writer::<DynamicRow, _, _>(path.as_path(), file, &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn write_csv_to_writer_creates_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("writer.csv");
    let data = vec![DynamicRow::default()];
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![];
    let options = WriteOptions::default();
    let file = std::fs::File::create(&path).map_err(test_error)?;
    write_csv_to_writer::<DynamicRow, _, _>(path.as_path(), file, &options, data, &mut handlers)?;
    assert!(path.exists());
    Ok(())
}
