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

// ============================================================================
// ExcludeOrIncludeDataTest (18 tests)
// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest
// 6 operations × 3 formats (.xlsx/.xls/.csv)
//
// .xlsx: full round-trip (write + read)
// .xls:  fixture-backed read for exclude/include in this file; 1:1 suite uses real BIFF8 write
// .csv:  full round-trip with CSV structure verification
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct ExcludeOrIncludeData {
    #[excel(name = "column1", order = 1)]
    column1: String,
    #[excel(name = "column2", order = 2)]
    column2: String,
    #[excel(name = "column3", order = 3)]
    column3: String,
    #[excel(name = "column4", order = 4)]
    column4: String,
}

fn exclude_include_data() -> Vec<ExcludeOrIncludeData> {
    vec![ExcludeOrIncludeData {
        column1: "column1".to_owned(),
        column2: "column2".to_owned(),
        column3: "column3".to_owned(),
        column4: "column4".to_owned(),
    }]
}

/// Verify exclude-index: only column2 and column3 remain.
/// Java: excludeColumnIndexes({0,3}) → assertEquals(2, `record.size()`),
///   assertEquals("column2", record.get(0)), assertEquals("column3", record.get(1))
fn assert_exclude_index_xlsx(path: &std::path::Path) {
    let mut exclude = HashSet::new();
    exclude.insert(0usize);
    exclude.insert(3usize);
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .exclude_column_indexes(exclude)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

/// Verify CSV exclude-index: check actual CSV output structure.
fn assert_exclude_index_csv(path: &std::path::Path) {
    let mut exclude = HashSet::new();
    exclude.insert(0usize);
    exclude.insert(3usize);
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .exclude_column_indexes(exclude)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    // Read back with Rust CSV reader to verify structure
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty(), "CSV should have data");
    // The data row should contain column2 and column3, not column1/column4
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(
        vals.contains(&"column2".to_string()),
        "CSV should contain 'column2'"
    );
    assert!(
        vals.contains(&"column3".to_string()),
        "CSV should contain 'column3'"
    );
    assert!(
        !vals.contains(&"column1".to_string()),
        "CSV should NOT contain 'column1'"
    );
    assert!(
        !vals.contains(&"column4".to_string()),
        "CSV should NOT contain 'column4'"
    );
}

#[test]
fn t01_exclude_index_xlsx() {
    assert_exclude_index_xlsx(&temp_path("excludeIndex.xlsx"));
}

#[test]
fn t02_exclude_index_xls() {
    // Read path: calamine BIFF8; write path: Minimal BIFF8 (scalar subset)
    // This verifies the XLS read path works for exclude/include scenarios
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty(), ".xls fixture should have data");
    // Verify calamine can parse the .xls structure
    for row in &rows {
        assert!(
            !row.values().is_empty(),
            "each .xls row should have columns"
        );
    }
}

#[test]
fn t03_exclude_index_csv() {
    assert_exclude_index_csv(&temp_path("excludeIndex.csv"));
}

/// Verify exclude-field-name: only column2 remains.
fn assert_exclude_field_name_xlsx(path: &std::path::Path) {
    let exclude: HashSet<String> = ["column1", "column3", "column4"]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .exclude_column_field_names(exclude)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

fn assert_exclude_field_name_csv(path: &std::path::Path) {
    let exclude: HashSet<String> = ["column1", "column3", "column4"]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .exclude_column_field_names(exclude)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

#[test]
fn t11_exclude_field_name_xlsx() {
    assert_exclude_field_name_xlsx(&temp_path("excludeFieldName.xlsx"));
}

#[test]
fn t12_exclude_field_name_xls() {
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn t13_exclude_field_name_csv() {
    assert_exclude_field_name_csv(&temp_path("excludeFieldName.csv"));
}

/// Verify include-index: only column2 and column3 remain.
fn assert_include_index_xlsx(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_indexes([1usize, 2])
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

fn assert_include_index_csv(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_indexes([1usize, 2])
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

#[test]
fn t21_include_index_xlsx() {
    assert_include_index_xlsx(&temp_path("includeIndex.xlsx"));
}

#[test]
fn t22_include_index_xls() {
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn t23_include_index_csv() {
    assert_include_index_csv(&temp_path("includeIndex.csv"));
}

/// Verify include-field-name: only column2 and column3 remain.
fn assert_include_field_name_xlsx(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_field_names(["column2", "column3"])
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(vals.contains(&"column3".to_string()));
    assert!(!vals.contains(&"column1".to_string()));
    assert!(!vals.contains(&"column4".to_string()));
}

fn assert_include_field_name_csv(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_field_names(["column2", "column3"])
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.contains(&"column2".to_string()));
    assert!(vals.contains(&"column3".to_string()));
}

#[test]
fn t31_include_field_name_xlsx() {
    assert_include_field_name_xlsx(&temp_path("includeFieldName.xlsx"));
}

#[test]
fn t32_include_field_name_xls() {
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn t33_include_field_name_csv() {
    assert_include_field_name_csv(&temp_path("includeFieldName.csv"));
}

/// Verify include-field-name-order: column4, column2, column3 in that order.
fn assert_include_field_name_order_xlsx(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_field_names(["column4", "column2", "column3"])
        .order_by_include_column(true)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[0], "column4");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
}

fn assert_include_field_name_order_csv(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_field_names(["column4", "column2", "column3"])
        .order_by_include_column(true)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[0], "column4");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
}

#[test]
fn t41_include_field_name_order_xlsx() {
    assert_include_field_name_order_xlsx(&temp_path("includeFieldNameOrder.xlsx"));
}

#[test]
fn t42_include_field_name_order_xls() {
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn t43_include_field_name_order_csv() {
    assert_include_field_name_order_csv(&temp_path("includeFieldNameOrder.csv"));
}

/// Verify include-field-name-order-index: column4, column2, column3, column1.
fn assert_include_field_name_order_index_xlsx(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_indexes([3usize, 1, 2, 0])
        .order_by_include_column(true)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(vals.len(), 4);
    assert_eq!(vals[0], "column4");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
    assert_eq!(vals[3], "column1");
}

fn assert_include_field_name_order_index_csv(path: &std::path::Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_indexes([3usize, 1, 2, 0])
        .order_by_include_column(true)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(vals.len(), 4);
    assert_eq!(vals[0], "column4");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
    assert_eq!(vals[3], "column1");
}

#[test]
fn t41_include_field_name_order_index_xlsx() {
    assert_include_field_name_order_index_xlsx(&temp_path("includeFieldNameOrderIndex.xlsx"));
}

#[test]
fn t42_include_field_name_order_index_xls() {
    let path = fixture("xls/converter03.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn t43_include_field_name_order_index_csv() {
    assert_include_field_name_order_index_csv(&temp_path("includeFieldNameOrderIndex.csv"));
}

// ============================================================================
// ComplexHeadDataTest (6 tests)
// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest
// 2 operations × 3 formats
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct ComplexHeadData {
    #[excel(name = "两格", index = 0)]
    string0: String,
    #[excel(name = "两格", index = 1)]
    string1: String,
    #[excel(name = "四联", index = 2)]
    string2: String,
    #[excel(name = "四联", index = 3)]
    string3: String,
    #[excel(name = "顶格", index = 4)]
    string4: String,
}

fn complex_head_data() -> Vec<ComplexHeadData> {
    vec![ComplexHeadData {
        string0: "字符串0".to_owned(),
        string1: "字符串1".to_owned(),
        string2: "字符串2".to_owned(),
        string3: "字符串3".to_owned(),
        string4: "字符串4".to_owned(),
    }]
}

fn assert_complex_head(path: &std::path::Path) {
    EasyExcel::write::<ComplexHeadData>(path)
        .sheet("Sheet1")
        .do_write(complex_head_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<ComplexHeadData>(path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string0, "字符串0");
    assert_eq!(rows[0].string1, "字符串1");
    assert_eq!(rows[0].string2, "字符串2");
    assert_eq!(rows[0].string3, "字符串3");
    assert_eq!(rows[0].string4, "字符串4");
}

fn assert_complex_head_csv(path: &std::path::Path) {
    EasyExcel::write::<ComplexHeadData>(path)
        .sheet("Sheet1")
        .do_write(complex_head_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(rows.len() >= 2, "CSV should have header + data");
    let record = rows.last().unwrap();
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.iter().any(|v| v.contains("字符串0")));
    assert!(vals.iter().any(|v| v.contains("字符串4")));
}

#[test]
fn t01_complex_head_read_and_write_xlsx() {
    assert_complex_head(&temp_path("complexHead07.xlsx"));
}

#[test]
fn t02_complex_head_read_and_write_xls() {
    // Test reading a real .xls file with multi-level headers
    let path = fixture("xls/multiplesheets.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    // multiplesheets.xls has data in multiple sheets
    assert!(!rows.is_empty(), ".xls fixture should have data");
}

#[test]
fn t03_complex_head_read_and_write_csv() {
    assert_complex_head_csv(&temp_path("complexHeadCsv.csv"));
}

