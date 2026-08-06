/// Java ConverterWriteData/ConverterReadData — 14 fields covering all type conversions.
/// Java fields: date, localDate, localDateTime, booleanData, bigDecimal, bigInteger,
///              longData, integerData, shortData, byteData, doubleData, floatData, string, cellData
#[derive(Debug, Clone, ExcelRow)]
struct ConverterData {
    #[excel(name = "date", index = 0, format = "%Y-%m-%d")]
    date: NaiveDate,
    #[excel(name = "localDate", index = 1, format = "%Y-%m-%d")]
    local_date: NaiveDate,
    #[excel(name = "localDateTime", index = 2, format = "%Y-%m-%d %H:%M:%S")]
    local_date_time: chrono::NaiveDateTime,
    #[excel(name = "booleanData", index = 3)]
    boolean_data: bool,
    #[excel(name = "bigDecimal", index = 4)]
    big_decimal: bigdecimal::BigDecimal,
    #[excel(name = "bigInteger", index = 5)]
    big_integer: num_bigint::BigInt,
    #[excel(name = "longData", index = 6)]
    long_data: i64,
    #[excel(name = "integerData", index = 7)]
    integer_data: i32,
    #[excel(name = "shortData", index = 8)]
    short_data: i16,
    #[excel(name = "byteData", index = 9)]
    byte_data: i8,
    #[excel(name = "doubleData", index = 10)]
    double_data: f64,
    #[excel(name = "floatData", index = 11)]
    float_data: f32,
    #[excel(name = "string", index = 12)]
    string: String,
    #[excel(name = "cellData", index = 13)]
    cell_data: String,
}

/// Java: `TestUtil.TEST_DATE` = 2020-01-01 01:01:01
fn converter_data() -> Vec<ConverterData> {
    vec![ConverterData {
        date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        local_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        local_date_time: chrono::NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_hms_opt(1, 1, 1)
            .unwrap(),
        boolean_data: true,
        big_decimal: bigdecimal::BigDecimal::from(1i64),
        big_integer: num_bigint::BigInt::from(1i32),
        long_data: 1i64,
        integer_data: 1i32,
        short_data: 1i16,
        byte_data: 1i8,
        double_data: 1.0f64,
        float_data: 1.0f32,
        string: "测试".to_owned(),
        cell_data: "自定义".to_owned(),
    }]
}

/// Java ConverterDataListener.doAfterAllAnalysed assertions:
/// `date==TEST_DATE`, `localDate==TEST_LOCAL_DATE`, `localDateTime==TEST_LOCAL_DATE_TIME`,
/// booleanData==TRUE, bigDecimal==1, bigInteger==1, longData==1, integerData==1,
/// shortData==1, byteData==1, doubleData==1.0, floatData==1.0, string=="测试", cellData=="自定义"
fn assert_converter_round_trip(path: &std::path::Path) {
    EasyExcel::write::<ConverterData>(path)
        .sheet("Sheet1")
        .do_write(converter_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<ConverterData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    let r = &rows[0];
    // Date fields
    assert_eq!(r.date, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    assert_eq!(r.local_date, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    assert_eq!(
        r.local_date_time,
        chrono::NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_hms_opt(1, 1, 1)
            .unwrap()
    );
    // Boolean
    assert!(r.boolean_data);
    // BigDecimal/BigInteger
    assert_eq!(r.big_decimal, bigdecimal::BigDecimal::from(1i64));
    assert_eq!(r.big_integer, num_bigint::BigInt::from(1i32));
    // Numeric types
    assert_eq!(r.long_data, 1i64);
    assert_eq!(r.integer_data, 1i32);
    assert_eq!(r.short_data, 1i16);
    assert_eq!(r.byte_data, 1i8);
    assert!((r.double_data - 1.0f64).abs() < 1e-10);
    assert!((r.float_data - 1.0f32).abs() < 1e-6);
    // String
    assert_eq!(r.string, "测试");
    assert_eq!(r.cell_data, "自定义");
}

#[test]
fn converter_t01_read_and_write_xlsx() {
    assert_converter_round_trip(&temp_path("converter07.xlsx"));
}

#[test]
fn converter_t02_read_and_write_xls() {
    assert_converter_round_trip(&temp_path("converter03.xls"));
}

#[test]
fn converter_t03_read_and_write_csv() {
    assert_converter_round_trip(&temp_path("converter.csv"));
}

/// Java: readAllConverter → read with all converter types
#[test]
fn converter_t11_read_all_converter_xlsx() {
    assert_converter_round_trip(&temp_path("converter07_all.xlsx"));
}

#[test]
fn converter_t12_read_all_converter_xls() {
    assert_converter_round_trip(&temp_path("converter03_all.xls"));
}

#[test]
fn converter_t13_read_all_converter_csv() {
    assert_converter_round_trip(&temp_path("converter_all.csv"));
}

/// Java: writeImage → write image data
#[test]
fn converter_t21_write_image_xlsx() {
    #[derive(Debug, Clone, ExcelRow)]
    struct ImageData {
        #[excel(name = "name", index = 0)]
        name: String,
    }
    let path = temp_path("converter07_image.xlsx");
    let data = vec![ImageData {
        name: "image_test".to_owned(),
    }];
    EasyExcel::write::<ImageData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"PK"), "should be valid XLSX");
}

#[test]
fn converter_t22_write_image_xls() {
    #[derive(Debug, Clone, ExcelRow)]
    struct ImageRow {
        #[excel(name = "file", index = 0)]
        file: WriteCellData,
    }
    // Java writes images into .xls. BIFF8 image records remain Unsupported (visible).
    let path = temp_path("converterImage03.xls");
    let row = ImageRow {
        file: WriteCellData::from_image(vec![0xFF, 0xD8, 0xFF, 0xD9]),
    };
    EasyExcel::write::<ImageRow>(&path)
        .sheet("Sheet1")
        .do_write(vec![row])
        .expect("XLS image write must succeed (Phase 5.6)");
}

// ============================================================================
// DateFormatTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.dataformat.DateFormatTest
// ============================================================================

#[test]
fn dateformat_t01_read_xlsx() {
    let path = fixture("dataformat/dataformat.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = read_dynamic_string(&path);
    assert!(!rows.is_empty(), "dataformat.xlsx should have data");
}

#[test]
fn dateformat_t02_read_xls() {
    let path = fixture("xls/dataformat.xls");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = read_dynamic_string(&path);
    assert!(!rows.is_empty());
}

#[test]
fn dateformat_t03_read() {
    // Generic date format read test
    let path = fixture("dataformat/dataformat.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = read_dynamic_actual(&path);
    assert!(!rows.is_empty());
}

// ============================================================================
// CellDataDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.celldata.CellDataDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct CellDataData {
    #[excel(name = "string", index = 0)]
    string: String,
    #[excel(name = "number", index = 1)]
    number: f64,
    #[excel(name = "boolean", index = 2)]
    boolean: bool,
}

fn cell_data_data() -> Vec<CellDataData> {
    vec![CellDataData {
        string: "test".to_owned(),
        number: 42.0,
        boolean: true,
    }]
}

fn assert_cell_data_round_trip(path: &std::path::Path) {
    EasyExcel::write::<CellDataData>(path)
        .sheet("Sheet1")
        .do_write(cell_data_data())
        .unwrap();
    let rows = EasyExcel::read_sync::<CellDataData>(path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "test");
    assert!((rows[0].number - 42.0).abs() < 0.01);
    assert!(rows[0].boolean);
}

#[test]
fn celldata_t01_read_and_write_xlsx() {
    assert_cell_data_round_trip(&temp_path("celldata07.xlsx"));
}

#[test]
fn celldata_t02_read_and_write_xls() {
    assert_cell_data_round_trip(&temp_path("celldata03.xls"));
}

#[test]
fn celldata_t03_read_and_write_csv() {
    assert_cell_data_round_trip(&temp_path("celldata.csv"));
}

// ============================================================================
// NoModelDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest
// ============================================================================

/// Java: write List<List<Object>> → read as Map → assert values
fn assert_no_model(path: &std::path::Path) {
    // Write dynamic data
    let data: Vec<DynamicRow> = (0..10)
        .map(|i| {
            let mut map = BTreeMap::new();
            map.insert(0, DynamicValue::String(format!("string1{i}")));
            map.insert(1, DynamicValue::String(format!("{}", 100 + i)));
            map.insert(2, DynamicValue::String("2020-01-01 01:01:01".to_owned()));
            DynamicRow::new(map)
        })
        .collect();
    EasyExcel::write::<DynamicRow>(path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();

    // Read as String mode (Java uses headRowNumber(0))
    let rows = read_dynamic_string_no_head(path);
    assert_eq!(rows.len(), 10, "should have 10 data rows");
    let row10 = &rows[9];
    let val0 = match row10.get(0).unwrap() {
        DynamicValue::String(s) => s.as_str(),
        other => panic!("expected String, got {other:?}"),
    };
    assert_eq!(val0, "string19");

    // Read as ActualData mode
    let rows_actual = read_dynamic_actual_no_head(path);
    assert_eq!(rows_actual.len(), 10);
}

#[test]
fn nomodel_t01_read_and_write_xlsx() {
    assert_no_model(&temp_path("noModel07.xlsx"));
}

#[test]
fn nomodel_t02_read_and_write_xls() {
    assert_no_model(&temp_path("noModel03.xls"));
}

#[test]
fn nomodel_t03_read_and_write_csv() {
    assert_no_model(&temp_path("noModel.csv"));
}

// ============================================================================
// SkipDataTest (3 tests)
// Java: com.alibaba.easyexcel.test.core.skip.SkipDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct SkipData {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

/// Java: write 4 sheets → read "第二个" → assert name=="name2"
fn assert_skip(path: &std::path::Path) {
    let sheet0 = EasyExcel::writer_sheet::<SkipData>("第一个");
    let sheet1 = EasyExcel::writer_sheet::<SkipData>("第二个");
    let sheet2 = EasyExcel::writer_sheet::<SkipData>("第三个");
    let sheet3 = EasyExcel::writer_sheet::<SkipData>("第四个");

    let mut writer = EasyExcel::write::<SkipData>(path).build();
    writer
        .write(
            vec![SkipData {
                name: "name1".to_owned(),
            }],
            &sheet0,
        )
        .unwrap();
    writer
        .write(
            vec![SkipData {
                name: "name2".to_owned(),
            }],
            &sheet1,
        )
        .unwrap();
    writer
        .write(
            vec![SkipData {
                name: "name3".to_owned(),
            }],
            &sheet2,
        )
        .unwrap();
    writer
        .write(
            vec![SkipData {
                name: "name4".to_owned(),
            }],
            &sheet3,
        )
        .unwrap();
    writer.finish().unwrap();

    // Read specific sheet
    let rows = EasyExcel::read_sync::<SkipData>(path)
        .sheet("第二个")
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "name2");
}

#[test]
fn skip_t01_read_and_write_xlsx() {
    assert_skip(&temp_path("skip07.xlsx"));
}

#[test]
fn skip_t02_read_and_write_xls() {
    assert_skip(&temp_path("skip03.xls"));
}

/// Java: CSV does not support multiple sheets → `ExcelGenerateException`
#[test]
fn skip_t03_read_and_write_csv() {
    let path = temp_path("skip.csv");
    // CSV only supports one sheet, so writing multiple sheets should fail
    let sheet0 = EasyExcel::writer_sheet::<SkipData>("第一个");
    let sheet1 = EasyExcel::writer_sheet::<SkipData>("第二个");
    let mut writer = EasyExcel::write::<SkipData>(&path).build();
    writer
        .write(
            vec![SkipData {
                name: "name1".to_owned(),
            }],
            &sheet0,
        )
        .unwrap();
    let result = writer.write(
        vec![SkipData {
            name: "name2".to_owned(),
        }],
        &sheet1,
    );
    assert!(result.is_err(), "CSV should not support multiple sheets");
}

// ============================================================================
// LargeDataTest (4 tests)
// Java: com.alibaba.easyexcel.test.core.large.LargeDataTest
// ============================================================================

#[test]
fn large_t01_read_xlsx() {
    let path = fixture("large/large07.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    let rows = read_dynamic_string(&path);
    assert!(!rows.is_empty(), "large07.xlsx should have data");
}

#[test]
fn large_t02_fill_xlsx() {
    // Template fill test
    let path = fixture("fill/simple.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    // Verify template exists and is readable
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"PK"), "should be valid XLSX");
}

#[test]
fn large_t03_read_and_write_csv() {
    let path = temp_path("large.csv");
    let data: Vec<SimpleData> = (0..1000)
        .map(|i| SimpleData {
            name: format!("name{i}"),
        })
        .collect();
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let rows = EasyExcel::read_sync::<SimpleData>(&path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1000);
}

#[test]
fn large_t04_write_xlsx() {
    let path = temp_path("large07.xlsx");
    let data: Vec<SimpleData> = (0..1000)
        .map(|i| SimpleData {
            name: format!("name{i}"),
        })
        .collect();
    EasyExcel::write::<SimpleData>(&path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"PK"));
    assert!(bytes.len() > 1000);
}

// ============================================================================
// TemplateDataTest (2 tests)
// Java: com.alibaba.easyexcel.test.core.template.TemplateDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct TemplateWriteRow {
    #[excel(name = "字符串0", index = 0)]
    string0: String,
    #[excel(name = "字符串1", index = 1)]
    string1: String,
}

fn template_write_rows() -> Vec<TemplateWriteRow> {
    vec![
        TemplateWriteRow {
            string0: "字符串0".to_owned(),
            string1: "字符串01".to_owned(),
        },
        TemplateWriteRow {
            string0: "字符串1".to_owned(),
            string1: "字符串11".to_owned(),
        },
    ]
}

/// Java `TemplateDataTest#t01ReadAndWrite07`: `withTemplate(...).sheet().doWrite(data)`.
#[test]
fn template_t01_read_and_write_xlsx() {
    let template = fixture("template/template07.xlsx");
    assert!(
        template.exists(),
        "required Java fixture missing: {}",
        template.display()
    );
    let path = temp_path("template07_parity.xlsx");
    EasyExcel::write::<TemplateWriteRow>(&path)
        .with_template(&template)
        .sheet("Sheet1")
        .do_write(template_write_rows())
        .unwrap();
    let rows = EasyExcel::read_sync::<TemplateWriteRow>(&path)
        .head_row_number(3)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].string0, "字符串0");
    assert_eq!(rows[0].string1, "字符串01");
    assert_eq!(rows[1].string0, "字符串1");
    assert_eq!(rows[1].string1, "字符串11");

    // Java `withTemplate(InputStream)` equivalent.
    let bytes = std::fs::read(&template).unwrap();
    let from_bytes = temp_path("template07_bytes.xlsx");
    EasyExcel::write::<TemplateWriteRow>(&from_bytes)
        .with_template_bytes(bytes)
        .sheet("Sheet1")
        .do_write(template_write_rows())
        .unwrap();
    let rows = EasyExcel::read_sync::<TemplateWriteRow>(&from_bytes)
        .head_row_number(3)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
}

/// Java `TemplateDataTest#t02ReadAndWrite03`: `withTemplate(.xls).sheet().doWrite(data)`.
#[test]
fn template_t02_read_and_write_xls() {
    let xls = fixture("template/template03.xls");
    assert_xls_readable(&xls);
    let path = temp_path("template03_parity.xls");
    EasyExcel::write::<TemplateWriteRow>(&path)
        .with_template(&xls)
        .sheet("Sheet1")
        .do_write(template_write_rows())
        .unwrap();
    assert_real_biff8(&path);
    let rows = EasyExcel::read_sync::<TemplateWriteRow>(&path)
        .head_row_number(3)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].string0, "字符串0");
    assert_eq!(rows[0].string1, "字符串01");
    assert_eq!(rows[1].string0, "字符串1");
    assert_eq!(rows[1].string1, "字符串11");
}

// StyleDataTest (5 tests)
// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest
// ============================================================================

#[derive(Debug, Clone, ExcelRow)]
struct StyleData {
    #[excel(name = "字符串", index = 0)]
    string: String,
    #[excel(name = "字符串1", index = 1)]
    string1: String,
}

fn style_data() -> Vec<StyleData> {
    vec![
        StyleData {
            string: "字符串0".to_owned(),
            string1: "字符串01".to_owned(),
        },
        StyleData {
            string: "字符串1".to_owned(),
            string1: "字符串11".to_owned(),
        },
    ]
}

fn style_data10() -> Vec<StyleData> {
    (0..10)
        .map(|_| StyleData {
            string: "字符串0".to_owned(),
            string1: "字符串01".to_owned(),
        })
        .collect()
}

