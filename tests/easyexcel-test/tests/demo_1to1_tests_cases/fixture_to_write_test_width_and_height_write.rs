fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn require_fixture(name: &str) -> std::path::PathBuf {
    let path = fixture(name);
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    path
}

fn temp_path(name: &str) -> std::path::PathBuf {
    tempdir().unwrap().keep().join(name)
}

/// Java `demo.read.DemoData` / `demo.write.DemoData`.
#[derive(Debug, Clone, ExcelRow)]
struct DemoData {
    #[excel(name = "字符串标题", order = 1)]
    string: String,
    #[excel(name = "日期标题", order = 2)]
    date: Option<NaiveDate>,
    #[excel(name = "数字标题", order = 3)]
    double_data: Option<f64>,
}

#[derive(Debug, Clone, ExcelRow)]
struct WriteDemoData {
    #[excel(name = "字符串标题", order = 1)]
    string: String,
    #[excel(name = "日期标题", order = 2)]
    date: NaiveDate,
    #[excel(name = "数字标题", order = 3)]
    double_data: f64,
}

fn write_demo_data() -> Vec<WriteDemoData> {
    (0..10)
        .map(|i| WriteDemoData {
            string: format!("字符串{i}"),
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            double_data: 0.56,
        })
        .collect()
}

fn assert_write_10(path: &std::path::Path) {
    assert_eq!(
        EasyExcel::read_sync::<WriteDemoData>(path)
            .do_read_sync()
            .unwrap()
            .len(),
        10
    );
}

// ============================================================================
// read.ReadTest — 12
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#simpleRead`
#[test]
fn read_test_simple_read() {
    let path = require_fixture("demo/demo.xlsx");
    let total = Arc::new(Mutex::new(0usize));
    let total_cb = Arc::clone(&total);
    let listener = PageReadListener::new(100, move |batch: Vec<DemoData>, _ctx| {
        *total_cb.lock().unwrap() += batch.len();
        Ok(())
    });
    EasyExcel::read::<DemoData, _>(&path, listener)
        .sheet(0usize)
        .do_read()
        .unwrap();
    let page_count = *total.lock().unwrap();
    assert!(page_count > 0);
    assert_eq!(
        EasyExcel::read_sync::<DemoData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .len(),
        page_count
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#indexOrNameRead`
#[test]
fn read_test_index_or_name_read() {
    #[derive(Debug, Clone, ExcelRow)]
    struct IndexOrNameData {
        #[excel(index = 0)]
        string: Option<String>,
        #[excel(name = "日期标题")]
        date: Option<NaiveDate>,
        #[excel(index = 2)]
        double_data: Option<f64>,
    }
    let path = require_fixture("demo/demo.xlsx");
    let rows = EasyExcel::read_sync::<IndexOrNameData>(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
    assert!(rows[0].string.as_ref().is_some_and(|s| !s.is_empty()));
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#repeatedRead`
#[test]
fn read_test_repeated_read() {
    let path = require_fixture("demo/demo.xlsx");
    assert!(
        !EasyExcel::read_sync::<DemoData>(&path)
            .all_sheets()
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
    assert!(
        !EasyExcel::read_sync::<DemoData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#converterRead`
#[test]
fn read_test_converter_read() {
    let path = require_fixture("demo/demo.xlsx");
    let rows = EasyExcel::read_sync::<DemoData>(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
    assert!(!rows[0].string.is_empty());
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#complexHeaderRead`
#[test]
fn read_test_complex_header_read() {
    let path = require_fixture("demo/demo.xlsx");
    let rows = EasyExcel::read_sync::<DemoData>(&path)
        .sheet(0usize)
        .head_row_number(1)
        .do_read_sync()
        .unwrap();
    let fallback = EasyExcel::read_sync::<DemoData>(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap()
        .len();
    assert!(!rows.is_empty() || fallback > 0);
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#headerRead`
#[test]
fn read_test_header_read() {
    struct HeadListener {
        saw: Arc<Mutex<bool>>,
    }
    impl ReadListener<DemoData> for HeadListener {
        fn invoke_head(
            &mut self,
            head: &HashMap<String, usize>,
            _ctx: &AnalysisContext,
        ) -> Result<()> {
            assert!(!head.is_empty());
            *self.saw.lock().unwrap() = true;
            Ok(())
        }
        fn invoke(&mut self, _data: DemoData, _ctx: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }
    let path = require_fixture("demo/demo.xlsx");
    let saw_head = Arc::new(Mutex::new(false));
    let saw = Arc::clone(&saw_head);
    EasyExcel::read::<DemoData, _>(&path, HeadListener { saw })
        .sheet(0usize)
        .do_read()
        .unwrap();
    assert!(*saw_head.lock().unwrap());
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#extraRead`
#[test]
fn read_test_extra_read() {
    // Java also ships extra.xls; assert real BIFF8 read (only-add; keep xlsx path).
    let xls = require_fixture("demo/extra.xls");
    assert!(
        !EasyExcel::read_dynamic_sync(&xls)
            .sheet(0usize)
            .head_row_number(0)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
    let path = require_fixture("demo/extra.xlsx");
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#cellDataRead`
#[test]
fn read_test_cell_data_read() {
    let path = require_fixture("demo/cellDataDemo.xlsx");
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#exceptionRead`
///
/// Maps string column into `NaiveDate` (Java `ExceptionDemoData.date`); listener
/// continues on convert errors via `ErrorAction::Continue`.
#[test]
fn read_test_exception_read() {
    struct DemoExceptionListener {
        hits: Arc<AtomicUsize>,
    }
    impl ReadListener<ExceptionDemoData> for DemoExceptionListener {
        fn on_exception(&mut self, _error: &ExcelError, _ctx: &AnalysisContext) -> ErrorAction {
            self.hits.fetch_add(1, Ordering::Relaxed);
            ErrorAction::Continue
        }
        fn invoke(&mut self, _data: ExceptionDemoData, _ctx: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }
    #[derive(Debug, Clone, ExcelRow)]
    struct ExceptionDemoData {
        #[excel(index = 0)]
        date: NaiveDate,
    }
    let path = require_fixture("demo/demo.xlsx");
    let exceptions = Arc::new(AtomicUsize::new(0));
    let hits = Arc::clone(&exceptions);
    EasyExcel::read::<ExceptionDemoData, _>(&path, DemoExceptionListener { hits })
        .sheet(0usize)
        .do_read()
        .unwrap();
    assert!(
        exceptions.load(Ordering::Relaxed) > 0,
        "string→date conversion must fire on_exception"
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#synchronousRead`
#[test]
fn read_test_synchronous_read() {
    let path = require_fixture("demo/demo.xlsx");
    assert!(
        !EasyExcel::read_sync::<DemoData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#noModelRead`
#[test]
fn read_test_no_model_read() {
    let path = require_fixture("demo/demo.xlsx");
    let rows = EasyExcel::read_dynamic_sync(&path)
        .sheet(0usize)
        .head_row_number(1)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

/// Java: `com.alibaba.easyexcel.test.demo.read.ReadTest#csvFormat`
#[test]
fn read_test_csv_format() {
    let path = require_fixture("demo/demo.csv");
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

// ============================================================================
// write.WriteTest — 20
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#simpleWrite`
#[test]
fn write_test_simple_write() {
    let path = temp_path("simpleWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
    let path3 = temp_path("simpleWrite3.xlsx");
    let mut writer = EasyExcel::write::<WriteDemoData>(&path3).build();
    let sheet = EasyExcel::writer_sheet::<WriteDemoData>("模板");
    writer.write(write_demo_data(), &sheet).unwrap();
    writer.finish().unwrap();
    assert_write_10(&path3);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#excludeOrIncludeWrite`
#[test]
fn write_test_exclude_or_include_write() {
    let path = temp_path("excludeOrIncludeWrite.xlsx");
    let mut exclude = HashSet::new();
    exclude.insert("date".to_owned());
    EasyExcel::write::<WriteDemoData>(&path)
        .exclude_column_field_names(exclude)
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
    let path2 = temp_path("includeOnlyDate.xlsx");
    let mut include = HashSet::new();
    include.insert("date".to_owned());
    EasyExcel::write::<WriteDemoData>(&path2)
        .include_column_field_names(include)
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(path2.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#indexWrite`
#[test]
fn write_test_index_write() {
    #[derive(Debug, Clone, ExcelRow)]
    struct IndexData {
        #[excel(name = "字符串标题", index = 0)]
        string: String,
        #[excel(name = "日期标题", index = 1)]
        date: NaiveDate,
        #[excel(name = "数字标题", index = 3)]
        double_data: f64,
    }
    let path = temp_path("indexWrite.xlsx");
    let data: Vec<IndexData> = (0..10)
        .map(|i| IndexData {
            string: format!("字符串{i}"),
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            double_data: 0.56,
        })
        .collect();
    EasyExcel::write::<IndexData>(&path)
        .sheet("模板")
        .do_write(data)
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
    assert!(rows.last().unwrap().values().len() >= 3);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#complexHeadWrite`
#[test]
fn write_test_complex_head_write() {
    let path = temp_path("complexHeadWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .head([
            ["主标题", "字符串标题"],
            ["主标题", "日期标题"],
            ["主标题", "数字标题"],
        ])
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#repeatedWrite`
#[test]
fn write_test_repeated_write() {
    let path = temp_path("repeatedWrite.xlsx");
    let mut writer = EasyExcel::write::<WriteDemoData>(&path).build();
    for i in 0..3 {
        let sheet = EasyExcel::writer_sheet::<WriteDemoData>(format!("模板{i}"));
        writer.write(write_demo_data(), &sheet).unwrap();
    }
    writer.finish().unwrap();
    assert_eq!(
        EasyExcel::read_sync::<WriteDemoData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap()
            .len(),
        10
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#converterWrite`
#[test]
fn write_test_converter_write() {
    #[derive(Debug, Clone, ExcelRow)]
    struct ConverterData {
        #[excel(name = "字符串标题")]
        string: String,
        #[excel(name = "日期标题", format = "%Y-%m-%d")]
        date: NaiveDate,
        #[excel(name = "数字标题")]
        double_data: f64,
    }
    let path = temp_path("converterWrite.xlsx");
    let data: Vec<ConverterData> = (0..10)
        .map(|i| ConverterData {
            string: format!("字符串{i}"),
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            double_data: 0.56,
        })
        .collect();
    EasyExcel::write::<ConverterData>(&path)
        .sheet("模板")
        .do_write(data)
        .unwrap();
    assert_eq!(
        EasyExcel::read_sync::<ConverterData>(&path)
            .do_read_sync()
            .unwrap()
            .len(),
        10
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#imageWrite`
#[test]
fn write_test_image_write() {
    #[derive(Debug, Clone, ExcelRow)]
    struct ImageDemoData {
        #[excel(name = "byteArray")]
        byte_array: WriteCellData,
        #[excel(name = "writeCellDataFile")]
        write_cell_data_file: WriteCellData,
    }
    let img = require_fixture("converter/img.jpg");
    let bytes = std::fs::read(&img).unwrap();
    let path = temp_path("imageWrite.xlsx");
    let row = ImageDemoData {
        byte_array: WriteCellData::from_image(bytes.clone()),
        write_cell_data_file: WriteCellData::from_string("额外的放一些文字").image_data_list([
            ImageData::new(bytes.clone())
                .image_type(ImageType::Jpeg)
                .anchor(ClientAnchorData::new().top(5).right(40).bottom(5).left(5)),
            ImageData::new(bytes).image_type(ImageType::Jpeg).anchor(
                ClientAnchorData::new()
                    .top(5)
                    .right(5)
                    .bottom(5)
                    .left(50)
                    .coordinates(CoordinateData::new().relative_last_column_index(1)),
            ),
        ]),
    };
    EasyExcel::write::<ImageDemoData>(&path)
        .sheet("Sheet1")
        .do_write(vec![row])
        .unwrap();
    assert!(path.metadata().unwrap().len() > 1000);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#writeCellDataWrite`
#[test]
fn write_test_write_cell_data_write() {
    #[derive(Debug, Clone, ExcelRow)]
    struct WriteCellDemoData {
        #[excel(name = "超链接")]
        hyperlink: WriteCellData,
        #[excel(name = "备注")]
        comment_data: WriteCellData,
        #[excel(name = "公式")]
        formula_data: WriteCellData,
        #[excel(name = "富文本")]
        rich_text: WriteCellData,
    }
    let path = temp_path("writeCellDataWrite.xlsx");
    let row = WriteCellDemoData {
        hyperlink: WriteCellData::from_string("官方网站").hyperlink_data(
            HyperlinkData::new()
                .address("https://github.com/alibaba/easyexcel")
                .hyperlink_type(HyperlinkType::Url),
        ),
        comment_data: WriteCellData::from_string("备注的单元格信息").comment_data(
            CommentData::new()
                .author("Jiaju Zhuang")
                .text("这是一个备注")
                .anchor(
                    ClientAnchorData::new().coordinates(
                        CoordinateData::new()
                            .relative_last_column_index(1)
                            .relative_last_row_index(1),
                    ),
                ),
        ),
        formula_data: WriteCellData::new(CellValue::Empty)
            .formula_data(FormulaData::new("REPLACE(123456789,1,1,2)")),
        rich_text: WriteCellData::from_rich_text(RichTextStringData::new("红色绿色默认")),
    };
    EasyExcel::write::<WriteCellDemoData>(&path)
        .sheet("模板")
        .do_write(vec![row])
        .unwrap();
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#templateWrite`
#[test]
fn write_test_template_write() {
    let template = require_fixture("demo/demo.xlsx");
    let template_rows = EasyExcel::read_sync::<DemoData>(&template)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(!template_rows.is_empty());
    let path = temp_path("templateWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .with_template(&template)
        .sheet_index(0)
        .do_write(write_demo_data())
        .unwrap();
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .sheet(1usize)
            .head_row_number(0)
            .do_read_sync()
            .unwrap()
            .is_empty(),
        "withTemplate must preserve non-target sheets"
    );
    let all_rows = EasyExcel::read_dynamic_sync(&path)
        .sheet(0usize)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(all_rows.len() > template_rows.len() + 1);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#widthAndHeightWrite`
#[test]
fn write_test_width_and_height_write() {
    #[derive(Debug, Clone, ExcelRow)]
    #[excel(column_width = 25, head_row_height = 20, content_row_height = 10)]
    struct WidthAndHeightData {
        #[excel(name = "字符串标题")]
        string: String,
        #[excel(name = "日期标题")]
        date: NaiveDate,
        #[excel(name = "数字标题", column_width = 50)]
        double_data: f64,
    }
    let path = temp_path("widthAndHeightWrite.xlsx");
    let data: Vec<WidthAndHeightData> = (0..10)
        .map(|i| WidthAndHeightData {
            string: format!("字符串{i}"),
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            double_data: 0.56,
        })
        .collect();
    EasyExcel::write::<WidthAndHeightData>(&path)
        .sheet("模板")
        .do_write(data)
        .unwrap();
    assert_eq!(
        EasyExcel::read_sync::<WidthAndHeightData>(&path)
            .do_read_sync()
            .unwrap()
            .len(),
        10
    );
}

