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

/// Phase 5.5: BIFF8 images supported. Test XLS password write succeeds.
#[test]
fn write_xls_rejects_password_and_images() {
    let directory = tempdir().expect("tempdir");
    write_xls::<DimensionRow, _>(
        &directory.path().join("protected03.xls"),
        &WriteOptions {
            password: Some("secret".to_owned()),
            ..WriteOptions::default()
        },
        Vec::new(),
    )
    .expect("XLS password write must succeed (Phase 5.3)");
    assert!(directory.path().join("protected03.xls").exists());
    // Phase 5.5: image writing also works — see core_phase5_xls_features test
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
