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
