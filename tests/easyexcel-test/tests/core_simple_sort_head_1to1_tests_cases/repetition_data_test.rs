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
