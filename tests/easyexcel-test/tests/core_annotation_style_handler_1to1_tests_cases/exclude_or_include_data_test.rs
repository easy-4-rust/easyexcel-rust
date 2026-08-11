mod exclude_or_include_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t01ExcludeIndex07
    #[test]
    fn t01_exclude_index07() {
        assert_exclude_index(&temp_path("excludeIndex.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t02ExcludeIndex03
    #[test]
    fn t02_exclude_index03() {
        assert_exclude_index(&temp_path("excludeIndex03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t03ExcludeIndexCsv
    #[test]
    fn t03_exclude_index_csv() {
        assert_exclude_index(&temp_path("excludeIndex.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t11ExcludeFieldName07
    #[test]
    fn t11_exclude_field_name07() {
        assert_exclude_field_name(&temp_path("excludeFieldName.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t12ExcludeFieldName03
    #[test]
    fn t12_exclude_field_name03() {
        assert_exclude_field_name(&temp_path("excludeFieldName03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t13ExcludeFieldNameCsv
    #[test]
    fn t13_exclude_field_name_csv() {
        assert_exclude_field_name(&temp_path("excludeFieldName.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t21IncludeIndex07
    #[test]
    fn t21_include_index07() {
        assert_include_index(&temp_path("includeIndex.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t22IncludeIndex03
    #[test]
    fn t22_include_index03() {
        assert_include_index(&temp_path("includeIndex03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t23IncludeIndexCsv
    #[test]
    fn t23_include_index_csv() {
        assert_include_index(&temp_path("includeIndex.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t31IncludeFieldName07
    #[test]
    fn t31_include_field_name07() {
        assert_include_field_name(&temp_path("includeFieldName.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t32IncludeFieldName03
    #[test]
    fn t32_include_field_name03() {
        assert_include_field_name(&temp_path("includeFieldName03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t33IncludeFieldNameCsv
    #[test]
    fn t33_include_field_name_csv() {
        assert_include_field_name(&temp_path("includeFieldName.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t41IncludeFieldNameOrder07
    #[test]
    fn t41_include_field_name_order07() {
        assert_include_field_name_order(&temp_path("includeFieldNameOrder.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t42IncludeFieldNameOrder03
    #[test]
    fn t42_include_field_name_order03() {
        assert_include_field_name_order(&temp_path("includeFieldNameOrder03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t43IncludeFieldNameOrderCsv
    #[test]
    fn t43_include_field_name_order_csv() {
        assert_include_field_name_order(&temp_path("includeFieldNameOrder.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t41IncludeFieldNameOrderIndex07
    #[test]
    fn t41_include_field_name_order_index07() {
        assert_include_field_name_order_index(&temp_path("includeFieldNameOrderIndex.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t42IncludeFieldNameOrderIndex03
    #[test]
    fn t42_include_field_name_order_index03() {
        assert_include_field_name_order_index(&temp_path("includeFieldNameOrderIndex03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t43IncludeFieldNameOrderIndexCsv
    #[test]
    fn t43_include_field_name_order_index_csv() {
        assert_include_field_name_order_index(&temp_path("includeFieldNameOrderIndex.csv"));
    }
}
