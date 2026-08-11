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

    /// Java HSSF：明文/加密 XLS 模板均可按调用级密码解密、修改并重新加密。
    #[test]
    fn t02_read_and_write03_with_password() {
        let template = require_fixture("template/template03.xls");
        let encrypted = temp_path("template03_encrypted.xls");
        EasyExcel::write::<TemplateData>(&encrypted)
            .with_template(&template)
            .password("123456")
            .sheet("Sheet1")
            .do_write(template_data())
            .unwrap();
        let rows = EasyExcel::read_sync::<TemplateData>(&encrypted)
            .password("123456")
            .head_row_number(3)
            .do_read_sync()
            .unwrap();
        assert_eq!(rows.len(), 2);

        let rewritten = temp_path("template03_reencrypted.xls");
        EasyExcel::write::<TemplateData>(&rewritten)
            .with_template(&encrypted)
            .password("123456")
            .sheet("Sheet1")
            .do_write(template_data())
            .unwrap();
        let rows = EasyExcel::read_dynamic_sync(&rewritten)
            .password("123456")
            .head_row_number(0)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty());
    }
}
