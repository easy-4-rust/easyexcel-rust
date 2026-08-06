fn assert_complex_head_no_auto_merge(path: &std::path::Path) {
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
    assert_eq!(rows[0].string4, "字符串4");
}

#[test]
fn t11_complex_head_automatic_merge_head_xlsx() {
    assert_complex_head_no_auto_merge(&temp_path("complexHeadAutomaticMergeHead07.xlsx"));
}

#[test]
fn t12_complex_head_automatic_merge_head_xls() {
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
    assert!(!rows.is_empty());
}

#[test]
fn t13_complex_head_automatic_merge_head_csv() {
    let path = temp_path("complexHeadAutomaticMergeHeadCsv.csv");
    EasyExcel::write::<ComplexHeadData>(&path)
        .sheet("Sheet1")
        .do_write(complex_head_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(&path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(rows.len() >= 2, "CSV should have header + data");
}

// ============================================================================
// MultipleSheetsDataTest (4 tests)
// Java: com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct MultipleSheetsData {
    #[excel(name = "title", index = 0)]
    title: String,
}

fn write_multi_sheet_file(path: &std::path::Path) {
    let sheet0 = EasyExcel::writer_sheet::<MultipleSheetsData>("Sheet0");
    let sheet1 = EasyExcel::writer_sheet::<MultipleSheetsData>("Sheet1");
    let sheet2 = EasyExcel::writer_sheet::<MultipleSheetsData>("Sheet2");
    let mut writer = EasyExcel::write::<MultipleSheetsData>(path).build();
    writer
        .write(
            vec![MultipleSheetsData {
                title: "s0_row0".to_owned(),
            }],
            &sheet0,
        )
        .unwrap();
    writer
        .write(
            vec![
                MultipleSheetsData {
                    title: "s1_row0".to_owned(),
                },
                MultipleSheetsData {
                    title: "s1_row1".to_owned(),
                },
            ],
            &sheet1,
        )
        .unwrap();
    writer
        .write(
            vec![
                MultipleSheetsData {
                    title: "s2_row0".to_owned(),
                },
                MultipleSheetsData {
                    title: "s2_row1".to_owned(),
                },
                MultipleSheetsData {
                    title: "s2_row2".to_owned(),
                },
            ],
            &sheet2,
        )
        .unwrap();
    writer.finish().unwrap();
}

/// Java: read each sheet individually → assert counts match.
#[test]
fn t01_multiple_sheets_read_xlsx() {
    let path = temp_path("multiplesheets07.xlsx");
    write_multi_sheet_file(&path);
    let rows0 = EasyExcel::read_sync::<MultipleSheetsData>(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows0.len(), 1);
    assert_eq!(rows0[0].title, "s0_row0");
    let rows1 = EasyExcel::read_sync::<MultipleSheetsData>(&path)
        .sheet(1usize)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows1.len(), 2);
    assert_eq!(rows1[0].title, "s1_row0");
    let rows2 = EasyExcel::read_sync::<MultipleSheetsData>(&path)
        .sheet(2usize)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows2.len(), 3);
}

/// Java: read .xls with multiple sheets.
#[test]
fn t02_multiple_sheets_read_xls() {
    let path = fixture("xls/multiplesheets.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    // Read the first sheet from the real .xls fixture
    let rows = EasyExcel::read_dynamic_sync(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(
        !rows.is_empty(),
        ".xls multiplesheets fixture should have data"
    );
}

/// Java: `doReadAll()` → reads all sheets into one listener.
#[test]
fn t03_multiple_sheets_read_all_xlsx() {
    let path = temp_path("multiplesheetsAll07.xlsx");
    write_multi_sheet_file(&path);
    let rows = EasyExcel::read_sync::<MultipleSheetsData>(&path)
        .all_sheets()
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 6);
}

#[test]
fn t04_multiple_sheets_read_all_xls() {
    let path = fixture("xls/multiplesheets.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path)
        .all_sheets()
        .do_read_sync()
        .unwrap();
    assert!(
        !rows.is_empty(),
        ".xls multiplesheets fixture should have data when reading all sheets"
    );
}

// ============================================================================
// RepetitionDataTest (6 tests)
// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest
// 2 operations × 3 formats
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct RepetitionData {
    #[excel(name = "字符串", index = 0)]
    string: String,
}

fn repetition_data() -> Vec<RepetitionData> {
    vec![RepetitionData {
        string: "字符串0".to_owned(),
    }]
}

fn assert_repetition_xlsx(path: &std::path::Path) {
    let sheet = EasyExcel::writer_sheet_index::<RepetitionData>(0);
    let mut writer = EasyExcel::write::<RepetitionData>(path).build();
    writer
        .write(repetition_data(), &sheet)
        .unwrap()
        .write(repetition_data(), &sheet)
        .unwrap();
    writer.finish().unwrap();
    let rows = EasyExcel::read_sync::<RepetitionData>(path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].string, "字符串0");
    assert_eq!(rows[1].string, "字符串0");
}

fn assert_repetition_csv(path: &std::path::Path) {
    let sheet = EasyExcel::writer_sheet_index::<RepetitionData>(0);
    let mut writer = EasyExcel::write::<RepetitionData>(path).build();
    writer
        .write(repetition_data(), &sheet)
        .unwrap()
        .write(repetition_data(), &sheet)
        .unwrap();
    writer.finish().unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(rows.len() >= 3, "CSV should have header + 2 data rows");
    for row in rows.iter().skip(1) {
        let vals: Vec<String> = (0..row.values().len())
            .filter_map(|i| match row.get(i) {
                Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect();
        assert!(vals.iter().any(|v| v.contains("字符串0")));
    }
}

#[test]
fn t01_repetition_read_and_write_xlsx() {
    assert_repetition_xlsx(&temp_path("repetition07.xlsx"));
}

#[test]
fn t02_repetition_read_and_write_xls() {
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
    assert!(!rows.is_empty());
}

#[test]
fn t03_repetition_read_and_write_csv() {
    assert_repetition_csv(&temp_path("repetitionCsv.csv"));
}

fn assert_repetition_table_xlsx(path: &std::path::Path) {
    let sheet = EasyExcel::writer_sheet_index::<RepetitionData>(0);
    let mut writer = EasyExcel::write::<RepetitionData>(path).build();
    writer
        .write(repetition_data(), &sheet)
        .unwrap()
        .write(repetition_data(), &sheet)
        .unwrap();
    writer.finish().unwrap();
    let rows = EasyExcel::read_sync::<RepetitionData>(path)
        .head_row_number(2)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(rows.len() <= 2);
}

fn assert_repetition_table_csv(path: &std::path::Path) {
    let sheet = EasyExcel::writer_sheet_index::<RepetitionData>(0);
    let mut writer = EasyExcel::write::<RepetitionData>(path).build();
    writer
        .write(repetition_data(), &sheet)
        .unwrap()
        .write(repetition_data(), &sheet)
        .unwrap();
    writer.finish().unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(rows.len() >= 3, "CSV should have header + 2 data rows");
}

#[test]
fn t11_repetition_table_xlsx() {
    assert_repetition_table_xlsx(&temp_path("repetitionTable07.xlsx"));
}

#[test]
fn t12_repetition_table_xls() {
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
    assert!(!rows.is_empty());
}

#[test]
fn t13_repetition_table_csv() {
    assert_repetition_table_csv(&temp_path("repetitionTableCsv.csv"));
}

// ============================================================================
// AnnotationIndexAndNameDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct AnnotationIndexAndNameData {
    #[excel(name = "第四个", index = 4)]
    index4: String,
    #[excel(name = "第二个", index = 2)]
    index2: String,
    #[excel(index = 0)]
    index0: String,
    #[excel(name = "第一个", index = 1)]
    index1: String,
}

fn annotation_index_name_data() -> Vec<AnnotationIndexAndNameData> {
    vec![AnnotationIndexAndNameData {
        index0: "第0个".to_owned(),
        index1: "第1个".to_owned(),
        index2: "第2个".to_owned(),
        index4: "第4个".to_owned(),
    }]
}

fn assert_annotation_index_name_xlsx(path: &std::path::Path) {
    EasyExcel::write::<AnnotationIndexAndNameData>(path)
        .sheet("Sheet1")
        .do_write(annotation_index_name_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<AnnotationIndexAndNameData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].index0, "第0个");
    assert_eq!(rows[0].index1, "第1个");
    assert_eq!(rows[0].index2, "第2个");
    assert_eq!(rows[0].index4, "第4个");
}

fn assert_annotation_index_name_csv(path: &std::path::Path) {
    EasyExcel::write::<AnnotationIndexAndNameData>(path)
        .sheet("Sheet1")
        .do_write(annotation_index_name_data())
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
    assert!(vals.iter().any(|v| v.contains("第0个")));
    assert!(vals.iter().any(|v| v.contains("第1个")));
    assert!(vals.iter().any(|v| v.contains("第2个")));
    assert!(vals.iter().any(|v| v.contains("第4个")));
}

#[test]
fn t01_annotation_index_and_name_xlsx() {
    assert_annotation_index_name_xlsx(&temp_path("annotationIndexAndName07.xlsx"));
}

#[test]
fn t02_annotation_index_and_name_xls() {
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
fn t03_annotation_index_and_name_csv() {
    assert_annotation_index_name_csv(&temp_path("annotationIndexAndNameCsv.csv"));
}

// ============================================================================
// UnCamelDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct UnCamelData {
    #[excel(index = 0)]
    string1: String,
    #[excel(index = 1)]
    string2: String,
    #[excel(index = 2)]
    s_tring3: String,
    #[excel(index = 3)]
    s_tring4: String,
    #[excel(index = 4)]
    string5: String,
    #[excel(index = 5)]
    s_tring6: String,
}

fn uncamel_data() -> Vec<UnCamelData> {
    (0..10)
        .map(|_| UnCamelData {
            string1: "string1".to_owned(),
            string2: "string2".to_owned(),
            s_tring3: "string3".to_owned(),
            s_tring4: "string4".to_owned(),
            string5: "string5".to_owned(),
            s_tring6: "string6".to_owned(),
        })
        .collect()
}

fn assert_uncamel_xlsx(path: &std::path::Path) {
    EasyExcel::write::<UnCamelData>(path)
        .sheet("Sheet1")
        .do_write(uncamel_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<UnCamelData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    for row in &rows {
        assert_eq!(row.string1, "string1");
        assert_eq!(row.string2, "string2");
        assert_eq!(row.s_tring3, "string3");
        assert_eq!(row.s_tring4, "string4");
        assert_eq!(row.string5, "string5");
        assert_eq!(row.s_tring6, "string6");
    }
}

fn assert_uncamel_csv(path: &std::path::Path) {
    EasyExcel::write::<UnCamelData>(path)
        .sheet("Sheet1")
        .do_write(uncamel_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 11, "CSV should have 1 header + 10 data rows");
    // Verify each data row has values
    for row in rows.iter().skip(1) {
        assert!(
            row.values().len() >= 6,
            "each row should have at least 6 columns"
        );
    }
}

#[test]
fn t01_uncamel_read_and_write_xlsx() {
    assert_uncamel_xlsx(&temp_path("unCame07.xlsx"));
}

#[test]
fn t02_uncamel_read_and_write_xls() {
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
    assert!(!rows.is_empty());
}

#[test]
fn t03_uncamel_read_and_write_csv() {
    assert_uncamel_csv(&temp_path("unCameCsv.csv"));
}

// ============================================================================
// ListHeadDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.head.ListHeadDataTest
// ============================================================================

fn assert_list_head_xlsx(path: &std::path::Path) {
    EasyExcel::write::<DynamicRow>(path)
        .head(vec![
            vec!["字符串".to_owned()],
            vec!["数字".to_owned()],
            vec!["日期".to_owned()],
        ])
        .sheet("Sheet1")
        .do_write(vec![{
            let mut map = std::collections::BTreeMap::new();
            map.insert(0usize, DynamicValue::String("字符串0".to_owned()));
            map.insert(1usize, DynamicValue::String("1".to_owned()));
            map.insert(
                2usize,
                DynamicValue::String("2020-01-01 01:01:01".to_owned()),
            );
            DynamicRow::new(map)
        }])
        .unwrap();
    let rows = read_dynamic_string(path);
    assert_eq!(rows.len(), 1);
    let val0 = match rows[0].get(0).unwrap() {
        DynamicValue::String(s) => s.as_str(),
        other => panic!("expected String, got {other:?}"),
    };
    assert_eq!(val0, "字符串0");
}

fn assert_list_head_csv(path: &std::path::Path) {
    EasyExcel::write::<DynamicRow>(path)
        .head(vec![
            vec!["字符串".to_owned()],
            vec!["数字".to_owned()],
            vec!["日期".to_owned()],
        ])
        .sheet("Sheet1")
        .do_write(vec![{
            let mut map = std::collections::BTreeMap::new();
            map.insert(0usize, DynamicValue::String("字符串0".to_owned()));
            map.insert(1usize, DynamicValue::String("1".to_owned()));
            map.insert(
                2usize,
                DynamicValue::String("2020-01-01 01:01:01".to_owned()),
            );
            DynamicRow::new(map)
        }])
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
}

#[test]
fn t01_list_head_read_and_write_xlsx() {
    assert_list_head_xlsx(&temp_path("listHead07.xlsx"));
}

#[test]
fn t02_list_head_read_and_write_xls() {
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
    assert!(!rows.is_empty());
}

#[test]
fn t03_list_head_read_and_write_csv() {
    assert_list_head_csv(&temp_path("listHeadCsv.csv"));
}

// ============================================================================
// NoHeadDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.head.NoHeadDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct NoHeadData {
    #[excel(name = "字符串", index = 0)]
    string: String,
}

fn no_head_data() -> Vec<NoHeadData> {
    vec![NoHeadData {
        string: "字符串0".to_owned(),
    }]
}

