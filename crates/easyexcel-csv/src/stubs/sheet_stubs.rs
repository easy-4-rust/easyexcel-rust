//! CsvSheet 的 STUB 方法集中文件。
//!
//! 包含 CSV 格式不支持的 Excel 工作表功能的 no-op 实现。
//! 对应 Java：com.alibaba.excel.metadata.csv.CsvSheet 中的 no-op 方法。

use crate::csv::{CsvCellValue, CsvRow, CsvSheet};

/// CsvSheet 的 STUB 方法实现。
///
/// 这些方法对应 Java CsvSheet 中因 CSV 格式限制而无法实现的功能，
/// 保留 no-op 语义以维持 Java API 调用兼容性。
impl<V: CsvCellValue> CsvSheet<V> {
    // ─── 合并区域 (Merged Region) ───

    /// CSV 的合并单元格在 Java 实现中是 no-op，并返回固定索引 `0`。
    /// 对应 Java: CsvSheet#addMergedRegion no-op
    #[must_use]
    pub const fn add_merged_region(&mut self) -> usize {
        0
    }

    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#numberOfMergedRegions no-op
    #[must_use]
    pub const fn number_of_merged_regions(&self) -> usize {
        0
    }
    /// 对应 Java: CsvSheet#getNumMergedRegions no-op
    pub const fn get_num_merged_regions(&self) -> usize { self.number_of_merged_regions() }

    /// CSV 不保存合并区域；不安全版本。
    /// 对应 Java: CsvSheet#addMergedRegion no-op
    pub const fn add_merged_region_unsafe(&mut self) -> usize { 0 }
    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#getMergedRegion no-op
    #[must_use] pub const fn get_merged_region(&self, _index: usize) -> Option<&str> { None }
    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#getMergedRegions no-op
    #[must_use] pub const fn get_merged_regions(&self) -> Vec<&str> { Vec::new() }
    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#removeMergedRegion no-op
    pub const fn remove_merged_region(&mut self, _index: usize) {}
    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#removeMergedRegions no-op
    pub const fn remove_merged_regions(&mut self, _indexes: &[usize]) {}
    /// CSV 不保存合并区域。
    /// 对应 Java: CsvSheet#validateMergedRegions no-op
    pub const fn validate_merged_regions(&self) {}

    // ─── 列宽/隐藏 (Column Width/Hidden) ───

    /// CSV 不保存列宽，Java getter 返回 `0`。
    /// 对应 Java: CsvSheet#getColumnWidth no-op
    #[must_use]
    pub const fn column_width(&self, _column_index: usize) -> usize {
        0
    }
    /// 对应 Java: CsvSheet#getColumnWidth no-op
    pub const fn get_column_width(&self, column_index: usize) -> usize {
        self.column_width(column_index)
    }

    /// CSV 列隐藏状态不持久化。
    /// 对应 Java: CsvSheet#isColumnHidden no-op
    #[must_use]
    pub const fn is_column_hidden(&self, _column_index: usize) -> bool {
        false
    }

    // ─── 冻结窗格 (Freeze Pane) ───

    /// CSV 不保存冻结窗格；保留 Java no-op 调用体验。
    /// 对应 Java: CsvSheet#createFreezePane no-op
    pub const fn create_freeze_pane(&mut self, _column_split: usize, _row_split: usize) {}

    /// CSV 不保存分割窗格；保留 Java no-op 调用体验。
    /// 对应 Java: CsvSheet#createSplitPane no-op
    pub const fn create_split_pane(&mut self, _x_split: usize, _y_split: usize, _left: usize, _top: usize) {}

    /// CSV 不保存窗格显示；保留 Java no-op 调用体验。
    /// 对应 Java: CsvSheet#showInPane no-op
    pub const fn show_in_pane(&mut self, _top_row: usize, _left_column: usize) {}

    // ─── 缩放 (Zoom) ───

    /// CSV 不保存缩放；保留 Java no-op 调用体验。
    /// 对应 Java: CsvSheet#setZoom no-op
    pub const fn set_zoom(&mut self, _scale: usize) {}

    /// CSV 不保存缩放比例。
    /// 对应 Java: CsvSheet#getZoom no-op
    #[must_use]
    pub const fn get_zoom(&self) -> usize { 0 }

    // ─── 公式重算 (Formula Recalc) ───

    /// CSV 本身不存储公式重算标志。
    /// 对应 Java: CsvSheet#forceFormulaRecalculation no-op
    #[must_use]
    pub const fn force_formula_recalculation(&self) -> bool {
        false
    }
    /// 对应 Java: CsvSheet#getForceFormulaRecalculation no-op
    pub const fn get_force_formula_recalculation(&self) -> bool {
        self.force_formula_recalculation()
    }

    /// CSV 不存储公式重算标志。
    /// 对应 Java: CsvSheet#setForceFormulaRecalculation no-op
    pub const fn set_force_formula_recalculation(&mut self, _value: bool) {}

    // ─── 视图属性 Getter (View Properties Getter) ───

    /// Java CSV Sheet 的不持久化视图属性。
    /// 对应 Java: CsvSheet#getDefaultColumnWidth no-op
    pub const fn get_default_column_width(&self) -> usize { 0 }
    /// 对应 Java: CsvSheet#getDefaultRowHeight no-op
    pub const fn get_default_row_height(&self) -> u16 { 0 }
    /// 对应 Java: CsvSheet#getDefaultRowHeightInPoints no-op
    pub const fn get_default_row_height_in_points(&self) -> f32 { 0.0 }
    /// 对应 Java: CsvSheet#getHorizontallyCenter no-op
    pub const fn get_horizontally_center(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getVerticallyCenter no-op
    pub const fn get_vertically_center(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isDisplayZeros no-op
    pub const fn is_display_zeros(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isDisplayFormulas no-op
    pub const fn is_display_formulas(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isPrintGridlines no-op
    pub const fn is_print_gridlines(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isSelected no-op
    pub const fn is_selected(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isRightToLeft no-op
    pub const fn is_right_to_left(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getTopRow no-op
    pub const fn get_top_row(&self) -> usize { 0 }
    /// 对应 Java: CsvSheet#getLeftCol no-op
    pub const fn get_left_col(&self) -> usize { 0 }
    /// 对应 Java: CsvSheet#getMargin no-op
    pub const fn get_margin(&self, _margin: usize) -> f64 { 0.0 }

    // ─── 视图 Setter (View Setter) ───

    /// 对应 Java: CsvSheet#setDefaultColumnWidth no-op
    pub const fn set_default_column_width(&mut self, _width: usize) {}
    /// 对应 Java: CsvSheet#setDefaultRowHeight no-op
    pub const fn set_default_row_height(&mut self, _height: u16) {}
    /// 对应 Java: CsvSheet#setDefaultRowHeightInPoints no-op
    pub const fn set_default_row_height_in_points(&mut self, _height: f32) {}
    /// 对应 Java: CsvSheet#setHorizontallyCenter no-op
    pub const fn set_horizontally_center(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setVerticallyCenter no-op
    pub const fn set_vertically_center(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setDisplayZeros no-op
    pub const fn set_display_zeros(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setDisplayFormulas no-op
    pub const fn set_display_formulas(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setPrintGridlines no-op
    pub const fn set_print_gridlines(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setSelected no-op
    pub const fn set_selected(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setRightToLeft no-op
    pub const fn set_right_to_left(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setZoom no-op
    // (already defined above in Zoom section)

    // ─── 行操作 (Row Operations) ───

    /// CSV 不支持行移动。
    /// 对应 Java: CsvSheet#shiftRows no-op
    pub const fn shift_rows(&mut self, _start: u32, _end: u32, _count: i32) {}
    /// CSV 不支持列移动。
    /// 对应 Java: CsvSheet#shiftColumns no-op
    pub const fn shift_columns(&mut self, _start: u16, _end: u16, _count: i32) {}

    /// CSV 行迭代器；STUB 委托给 rows()。
    /// 对应 Java: CsvSheet#rowIterator no-op
    pub fn row_iterator(&self) -> impl Iterator<Item = &CsvRow<V>> { self.rows() }

    // ─── 列样式/轮廓/分页符 (Column Style/Outline/Breaks) ───

    /// CSV 不保存列样式、轮廓、分页符或窗格。
    /// 对应 Java: CsvSheet#getColumnStyle no-op
    #[must_use] pub const fn get_column_style(&self, _column: usize) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getColumnWidthInPixels no-op
    #[must_use] pub const fn get_column_width_in_pixels(&self, _column: usize) -> f32 { 0.0 }
    /// 对应 Java: CsvSheet#getColumnOutlineLevel no-op
    #[must_use] pub const fn get_column_outline_level(&self, _column: usize) -> u8 { 0 }
    /// 对应 Java: CsvSheet#getColumnBreaks no-op
    #[must_use] pub const fn get_column_breaks(&self) -> Vec<usize> { Vec::new() }
    /// 对应 Java: CsvSheet#getRowBreaks no-op
    #[must_use] pub const fn get_row_breaks(&self) -> Vec<usize> { Vec::new() }
    /// 对应 Java: CsvSheet#isColumnBroken no-op
    #[must_use] pub const fn is_column_broken(&self, _column: usize) -> bool { false }
    /// 对应 Java: CsvSheet#isRowBroken no-op
    #[must_use] pub const fn is_row_broken(&self, _row: usize) -> bool { false }
    /// 对应 Java: CsvSheet#getPaneInformation no-op
    #[must_use] pub const fn get_pane_information(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#setColumnBreak no-op
    pub const fn set_column_break(&mut self, _column: usize) {}
    /// 对应 Java: CsvSheet#removeColumnBreak no-op
    pub const fn remove_column_break(&mut self, _column: usize) {}
    /// 对应 Java: CsvSheet#setRowBreak no-op
    pub const fn set_row_break(&mut self, _row: usize) {}
    /// 对应 Java: CsvSheet#removeRowBreak no-op
    pub const fn remove_row_break(&mut self, _row: usize) {}
    /// 对应 Java: CsvSheet#groupColumn no-op
    pub const fn group_column(&mut self, _from: usize, _to: usize) {}
    /// 对应 Java: CsvSheet#ungroupColumn no-op
    pub const fn ungroup_column(&mut self, _from: usize, _to: usize) {}
    /// 对应 Java: CsvSheet#groupRow no-op
    pub const fn group_row(&mut self, _from: usize, _to: usize) {}
    /// 对应 Java: CsvSheet#ungroupRow no-op
    pub const fn ungroup_row(&mut self, _from: usize, _to: usize) {}
    /// 对应 Java: CsvSheet#setColumnGroupCollapsed no-op
    pub const fn set_column_group_collapsed(&mut self, _column: usize, _collapsed: bool) {}
    /// 对应 Java: CsvSheet#setRowGroupCollapsed no-op
    pub const fn set_row_group_collapsed(&mut self, _row: usize, _collapsed: bool) {}
    /// 对应 Java: CsvSheet#setColumnHidden no-op
    pub const fn set_column_hidden(&mut self, _column: usize, _hidden: bool) {}
    /// 对应 Java: CsvSheet#setDefaultColumnStyle no-op
    pub const fn set_default_column_style(&mut self, _column: usize, _style: Option<&str>) {}
    /// 对应 Java: CsvSheet#autoSizeColumn no-op
    pub const fn auto_size_column(&mut self, _column: usize, _use_merged_cells: bool) {}

    // ─── 打印/显示状态 (Print/Display State) ───

    /// Java CSV Sheet 的打印/显示 no-op 状态。
    /// 对应 Java: CsvSheet#isDisplayGridlines no-op
    #[must_use] pub const fn is_display_gridlines(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isDisplayRowColHeadings no-op
    #[must_use] pub const fn is_display_row_col_headings(&self) -> bool { false }
    /// 对应 Java: CsvSheet#isPrintRowAndColumnHeadings no-op
    #[must_use] pub const fn is_print_row_and_column_headings(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getAutobreaks no-op
    #[must_use] pub const fn get_autobreaks(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getDisplayGuts no-op
    #[must_use] pub const fn get_display_guts(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getFitToPage no-op
    #[must_use] pub const fn get_fit_to_page(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getRowSumsBelow no-op
    #[must_use] pub const fn get_row_sums_below(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getRowSumsRight no-op
    #[must_use] pub const fn get_row_sums_right(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getScenarioProtect no-op
    #[must_use] pub const fn get_scenario_protect(&self) -> bool { false }
    /// 对应 Java: CsvSheet#getProtect no-op
    #[must_use] pub const fn get_protect(&self) -> bool { false }
    /// 对应 Java: CsvSheet#setDisplayGridlines no-op
    pub const fn set_display_gridlines(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setDisplayRowColHeadings no-op
    pub const fn set_display_row_col_headings(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setPrintRowAndColumnHeadings no-op
    pub const fn set_print_row_and_column_headings(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setAutobreaks no-op
    pub const fn set_autobreaks(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setDisplayGuts no-op
    pub const fn set_display_guts(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setFitToPage no-op
    pub const fn set_fit_to_page(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setRowSumsBelow no-op
    pub const fn set_row_sums_below(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setRowSumsRight no-op
    pub const fn set_row_sums_right(&mut self, _value: bool) {}
    /// 对应 Java: CsvSheet#setMargin no-op
    pub const fn set_margin(&mut self, _margin: usize, _size: f64) {}
    /// 对应 Java: CsvSheet#setAutoFilter no-op
    pub const fn set_auto_filter(&mut self, _range: &str) {}
    /// 对应 Java: CsvSheet#setRepeatingColumns no-op
    pub const fn set_repeating_columns(&mut self, _range: Option<&str>) {}
    /// 对应 Java: CsvSheet#setRepeatingRows no-op
    pub const fn set_repeating_rows(&mut self, _range: Option<&str>) {}
    /// 对应 Java: CsvSheet#getRepeatingColumns no-op
    #[must_use] pub const fn get_repeating_columns(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getRepeatingRows no-op
    #[must_use] pub const fn get_repeating_rows(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getActiveCell no-op
    #[must_use] pub const fn get_active_cell(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#setActiveCell no-op
    pub const fn set_active_cell(&mut self, _reference: &str) {}

    // ─── 批注/超链接/验证 (Comment/Hyperlink/Validation) ───

    /// CSV 不保存批注。
    /// 对应 Java: CsvSheet#getCellComments no-op
    #[must_use] pub const fn get_cell_comments(&self) -> Vec<&str> { Vec::new() }
    /// CSV 不保存批注。
    /// 对应 Java: CsvSheet#getCellComment no-op
    #[must_use] pub const fn get_cell_comment(&self, _reference: &str) -> Option<&str> { None }
    /// CSV 不保存超链接。
    /// 对应 Java: CsvSheet#getHyperlinkList no-op
    #[must_use] pub const fn get_hyperlink_list(&self) -> Vec<&str> { Vec::new() }
    /// CSV 不保存数据验证。
    /// 对应 Java: CsvSheet#getDataValidations no-op
    #[must_use] pub const fn get_data_validations(&self) -> Vec<&str> { Vec::new() }
    /// 对应 Java: CsvSheet#addValidationData no-op
    pub const fn add_validation_data(&mut self, _validation: &str) {}
    /// 对应 Java: CsvSheet#getDrawingPatriarch no-op
    #[must_use] pub const fn get_drawing_patriarch(&self) -> Option<&str> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvSheet#createDrawingPatriarch no-op
    #[must_use] pub const fn create_drawing_patriarch(&mut self) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvSheet#setArrayFormula no-op
    #[must_use] pub const fn set_array_formula(&mut self, _formula: &str, _range: &str) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvSheet#removeArrayFormula no-op
    #[must_use] pub const fn remove_array_formula(&mut self, _row: u32, _column: u16) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvSheet#getHyperlink no-op
    #[must_use] pub const fn get_hyperlink(&self, _row: u32, _column: u16) -> Option<()> { None }
    /// 对应 Java: CsvSheet#getSheetConditionalFormatting no-op
    #[must_use] pub const fn get_sheet_conditional_formatting(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getDataValidationHelper no-op
    #[must_use] pub const fn get_data_validation_helper(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getPrintSetup no-op
    #[must_use] pub const fn get_print_setup(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getHeader no-op
    #[must_use] pub const fn get_header(&self) -> Option<&str> { None }
    /// 对应 Java: CsvSheet#getFooter no-op
    #[must_use] pub const fn get_footer(&self) -> Option<&str> { None }
}
