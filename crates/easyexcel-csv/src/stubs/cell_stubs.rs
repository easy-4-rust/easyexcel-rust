//! CsvCell 的 STUB 方法集中文件。
//!
//! 包含 CSV 格式不支持的 Excel 单元格功能的 no-op 实现。
//! 对应 Java：com.alibaba.excel.metadata.csv.CsvCell 中的 no-op 方法。

use crate::csv::{CsvCell, CsvCellValue};

/// CsvCell 的 STUB 方法实现。
///
/// 这些方法对应 Java CsvCell 中因 CSV 格式限制而无法实现的功能，
/// 保留 no-op 语义以维持 Java API 调用兼容性。
impl<V: CsvCellValue> CsvCell<V> {
    // ─── 批注 (Comment) ───

    /// CSV 不承载批注；与 Java CSV 适配器的 no-op 语义一致。
    /// 对应 Java: CsvCell#removeCellComment no-op
    pub const fn remove_cell_comment(&mut self) {}
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvCell#getCellComment no-op
    #[must_use] pub const fn get_cell_comment(&self) -> Option<()> { None }
    /// Java CSV 为空操作。
    /// 对应 Java: CsvCell#setCellComment no-op
    pub const fn set_cell_comment(&mut self, _comment: Option<()>) {}

    // ─── 超链接 (Hyperlink) ───

    /// CSV 不承载超链接；与 Java CSV 适配器的 no-op 语义一致。
    /// 对应 Java: CsvCell#removeHyperlink no-op
    pub const fn remove_hyperlink(&mut self) {}
    /// Java CSV 返回 `null`。
    /// 对应 Java: CsvCell#getHyperlink no-op
    #[must_use] pub const fn get_hyperlink(&self) -> Option<()> { None }
    /// Java CSV 为空操作。
    /// 对应 Java: CsvCell#setHyperlink no-op
    pub const fn set_hyperlink(&mut self, _hyperlink: Option<()>) {}

    // ─── 数组公式 (Array Formula) ───

    /// CSV 不支持数组公式。
    /// 对应 Java: CsvCell#isPartOfArrayFormulaGroup no-op
    #[must_use]
    pub const fn is_part_of_array_formula_group(&self) -> bool { false }

    /// Java CSV 返回 `null`，因为 CSV 不支持数组公式。
    /// 对应 Java: CsvCell#getArrayFormulaRange no-op
    #[must_use] pub const fn get_array_formula_range(&self) -> Option<()> { None }

    // ─── 活动单元格 (Active Cell) ───

    /// CSV 不维护活动单元格状态。
    /// 对应 Java: CsvCell#setAsActiveCell no-op
    pub const fn set_as_active_cell(&mut self) {}

    // ─── 数组公式 Java 兼容别名 ───

    /// CSV 不支持数组公式；Java 兼容别名。
    /// 对应 Java: CsvCell#isPartOfArrayFormulaGroup no-op
    pub const fn is_part_of_array_formula_group_java(&self) -> bool {
        self.is_part_of_array_formula_group()
    }
}
