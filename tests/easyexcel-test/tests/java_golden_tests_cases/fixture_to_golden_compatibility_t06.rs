fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn golden_artifact(name: &str) -> PathBuf {
    golden_dir().join("artifacts").join(name)
}

/// JSON schema written by `scripts/java-golden-export` / `export-java-golden.sh`.
#[derive(Debug, Deserialize)]
struct GoldenExpectation {
    /// Java test class#method that owns this fixture assertion.
    source: String,
    /// Relative fixture path under `tests/fixtures` or `artifacts/...`.
    #[serde(default)]
    fixture: String,
    /// Sheet index used by the Java export.
    #[serde(default)]
    sheet_index: usize,
    /// Optional sheet name (takes precedence over `sheet_index` when set).
    #[serde(default)]
    sheet_name: Option<String>,
    /// `headRowNumber` used by the Java export.
    #[serde(default)]
    head_row_number: u32,
    /// Optional workbook password (encrypt scenarios).
    #[serde(default)]
    password: Option<String>,
    /// Optional CSV charset (e.g. `GBK` / `UTF-8`).
    #[serde(default)]
    charset: Option<String>,
    /// Number of rows returned by Java `doReadSync`.
    row_count: usize,
    /// Key cells as `"row.col" → display text` (Java STRING mode).
    #[serde(default)]
    cells: BTreeMap<String, String>,
    /// Full sheet rows as display strings (optional; compared when present).
    #[serde(default)]
    rows: Vec<Vec<String>>,
}

/// Load a golden file; **fails** if missing or invalid JSON.
fn load_golden(name: &str) -> GoldenExpectation {
    let path = golden_dir().join(name);
    assert!(
        path.is_file(),
        "required Java golden missing (run scripts/export-java-golden.sh): {}",
        path.display()
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("failed to read golden {}: {e}", path.display());
    });
    serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("invalid golden JSON {}: {e}", path.display());
    })
}

/// Resolve the file path referenced by a golden (`fixtures/...` or `artifacts/...`).
fn resolve_golden_path(golden: &GoldenExpectation) -> PathBuf {
    let rel = golden.fixture.as_str();
    assert!(
        !rel.is_empty(),
        "golden has empty fixture field (source={}); re-run scripts/export-java-golden.sh",
        golden.source
    );
    if let Some(rest) = rel.strip_prefix("artifacts/") {
        let path = golden_artifact(rest);
        assert!(
            path.is_file(),
            "required Java write artifact missing (run scripts/export-java-golden.sh): {}",
            path.display()
        );
        return path;
    }
    let path = fixture(rel);
    assert!(
        path.is_file(),
        "required Java fixture missing: {}",
        path.display()
    );
    path
}

/// Convert a Rust `DynamicValue` to display text comparable with Java STRING mode.
fn display_text(value: &DynamicValue) -> String {
    match value {
        DynamicValue::Null => String::new(),
        DynamicValue::String(s) => s.clone(),
        DynamicValue::ActualData(cv) => cv.as_text(),
        DynamicValue::ReadCellData(cell) => cell.display_value().to_owned(),
    }
}

/// Read a path with the same sheet / head / password / charset options as the Java golden export.
fn read_display_rows(path: &Path, golden: &GoldenExpectation) -> Vec<DynamicRow> {
    let mut builder = EasyExcel::read_sync::<DynamicRow>(path)
        .head_row_number(golden.head_row_number)
        .read_default_return(ReadDefaultReturn::String);
    if let Some(password) = golden.password.as_deref()
        && !password.is_empty()
    {
        builder = builder.password(password);
    }
    if let Some(charset) = golden.charset.as_deref()
        && !charset.is_empty()
    {
        builder = builder.charset(charset);
    }
    builder = match golden.sheet_name.as_deref() {
        Some(name) if !name.is_empty() => builder.sheet(name),
        _ => builder.sheet(golden.sheet_index),
    };
    builder
        .do_read_sync()
        .unwrap_or_else(|e| panic!("Rust read failed for {}: {e}", path.display()))
}

/// Assert Rust rows match golden `row_count`, key `cells`, and full `rows` when present.
/// Date columns are compared like any other STRING cell (no soft-skip).
fn assert_matches_golden(golden: &GoldenExpectation, rows: &[DynamicRow]) {
    assert_eq!(
        rows.len(),
        golden.row_count,
        "row_count mismatch vs Java golden ({})",
        golden.source
    );

    for (coord, expected) in &golden.cells {
        let (row_idx, col_idx) = parse_coord(coord);
        let actual = rows
            .get(row_idx)
            .and_then(|r| r.get(col_idx))
            .map(display_text)
            .unwrap_or_default();
        assert_eq!(
            actual, *expected,
            "cell {coord} mismatch vs Java golden ({})",
            golden.source
        );
    }

    if golden.rows.is_empty() {
        return;
    }
    assert_eq!(
        rows.len(),
        golden.rows.len(),
        "full rows length mismatch vs Java golden ({})",
        golden.source
    );
    for (r, expected_row) in golden.rows.iter().enumerate() {
        for (c, expected_cell) in expected_row.iter().enumerate() {
            let actual = rows
                .get(r)
                .and_then(|row| row.get(c))
                .map(display_text)
                .unwrap_or_default();
            // Trailing empty columns may be omitted on the sparse side; treat missing as "".
            if actual.is_empty() && expected_cell.is_empty() {
                continue;
            }
            assert_eq!(
                actual, *expected_cell,
                "rows[{r}][{c}] mismatch vs Java golden ({})",
                golden.source
            );
        }
    }
}

/// Load golden, resolve path, read with Rust, assert full STRING match.
fn assert_golden_file(golden_name: &str) {
    let golden = load_golden(golden_name);
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Parse `"row.col"` coordinate used in golden `cells`.
fn parse_coord(coord: &str) -> (usize, usize) {
    let mut parts = coord.split('.');
    let row = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("bad golden cell coord: {coord}"));
    let col = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("bad golden cell coord: {coord}"));
    (row, col)
}

/// Sample rows matching Java `SimpleDataTest#data()` (姓名0..9).
fn simple_names() -> Vec<String> {
    (0..10).map(|i| format!("姓名{i}")).collect()
}

/// Java CompatibilityTest#t02 — fixed expected cell + full golden file.
#[test]
fn golden_compatibility_t02() {
    let path = fixture("compatibility/t02.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    let golden = load_golden("compatibility_t02.expected.json");
    assert!(
        golden.source.contains("CompatibilityTest#t02"),
        "unexpected source: {}",
        golden.source
    );

    // Hard-coded Java assertion (CompatibilityTest#t02) — keep alongside golden.
    let actual_data_rows = EasyExcel::read_sync::<DynamicRow>(&path)
        .head_row_number(0)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()
        .unwrap();
    assert_eq!(actual_data_rows.len(), 3);
    let val = match actual_data_rows[2].get(2).unwrap() {
        DynamicValue::ActualData(CellValue::String(s)) | DynamicValue::String(s) => s.as_str(),
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(val, "1，2-戊二醇");

    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Java CompatibilityTest#t04 — merged-cell fixture row count + key cell + golden.
#[test]
fn golden_compatibility_t04() {
    let path = fixture("compatibility/t04.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    let golden = load_golden("compatibility_t04.expected.json");
    assert!(
        golden.source.contains("CompatibilityTest#t04"),
        "unexpected source: {}",
        golden.source
    );

    let actual_data_rows = EasyExcel::read_sync::<DynamicRow>(&path)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()
        .unwrap();
    assert_eq!(actual_data_rows.len(), 56);
    let val = match actual_data_rows[0].get(5).unwrap() {
        DynamicValue::ActualData(CellValue::String(s)) | DynamicValue::String(s) => s.as_str(),
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(val, "QQSJK28F152A012242S0081");

    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// CompatibilityTest#t01 — Java .xls fixture STRING read对照.
#[test]
// 语义敏感：golden fixture 文件名规范为小写 `.xls`，精确匹配即测试意图。
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn golden_compatibility_t01_xls() {
    let golden = load_golden("compatibility_t01_xls.expected.json");
    assert!(
        golden.source.contains("CompatibilityTest#t01"),
        "unexpected source: {}",
        golden.source
    );
    // golden 文件名为固定小写；Path::extension 比较保持大小写敏感语义
    assert!(
        std::path::Path::new(&golden.fixture)
            .extension()
            .is_some_and(|ext| ext == "xls"),
        "expected .xls fixture, got {}",
        golden.fixture
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// BOM fixtures must match Java `BomDataTest` expectations + Java golden JSON.
#[test]
fn golden_bom_office_csv() {
    #[derive(Debug, Clone, easyexcel::ExcelRow)]
    struct BomData {
        #[excel(name = "姓名")]
        name: String,
        #[excel(name = "年纪")]
        age: i64,
    }
    let path = fixture("bom/office_bom.csv");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    let typed = EasyExcel::read_sync::<BomData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(typed.len(), 10);
    assert_eq!(typed[0].name, "姓名0");
    assert_eq!(typed[0].age, 20);

    let golden = load_golden("bom_office_bom.expected.json");
    assert!(
        golden.source.contains("BomDataTest"),
        "unexpected source: {}",
        golden.source
    );
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `no_bom.csv` — same logical content as `office_bom` without UTF-8 BOM.
#[test]
fn golden_bom_no_bom_csv() {
    assert_golden_file("bom_no_bom.expected.json");
}

/// Java ReadTest#simpleRead — demo.xlsx first sheet vs Java golden (date col included).
#[test]
fn golden_demo_demo_sheet0() {
    let golden = load_golden("demo_demo_sheet0.expected.json");
    assert!(
        golden
            .source
            .contains("com.alibaba.easyexcel.test.demo.read.ReadTest"),
        "unexpected source: {}",
        golden.source
    );
    assert!(
        golden.cells.contains_key("1.1"),
        "demo golden must include date cell 1.1 for STRING alignment"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// demo.csv — full STRING-mode display vs Java golden including date column.
#[test]
fn golden_demo_demo_csv() {
    let golden = load_golden("demo_demo_csv.expected.json");
    assert!(
        golden.cells.contains_key("1.1"),
        "demo csv golden must include date cell 1.1"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// SimpleDataTest#t21 — sheet name `simple` on simple07.xlsx.
#[test]
fn golden_simple_simple07_sheet_name() {
    assert_golden_file("simple_simple07.expected.json");
}

/// Java-written `SimpleData` xlsx artifact must match golden; Rust write+read must match too.
#[test]
fn golden_simple_data_xlsx_write() {
    #[derive(Debug, Clone, easyexcel::ExcelRow)]
    struct SimpleData {
        #[excel(name = "姓名")]
        name: String,
    }

    let golden = load_golden("simple_data.expected.json");
    assert!(
        golden.source.contains("SimpleDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert_eq!(golden.row_count, 10);

    let java_artifact = golden_artifact("simple_data.xlsx");
    assert!(
        java_artifact.is_file(),
        "required Java write artifact missing (run scripts/export-java-golden.sh): {}",
        java_artifact.display()
    );
    let java_rows = read_display_rows(&java_artifact, &golden);
    assert_matches_golden(&golden, &java_rows);

    let path = tempfile::tempdir()
        .unwrap()
        .keep()
        .join("simple_golden.xlsx");
    let data: Vec<SimpleData> = simple_names()
        .into_iter()
        .map(|name| SimpleData { name })
        .collect();
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let rust_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &rust_rows);
}

/// Java-written `SimpleData` csv artifact + Rust csv write/read vs golden.
#[test]
fn golden_simple_data_csv_write() {
    #[derive(Debug, Clone, easyexcel::ExcelRow)]
    struct SimpleData {
        #[excel(name = "姓名")]
        name: String,
    }

    let golden = load_golden("simple_data_csv.expected.json");
    assert!(
        golden.source.contains("SimpleDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert_eq!(golden.row_count, 10);

    let java_artifact = golden_artifact("simple_data.csv");
    assert!(
        java_artifact.is_file(),
        "required Java write artifact missing (run scripts/export-java-golden.sh): {}",
        java_artifact.display()
    );
    let java_rows = read_display_rows(&java_artifact, &golden);
    assert_matches_golden(&golden, &java_rows);

    let path = tempfile::tempdir()
        .unwrap()
        .keep()
        .join("simple_golden.csv");
    let data: Vec<SimpleData> = simple_names()
        .into_iter()
        .map(|name| SimpleData { name })
        .collect();
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let rust_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &rust_rows);
}

/// Java-written `SimpleData` `.xls` artifact — Rust **read**对照 only (write unsupported).
#[test]
fn golden_simple_data_xls_read() {
    let golden = load_golden("simple_data_xls.expected.json");
    assert!(
        golden.source.contains("SimpleDataTest"),
        "unexpected source: {}",
        golden.source
    );
    // golden 文件名为固定小写；Path::extension 比较保持大小写敏感语义
    assert!(
        std::path::Path::new(&golden.fixture)
            .extension()
            .is_some_and(|ext| ext == "xls"),
        "expected .xls artifact, got {}",
        golden.fixture
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Converter fixture converter07.xlsx — STRING read including date columns.
#[test]
fn golden_converter_converter07() {
    let golden = load_golden("converter_converter07.expected.json");
    assert!(
        golden.source.contains("ConverterDataTest"),
        "unexpected source: {}",
        golden.source
    );
    // Date columns (e.g. 0.12 / 0.13) must be present and compared.
    assert!(
        golden.cells.contains_key("0.12") && golden.cells.contains_key("0.13"),
        "converter golden must include date STRING cells"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Converter `.xls` fixture — full-table STRING including short dates (`xls_display`).
#[test]
fn golden_converter_converter03_xls() {
    let golden = load_golden("converter_converter03_xls.expected.json");
    // golden 文件名为固定小写；Path::extension 比较保持大小写敏感语义
    assert!(
        std::path::Path::new(&golden.fixture)
            .extension()
            .is_some_and(|ext| ext == "xls"),
        "expected .xls fixture, got {}",
        golden.fixture
    );
    assert_eq!(
        golden.cells.get("0.12").map(String::as_str),
        Some("2020-1-1 1:01")
    );
    assert_eq!(
        golden.cells.get("0.13").map(String::as_str),
        Some("2020-01-01 01:01:01")
    );
    assert!(
        !golden.rows.is_empty(),
        "converter03.xls must be full-table STRING"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Converter csv fixture.
#[test]
fn golden_converter_converter_csv() {
    assert_golden_file("converter_converter_csv.expected.json");
}

/// Java `ConverterWriteData` artifact — date / localDate / localDateTime STRING对齐.
#[test]
fn golden_converter_write() {
    let golden = load_golden("converter_write.expected.json");
    assert!(
        golden.cells.get("0.0").map(String::as_str) == Some("2020-01-01 01:01:01"),
        "converter write golden date cell missing/mismatched"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// FillTest#simpleFill — Java filled artifact vs Rust STRING read.
#[test]
fn golden_fill_simple() {
    let golden = load_golden("fill_simple.expected.json");
    assert!(
        golden.source.contains("FillTest"),
        "unexpected source: {}",
        golden.source
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Multi-sheet xlsx — sheet 0.
#[test]
fn golden_multiplesheets_sheet0() {
    assert_golden_file("multiplesheets_sheet0.expected.json");
}

/// Multi-sheet xlsx — sheet 1.
#[test]
fn golden_multiplesheets_sheet1() {
    assert_golden_file("multiplesheets_sheet1.expected.json");
}

/// Multi-sheet `.xls` — sheet 0 Rust read.
#[test]
fn golden_multiplesheets_xls_sheet0() {
    assert_golden_file("multiplesheets_xls_sheet0.expected.json");
}

/// Multi-sheet `.xls` — sheet 1 Rust read.
#[test]
fn golden_multiplesheets_xls_sheet1() {
    assert_golden_file("multiplesheets_xls_sheet1.expected.json");
}

/// CompatibilityTest#t03 — sparse null leading columns.
#[test]
fn golden_compatibility_t03() {
    assert_golden_file("compatibility_t03.expected.json");
}

/// CompatibilityTest#t05 — date rounding full STRING cells.
#[test]
fn golden_compatibility_t05_dates() {
    let golden = load_golden("compatibility_t05.expected.json");
    assert!(
        golden.cells.get("0.0").map(String::as_str) == Some("2023-01-01 00:00:00"),
        "t05 date STRING cell missing"
    );
    assert!(
        golden.cells.get("3.0").map(String::as_str) == Some("2023-01-01 00:00:01"),
        "t05 rounded second cell missing"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// CompatibilityTest#t06 — numeric precision STRING.
#[test]
fn golden_compatibility_t06() {
    let golden = load_golden("compatibility_t06.expected.json");
    assert_eq!(golden.cells.get("0.2").map(String::as_str), Some("2087.03"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

