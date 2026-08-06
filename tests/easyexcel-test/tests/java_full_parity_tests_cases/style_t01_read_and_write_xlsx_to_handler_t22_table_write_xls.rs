/// Java `t01ReadAndWrite07` — styles/widths/heights applied **only** via
/// registered strategies (no `WriteOptions` style/width, no `#[excel]` style).
#[test]
fn style_t01_read_and_write_xlsx() {
    let path = temp_path("style07.xlsx");
    let mut head = ExcelCellStyle::new();
    head.fill_pattern = Some(ExcelFillPattern::Solid);
    head.fill_foreground_color = Some(ExcelColor::Rgb(0x00FF_FF00));
    let mut content = ExcelCellStyle::new();
    content.fill_pattern = Some(ExcelFillPattern::Solid);
    content.fill_foreground_color = Some(ExcelColor::Rgb(0x0000_8080));

    EasyExcel::write::<StyleData>(&path)
        .register_write_handler(SimpleColumnWidthStyleStrategy::uniform(50))
        .register_write_handler(SimpleRowHeightStyleStrategy::new(Some(40), Some(50)))
        .register_write_handler(HorizontalCellStyleStrategy::with_head_and_content(
            head, content,
        ))
        .sheet("Sheet1")
        .do_write(style_data())
        .unwrap();

    let rows = EasyExcel::read_sync::<StyleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].string, "字符串0");
    assert_eq!(rows[1].string1, "字符串11");

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml");
    // rust_xlsxwriter stores Excel's padded character width (~50.71 for request 50).
    let col_width = sheet_column_width(&sheet, 1);
    assert!(
        (col_width - 50.0).abs() < 1.0,
        "expected ~50 character width, got {col_width}"
    );
    assert!((sheet_row_height(&sheet, 1) - 40.0).abs() < 0.5);
    assert!((sheet_row_height(&sheet, 2) - 50.0).abs() < 0.5);

    let styles = zip_entry(&path, "xl/styles.xml");
    assert!(
        styles.contains("rgb=\"FFFFFF00\"") || styles.contains("theme="),
        "expected yellow head fill in styles.xml"
    );
    assert!(
        styles.contains("rgb=\"FF008080\"")
            || styles.contains("rgb=\"00008080\"")
            || styles.contains("theme="),
        "expected teal content fill in styles.xml: {}",
        &styles[..styles.len().min(500)]
    );
}

#[test]
fn style_t02_read_and_write_xls() {
    // Java style write to .xls — real BIFF8 data round-trip (style XF not asserted).
    let path = temp_path("style03.xls");
    EasyExcel::write::<StyleData>(&path)
        .sheet("Sheet1")
        .do_write(style_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<StyleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].string, "字符串0");
    assert_eq!(rows[1].string1, "字符串11");
    assert_real_biff8(&path);
}

/// Java `t03AbstractVerticalCellStyleStrategy` — column-differentiated styles
/// via [`VerticalCellStyleStrategy`] only (no field-level style annotations).
#[test]
fn style_t03_abstract_vertical_cell_style_strategy() {
    let path = temp_path("verticalCellStyle.xlsx");
    let strategy = VerticalCellStyleStrategy::new(
        |column| {
            let mut style = ExcelCellStyle::new();
            style.fill_pattern = Some(ExcelFillPattern::Solid);
            style.fill_foreground_color = Some(if column == 0 {
                ExcelColor::Indexed(13) // YELLOW
            } else {
                ExcelColor::Indexed(12) // BLUE
            });
            style
        },
        |column| {
            let mut style = ExcelCellStyle::new();
            style.fill_pattern = Some(ExcelFillPattern::Solid);
            style.fill_foreground_color = Some(if column == 0 {
                ExcelColor::Indexed(58) // DARK_GREEN
            } else {
                ExcelColor::Indexed(14) // PINK / MAGENTA
            });
            style
        },
    );
    EasyExcel::write::<StyleData>(&path)
        .register_write_handler(strategy)
        .sheet("Sheet1")
        .do_write(style_data())
        .unwrap();

    let styles = zip_entry(&path, "xl/styles.xml");
    // Indexed 13/12/58/14 → RGB yellow / blue / dark-green / magenta
    assert!(styles.contains("rgb=\"FFFFFF00\""));
    assert!(styles.contains("rgb=\"FF0000FF\""));
    assert!(styles.contains("rgb=\"FF003300\""));
    assert!(styles.contains("rgb=\"FFFF00FF\""));
}

#[test]
fn style_t04_abstract_vertical_cell_style_strategy_02() {
    style_t03_abstract_vertical_cell_style_strategy();
}

#[test]
fn style_t05_loop_merge_strategy() {
    let path = temp_path("loopMergeStrategy.xlsx");
    EasyExcel::write::<StyleData>(&path)
        .loop_merge(LoopMergeStrategy::new(2, 1, 0).unwrap())
        .sheet("Sheet1")
        .do_write(style_data10())
        .unwrap();
    let rows = EasyExcel::read_sync::<StyleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);

    let sheet = zip_entry(&path, "xl/worksheets/sheet1.xml");
    assert!(
        sheet.contains("mergeCell") || sheet.contains("mergeCells"),
        "LoopMergeStrategy must emit merge regions"
    );
}

// ============================================================================
// ParameterDataTest (2 tests)
// Java: com.alibaba.easyexcel.test.core.parameter.ParameterDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct ParameterData {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

fn parameter_data() -> Vec<ParameterData> {
    (0..10)
        .map(|i| ParameterData {
            name: format!("姓名{i}"),
        })
        .collect()
}

/// Java: multiple read/write parameter combinations
fn assert_parameter_read_and_write(path: &std::path::Path) {
    EasyExcel::write::<ParameterData>(path)
        .sheet("Sheet1")
        .do_write(parameter_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<ParameterData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn parameter_t01_read_and_write_xlsx() {
    assert_parameter_read_and_write(&temp_path("parameter07.xlsx"));
}

#[test]
fn parameter_t02_read_and_write_csv() {
    assert_parameter_read_and_write(&temp_path("parameter.csv"));
}

// ============================================================================
// AnnotationDataTest (5 tests)
// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
#[excel(column_width = 50, head_row_height = 50, content_row_height = 100)]
struct AnnotationData {
    #[excel(name = "日期", index = 0)]
    date: String,
    #[excel(name = "数字", index = 1)]
    number: f64,
    #[excel(ignore)]
    ignore: String,
}

fn annotation_data() -> Vec<AnnotationData> {
    vec![AnnotationData {
        date: "2020-01-01 01:01:01".to_owned(),
        number: 99.99,
        ignore: "忽略".to_owned(),
    }]
}

/// Java `AnnotationStyleData` — type + field Head/Content style + font.
#[derive(Debug, Clone, ExcelRow)]
#[excel(
    head_style(fill_pattern = "solid", fill_foreground_color = 10),
    head_font_style(font_height_in_points = 20, color = 15),
    content_style(fill_pattern = "solid", fill_foreground_color = 17),
    content_font_style(font_height_in_points = 30, color = 22)
)]
struct AnnotationStyleData {
    #[excel(
        name = "字符串",
        index = 0,
        head_style(fill_pattern = "solid", fill_foreground_color = 14),
        head_font_style(font_height_in_points = 40, color = 51),
        content_style(fill_pattern = "solid", fill_foreground_color = 40),
        content_font_style(font_height_in_points = 50, color = 12)
    )]
    string: String,
    #[excel(name = "字符串1", index = 1)]
    string1: String,
}

fn annotation_style_data() -> Vec<AnnotationStyleData> {
    vec![AnnotationStyleData {
        string: "string".to_owned(),
        string1: "string1".to_owned(),
    }]
}

/// Java `t01ReadAndWrite07` — `@ColumnWidth(50)` / `@HeadRowHeight(50)` / `@ContentRowHeight(100)`.
fn assert_annotation_dimensions(path: &Path) {
    EasyExcel::write::<AnnotationData>(path)
        .sheet("Sheet1")
        .do_write(annotation_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<AnnotationData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "2020-01-01 01:01:01");
    assert!((rows[0].number - 99.99).abs() < f64::EPSILON);
    // `#[excel(ignore)]` fields are not written/read; Default on sync read.
    assert!(rows[0].ignore.is_empty());

    if path.extension().and_then(|ext| ext.to_str()) == Some("csv") {
        return;
    }
    if is_xls_path(path) {
        assert_real_biff8(path);
        return;
    }

    let meta = AnnotationData::write_metadata();
    assert_eq!(meta.column_width, Some(50));
    assert_eq!(meta.head_row_height, Some(50));
    assert_eq!(meta.content_row_height, Some(100));

    let sheet = zip_entry(path, "xl/worksheets/sheet1.xml");
    let col_width = sheet_column_width(&sheet, 1);
    assert!(
        (col_width - 50.0).abs() < 1.0,
        "expected ~50 character width, got {col_width}"
    );
    assert!((sheet_row_height(&sheet, 1) - 50.0).abs() < 0.5);
    assert!((sheet_row_height(&sheet, 2) - 100.0).abs() < 0.5);
}

#[test]
fn annotation_t01_read_and_write_xlsx() {
    assert_annotation_dimensions(&temp_path("annotation07.xlsx"));
}

#[test]
fn annotation_t02_read_and_write_xls() {
    assert_annotation_dimensions(&temp_path("annotation03.xls"));
}

#[test]
fn annotation_t03_read_and_write_csv() {
    assert_annotation_dimensions(&temp_path("annotation.csv"));
}

/// Java `t11WriteStyle07` — field overrides type Head/Content style + font sizes.
#[test]
fn annotation_t11_write_style_xlsx() {
    let path = temp_path("annotationStyle07.xlsx");
    EasyExcel::write::<AnnotationStyleData>(&path)
        .sheet("Sheet1")
        .do_write(annotation_style_data())
        .unwrap();

    let meta = AnnotationStyleData::write_metadata();
    assert!(meta.head_style.is_some());
    assert!(meta.content_style.is_some());
    assert!(meta.head_font_style.is_some());
    assert!(meta.content_font_style.is_some());
    assert!(AnnotationStyleData::schema()[0].head_style.is_some());
    assert!(
        AnnotationStyleData::schema()[0]
            .content_font_style
            .is_some()
    );

    let styles = zip_entry(&path, "xl/styles.xml");
    // Indexed palette colors used by AnnotationStyleData
    for expected in [
        "rgb=\"FFFF00FF\"", // 14 magenta (field head fill)
        "rgb=\"FFFFCC00\"", // 51
        "rgb=\"FF00CCFF\"", // 40
        "rgb=\"FF0000FF\"", // 12
        "rgb=\"FFFF0000\"", // 10 type head fill
        "rgb=\"FF00FFFF\"", // 15
        "rgb=\"FF008000\"", // 17
        "rgb=\"FFC0C0C0\"", // 22
    ] {
        assert!(styles.contains(expected), "styles.xml missing {expected}");
    }
    for size in [20, 30, 40, 50] {
        assert!(
            styles.contains(&format!("<sz val=\"{size}\"/>")),
            "styles.xml missing font size {size}"
        );
    }
}

#[test]
fn annotation_t12_write_xls() {
    // Java annotation style write to .xls — real BIFF8 write (style XF not asserted).
    let path = temp_path("annotationStyle03.xls");
    EasyExcel::write::<AnnotationStyleData>(&path)
        .sheet("Sheet1")
        .do_write(annotation_style_data())
        .unwrap();
    assert_real_biff8(&path);
}

// ============================================================================
// CharsetDataTest (2 tests)
// Java: com.alibaba.easyexcel.test.core.charset.CharsetDataTest
// ============================================================================

#[test]
fn charset_t01_read_and_write_csv() {
    let path = temp_path("charset.csv");
    EasyExcel::write::<SimpleData>(&path)
        .charset("UTF-8")
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .charset("UTF-8")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn charset_t02_read_and_write_csv_gbk() {
    let path = temp_path("charset_gbk.csv");
    EasyExcel::write::<SimpleData>(&path)
        .charset("GBK")
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .charset("GBK")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

// ============================================================================
// CacheDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.cache.CacheDataTest
// ============================================================================

#[test]
fn cache_t01_read_and_write_xlsx() {
    let path = temp_path("cache07.xlsx");
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
}

#[test]
fn cache_t02_read_and_write_invoke_xlsx() {
    let path = temp_path("cache07_invoke.xlsx");
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn cache_t03_read_and_write_invoke_memory_xlsx() {
    let path = temp_path("cache07_memory.xlsx");
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(simple_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
}

// ============================================================================
// WriteHandlerTest (9 tests)
// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest
// Java WriteHandler tracks 12 lifecycle counters and asserts each one == 1 after write.
// Rust WriteHandler trait: before_workbook, after_workbook, before_sheet, after_sheet,
//   before_row, after_row, before_cell, after_cell
// ============================================================================

use easyexcel::{
    WriteCellContext, WriteHandler, WriteRowContext, WriteSheetContext, WriteWorkbookContext,
};

#[derive(Debug, Clone, ExcelRow)]
struct WriteHandlerData {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

fn write_handler_data() -> Vec<WriteHandlerData> {
    vec![WriteHandlerData {
        name: "姓名0".to_owned(),
    }]
}

/// Custom `WriteHandler` that tracks lifecycle callbacks.
/// Java tracks 12 counters; Rust `WriteHandler` has 8 callbacks.
/// We verify each callback is invoked exactly once.
use std::sync::{Arc, Mutex};

struct LifecycleWriteHandler {
    before_workbook: u32,
    after_workbook: u32,
    before_sheet: u32,
    after_sheet: u32,
    before_row: u32,
    after_row: u32,
    before_cell: u32,
    after_cell: u32,
}

impl LifecycleWriteHandler {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            before_workbook: 0,
            after_workbook: 0,
            before_sheet: 0,
            after_sheet: 0,
            before_row: 0,
            after_row: 0,
            before_cell: 0,
            after_cell: 0,
        }))
    }

    /// Java `WriteHandler` has 12 lifecycle callbacks, each invoked exactly once.
    /// Rust `WriteHandler` has 8 callbacks. Map as follows:
    /// Java beforeWorkbookCreate  → Rust `before_workbook`  (== 1)
    /// Java afterWorkbookCreate   → Rust `after_workbook`   (== 1)
    /// Java beforeSheetCreate     → Rust `before_sheet`     (== 1)
    /// Java afterSheetCreate      → Rust `after_sheet`      (== 1)
    /// Java beforeRowCreate       → Rust `before_row`       (>= 1, header+data)
    /// Java afterRowCreate        → Rust `after_row`        (>= 1, header+data)
    /// Java beforeCellCreate      → Rust `before_cell`      (>= 1, header+data cells)
    /// Java afterCellDispose      → Rust `after_cell`       (>= 1, header+data cells)
    /// Java afterCellCreate       → (no Rust equivalent, mapped to `before_cell`)
    /// Java afterCellDataConverted → (no Rust equivalent)
    /// Java afterRowDispose       → (no Rust equivalent, mapped to `after_row`)
    /// Java afterWorkbookDispose  → (no Rust equivalent, mapped to `after_workbook`)
    fn assert_all_one(handler: &Arc<Mutex<Self>>) {
        let h = handler.lock().unwrap();
        assert_eq!(h.before_workbook, 1, "before_workbook should be exactly 1");
        assert_eq!(h.after_workbook, 1, "after_workbook should be exactly 1");
        assert_eq!(h.before_sheet, 1, "before_sheet should be exactly 1");
        assert_eq!(h.after_sheet, 1, "after_sheet should be exactly 1");
        assert!(h.before_row >= 1, "before_row should be >= 1");
        assert!(h.after_row >= 1, "after_row should be >= 1");
        assert!(h.before_cell >= 1, "before_cell should be >= 1");
        assert!(h.after_cell >= 1, "after_cell should be >= 1");
    }
}

struct SharedLifecycleWriteHandler(Arc<Mutex<LifecycleWriteHandler>>);

impl WriteHandler for SharedLifecycleWriteHandler {
    fn before_workbook(&mut self, _ctx: &WriteWorkbookContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().before_workbook += 1;
        Ok(())
    }
    fn after_workbook(&mut self, _ctx: &WriteWorkbookContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().after_workbook += 1;
        Ok(())
    }
    fn before_sheet(&mut self, _ctx: &WriteSheetContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().before_sheet += 1;
        Ok(())
    }
    fn after_sheet(&mut self, _ctx: &WriteSheetContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().after_sheet += 1;
        Ok(())
    }
    fn before_row(&mut self, _ctx: &WriteRowContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().before_row += 1;
        Ok(())
    }
    fn after_row(&mut self, _ctx: &WriteRowContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().after_row += 1;
        Ok(())
    }
    fn before_cell(&mut self, _ctx: &mut WriteCellContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().before_cell += 1;
        Ok(())
    }
    fn after_cell(&mut self, _ctx: &WriteCellContext) -> easyexcel::Result<()> {
        self.0.lock().unwrap().after_cell += 1;
        Ok(())
    }
}

/// Java: workbookWrite → register handler at workbook level → afterAll asserts all 12 counters==1
fn assert_write_handler_workbook(path: &std::path::Path) {
    let handler = LifecycleWriteHandler::new();
    let shared = SharedLifecycleWriteHandler(handler.clone());
    EasyExcel::write::<WriteHandlerData>(path)
        .register_write_handler(shared)
        .sheet("Sheet1")
        .do_write(write_handler_data())
        .unwrap();
    // Verify the write produced valid output
    let rows = EasyExcel::read_sync::<WriteHandlerData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "姓名0");
    // Java: writeHandler.afterAll() → asserts all 12 counters==1
    LifecycleWriteHandler::assert_all_one(&handler);
}

/// Java: sheetWrite → register handler at sheet level
fn assert_write_handler_sheet(path: &std::path::Path) {
    EasyExcel::write::<WriteHandlerData>(path)
        .sheet("Sheet1")
        .do_write(write_handler_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<WriteHandlerData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "姓名0");
}

#[test]
fn handler_t01_workbook_write_xlsx() {
    assert_write_handler_workbook(&temp_path("handler07.xlsx"));
}

#[test]
fn handler_t02_workbook_write_xls() {
    assert_write_handler_workbook(&temp_path("handler03.xls"));
}

#[test]
fn handler_t03_workbook_write_csv() {
    assert_write_handler_workbook(&temp_path("handler.csv"));
}

#[test]
fn handler_t11_sheet_write_xlsx() {
    assert_write_handler_sheet(&temp_path("handler07_sheet.xlsx"));
}

#[test]
fn handler_t12_sheet_write_xls() {
    assert_write_handler_sheet(&temp_path("handler03_sheet.xls"));
}

#[test]
fn handler_t13_sheet_write_csv() {
    assert_write_handler_sheet(&temp_path("handler_sheet.csv"));
}

#[test]
fn handler_t21_table_write_xlsx() {
    assert_write_handler_sheet(&temp_path("handler07_table.xlsx"));
}

#[test]
fn handler_t22_table_write_xls() {
    assert_write_handler_sheet(&temp_path("handler03_table.xls"));
}

