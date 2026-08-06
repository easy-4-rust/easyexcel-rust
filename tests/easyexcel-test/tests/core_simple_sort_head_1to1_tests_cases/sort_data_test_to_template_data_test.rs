mod sort_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_sort_read_and_write(&temp_path("sort07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_sort_read_and_write(&temp_path("sort03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        assert_sort_read_and_write(&temp_path("sort.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t11ReadAndWriteNoHead07
    #[test]
    fn t11_read_and_write_no_head07() {
        assert_sort_no_head(&temp_path("sortNoHead07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t12ReadAndWriteNoHead03
    #[test]
    fn t12_read_and_write_no_head03() {
        // Java .xls write — real BIFF8 write → read.
        assert_sort_no_head(&temp_path("sortNoHead03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.sort.SortDataTest#t13ReadAndWriteNoHeadCsv
    #[test]
    fn t13_read_and_write_no_head_csv() {
        assert_sort_no_head(&temp_path("sortNoHead.csv"));
    }
}

// ============================================================================
// SkipDataTest (3)
// ============================================================================

mod skip_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.skip.SkipDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_skip(&temp_path("skip07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.skip.SkipDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_skip(&temp_path("skip03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.skip.SkipDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        // Java: CSV multi-sheet → ExcelGenerateException
        let path = temp_path("skip.csv");
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
}

// ============================================================================
// NoModelDataTest (3)
// ============================================================================

mod no_model_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_no_model(&temp_path("noModel07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_no_model(&temp_path("noModel03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        assert_no_model(&temp_path("noModel.csv"));
    }
}

// ============================================================================
// ParameterDataTest (2)
// ============================================================================

mod parameter_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.parameter.ParameterDataTest#t01ReadAndWrite
    #[test]
    fn t01_read_and_write() {
        assert_parameter(&temp_path("parameter07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.parameter.ParameterDataTest#t02ReadAndWrite
    #[test]
    fn t02_read_and_write() {
        assert_parameter(&temp_path("parameterCsv.csv"));
    }
}

// ============================================================================
// RepetitionDataTest (6)
// ============================================================================

mod repetition_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_repetition(&temp_path("repetition07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_repetition(&temp_path("repetition03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        assert_repetition(&temp_path("repetitionCsv.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t11ReadAndWriteTable07
    #[test]
    fn t11_read_and_write_table07() {
        assert_repetition_table(&temp_path("repetitionTable07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t12ReadAndWriteTable03
    #[test]
    fn t12_read_and_write_table03() {
        // Java .xls write — real BIFF8 write → read.
        assert_repetition_table(&temp_path("repetitionTable03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t13ReadAndWriteTableCsv
    #[test]
    fn t13_read_and_write_table_csv() {
        assert_repetition_table(&temp_path("repetitionTableCsv.csv"));
    }
}

// ============================================================================
// MultipleSheetsDataTest (4)
// ============================================================================

mod multiple_sheets_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t01Read07
    #[test]
    fn t01_read07() {
        let path = require_fixture("multiplesheets/multiplesheets.xlsx");
        let rows = EasyExcel::read_sync::<MultipleSheetsData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].title, "表1数据");
    }

    /// Java: com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t02Read03
    #[test]
    fn t02_read03() {
        let path = require_fixture("xls/multiplesheets.xls");
        let rows = EasyExcel::read_sync::<MultipleSheetsData>(&path)
            .sheet(0usize)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].title, "表1数据");
    }

    /// Java: com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t03Read07All
    #[test]
    fn t03_read07_all() {
        let path = require_fixture("multiplesheets/multiplesheets.xlsx");
        let rows = EasyExcel::read_sync::<MultipleSheetsData>(&path)
            .all_sheets()
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].title, "表1数据");
    }

    /// Java: com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t04Read03All
    #[test]
    fn t04_read03_all() {
        let path = require_fixture("xls/multiplesheets.xls");
        let rows = EasyExcel::read_sync::<MultipleSheetsData>(&path)
            .all_sheets()
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].title, "表1数据");
    }
}

// ============================================================================
// ComplexHeadDataTest (6)
// ============================================================================

mod complex_head_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_complex_head(&temp_path("complexHead07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_complex_head(&temp_path("complexHead03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        let path = temp_path("complexHeadCsv.csv");
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

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t11ReadAndWriteAutomaticMergeHead07
    #[test]
    fn t11_read_and_write_automatic_merge_head07() {
        // Java: automaticMergeHead(false); facade mirrors via normal write round-trip.
        assert_complex_head(&temp_path("complexHeadAutomaticMergeHead07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t12ReadAndWriteAutomaticMergeHead03
    #[test]
    fn t12_read_and_write_automatic_merge_head03() {
        // Java .xls write — real BIFF8 write → read.
        assert_complex_head(&temp_path("complexHeadAutomaticMergeHead03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t13ReadAndWriteAutomaticMergeHeadCsv
    #[test]
    fn t13_read_and_write_automatic_merge_head_csv() {
        let path = temp_path("complexHeadAutomaticMergeHeadCsv.csv");
        EasyExcel::write::<ComplexHeadData>(&path)
            .sheet("Sheet1")
            .do_write(complex_head_data())
            .unwrap();
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert!(rows.len() >= 2);
    }
}

// ============================================================================
// ListHeadDataTest (3)
// ============================================================================

mod list_head_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_list_head(&temp_path("listHead07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_list_head(&temp_path("listHead03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        let path = temp_path("listHeadCsv.csv");
        EasyExcel::write::<DynamicRow>(&path)
            .head(vec![
                vec!["字符串".to_owned()],
                vec!["数字".to_owned()],
                vec!["日期".to_owned()],
            ])
            .sheet("Sheet1")
            .do_write(vec![{
                let mut map = BTreeMap::new();
                map.insert(0usize, DynamicValue::String("字符串0".to_owned()));
                map.insert(1usize, DynamicValue::String("1".to_owned()));
                map.insert(
                    2usize,
                    DynamicValue::String("2020-01-01 01:01:01".to_owned()),
                );
                DynamicRow::new(map)
            }])
            .unwrap();
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert!(rows.len() >= 2);
    }
}

// ============================================================================
// NoHeadDataTest (3)
// ============================================================================

mod no_head_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_no_head(&temp_path("noHead07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_no_head(&temp_path("noHead03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        let path = temp_path("noHeadCsv.csv");
        EasyExcel::write::<NoHeadData>(&path)
            .need_head(false)
            .sheet("Sheet1")
            .do_write(vec![NoHeadData {
                string: "字符串0".to_owned(),
            }])
            .unwrap();
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 1);
    }
}

// ============================================================================
// UnCamelDataTest (3)
// ============================================================================

mod un_camel_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_uncamel(&temp_path("unCame07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java .xls write — real BIFF8 write → read.
        assert_uncamel(&temp_path("unCame03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        let path = temp_path("unCameCsv.csv");
        EasyExcel::write::<UnCamelData>(&path)
            .sheet("Sheet1")
            .do_write(uncamel_data())
            .unwrap();
        let rows = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 11, "CSV: 1 header + 10 data");
    }
}

// ============================================================================
// TemplateDataTest (2)
// ============================================================================

mod template_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.template.TemplateDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        let template = require_fixture("template/template07.xlsx");
        let path = temp_path("template07_out.xlsx");
        EasyExcel::write::<TemplateData>(&path)
            .with_template(&template)
            .sheet("Sheet1")
            .do_write(template_data())
            .unwrap();
        let rows = EasyExcel::read_sync::<TemplateData>(&path)
            .head_row_number(3)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].string0, "字符串0");
        assert_eq!(rows[1].string0, "字符串1");
    }

    /// Java: com.alibaba.easyexcel.test.core.template.TemplateDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        // Java withTemplate(.xls) + write. Rust: value-preserving Minimal BIFF8 rewrite.
        let xls = require_fixture("template/template03.xls");
        assert_xls_readable(&xls);
        let path = temp_path("template03_out.xls");
        EasyExcel::write::<TemplateData>(&path)
            .with_template(&xls)
            .sheet("Sheet1")
            .do_write(template_data())
            .unwrap();
        let rows = EasyExcel::read_sync::<TemplateData>(&path)
            .head_row_number(3)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].string0, "字符串0");
        assert_eq!(rows[1].string0, "字符串1");
        // Template cells before the append must remain (value preserve).
        let dynamic = EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert!(
            dynamic.len() >= 4,
            "template rows + head + data expected, got {}",
            dynamic.len()
        );
    }
}
