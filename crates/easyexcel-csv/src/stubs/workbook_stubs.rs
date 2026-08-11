//! CsvWorkbook 的 STUB 方法集中文件。
//!
//! 包含 CSV 格式不支持的 Excel 工作簿功能的 no-op 实现。
//! 对应 Java：com.alibaba.excel.metadata.csv.CsvWorkbook 中的 no-op 方法。

use crate::csv::{CsvCellValue, CsvWorkbook};

/// CsvWorkbook 的 STUB 方法实现。
///
/// 这些方法对应 Java CsvWorkbook 中因 CSV 格式限制而无法实现的功能，
/// 保留 no-op 语义以维持 Java API 调用兼容性。
impl<V: CsvCellValue> CsvWorkbook<V> {
    // ─── 字体/名称表 (Font/Name Table) ───

    /// CSV 没有字体表，对齐 Java 的固定返回值。
    /// 对应 Java: CsvWorkbook#numberOfFonts no-op
    #[must_use]
    pub const fn number_of_fonts(&self) -> usize {
        0
    }
    /// 对应 Java: CsvWorkbook#getNumberOfFonts no-op
    pub const fn get_number_of_fonts(&self) -> usize { self.number_of_fonts() }
    /// 对应 Java: CsvWorkbook#getNumberOfFontsAsInt no-op
    pub const fn get_number_of_fonts_as_int(&self) -> usize { self.number_of_fonts() }

    /// CSV 没有名称表，对齐 Java 的固定返回值。
    /// 对应 Java: CsvWorkbook#numberOfNames no-op
    #[must_use]
    pub const fn number_of_names(&self) -> usize {
        0
    }
    /// 对应 Java: CsvWorkbook#getNumberOfNames no-op
    pub const fn get_number_of_names(&self) -> usize { self.number_of_names() }

    // ─── 隐藏状态 (Hidden State) ───

    /// CSV 工作簿本身不持久化隐藏状态。
    /// 对应 Java: CsvWorkbook#isHidden no-op
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        false
    }

    // ─── 公式重算 (Formula Recalculation) ───

    /// CSV 工作簿本身不执行公式重算。
    /// 对应 Java: CsvWorkbook#forceFormulaRecalculation no-op
    #[must_use]
    pub const fn force_formula_recalculation(&self) -> bool {
        false
    }
    /// 对应 Java: CsvWorkbook#getForceFormulaRecalculation no-op
    pub const fn get_force_formula_recalculation(&self) -> bool {
        self.force_formula_recalculation()
    }

    /// CSV 不存储公式重算标志。
    /// 对应 Java: CsvWorkbook#setForceFormulaRecalculation no-op
    pub const fn set_force_formula_recalculation(&mut self, _value: bool) {}

    // ─── 工作表导航 (Sheet Navigation) ───

    /// Java CSV Workbook 对 POI 非 CSV 能力的确定性默认语义。
    /// 对应 Java: CsvWorkbook#getActiveSheetIndex no-op
    pub const fn get_active_sheet_index(&self) -> usize { 0 }
    /// 对应 Java: CsvWorkbook#getFirstVisibleTab no-op
    pub const fn get_first_visible_tab(&self) -> usize { 0 }
    /// 对应 Java: CsvWorkbook#getSheetIndex no-op
    pub const fn get_sheet_index(&self, _name: &str) -> usize { 0 }
    /// 对应 Java: CsvWorkbook#setActiveSheet no-op
    pub const fn set_active_sheet(&mut self, _index: usize) {}
    /// 对应 Java: CsvWorkbook#setFirstVisibleTab no-op
    pub const fn set_first_visible_tab(&mut self, _index: usize) {}
    /// 对应 Java: CsvWorkbook#setSelectedTab no-op
    pub const fn set_selected_tab(&mut self, _index: usize) {}
    /// 对应 Java: CsvWorkbook#setSheetOrder no-op
    pub const fn set_sheet_order(&mut self, _name: &str, _index: usize) {}
    /// 对应 Java: CsvWorkbook#setSheetName no-op
    pub const fn set_sheet_name(&mut self, _index: usize, _name: &str) {}

    // ─── 工作表可见性 (Sheet Visibility) ───

    /// 对应 Java: CsvWorkbook#isSheetHidden no-op
    pub const fn is_sheet_hidden(&self, _index: usize) -> bool { false }
    /// 对应 Java: CsvWorkbook#isSheetVeryHidden no-op
    pub const fn is_sheet_very_hidden(&self, _index: usize) -> bool { false }
    /// 对应 Java: CsvWorkbook#setHidden no-op
    pub const fn set_hidden(&mut self, _hidden: bool) {}
    /// 对应 Java: CsvWorkbook#setSheetHidden no-op
    pub const fn set_sheet_hidden(&mut self, _index: usize, _hidden: bool) {}
    /// 对应 Java: CsvWorkbook#getSheetVisibility no-op
    #[must_use] pub const fn get_sheet_visibility(&self, _sheet_index: usize) -> &'static str { "VISIBLE" }
    /// 对应 Java: CsvWorkbook#setSheetVisibility no-op
    pub const fn set_sheet_visibility(&mut self, _sheet_index: usize, _visibility: &str) {}

    // ─── 迭代器 (Iterator) ───

    /// CSV 工作簿迭代器；STUB 委托给 sheets()。
    /// 对应 Java: CsvWorkbook#iterator no-op
    pub fn iterator(&self) -> impl Iterator<Item = &crate::csv::CsvSheet<V>> { self.sheets() }
    /// CSV 工作簿迭代器；STUB 委托给 sheets()。
    /// 对应 Java: CsvWorkbook#sheetIterator no-op
    pub fn sheet_iterator(&self) -> impl Iterator<Item = &crate::csv::CsvSheet<V>> { self.sheets() }

    // ─── 缺失单元格策略 (Missing Cell Policy) ───

    /// 对应 Java: CsvWorkbook#getMissingCellPolicy no-op
    pub const fn get_missing_cell_policy(&self) -> u8 { 0 }
    /// 对应 Java: CsvWorkbook#setMissingCellPolicy no-op
    pub const fn set_missing_cell_policy(&mut self, _policy: u8) {}

    // ─── 名称管理 (Name Management) ───

    /// CSV 不保存名称。
    /// 对应 Java: CsvWorkbook#getAllNames no-op
    #[must_use] pub const fn get_all_names(&self) -> Vec<&str> { Vec::new() }
    /// CSV 不保存名称。
    /// 对应 Java: CsvWorkbook#getNames no-op
    #[must_use] pub const fn get_names(&self, _name: &str) -> Vec<&str> { Vec::new() }
    /// CSV 不支持删除名称。
    /// 对应 Java: CsvWorkbook#removeName no-op
    pub const fn remove_name(&mut self, _name: &str) {}

    // ─── 图片 (Picture) ───

    /// CSV 不保存图片。
    /// 对应 Java: CsvWorkbook#getAllPictures no-op
    #[must_use] pub const fn get_all_pictures(&self) -> Vec<&[u8]> { Vec::new() }

    // ─── 打印区域 (Print Area) ───

    /// CSV 不保存打印区域。
    /// 对应 Java: CsvWorkbook#getPrintArea no-op
    #[must_use] pub const fn get_print_area(&self, _sheet_index: usize) -> Option<&str> { None }
    /// CSV 不保存打印区域。
    /// 对应 Java: CsvWorkbook#setPrintArea no-op
    pub const fn set_print_area(&mut self, _sheet_index: usize, _reference: &str) {}
    /// CSV 不保存打印区域。
    /// 对应 Java: CsvWorkbook#removePrintArea no-op
    pub const fn remove_print_area(&mut self, _sheet_index: usize) {}

    // ─── 字体查询 (Font Query) ───

    /// CSV 不保存字体。
    /// 对应 Java: CsvWorkbook#getFontAt no-op
    #[must_use] pub const fn get_font_at(&self, _index: usize) -> Option<&str> { None }
    /// CSV 不保存字体。
    /// 对应 Java: CsvWorkbook#findFont no-op
    #[must_use] pub const fn find_font(&self) -> Option<&str> { None }

    // ─── 其他 (Miscellaneous) ───

    /// 对应 Java: CsvWorkbook#getSpreadsheetVersion no-op
    #[must_use] pub const fn get_spreadsheet_version(&self) -> &'static str { "EXCEL2007" }
    /// 对应 Java: CsvWorkbook#flushData no-op
    pub const fn flush_data(&mut self) {}
    /// Java CSV 为空操作。
    /// 对应 Java: CsvWorkbook#addToolPack no-op
    pub const fn add_tool_pack(&mut self) {}
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvWorkbook#createEvaluationWorkbook no-op
    #[must_use] pub const fn create_evaluation_workbook(&self) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvWorkbook#getCreationHelper no-op
    #[must_use] pub const fn get_creation_helper(&self) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvWorkbook#getCellReferenceType no-op
    #[must_use] pub const fn get_cell_reference_type(&self) -> Option<()> { None }
    /// Java CSV 为空操作。
    /// 对应 Java: CsvWorkbook#setCellReferenceType no-op
    pub const fn set_cell_reference_type(&mut self, _value: Option<()>) {}
}
