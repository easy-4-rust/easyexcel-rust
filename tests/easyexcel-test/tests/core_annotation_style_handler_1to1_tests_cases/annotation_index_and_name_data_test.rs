mod annotation_index_and_name_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_annotation_index_name(&temp_path("annotationIndexAndName07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        assert_annotation_index_name(&temp_path("annotationIndexAndName03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        assert_annotation_index_name(&temp_path("annotationIndexAndNameCsv.csv"));
    }
}
