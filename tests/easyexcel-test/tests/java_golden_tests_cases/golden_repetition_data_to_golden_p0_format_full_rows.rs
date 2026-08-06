/// `RepetitionDataTest` — double write → 2 data rows.
#[test]
fn golden_repetition_data() {
    let golden = load_golden("repetition_data.expected.json");
    assert_eq!(golden.row_count, 2);
    assert!(!golden.rows.is_empty());
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// `SkipDataTest` — sheet name `第二个`.
#[test]
fn golden_skip_sheet1() {
    let golden = load_golden("skip_sheet1.expected.json");
    assert_eq!(golden.sheet_name.as_deref(), Some("第二个"));
    assert_eq!(golden.cells.get("0.0").map(String::as_str), Some("name2"));
    let path = resolve_golden_path(&golden);
    let display_rows = read_display_rows(&path, &golden);
    assert_matches_golden(&golden, &display_rows);
}

/// Every checked-in `*.expected.json` must pass (guards coverage ≥100, no soft-skip).
#[test]
fn golden_all_expected_json_files() {
    let dir = golden_dir();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("golden dir missing {}: {e}", dir.display()))
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && e.file_name().to_string_lossy().ends_with(".expected.json")
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert!(
        names.len() >= 108,
        "expected ≥108 golden JSON files, found {} — run scripts/export-java-golden.sh",
        names.len()
    );
    for name in &names {
        assert_golden_file(name);
    }
}

/// Missing golden must fail loudly (regression guard for soft-skip).
#[test]
#[should_panic(expected = "required Java golden missing")]
fn golden_missing_file_fails() {
    let _ = load_golden("__does_not_exist__.expected.json");
}

/// P0 STRING 全表回归：dataformat / annotation / converter03 / t07 / celldata / `list_head`。
#[test]
fn golden_p0_format_full_rows() {
    for (label, name) in [
        ("dataformat_v2", "dataformat_v2.expected.json"),
        ("dataformat_xlsx", "dataformat_xlsx.expected.json"),
        ("dataformat_xls", "dataformat_xls.expected.json"),
        ("annotation_data", "annotation_data.expected.json"),
        ("converter03_xls", "converter_converter03_xls.expected.json"),
        ("compatibility_t07", "compatibility_t07.expected.json"),
        ("celldata_csv", "celldata_data_csv.expected.json"),
        ("celldata_xls", "celldata_data_xls.expected.json"),
        ("list_head", "list_head.expected.json"),
        ("list_head_csv", "list_head_csv.expected.json"),
        ("list_head_xls", "list_head_xls.expected.json"),
    ] {
        let golden = load_golden(name);
        assert!(
            !golden.rows.is_empty(),
            "{label} must export full rows (ofNoRows cleared)"
        );
        let path = resolve_golden_path(&golden);
        let rows = read_display_rows(&path, &golden);
        assert_matches_golden(&golden, &rows);
    }
}
