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
