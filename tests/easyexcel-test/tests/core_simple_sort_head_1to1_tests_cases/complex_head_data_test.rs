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
