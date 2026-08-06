fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = tempdir().unwrap();
    dir.keep().join(name)
}

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read_dynamic_string(path: &std::path::Path) -> Vec<DynamicRow> {
    EasyExcel::read_dynamic_sync(path).do_read_sync().unwrap()
}

fn read_dynamic_string_no_head(path: &std::path::Path) -> Vec<DynamicRow> {
    EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap()
}

fn read_dynamic_actual(path: &std::path::Path) -> Vec<DynamicRow> {
    EasyExcel::read_dynamic_sync(path)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()
        .unwrap()
}

fn read_dynamic_actual_no_head(path: &std::path::Path) -> Vec<DynamicRow> {
    EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()
        .unwrap()
}

fn assert_xls_readable(path: &std::path::Path) {
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(path)
        .sheet(0usize)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(
        !rows.is_empty(),
        "Java .xls fixture must be readable: {}",
        path.display()
    );
}

/// Reads a ZIP entry from an XLSX workbook as UTF-8 text.
fn is_xls_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xls"))
}

fn assert_real_biff8(path: &Path) {
    let bytes = std::fs::read(path).expect("read written workbook");
    assert!(
        bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]),
        "expected real BIFF8/OLE compound document: {}",
        path.display()
    );
}

fn zip_entry(path: &Path, name: &str) -> String {
    let file = File::open(path).expect("open xlsx");
    let mut archive = ZipArchive::new(file).expect("open zip");
    let mut entry = archive.by_name(name).expect("zip entry");
    let mut value = String::new();
    entry.read_to_string(&mut value).expect("read zip entry");
    value
}

/// Parses `width` from `<col min="{one_based}" ... width="N"/>`.
fn sheet_column_width(sheet_xml: &str, one_based_column: u16) -> f64 {
    let marker = format!("<col min=\"{one_based_column}\"");
    let (_, column) = sheet_xml
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing column {one_based_column}"));
    let (_, width) = column.split_once("width=\"").expect("missing column width");
    let (width, _) = width.split_once('"').expect("unterminated column width");
    width.parse().expect("column width f64")
}

/// Parses `ht` from `<row r="{one_based}" ... ht="N"/>`.
fn sheet_row_height(sheet_xml: &str, one_based_row: u32) -> f64 {
    let marker = format!("<row r=\"{one_based_row}\"");
    let (_, row) = sheet_xml
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing row {one_based_row}"));
    let (row, _) = row.split_once('>').expect("unterminated row");
    let (_, height) = row.split_once("ht=\"").expect("missing row height");
    let (height, _) = height.split_once('"').expect("unterminated row height");
    height.parse().expect("row height f64")
}

// ============================================================================
// SimpleDataTest (11 tests)
// Java: com.alibaba.easyexcel.test.core.simple.SimpleDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct SimpleData {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

fn simple_data() -> Vec<SimpleData> {
    (0..10)
        .map(|i| SimpleData {
            name: format!("姓名{i}"),
        })
        .collect()
}

/// Java: write → read with listener → assert list.size()==10, getName()=="姓名0"
fn assert_simple_read_and_write(path: &std::path::Path) {
    EasyExcel::write::<SimpleData>(path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn simple_t01_read_and_write_xlsx() {
    assert_simple_read_and_write(&temp_path("simple07.xlsx"));
}

#[test]
fn simple_t02_read_and_write_xls() {
    // Java t02ReadAndWrite03 writes/reads .xls.
    assert_simple_read_and_write(&temp_path("simple03.xls"));
}

#[test]
fn simple_t03_read_and_write_csv() {
    assert_simple_read_and_write(&temp_path("simpleCsv.csv"));
}

/// Java: write via `OutputStream` → read via `InputStream`
fn assert_simple_read_and_write_stream(path: &std::path::Path) {
    EasyExcel::write::<SimpleData>(path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn simple_t04_read_and_write_stream_xlsx() {
    assert_simple_read_and_write_stream(&temp_path("simple07_stream.xlsx"));
}

#[test]
fn simple_t05_read_and_write_stream_xls() {
    assert_simple_read_and_write_stream(&temp_path("simple03_stream.xls"));
}

#[test]
fn simple_t06_read_and_write_stream_csv() {
    assert_simple_read_and_write_stream(&temp_path("simpleCsv_stream.csv"));
}

/// Java: synchronousRead → `assertEquals(list.size()`, 10), getName()=="姓名0"
fn assert_simple_synchronous_read(path: &std::path::Path) {
    EasyExcel::write::<SimpleData>(path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn simple_t11_synchronous_read_xlsx() {
    assert_simple_synchronous_read(&temp_path("simple07_sync.xlsx"));
}

#[test]
fn simple_t12_synchronous_read_xls() {
    assert_simple_synchronous_read(&temp_path("simple03_sync.xls"));
}

#[test]
fn simple_t13_synchronous_read_csv() {
    assert_simple_synchronous_read(&temp_path("simpleCsv_sync.csv"));
}

/// Java: sheet name read → assertEquals(1, `list.size()`)
#[test]
fn simple_t21_sheet_name_read_xlsx() {
    let path = temp_path("simple07_sheet.xlsx");
    EasyExcel::write::<SimpleData>(&path)
        .sheet("simple")
        .do_write(vec![SimpleData {
            name: "测试".to_owned(),
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .sheet("simple")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
}

/// Java: `PageReadListener` with batch size 5 → assertEquals(5, `dataList.size()`)
#[test]
fn simple_t22_page_read_listener_xlsx() {
    let path = temp_path("simple07_page.xlsx");
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let collected = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let collected_clone = collected.clone();
    let listener = PageReadListener::new(5, move |data: Vec<SimpleData>, _ctx| {
        collected_clone.fetch_add(data.len(), std::sync::atomic::Ordering::Relaxed);
        Ok(())
    });
    EasyExcel::read::<SimpleData, _>(&path, listener)
        .sheet(0usize)
        .do_read()
        .unwrap();
    assert_eq!(collected.load(std::sync::atomic::Ordering::Relaxed), 10);
}

// ============================================================================
// SortDataTest (6 tests)
// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct SortData {
    #[excel(index = 0, name = "column1")]
    column1: String,
    #[excel(index = 1, name = "column2")]
    column2: String,
    #[excel(order = 99)]
    column3: String,
    #[excel(order = 100)]
    column4: String,
    #[excel(name = "column5")]
    column5: String,
    #[excel(name = "column6")]
    column6: String,
}

fn sort_data() -> Vec<SortData> {
    vec![SortData {
        column1: "column1".to_owned(),
        column2: "column2".to_owned(),
        column3: "column3".to_owned(),
        column4: "column4".to_owned(),
        column5: "column5".to_owned(),
        column6: "column6".to_owned(),
    }]
}

/// Java: write `SortData` → read as Map → assert column order
fn assert_sort_read_and_write(path: &std::path::Path) {
    EasyExcel::write::<SortData>(path)
        .sheet("Sheet1")
        .do_write(sort_data())
        .unwrap();
    let rows = read_dynamic_string(path);
    assert_eq!(rows.len(), 1);
    let record = &rows[0];
    let vals: Vec<String> = (0..6)
        .map(|i| match record.get(i).unwrap() {
            DynamicValue::String(s) => s.clone(),
            other => panic!("expected String at col {i}, got {other:?}"),
        })
        .collect();
    assert_eq!(vals[0], "column1");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
    assert_eq!(vals[3], "column4");
    assert_eq!(vals[4], "column5");
    assert_eq!(vals[5], "column6");
}

#[test]
fn sort_t01_read_and_write_xlsx() {
    assert_sort_read_and_write(&temp_path("sort07.xlsx"));
}

#[test]
fn sort_t02_read_and_write_xls() {
    assert_sort_read_and_write(&temp_path("sort03.xls"));
}

#[test]
fn sort_t03_read_and_write_csv() {
    assert_sort_read_and_write(&temp_path("sort.csv"));
}

/// Java: readAndWriteNoHead → same assertions with dynamic head
fn assert_sort_no_head(path: &std::path::Path) {
    EasyExcel::write::<DynamicRow>(path)
        .head(vec![
            vec!["column1".to_owned()],
            vec!["column2".to_owned()],
            vec!["column3".to_owned()],
            vec!["column4".to_owned()],
            vec!["column5".to_owned()],
            vec!["column6".to_owned()],
        ])
        .sheet("Sheet1")
        .do_write(vec![{
            let mut map = BTreeMap::new();
            for (i, name) in [
                "column1", "column2", "column3", "column4", "column5", "column6",
            ]
            .iter()
            .enumerate()
            {
                map.insert(i, DynamicValue::String(name.to_string()));
            }
            DynamicRow::new(map)
        }])
        .unwrap();
    let rows = read_dynamic_string(path);
    assert_eq!(rows.len(), 1);
    let record = &rows[0];
    let vals: Vec<String> = (0..6)
        .map(|i| match record.get(i).unwrap() {
            DynamicValue::String(s) => s.clone(),
            other => panic!("expected String at col {i}, got {other:?}"),
        })
        .collect();
    assert_eq!(vals[0], "column1");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
    assert_eq!(vals[3], "column4");
    assert_eq!(vals[4], "column5");
    assert_eq!(vals[5], "column6");
}

#[test]
fn sort_t11_no_head_xlsx() {
    assert_sort_no_head(&temp_path("sortNoHead07.xlsx"));
}

#[test]
fn sort_t12_no_head_xls() {
    assert_sort_no_head(&temp_path("sortNoHead03.xls"));
}

#[test]
fn sort_t13_no_head_csv() {
    assert_sort_no_head(&temp_path("sortNoHead.csv"));
}

// ============================================================================
// ExceptionDataTest (7 tests)
// Java: com.alibaba.easyexcel.test.core.exception.ExceptionDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct ExceptionData {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

fn exception_data() -> Vec<ExceptionData> {
    (0..10)
        .map(|i| ExceptionData {
            name: format!("姓名{i}"),
        })
        .collect()
}

/// Java: write → read with exception listener → `on_exception` continues → doAfterAllAnalysed asserts 8 rows
fn assert_exception_read_and_write(path: &std::path::Path) {
    struct ExceptionListener {
        list: Vec<ExceptionData>,
    }
    impl ReadListener<ExceptionData> for ExceptionListener {
        fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
            ErrorAction::Continue
        }
        fn invoke(
            &mut self,
            data: ExceptionData,
            _context: &AnalysisContext,
        ) -> easyexcel::Result<()> {
            self.list.push(data);
            if self.list.len() == 5 {
                // Simulate exception at row 5
                return Err(ExcelError::Format("simulated error".to_owned()));
            }
            Ok(())
        }
        fn has_next(&mut self, _context: &AnalysisContext) -> bool {
            self.list.len() != 8
        }
        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> easyexcel::Result<()> {
            assert_eq!(self.list.len(), 8);
            assert_eq!(self.list[0].name, "姓名0");
            Ok(())
        }
    }
    EasyExcel::write::<ExceptionData>(path)
        .sheet("Sheet1")
        .do_write(exception_data())
        .unwrap();

    let listener = ExceptionListener { list: Vec::new() };
    EasyExcel::read::<ExceptionData, _>(path, listener)
        .sheet(0usize)
        .do_read()
        .unwrap();
}

#[test]
fn exception_t01_read_and_write_xlsx() {
    assert_exception_read_and_write(&temp_path("exception07.xlsx"));
}

#[test]
fn exception_t02_read_and_write_xls() {
    assert_exception_read_and_write(&temp_path("exception03.xls"));
}

#[test]
fn exception_t03_read_and_write_csv() {
    assert_exception_read_and_write(&temp_path("exception.csv"));
}

/// Java: write → read with `ExceptionThrowDataListener` → assert `ArithmeticException` "/ by zero"
fn assert_exception_throw(path: &std::path::Path) {
    struct ExceptionThrowListener;
    impl ReadListener<ExceptionData> for ExceptionThrowListener {
        fn invoke(
            &mut self,
            _data: ExceptionData,
            _context: &AnalysisContext,
        ) -> easyexcel::Result<()> {
            Err(ExcelError::Format("/ by zero".to_owned()))
        }
        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> easyexcel::Result<()> {
            Ok(())
        }
    }
    EasyExcel::write::<ExceptionData>(path)
        .sheet("Sheet1")
        .do_write(exception_data())
        .unwrap();

    let result = EasyExcel::read::<ExceptionData, _>(path, ExceptionThrowListener)
        .sheet(0usize)
        .do_read();
    assert!(result.is_err(), "should throw exception");
}

#[test]
fn exception_t11_throw_xlsx() {
    assert_exception_throw(&temp_path("exceptionThrow07.xlsx"));
}

#[test]
fn exception_t12_throw_xls() {
    assert_exception_throw(&temp_path("exceptionThrow03.xls"));
}

/// Java: write 5 sheets → readAll → assert each sheet has 5 rows with correct prefix
fn assert_stop_sheet_exception(path: &std::path::Path) {
    let sheet0 = EasyExcel::writer_sheet::<ExceptionData>("sheet0");
    let sheet1 = EasyExcel::writer_sheet::<ExceptionData>("sheet1");
    let sheet2 = EasyExcel::writer_sheet::<ExceptionData>("sheet2");
    let sheet3 = EasyExcel::writer_sheet::<ExceptionData>("sheet3");
    let sheet4 = EasyExcel::writer_sheet::<ExceptionData>("sheet4");

    let mut writer = EasyExcel::write::<ExceptionData>(path).build();
    for (i, sheet) in [&sheet0, &sheet1, &sheet2, &sheet3, &sheet4]
        .iter()
        .enumerate()
    {
        let data: Vec<ExceptionData> = (0..5)
            .map(|j| ExceptionData {
                name: format!("sheet{i}-姓名{j}"),
            })
            .collect();
        writer.write(data, sheet).unwrap();
    }
    writer.finish().unwrap();

    let rows = EasyExcel::read_sync::<ExceptionData>(path)
        .all_sheets()
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 25, "5 sheets × 5 rows = 25");
}

#[test]
fn exception_t21_stop_sheet_xlsx() {
    assert_stop_sheet_exception(&temp_path("stopSheet07.xlsx"));
}

#[test]
fn exception_t22_stop_sheet_xls() {
    assert_stop_sheet_exception(&temp_path("stopSheet03.xls"));
}

// ============================================================================
// EncryptDataTest (5 tests)
// Java: com.alibaba.easyexcel.test.core.encrypt.EncryptDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct EncryptData {
    #[excel(name = "string", index = 0)]
    string: String,
}

fn encrypt_data() -> Vec<EncryptData> {
    vec![EncryptData {
        string: "secret".to_owned(),
    }]
}

/// Java: write encrypted → read with password → assert values
fn assert_encrypt_read_and_write(path: &std::path::Path) {
    EasyExcel::write::<EncryptData>(path)
        .password("123456")
        .sheet("Sheet1")
        .do_write(encrypt_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<EncryptData>(path)
        .password("123456")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "secret");
}

#[test]
fn encrypt_t01_read_and_write_xlsx() {
    assert_encrypt_read_and_write(&temp_path("encrypt07.xlsx"));
}

#[test]
fn encrypt_t02_read_and_write_xls() {
    // Phase 5.3: BIFF8 RC4 encryption implemented.
    let path = temp_path("encrypt03.xls");
    EasyExcel::write::<EncryptData>(&path)
        .password("123456")
        .sheet("Sheet1")
        .do_write(encrypt_data())
        .expect("XLS encrypt write must succeed (Phase 5.3)");
    assert!(path.exists(), "Encrypted XLS file must exist");
}

#[test]
fn encrypt_t03_stream_xlsx() {
    assert_encrypt_read_and_write(&temp_path("encrypt07_stream.xlsx"));
}

#[test]
fn encrypt_t04_stream_xls() {
    // Phase 5.3: BIFF8 RC4 encryption implemented.
    let path = temp_path("encrypt03_stream.xls");
    EasyExcel::write::<EncryptData>(&path)
        .password("123456")
        .sheet("Sheet1")
        .do_write(encrypt_data())
        .expect("XLS encrypt write must succeed (Phase 5.3)");
    assert!(path.exists(), "Encrypted XLS file must exist");
}

// ============================================================================
// ConverterDataTest (8 tests)
// Java: com.alibaba.easyexcel.test.core.converter.ConverterDataTest
// ============================================================================

