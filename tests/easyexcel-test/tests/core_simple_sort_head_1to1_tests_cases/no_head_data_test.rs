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
