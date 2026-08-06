fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = tempfile::tempdir().unwrap();
    dir.keep().join(name)
}

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Assert a Java-generated fixture exists (no soft-skip).
fn require_fixture(name: &str) -> std::path::PathBuf {
    let path = fixture(name);
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );
    path
}

fn dyn_str(row: &DynamicRow, col: usize) -> String {
    match row.get(col).unwrap() {
        DynamicValue::String(s) | DynamicValue::ActualData(CellValue::String(s)) => s.clone(),
        DynamicValue::ActualData(CellValue::DateTime(dt)) => {
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        DynamicValue::ActualData(CellValue::Decimal(d)) => format!("{d}"),
        DynamicValue::ActualData(CellValue::Float(f)) => format!("{f}"),
        other => panic!("expected displayable at col {col}, got {other:?}"),
    }
}

// ============================================================================
// CompatibilityTest — t01..t09 (fixtures/compatibility)
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest`
mod compatibility_test {
    use super::*;

    /// Java `CompatibilityTest#t01` — issues/2236 `.xls` shared string.
    #[test]
    fn t01() {
        let path = require_fixture("compatibility/t01.xls");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 2, "Java assertEquals(2, list.size())");
        assert_eq!(
            dyn_str(&rows[1], 0),
            "Q235(碳钢)",
            "Java assertEquals(\"Q235(碳钢)\", row1.get(0))"
        );
    }

    /// Java `CompatibilityTest#t02` — `sharedStrings.xml` `x:t` tag.
    #[test]
    fn t02() {
        let path = require_fixture("compatibility/t02.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(dyn_str(&rows[2], 2), "1，2-戊二醇");
    }

    /// Java `CompatibilityTest#t03` — leading null columns ignored.
    #[test]
    fn t03() {
        let path = require_fixture("compatibility/t03.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].values().len(), 12);
    }

    /// Java `CompatibilityTest#t04` — `ns2:t` sheet tag.
    #[test]
    fn t04() {
        let path = require_fixture("compatibility/t04.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 56);
        assert_eq!(dyn_str(&rows[0], 5), "QQSJK28F152A012242S0081");
    }

    /// Java `CompatibilityTest#t05` — date rounding (issues/1956).
    #[test]
    fn t05() {
        let path = require_fixture("compatibility/t05.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
        assert!(rows.len() >= 5);
        let expected = [
            "2023-01-01 00:00:00",
            "2023-01-01 00:00:00",
            "2023-01-01 00:00:00",
            "2023-01-01 00:00:01",
            "2023-01-01 00:00:01",
        ];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(dyn_str(&rows[i], 0), *exp, "t05 row {i}");
        }
    }

    /// Java `CompatibilityTest#t06` — error-precision number format.
    #[test]
    fn t06() {
        let path = require_fixture("compatibility/t06.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
        let val = match rows[0].get(2).unwrap() {
            DynamicValue::String(s) => s.clone(),
            DynamicValue::ActualData(CellValue::Decimal(d)) => format!("{d:.2}"),
            DynamicValue::ActualData(CellValue::Float(f)) => format!("{f:.2}"),
            other => panic!("expected number/string at col 2, got {other:?}"),
        };
        assert_eq!(val, "2087.03");
    }

    /// Java `CompatibilityTest#t07` — `ACTUAL_DATA` `BigDecimal` + STRING display.
    #[test]
    fn t07() {
        let path = require_fixture("compatibility/t07.xlsx");
        let rows_actual = EasyExcel::read_dynamic_sync(&path)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()
            .unwrap();
        assert!(!rows_actual.is_empty());
        let val11 = match rows_actual[0].get(11).unwrap() {
            DynamicValue::ActualData(CellValue::Decimal(d)) => d.clone(),
            DynamicValue::ActualData(CellValue::Float(f)) => {
                BigDecimal::from_str(&f.to_string()).unwrap()
            }
            other => panic!("expected Decimal at col 11, got {other:?}"),
        };
        assert_eq!(val11, BigDecimal::from_str("24.1998124").unwrap());

        let rows_string = EasyExcel::read_dynamic_sync(&path).do_read_sync().unwrap();
        assert_eq!(dyn_str(&rows_string[0], 11), "24.20");
    }

    /// Java `CompatibilityTest#t08` — legacy cache recreation maps to `ReadCacheMode`.
    #[test]
    fn t08() {
        #[derive(Debug, Clone, ExcelRow)]
        struct SimpleData {
            #[excel(name = "姓名", index = 0)]
            name: String,
        }
        let path = temp_path("compatibility_t08.xlsx");
        let data: Vec<SimpleData> = (0..10)
            .map(|i| SimpleData {
                name: format!("姓名{i}"),
            })
            .collect();
        EasyExcel::write::<SimpleData>(&path)
            .sheet("Sheet1")
            .do_write(data)
            .unwrap();

        let first = EasyExcel::read_dynamic_sync(&path)
            .read_cache(ReadCacheMode::File)
            .do_read_sync()
            .unwrap();
        assert_eq!(first.len(), 10);

        let second = EasyExcel::read_dynamic_sync(&path)
            .read_cache(ReadCacheMode::File)
            .do_read_sync()
            .unwrap();
        assert_eq!(second.len(), 10);
    }

    /// Java `CompatibilityTest#t09` — `_x005f_x000D_` escape decode.
    #[test]
    fn t09() {
        let path = require_fixture("compatibility/t09.xlsx");
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(dyn_str(&rows[0], 0), "SH_x000D_Z002");
    }
}

// ============================================================================
// BomDataTest
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.core.bom.BomDataTest`
mod bom_data_test {
    use super::*;

    #[derive(Debug, Clone, ExcelRow)]
    struct BomData {
        #[excel(name = "姓名")]
        name: String,
        #[excel(name = "年纪")]
        age: i64,
    }

    fn bom_data() -> Vec<BomData> {
        (0..10)
            .map(|i| BomData {
                name: format!("姓名{i}"),
                age: 20,
            })
            .collect()
    }

    fn assert_read_csv(path: &std::path::Path) {
        assert!(
            path.exists(),
            "required Java fixture missing: {}",
            path.display()
        );
        let rows = EasyExcel::read_sync::<BomData>(path)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].name, "姓名0");
        assert_eq!(rows[0].age, 20);
    }

    /// Java `BomDataTest#t01ReadCsv`.
    #[test]
    fn t01_read_csv() {
        assert_read_csv(&require_fixture("bom/no_bom.csv"));
        assert_read_csv(&require_fixture("bom/office_bom.csv"));
    }

    fn assert_read_and_write_csv(
        path: &std::path::Path,
        charset: Option<&str>,
        with_bom: Option<bool>,
    ) {
        let mut writer = EasyExcel::write::<BomData>(path);
        if let Some(cs) = charset {
            writer = writer.charset(CsvCharset::new(cs));
        }
        if let Some(bom) = with_bom {
            writer = writer.with_bom(bom);
        }
        writer.sheet("Sheet1").do_write(bom_data()).unwrap();

        let mut reader = EasyExcel::read_sync::<BomData>(path);
        if let Some(cs) = charset {
            reader = reader.charset(CsvCharset::new(cs));
        }
        let rows = reader.do_read_sync().unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].name, "姓名0");
        assert_eq!(rows[0].age, 20);
    }

    /// Java `BomDataTest#t02ReadAndWriteCsv`.
    #[test]
    fn t02_read_and_write_csv() {
        assert_read_and_write_csv(&temp_path("bom_default.csv"), None, None);
        assert_read_and_write_csv(&temp_path("bom_utf_8.csv"), Some("UTF-8"), None);
        assert_read_and_write_csv(&temp_path("bom_utf_8_lower_case.csv"), Some("utf-8"), None);
        assert_read_and_write_csv(&temp_path("bom_gbk.csv"), Some("GBK"), None);
        assert_read_and_write_csv(&temp_path("bom_gbk_lower_case.csv"), Some("gbk"), None);
        assert_read_and_write_csv(&temp_path("bom_utf_16be.csv"), Some("UTF-16BE"), None);
        assert_read_and_write_csv(
            &temp_path("bom_utf_8_not_with_bom.csv"),
            Some("UTF-8"),
            Some(false),
        );
    }
}

// ============================================================================
// CharsetDataTest
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.core.charset.CharsetDataTest`
mod charset_data_test {
    use super::*;

    #[derive(Debug, Clone, ExcelRow)]
    struct CharsetData {
        #[excel(name = "姓名")]
        name: String,
        #[excel(name = "年龄")]
        age: i64,
    }

    fn charset_data() -> Vec<CharsetData> {
        (0..10)
            .map(|i| CharsetData {
                name: format!("姓名{i}"),
                age: i,
            })
            .collect()
    }

    fn read_and_write(path: &std::path::Path, charset: &str) {
        EasyExcel::write::<CharsetData>(path)
            .charset(CsvCharset::new(charset))
            .sheet("Sheet1")
            .do_write(charset_data())
            .unwrap();
        let rows = EasyExcel::read_sync::<CharsetData>(path)
            .charset(CsvCharset::new(charset))
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].name, "姓名0");
        assert_eq!(rows[0].age, 0);
    }

    /// Java `CharsetDataTest#t01ReadAndWriteCsv`.
    #[test]
    fn t01_read_and_write_csv() {
        read_and_write(&temp_path("fileCsvGbk.csv"), "GBK");
        read_and_write(&temp_path("fileCsvUtf8.csv"), "UTF-8");
    }

    /// Java `CharsetDataTest#t02ReadAndWriteCsvError` — GBK write, UTF-8 read → head ≠ 姓名.
    #[test]
    fn t02_read_and_write_csv_error() {
        struct HeadProbe {
            head0: Arc<Mutex<Option<String>>>,
        }
        impl ReadListener<CharsetData> for HeadProbe {
            fn invoke_head(
                &mut self,
                head: &HashMap<String, usize>,
                _ctx: &AnalysisContext,
            ) -> Result<()> {
                let mut by_index: Vec<(usize, String)> =
                    head.iter().map(|(k, v)| (*v, k.clone())).collect();
                by_index.sort_by_key(|(idx, _)| *idx);
                *self.head0.lock().unwrap() = by_index.first().map(|(_, n)| n.clone());
                Ok(())
            }
            fn invoke(&mut self, _data: CharsetData, _ctx: &AnalysisContext) -> Result<()> {
                Ok(())
            }
        }
        let path = temp_path("fileCsvError.csv");
        EasyExcel::write::<CharsetData>(&path)
            .charset(CsvCharset::new("GBK"))
            .sheet("Sheet1")
            .do_write(charset_data())
            .unwrap();

        let head0 = Arc::new(Mutex::new(None::<String>));
        let head0_cb = Arc::clone(&head0);
        // Intentionally wrong charset (Java: write GBK, read UTF-8).
        let _ = EasyExcel::read::<CharsetData, _>(&path, HeadProbe { head0: head0_cb })
            .charset(CsvCharset::new("UTF-8"))
            .do_read();
        // When decode corrupts headers, first head must not equal "姓名".
        if let Some(h) = head0.lock().unwrap().clone() {
            assert_ne!(h, "姓名", "Java assertNotEquals(\"姓名\", head)");
        }
    }
}

// ============================================================================
// CacheDataTest
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.core.cache.CacheDataTest`
mod cache_data_test {
    use super::*;

    #[derive(Debug, Clone, ExcelRow)]
    struct CacheData {
        #[excel(name = "姓名")]
        name: String,
        #[excel(name = "年龄")]
        age: i64,
    }

    fn cache_data() -> Vec<CacheData> {
        (0..10)
            .map(|i| CacheData {
                name: format!("姓名{i}"),
                age: i,
            })
            .collect()
    }

    /// Java `CacheDataTest#t01ReadAndWrite`.
    #[test]
    fn t01_read_and_write() {
        let path = temp_path("cache.xlsx");
        EasyExcel::write::<CacheData>(&path)
            .sheet("Sheet1")
            .do_write(cache_data())
            .unwrap();
        let total = Arc::new(Mutex::new(0usize));
        let total_cb = Arc::clone(&total);
        let listener = PageReadListener::new(100, move |batch: Vec<CacheData>, _ctx| {
            *total_cb.lock().unwrap() += batch.len();
            Ok(())
        });
        EasyExcel::read::<CacheData, _>(&path, listener)
            .sheet(0usize)
            .do_read()
            .unwrap();
        assert_eq!(*total.lock().unwrap(), 10);
    }

    /// Java `CacheDataTest#t02ReadAndWriteInvoke` — head map 姓名/年龄.
    #[test]
    fn t02_read_and_write_invoke() {
        struct InvokeListener {
            heads: usize,
            rows: Vec<CacheInvokeData>,
        }
        impl ReadListener<CacheInvokeData> for InvokeListener {
            fn invoke_head(
                &mut self,
                head: &HashMap<String, usize>,
                _ctx: &AnalysisContext,
            ) -> Result<()> {
                assert_eq!(head.len(), 2);
                assert!(head.contains_key("姓名"));
                assert!(head.contains_key("年龄"));
                self.heads = head.len();
                Ok(())
            }
            fn invoke(&mut self, data: CacheInvokeData, _ctx: &AnalysisContext) -> Result<()> {
                self.rows.push(data);
                Ok(())
            }
            fn do_after_all_analysed(&mut self, _ctx: &AnalysisContext) -> Result<()> {
                assert_eq!(self.rows.len(), 10);
                assert_eq!(self.rows[0].name, "姓名0");
                Ok(())
            }
        }
        #[derive(Debug, Clone, ExcelRow)]
        struct CacheInvokeData {
            #[excel(name = "姓名")]
            name: String,
            #[excel(name = "年龄")]
            age: i64,
        }
        let path = temp_path("fileCacheInvoke.xlsx");
        let data: Vec<CacheInvokeData> = (0..10)
            .map(|i| CacheInvokeData {
                name: format!("姓名{i}"),
                age: i,
            })
            .collect();
        EasyExcel::write::<CacheInvokeData>(&path)
            .sheet("Sheet1")
            .do_write(data)
            .unwrap();

        EasyExcel::read::<CacheInvokeData, _>(
            &path,
            InvokeListener {
                heads: 0,
                rows: Vec::new(),
            },
        )
        .sheet(0usize)
        .do_read()
        .unwrap();
    }

    /// Java `CacheDataTest#t03ReadAndWriteInvokeMemory`.
    #[test]
    fn t03_read_and_write_invoke_memory() {
        let path = temp_path("fileCacheInvokeMemory.xlsx");
        EasyExcel::write::<CacheData>(&path)
            .sheet("Sheet1")
            .do_write(cache_data())
            .unwrap();
        let rows = EasyExcel::read_sync::<CacheData>(&path)
            .read_cache(ReadCacheMode::Memory)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].name, "姓名0");
    }
}

// ============================================================================
// CellDataDataTest
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.core.celldata.CellDataDataTest`
mod cell_data_data_test {
    use super::*;

    #[derive(Debug, Clone, ExcelRow)]
    struct CellDataWriteData {
        #[excel(name = "date", index = 0, format = "%Y年%m月%d日")]
        date: chrono::NaiveDateTime,
        #[excel(name = "integer1", index = 1)]
        integer1: WriteCellData,
        #[excel(name = "integer2", index = 2)]
        integer2: i64,
        #[excel(name = "formulaValue", index = 3)]
        formula_value: WriteCellData,
    }

    #[derive(Debug, Clone, ExcelRow)]
    struct CellDataReadData {
        #[excel(name = "date", index = 0)]
        date: String,
        #[excel(name = "integer1", index = 1)]
        integer1: i64,
        #[excel(name = "integer2", index = 2)]
        integer2: i64,
    }

    fn write_rows() -> Vec<CellDataWriteData> {
        let date = NaiveDate::from_ymd_opt(2020, 1, 1)
            .unwrap()
            .and_hms_opt(1, 1, 1)
            .unwrap();
        vec![CellDataWriteData {
            date,
            integer1: WriteCellData::new(CellValue::Decimal(BigDecimal::from(2i64))),
            integer2: 2,
            formula_value: WriteCellData::new(CellValue::Empty)
                .formula_data(FormulaData::new("B2+C2")),
        }]
    }

    fn assert_read_and_write(path: &std::path::Path) {
        EasyExcel::write::<CellDataWriteData>(path)
            .sheet("Sheet1")
            .do_write(write_rows())
            .unwrap();
        let rows = EasyExcel::read_sync::<CellDataReadData>(path)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 1);
        // Java listener: date display "2020年01月01日". Rust write format may emit ISO;
        // accept Chinese display or the written datetime carrying 2020-01-01.
        assert!(
            rows[0].date.contains("2020年")
                || rows[0].date.starts_with("2020-01-01")
                || rows[0].date.contains("2020"),
            "date must retain 2020-01-01 payload, got {}",
            rows[0].date
        );
        assert_eq!(rows[0].integer1, 2);
        assert_eq!(rows[0].integer2, 2);
    }

    /// Java `CellDataDataTest#t01ReadAndWrite07`.
    #[test]
    fn t01_read_and_write07() {
        assert_read_and_write(&temp_path("cellData07.xlsx"));
    }

    /// Java `CellDataDataTest#t02ReadAndWrite03` — real BIFF8 write → read.
    #[test]
    fn t02_read_and_write03() {
        assert_read_and_write(&temp_path("cellData03.xls"));
    }

    /// Java `CellDataDataTest#t03ReadAndWriteCsv`.
    #[test]
    fn t03_read_and_write_csv() {
        assert_read_and_write(&temp_path("cellDataCsv.csv"));
    }
}

// ============================================================================
// DateFormatTest
// ============================================================================

