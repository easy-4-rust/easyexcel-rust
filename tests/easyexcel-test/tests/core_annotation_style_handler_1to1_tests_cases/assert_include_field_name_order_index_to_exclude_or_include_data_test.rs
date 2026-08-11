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

