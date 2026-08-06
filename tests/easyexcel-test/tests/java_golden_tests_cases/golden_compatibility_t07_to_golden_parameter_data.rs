/// CompatibilityTest#t07 — STRING "24.20" + trailing-space accounting (`-1.07 `).
#[test]
fn golden_compatibility_t07() {
    let golden = load_golden("compatibility_t07.expected.json");
    assert_eq!(golden.cells.get("0.11").map(String::as_str), Some("24.20"));
    assert_eq!(golden.cells.get("0.12").map(String::as_str), Some("-1.07 "));
    assert!(!golden.rows.is_empty(), "t07 must be full-table STRING");
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// CompatibilityTest#t09 — sharedStrings escape.
#[test]
fn golden_compatibility_t09() {
    let golden = load_golden("compatibility_t09.expected.json");
    assert_eq!(
        golden.cells.get("0.0").map(String::as_str),
        Some("SH_x000D_Z002")
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// DateFormatTest#t03Read — full STRING including unpadded month `2023-1-01`.
#[test]
fn golden_dataformat_v2() {
    let golden = load_golden("dataformat_v2.expected.json");
    assert_eq!(golden.cells.get("0.0").map(String::as_str), Some("15:00"));
    assert_eq!(
        golden.cells.get("1.0").map(String::as_str),
        Some("2023-1-01 00:00:00")
    );
    assert!(
        !golden.rows.is_empty(),
        "dataformatv2 must be full-table STRING"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// DateFormatTest#t01 — dataformat.xlsx full STRING (CN AM/PM, mmmmm PUA, ￥).
#[test]
fn golden_dataformat_xlsx() {
    let golden = load_golden("dataformat_xlsx.expected.json");
    assert_eq!(
        golden.cells.get("22.0").map(String::as_str),
        Some("上午1时01分")
    );
    assert!(
        !golden.rows.is_empty(),
        "dataformat_xlsx must be full-table STRING"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// DateFormatTest#t02 — dataformat.xls full STRING (BIFF ¥, CN AM/PM, mmmmm PUA).
#[test]
fn golden_dataformat_xls() {
    let golden = load_golden("dataformat_xls.expected.json");
    assert_eq!(golden.cells.get("2.4").map(String::as_str), Some("¥1.11"));
    assert_eq!(
        golden.cells.get("22.0").map(String::as_str),
        Some("上午1时01分")
    );
    assert!(
        !golden.rows.is_empty(),
        "dataformat_xls must be full-table STRING"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// issue2443 date1.xlsx STRING.
#[test]
fn golden_dataformat_date1() {
    assert_golden_file("dataformat_date1.expected.json");
}

/// issue2443 date2.xlsx STRING.
#[test]
fn golden_dataformat_date2() {
    assert_golden_file("dataformat_date2.expected.json");
}

/// `ExtraDataTest` content (xlsx).
#[test]
fn golden_demo_extra_xlsx() {
    assert_golden_file("demo_extra_xlsx.expected.json");
}

/// `ExtraDataTest` content (xls).
#[test]
fn golden_demo_extra_xls() {
    assert_golden_file("demo_extra_xls.expected.json");
}

/// cellDataDemo.xlsx.
#[test]
fn golden_demo_cell_data() {
    assert_golden_file("demo_cell_data.expected.json");
}

/// demo/simple07.xlsx sheet `simple`.
#[test]
fn golden_demo_simple07() {
    assert_golden_file("demo_simple07.expected.json");
}

/// template07.xlsx content read.
#[test]
fn golden_template_template07() {
    assert_golden_file("template_template07.expected.json");
}

/// template03.xls content read.
#[test]
fn golden_template_template03_xls() {
    assert_golden_file("template_template03_xls.expected.json");
}

/// `StyleDataTest` write artifact — STRING content对照 (styles are write-side).
#[test]
fn golden_style_data() {
    let golden = load_golden("style_data.expected.json");
    assert!(
        golden.source.contains("StyleDataTest"),
        "unexpected source: {}",
        golden.source
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `StyleDataTest` `.xls` artifact — Rust read对照.
#[test]
fn golden_style_data_xls() {
    assert_golden_file("style_data_xls.expected.json");
}

/// `AnnotationData` — `DateTimeFormat` + `#.##%` full STRING (`java_compat_display` → `9999%`).
#[test]
fn golden_annotation_data() {
    let golden = load_golden("annotation_data.expected.json");
    assert!(
        golden.source.contains("AnnotationDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert!(
        golden.cells.get("0.0").is_some_and(|s| s.contains("年")),
        "annotation date format cell missing"
    );
    assert_eq!(golden.cells.get("0.1").map(String::as_str), Some("9999%"));
    assert!(
        !golden.rows.is_empty(),
        "annotation_data must be full-table STRING"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
    // Confirm ignore column was not written (only date + number).
    assert!(
        display_rows[0]
            .get(2)
            .map(display_text)
            .unwrap_or_default()
            .is_empty(),
        "ExcelIgnore field must not appear as a third column"
    );
}

/// `ExcludeOrInclude` excludeColumnIndexes — only column2/column3 remain.
#[test]
fn golden_exclude_index() {
    let golden = load_golden("exclude_index.expected.json");
    assert!(
        golden.source.contains("ExcludeOrIncludeDataTest"),
        "unexpected source: {}",
        golden.source
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `ExcludeOrInclude` exclude index CSV.
#[test]
fn golden_exclude_index_csv() {
    assert_golden_file("exclude_index_csv.expected.json");
}

/// `ExcludeOrInclude` exclude field names.
#[test]
fn golden_exclude_field() {
    assert_golden_file("exclude_field.expected.json");
}

/// `ExcludeOrInclude` include indexes.
#[test]
fn golden_include_index() {
    assert_golden_file("include_index.expected.json");
}

/// `ExcludeOrInclude` include field names.
#[test]
fn golden_include_field() {
    assert_golden_file("include_field.expected.json");
}

/// `ExcludeOrInclude` include field names with order.
#[test]
fn golden_include_field_order() {
    assert_golden_file("include_field_order.expected.json");
}

/// FillDataTest#t02Fill03 — Java filled `.xls` artifact.
#[test]
fn golden_fill_simple_xls() {
    assert_golden_file("fill_simple_xls.expected.json");
}

/// FillDataTest#t05HorizontalFill07.
#[test]
fn golden_fill_horizontal() {
    let golden = load_golden("fill_horizontal.expected.json");
    assert_eq!(golden.cells.get("0.2").map(String::as_str), Some("张三"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `NoHeadDataTest` — needHead(false).
#[test]
fn golden_no_head_data() {
    assert_golden_file("no_head_data.expected.json");
}

/// `SortDataTest` — index/order columns.
#[test]
fn golden_sort_data() {
    assert_golden_file("sort_data.expected.json");
}

/// `EncryptDataTest` — Java encrypted artifact; Rust read with password.
#[test]
fn golden_encrypt_data() {
    let golden = load_golden("encrypt_data.expected.json");
    assert!(
        golden.source.contains("EncryptDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert_eq!(
        golden.password.as_deref(),
        Some("123456"),
        "encrypt golden must carry password"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `CacheDataTest` — 姓名/年龄 full-table STRING.
#[test]
fn golden_cache_data() {
    let golden = load_golden("cache_data.expected.json");
    assert!(
        golden.source.contains("CacheDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert!(
        !golden.rows.is_empty(),
        "cache golden must include full rows"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `CacheDataTest` `.xls` / `.csv` 格式变体.
#[test]
fn golden_cache_data_xls_csv() {
    assert_golden_file("cache_data_xls.expected.json");
    assert_golden_file("cache_data_csv.expected.json");
}

/// `CellDataDataTest` xlsx — date/number/formula STRING full table.
#[test]
fn golden_celldata_data() {
    let golden = load_golden("celldata_data.expected.json");
    assert!(
        golden.source.contains("CellDataDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert!(
        !golden.rows.is_empty(),
        "celldata golden must include full rows"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `CharsetDataTest` GBK CSV — charset field required.
#[test]
fn golden_charset_gbk() {
    let golden = load_golden("charset_gbk.expected.json");
    assert_eq!(golden.charset.as_deref(), Some("GBK"));
    assert!(
        !golden.rows.is_empty(),
        "charset golden must include full rows"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `CharsetDataTest` UTF-8 CSV.
#[test]
fn golden_charset_utf8() {
    let golden = load_golden("charset_utf8.expected.json");
    assert_eq!(golden.charset.as_deref(), Some("UTF-8"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `ExceptionDataTest` content.
#[test]
fn golden_exception_data() {
    assert_golden_file("exception_data.expected.json");
}

/// `ExceptionDataTest` multi-sheet stop fixture — sheet0.
#[test]
fn golden_exception_stop_sheet0() {
    let golden = load_golden("exception_stop_sheet0.expected.json");
    assert!(
        golden.source.contains("ExceptionDataTest"),
        "unexpected source: {}",
        golden.source
    );
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `WriteHandlerTest` content — 姓名0..9 full table.
#[test]
fn golden_handler_data() {
    let golden = load_golden("handler_data.expected.json");
    assert_eq!(golden.row_count, 10);
    assert!(!golden.rows.is_empty());
    assert_eq!(golden.cells.get("9.0").map(String::as_str), Some("姓名9"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `WriteHandlerTest` CSV content.
#[test]
fn golden_handler_data_csv() {
    assert_golden_file("handler_data_csv.expected.json");
}

/// `LargeDataTest` sample (100×25) — not large07.
#[test]
fn golden_large_sample() {
    let golden = load_golden("large_sample.expected.json");
    assert_eq!(golden.row_count, 100);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `LargeDataTest` CSV sample (100×25).
#[test]
fn golden_large_sample_csv() {
    let golden = load_golden("large_sample_csv.expected.json");
    assert_eq!(golden.row_count, 100);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Converter write `.xls` artifact — full STRING.
#[test]
fn golden_converter_write_xls() {
    assert_golden_file("converter_write_xls.expected.json");
}

/// Converter write CSV artifact — full STRING.
#[test]
fn golden_converter_write_csv() {
    assert_golden_file("converter_write_csv.expected.json");
}

/// `CellDataDataTest` `.xls` — full STRING（CN `DateTimeFormat` 已对齐）.
#[test]
fn golden_celldata_data_xls() {
    let golden = load_golden("celldata_data_xls.expected.json");
    assert!(
        !golden.rows.is_empty(),
        "celldata_xls must export full rows"
    );
    assert!(
        golden.cells.get("0.0").is_some_and(|s| s.contains("年")),
        "celldata xls must keep CN date text"
    );
    assert_eq!(golden.cells.get("0.1").map(String::as_str), Some("2"));
    assert_eq!(golden.cells.get("0.2").map(String::as_str), Some("2"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `CellDataDataTest` CSV — full STRING (literal CN date text).
#[test]
fn golden_celldata_data_csv() {
    let golden = load_golden("celldata_data_csv.expected.json");
    assert!(!golden.rows.is_empty());
    assert!(
        golden.cells.get("0.0").is_some_and(|s| s.contains("年")),
        "celldata csv must keep CN date text"
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `ComplexHeadDataTest` — multi-level head, headRowNumber(3).
#[test]
fn golden_complex_head() {
    let golden = load_golden("complex_head.expected.json");
    assert_eq!(golden.head_row_number, 3);
    assert!(!golden.rows.is_empty());
    assert_eq!(golden.cells.get("0.4").map(String::as_str), Some("字符串4"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `AnnotationIndexAndNameDataTest` — sparse column index 4.
#[test]
fn golden_annotation_index_name() {
    let golden = load_golden("annotation_index_name.expected.json");
    assert_eq!(golden.cells.get("0.0").map(String::as_str), Some("第0个"));
    assert_eq!(golden.cells.get("0.4").map(String::as_str), Some("第4个"));
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
    // Column 3 is intentionally empty (no @ExcelProperty index=3).
    assert!(
        display_rows[0]
            .get(3)
            .map(display_text)
            .unwrap_or_default()
            .is_empty(),
        "index gap at col3 must be empty"
    );
}

/// `ListHeadDataTest` xlsx — full STRING including date + 额外数据.
#[test]
fn golden_list_head() {
    let golden = load_golden("list_head.expected.json");
    assert!(
        !golden.rows.is_empty(),
        "list_head xlsx must export full rows"
    );
    assert_eq!(golden.cells.get("0.0").map(String::as_str), Some("字符串0"));
    assert_eq!(
        golden.cells.get("0.2").map(String::as_str),
        Some("2020-01-01 01:01:01")
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `ListHeadDataTest` `.xls` — full STRING.
#[test]
fn golden_list_head_xls() {
    assert_golden_file("list_head_xls.expected.json");
}

/// `ComplexHeadDataTest` `.xls` — headRowNumber(3).
#[test]
fn golden_complex_head_xls() {
    let golden = load_golden("complex_head_xls.expected.json");
    assert_eq!(golden.head_row_number, 3);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `AnnotationIndexAndNameDataTest` `.xls`.
#[test]
fn golden_annotation_index_name_xls() {
    assert_golden_file("annotation_index_name_xls.expected.json");
}

/// `LargeDataTest` sample `.xls` (100×25).
#[test]
fn golden_large_sample_xls() {
    let golden = load_golden("large_sample_xls.expected.json");
    assert_eq!(golden.row_count, 100);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `NoHeadDataTest` `.xls` / CSV.
#[test]
fn golden_no_head_data_xls() {
    assert_golden_file("no_head_data_xls.expected.json");
}

/// `NoHeadDataTest` CSV.
#[test]
fn golden_no_head_data_csv() {
    assert_golden_file("no_head_data_csv.expected.json");
}

/// `FillDataTest` horizontal `.xls`.
#[test]
fn golden_fill_horizontal_xls() {
    assert_golden_file("fill_horizontal_xls.expected.json");
}

/// `FillDataTest` byName Sheet2.
#[test]
fn golden_fill_by_name() {
    let golden = load_golden("fill_by_name.expected.json");
    assert_eq!(golden.sheet_name.as_deref(), Some("Sheet2"));
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// FillDataTest#t08ByNameFill03 + #t03/#t04 complex fill.
#[test]
fn golden_fill_by_name_xls_and_complex() {
    assert_golden_file("fill_by_name_xls.expected.json");
    let complex = load_golden("fill_complex.expected.json");
    assert_eq!(complex.head_row_number, 3);
    assert!(!complex.rows.is_empty());
    assert_golden_file("fill_complex.expected.json");
    assert_golden_file("fill_complex_xls.expected.json");
}

/// `ListHeadDataTest` CSV — full STRING including date text.
#[test]
fn golden_list_head_csv() {
    let golden = load_golden("list_head_csv.expected.json");
    assert!(!golden.rows.is_empty());
    assert_eq!(
        golden.cells.get("0.2").map(String::as_str),
        Some("2020-01-01 01:01:01")
    );
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `ParameterDataTest` `.xls` read对照.
#[test]
fn golden_parameter_data_xls() {
    assert_golden_file("parameter_data_xls.expected.json");
}

/// `NoModelDataTest` — headRowNumber(0) full table.
#[test]
fn golden_nomodel_data() {
    let golden = load_golden("nomodel_data.expected.json");
    assert_eq!(golden.head_row_number, 0);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `NoModelDataTest` repeat 写回 — xlsx/xls/csv.
#[test]
fn golden_nomodel_repeat_variants() {
    assert_golden_file("nomodel_repeat.expected.json");
    assert_golden_file("nomodel_repeat_xls.expected.json");
    assert_golden_file("nomodel_repeat_csv.expected.json");
}

/// `UnCamelDataTest`.
#[test]
fn golden_noncamel_data() {
    assert_golden_file("noncamel_data.expected.json");
}

/// `ParameterDataTest`.
#[test]
fn golden_parameter_data() {
    assert_golden_file("parameter_data.expected.json");
}

