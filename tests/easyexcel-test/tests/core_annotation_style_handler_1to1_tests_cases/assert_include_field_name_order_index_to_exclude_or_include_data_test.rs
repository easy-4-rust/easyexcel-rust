fn assert_include_field_name_order_index(path: &Path) {
    EasyExcel::write::<ExcludeOrIncludeData>(path)
        .include_column_indexes([3usize, 1, 2, 0])
        .order_by_include_column(true)
        .sheet("Sheet1")
        .do_write(exclude_include_data())
        .unwrap();
    let vals = dyn_strings(path);
    assert_eq!(vals.len(), 4);
    assert_eq!(vals[0], "column4");
    assert_eq!(vals[1], "column2");
    assert_eq!(vals[2], "column3");
    assert_eq!(vals[3], "column1");
}

// ============================================================================
// AnnotationDataTest (5 @Test)
// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest
// ============================================================================

mod annotation_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_annotation_dimensions(&temp_path("annotation07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        assert_annotation_dimensions(&temp_path("annotation03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t03ReadAndWriteCsv
    #[test]
    fn t03_read_and_write_csv() {
        assert_annotation_dimensions(&temp_path("annotationCsv.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t11WriteStyle07
    #[test]
    fn t11_write_style07() {
        assert_annotation_write_style(&temp_path("annotationStyle07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t12Write03
    #[test]
    fn t12_write03() {
        assert_annotation_write_style(&temp_path("annotationStyle03.xls"));
    }
}

// ============================================================================
// AnnotationIndexAndNameDataTest (3 @Test)
// Java: com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest
// ============================================================================

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

// ============================================================================
// StyleDataTest (5 @Test)
// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest
// ============================================================================

mod style_data_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest#t01ReadAndWrite07
    #[test]
    fn t01_read_and_write07() {
        assert_style_read_and_write(&temp_path("style07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest#t02ReadAndWrite03
    #[test]
    fn t02_read_and_write03() {
        assert_style_read_and_write(&temp_path("style03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest#t03AbstractVerticalCellStyleStrategy
    #[test]
    fn t03_abstract_vertical_cell_style_strategy() {
        assert_vertical_cell_style(&temp_path("verticalCellStyle.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest#t04AbstractVerticalCellStyleStrategy02
    #[test]
    fn t04_abstract_vertical_cell_style_strategy02() {
        // Java builds WriteCellStyle from StyleProperty/FontProperty; Rust uses same
        // column-differentiated VerticalCellStyleStrategy fills as t03.
        assert_vertical_cell_style(&temp_path("verticalCellStyle2.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.style.StyleDataTest#t05LoopMergeStrategy
    #[test]
    fn t05_loop_merge_strategy() {
        assert_loop_merge(&temp_path("loopMergeStrategy.xlsx"));
    }
}

// ============================================================================
// WriteHandlerTest (9 @Test)
// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest
// workbook / sheet / table × 07 / 03 / csv
// ============================================================================

mod write_handler_test {
    use super::*;

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t01WorkbookWrite07
    #[test]
    fn t01_workbook_write07() {
        assert_write_handler(&temp_path("writeHandler07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t02WorkbookWrite03
    #[test]
    fn t02_workbook_write03() {
        assert_write_handler(&temp_path("writeHandler03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t03WorkbookWriteCsv
    #[test]
    fn t03_workbook_write_csv() {
        assert_write_handler(&temp_path("writeHandlerCsv.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t11SheetWrite07
    #[test]
    fn t11_sheet_write07() {
        // Java: sheet().registerWriteHandler(...). Rust API registers at writer builder.
        assert_write_handler(&temp_path("writeHandlerSheet07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t12SheetWrite03
    #[test]
    fn t12_sheet_write03() {
        assert_write_handler(&temp_path("writeHandlerSheet03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t13SheetWriteCsv
    #[test]
    fn t13_sheet_write_csv() {
        assert_write_handler(&temp_path("writeHandlerSheetCsv.csv"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t21TableWrite07
    #[test]
    fn t21_table_write07() {
        // Java: sheet().table(0).registerWriteHandler(...). Rust registers at writer builder.
        assert_write_handler(&temp_path("writeHandlerTable07.xlsx"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t22TableWrite03
    #[test]
    fn t22_table_write03() {
        assert_write_handler(&temp_path("writeHandlerTable03.xls"));
    }

    /// Java: com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t23TableWriteCsv
    #[test]
    fn t23_table_write_csv() {
        assert_write_handler(&temp_path("writeHandlerTableCsv.csv"));
    }
}

// ============================================================================
// ExcludeOrIncludeDataTest (18 @Test)
// Java: com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest
// ============================================================================

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
