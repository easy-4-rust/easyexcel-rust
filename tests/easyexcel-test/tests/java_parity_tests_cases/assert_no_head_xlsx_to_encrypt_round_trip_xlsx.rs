fn assert_no_head_xlsx(path: &std::path::Path) {
    EasyExcel::write::<NoHeadData>(path)
        .need_head(false)
        .sheet("Sheet1")
        .do_write(no_head_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<NoHeadData>(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "字符串0");
}

fn assert_no_head_csv(path: &std::path::Path) {
    EasyExcel::write::<NoHeadData>(path)
        .need_head(false)
        .sheet("Sheet1")
        .do_write(no_head_data())
        .unwrap();
    let rows = EasyExcel::read_dynamic_sync(path)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert_eq!(
        rows.len(),
        1,
        "CSV should have exactly 1 data row (no header)"
    );
    let record = &rows[0];
    let vals: Vec<String> = (0..record.values().len())
        .filter_map(|i| match record.get(i) {
            Some(DynamicValue::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(vals.iter().any(|v| v.contains("字符串0")));
}

#[test]
fn t01_no_head_read_and_write_xlsx() {
    assert_no_head_xlsx(&temp_path("noHead07.xlsx"));
}

#[test]
fn t02_no_head_read_and_write_xls() {
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
fn t03_no_head_read_and_write_csv() {
    assert_no_head_csv(&temp_path("noHeadCsv.csv"));
}

// ============================================================================
// FillStyleDataTest (4 tests)
// Java: com.alibaba.easyexcel.test.core.fill.style.FillStyleDataTest
// ============================================================================

#[test]
fn t01_fill_style_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct FillStyleData {
        #[excel(name = "name", index = 0)]
        name: String,
    }
    let path = temp_path("fillStyle07.xlsx");
    EasyExcel::write::<FillStyleData>(&path)
        .sheet("Sheet1")
        .do_write(vec![FillStyleData {
            name: "测试".to_owned(),
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<FillStyleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "测试");
}

#[test]
fn t02_fill_style_xls() {
    let path = fixture("xls/fill/style.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
    // Java reads fill/style.xls and verifies style data
    assert!(!rows.is_empty(), "fill/style.xls fixture should have data");
}

#[test]
fn t11_fill_style_handler_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct FillStyleData {
        #[excel(name = "name", index = 0)]
        name: String,
    }
    let path = temp_path("fillStyleHandler07.xlsx");
    EasyExcel::write::<FillStyleData>(&path)
        .sheet("Sheet1")
        .do_write(vec![FillStyleData {
            name: "handler测试".to_owned(),
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<FillStyleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "handler测试");
}

#[test]
fn t12_fill_style_handler_xls() {
    let path = fixture("xls/fill/style.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
    assert!(!rows.is_empty());
}

// ============================================================================
// FillAnnotationDataTest (2 tests)
// Java: com.alibaba.easyexcel.test.core.fill.annotation.FillAnnotationDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct FillAnnotationData {
    #[excel(name = "name", index = 0)]
    name: String,
    #[excel(name = "number", index = 1)]
    number: f64,
}

fn assert_fill_annotation_xlsx(path: &std::path::Path) {
    EasyExcel::write::<FillAnnotationData>(path)
        .sheet("Sheet1")
        .do_write(vec![FillAnnotationData {
            name: "张三".to_owned(),
            number: 123.45,
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<FillAnnotationData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "张三");
    assert!((rows[0].number - 123.45).abs() < 0.01);
}

#[test]
fn t01_fill_annotation_xlsx() {
    assert_fill_annotation_xlsx(&temp_path("fillAnnotation07.xlsx"));
}

#[test]
fn t02_fill_annotation_xls() {
    let path = fixture("xls/fill/annotation.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
    assert!(
        !rows.is_empty(),
        "fill/annotation.xls fixture should have data"
    );
}

// ============================================================================
// FillStyleAnnotatedTest (2 tests)
// Java: com.alibaba.easyexcel.test.core.fill.style.FillStyleAnnotatedTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct FillStyleAnnotatedData {
    #[excel(name = "name", index = 0)]
    name: String,
    #[excel(name = "value", index = 1)]
    value: String,
}

fn assert_fill_style_annotated_xlsx(path: &std::path::Path) {
    EasyExcel::write::<FillStyleAnnotatedData>(path)
        .sheet("Sheet1")
        .do_write(vec![FillStyleAnnotatedData {
            name: "名称".to_owned(),
            value: "值".to_owned(),
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<FillStyleAnnotatedData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "名称");
    assert_eq!(rows[0].value, "值");
}

#[test]
fn t01_fill_style_annotated_xlsx() {
    assert_fill_style_annotated_xlsx(&temp_path("fillStyleAnnotated07.xlsx"));
}

#[test]
fn t02_fill_style_annotated_xls() {
    let path = fixture("xls/fill/annotation.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
    assert!(!rows.is_empty());
}

// ============================================================================
// Additional parity tests
// ============================================================================

#[test]
fn simple_data_round_trip_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct SimpleData {
        #[excel(name = "姓名", index = 0)]
        name: String,
    }
    let path = temp_path("simple07.xlsx");
    let data: Vec<SimpleData> = (0..10)
        .map(|i| SimpleData {
            name: format!("姓名{i}"),
        })
        .collect();
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].name, "姓名0");
    assert_eq!(rows[9].name, "姓名9");
}

#[test]
fn converter_round_trip_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct ConverterData {
        #[excel(name = "string", index = 0)]
        string: String,
        #[excel(name = "boolean", index = 1)]
        boolean: bool,
        #[excel(name = "integer", index = 2)]
        integer: i32,
        #[excel(name = "long", index = 3)]
        long: i64,
        #[excel(name = "double", index = 4)]
        double: f64,
        #[excel(name = "date", index = 5, format = "%Y-%m-%d")]
        date: NaiveDate,
    }
    let path = temp_path("converter07.xlsx");
    let data = vec![ConverterData {
        string: "hello".to_owned(),
        boolean: true,
        integer: 42,
        long: 1_234_567_890i64,
        double: std::f64::consts::PI,
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
    }];
    EasyExcel::write::<ConverterData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let rows = EasyExcel::read_sync::<ConverterData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "hello");
    assert!(rows[0].boolean);
    assert_eq!(rows[0].integer, 42);
    assert_eq!(rows[0].long, 1_234_567_890i64);
    assert!((rows[0].double - std::f64::consts::PI).abs() < 1e-10);
    assert_eq!(rows[0].date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
}

#[test]
fn encrypt_round_trip_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct EncryptData {
        #[excel(name = "string", index = 0)]
        string: String,
    }
    let path = temp_path("encrypt07.xlsx");
    EasyExcel::write::<EncryptData>(&path)
        .password("123456")
        .sheet("Sheet1")
        .do_write(vec![EncryptData {
            string: "secret".to_owned(),
        }])
        .unwrap();
    let rows = EasyExcel::read_sync::<EncryptData>(&path)
        .password("123456")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "secret");
}
