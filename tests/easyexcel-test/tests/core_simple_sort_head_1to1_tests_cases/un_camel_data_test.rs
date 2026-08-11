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
