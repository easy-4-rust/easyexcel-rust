#[test]
fn easy_excel_inherits_factory_entry_points_through_the_same_rust_type() {
    assert_eq!(
        std::any::TypeId::of::<EasyExcel>(),
        std::any::TypeId::of::<EasyExcelFactory>()
    );
    assert_eq!(EasyExcelFactory::writer_table(3).table_no(), 3);
    assert_eq!(
        EasyExcelFactory::writer_sheet_index::<Value>(4)
            .options()
            .sheet_index,
        Some(4)
    );
}

#[test]
fn easy_excel_factory_builds_all_unbound_sheet_and_table_overloads() {
    let default_read_sheet = EasyExcelFactory::read_sheet().build();
    assert!(!default_read_sheet.has_sheet_no());
    assert!(default_read_sheet.sheet_name().is_empty());

    let indexed_read_sheet = EasyExcelFactory::read_sheet_index(2).build();
    assert!(indexed_read_sheet.has_sheet_no());
    assert_eq!(indexed_read_sheet.sheet_no(), 2);

    let named_read_sheet = EasyExcelFactory::read_sheet_name("Named").build();
    assert!(!named_read_sheet.has_sheet_no());
    assert_eq!(named_read_sheet.sheet_name(), "Named");

    let combined_read_sheet = EasyExcelFactory::read_sheet_with(3, "Combined").build();
    assert_eq!(combined_read_sheet.sheet_no(), 3);
    assert_eq!(combined_read_sheet.sheet_name(), "Combined");

    let default_write_sheet = EasyExcelFactory::writer_sheet_builder().build();
    assert_eq!(default_write_sheet.sheet_no(), 0);
    assert!(default_write_sheet.sheet_name().is_empty());

    let indexed_write_sheet = EasyExcelFactory::writer_sheet_builder_index(4).build();
    assert_eq!(indexed_write_sheet.sheet_no(), 4);

    let named_write_sheet = EasyExcelFactory::writer_sheet_builder_name("Output").build();
    assert_eq!(named_write_sheet.sheet_name(), "Output");

    let combined_write_sheet =
        EasyExcelFactory::writer_sheet_builder_with(5, "CombinedOutput").build();
    assert_eq!(combined_write_sheet.sheet_no(), 5);
    assert_eq!(combined_write_sheet.sheet_name(), "CombinedOutput");

    assert_eq!(
        EasyExcelFactory::writer_table_builder_default()
            .build()
            .table_no(),
        0
    );
    assert_eq!(
        EasyExcelFactory::writer_table_builder(6).build().table_no(),
        6
    );
}

#[test]
fn easy_excel_factory_input_stream_uses_the_real_xlsx_reader_and_cleans_up() -> Result<()> {
    let directory = tempdir()?;
    let source = directory.path().join("factory-stream.xlsx");
    EasyExcelFactory::write::<Value>(&source)
        .need_head(false)
        .do_write([Value("from-stream".to_owned())])?;
    let bytes = fs::read(source)?;

    let events = Arc::new(Mutex::new(Vec::new()));
    let listener = OrderedReadListener {
        name: "stream",
        events: Arc::clone(&events),
    };
    let builder =
        EasyExcelFactory::reader_from_input_stream(Cursor::new(bytes))?.head_row_number(0);
    let temporary_path = builder
        .file
        .as_ref()
        .expect("input stream builder must expose its materialised path")
        .to_owned();
    assert!(temporary_path.exists());

    let builder = builder.register_read_listener::<Value, _>(listener);
    let mut reader = builder.build()?;
    assert!(reader.has_temporary_input());
    reader.read_all()?;
    reader.finish();
    assert!(!reader.has_temporary_input());

    assert_eq!(
        *events.lock().expect("factory stream events lock"),
        vec!["stream:from-stream"]
    );
    assert!(
        !temporary_path.exists(),
        "temporary input must be deleted immediately by finish"
    );
    Ok(())
}

#[test]
fn easy_excel_factory_input_stream_is_cleaned_when_analysis_fails() -> Result<()> {
    struct RejectRow;

    impl ReadListener<Value> for RejectRow {
        fn invoke(&mut self, _data: Value, _context: &AnalysisContext) -> Result<()> {
            Err(ExcelError::Format("listener rejected row".to_owned()))
        }
    }

    let directory = tempdir()?;
    let source = directory.path().join("factory-stream-error.xlsx");
    EasyExcelFactory::write::<Value>(&source)
        .need_head(false)
        .do_write([Value("reject-me".to_owned())])?;
    let builder = EasyExcelFactory::reader_from_input_stream(Cursor::new(fs::read(source)?))?
        .head_row_number(0);
    let temporary_path = builder
        .file
        .as_ref()
        .expect("materialised input path")
        .to_owned();
    let mut reader = builder.build(RejectRow)?;

    let error = reader.read_all().expect_err("listener must reject the row");
    assert!(error.to_string().contains("listener rejected row"));
    assert!(!reader.has_temporary_input());
    assert!(
        !temporary_path.exists(),
        "analysis failure must run finish and delete temporary input"
    );
    Ok(())
}

#[test]
fn easy_excel_factory_detects_an_encrypted_xlsx_input_stream() -> Result<()> {
    let directory = tempdir()?;
    let source = directory.path().join("factory-encrypted.xlsx");
    EasyExcelFactory::write::<Value>(&source)
        .password("stream-secret")
        .need_head(false)
        .do_write([Value("encrypted-stream".to_owned())])?;
    let bytes = fs::read(source)?;
    assert!(bytes.starts_with(b"\xD0\xCF\x11\xE0"));

    let events = Arc::new(Mutex::new(Vec::new()));
    let listener = OrderedReadListener {
        name: "encrypted",
        events: Arc::clone(&events),
    };
    let mut reader = EasyExcelFactory::reader_from_input_stream(Cursor::new(bytes))?
        .password("stream-secret")
        .head_row_number(0)
        .build(listener)?;
    reader.read_all()?;
    assert_eq!(
        *events.lock().expect("encrypted stream events lock"),
        vec!["encrypted:encrypted-stream"]
    );
    Ok(())
}

#[test]
fn easy_excel_factory_path_and_output_stream_builders_execute_real_writes() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("factory-path.xlsx");
    EasyExcelFactory::writer_to_path(&path)
        .sheet_name("Path")
        .expect("path-backed writer sheet")
        .need_head(false)
        .do_write([Value("path".to_owned())])?;
    assert_eq!(
        EasyExcel::read_sync::<Value>(&path)
            .sheet("Path")
            .head_row_number(0)
            .do_read_sync()?,
        vec![Value("path".to_owned())]
    );

    let output = ExcelOutputStream::new(Cursor::new(Vec::<u8>::new()));
    let inspect = output.clone();
    EasyExcelFactory::writer()
        .auto_close_stream(false)
        .output_stream(output)
        .sheet_name("Stream")
        .need_head(false)
        .do_write([Value("output-stream".to_owned())])?;
    let bytes = inspect
        .with_inner(|cursor| cursor.get_ref().clone())
        .expect("auto_close_stream(false) keeps output inspectable");
    assert!(bytes.starts_with(b"PK"));

    let observed = Arc::new(Mutex::new(Vec::new()));
    let listener = OrderedReadListener {
        name: "output",
        events: Arc::clone(&observed),
    };
    EasyExcelFactory::reader_from_input_stream(Cursor::new(bytes))?
        .head_row_number(0)
        .sheet_name("Stream")
        .build(listener)?
        .read_all()?;
    assert_eq!(
        *observed.lock().expect("factory output events lock"),
        vec!["output:output-stream"]
    );
    Ok(())
}

#[derive(Clone)]
struct FallibleValue {
    value: &'static str,
    fail: bool,
}

#[derive(Default)]
struct FacadeProbeWrite {
    bytes: Vec<u8>,
    fail_write: bool,
    fail_flush: bool,
    fail_flush_at: Option<usize>,
    flushes: usize,
}

impl Write for FacadeProbeWrite {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            Err(io::Error::other("injected facade write failure"))
        } else {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let flush = self.flushes;
        self.flushes += 1;
        if self.fail_flush || self.fail_flush_at == Some(flush) {
            Err(io::Error::other("injected facade flush failure"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct ToggleFacadeWrite {
    fail: Arc<AtomicBool>,
}

struct OrderedReadListener {
    name: &'static str,
    events: Arc<Mutex<Vec<String>>>,
}

impl ReadListener<Value> for OrderedReadListener {
    fn invoke(&mut self, data: Value, _context: &AnalysisContext) -> Result<()> {
        self.events
            .lock()
            .expect("ordered listener lock")
            .push(format!("{}:{}", self.name, data.0));
        Ok(())
    }
}

impl Write for ToggleFacadeWrite {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail.load(Ordering::SeqCst) {
            Err(io::Error::other("injected final encoding failure"))
        } else {
            Ok(buffer.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ExcelRow for FallibleValue {
    fn schema() -> &'static [ExcelColumn] {
        Value::schema()
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Err(ExcelError::Unsupported("write-only test row".to_owned()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        if self.fail {
            Err(ExcelError::Format("injected conversion failure".to_owned()))
        } else {
            Ok(vec![CellValue::String(self.value.to_owned())])
        }
    }
}

#[test]
fn writer_builder_excel_type_overrides_path_extension() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("values.data");

    EasyExcel::write::<Value>(&path)
        .excel_type(crate::support::ExcelTypeEnum::Csv)
        .with_bom(false)
        .do_write(vec![Value("one".to_owned())])?;

    assert_eq!(fs::read_to_string(path)?, "Value\none\n");
    Ok(())
}

#[test]
fn reader_builder_register_read_listener_dispatches_in_registration_order() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("listeners.csv");
    fs::write(&path, "Value\none\ntwo\n")?;
    let events = Arc::new(Mutex::new(Vec::new()));

    EasyExcel::read::<Value, _>(
        &path,
        OrderedReadListener {
            name: "first",
            events: Arc::clone(&events),
        },
    )
    .register_read_listener(OrderedReadListener {
        name: "second",
        events: Arc::clone(&events),
    })
    .do_read()?;

    assert_eq!(
        *events.lock().expect("ordered listener lock"),
        vec![
            "first:one".to_owned(),
            "second:one".to_owned(),
            "first:two".to_owned(),
            "second:two".to_owned(),
        ]
    );
    Ok(())
}

fn write_minimal_template(path: &Path, shared_strings: &str, worksheet: &str) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut archive = ZipWriter::new(file);
    archive
        .start_file("xl/sharedStrings.xml", SimpleFileOptions::default())
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    archive.write_all(shared_strings.as_bytes())?;
    archive
        .start_file("xl/worksheets/sheet1.xml", SimpleFileOptions::default())
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    archive.write_all(worksheet.as_bytes())?;
    archive
        .finish()
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    Ok(())
}

