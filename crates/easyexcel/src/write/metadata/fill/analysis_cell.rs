//! 对应 Java：`com.alibaba.excel.write.metadata.fill.AnalysisCell`.

use crate::WriteTemplateAnalysisCellType;

/// 对应 Java：com.alibaba.excel.write.metadata.fill.AnalysisCell。 Template placeholder discovered while filling data.
///
/// Rust port of Java `AnalysisCell`.
#[derive(Debug, Clone)]
pub struct AnalysisCell {
    /// Zero-based column index. (Java `columnIndex`)
    pub column_index: i32,
    /// Zero-based row index. (Java `rowIndex`)
    pub row_index: i32,
    /// Placeholder variables such as `{name}`. (Java `variableList`)
    pub variable_list: Vec<String>,
    /// Prepared data tokens. (Java `prepareDataList`)
    pub prepare_data_list: Vec<String>,
    /// Whether the cell contains exactly one variable. (Java `onlyOneVariable`)
    pub only_one_variable: Option<bool>,
    /// Template cell kind. (Java `cellType`)
    pub cell_type: WriteTemplateAnalysisCellType,
    /// Prefix before the first variable. (Java `prefix`)
    pub prefix: Option<String>,
    /// Whether this is the first row of a collection block. (Java `firstRow`)
    pub first_row: Option<bool>,
}

impl AnalysisCell {
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.AnalysisCell。 Creates a common template cell. (Java `initAnalysisCell`)
    #[must_use]
    pub fn new(row_index: i32, column_index: i32) -> Self {
        Self {
            column_index,
            row_index,
            variable_list: Vec::new(),
            prepare_data_list: Vec::new(),
            only_one_variable: None,
            cell_type: WriteTemplateAnalysisCellType::Common,
            prefix: None,
            first_row: None,
        }
    }

    /// Returns the column index. (Java `getColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.AnalysisCell。
    pub const fn column_index(&self) -> i32 {
        self.column_index
    }

    /// Returns the row index. (Java `getRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.AnalysisCell。
    pub const fn row_index(&self) -> i32 {
        self.row_index
    }

    /// Returns the template cell kind. (Java `getCellType()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.AnalysisCell。
    pub const fn cell_type(&self) -> WriteTemplateAnalysisCellType {
        self.cell_type
    }
    /// Java `getColumnIndex` 别名。
    #[must_use] pub const fn get_column_index(&self) -> i32 { self.column_index }
    /// Java `setColumnIndex`。
    pub const fn set_column_index(&mut self, value: i32) { self.column_index = value; }
    /// Java `getRowIndex` 别名。
    #[must_use] pub const fn get_row_index(&self) -> i32 { self.row_index }
    /// Java `setRowIndex`。
    pub const fn set_row_index(&mut self, value: i32) { self.row_index = value; }
    /// Java `getVariableList`。
    #[must_use] pub fn get_variable_list(&self) -> &[String] { &self.variable_list }
    /// Java `setVariableList`。
    pub fn set_variable_list(&mut self, value: Vec<String>) { self.variable_list = value; }
    /// Java `getPrepareDataList`。
    #[must_use] pub fn get_prepare_data_list(&self) -> &[String] { &self.prepare_data_list }
    /// Java `setPrepareDataList`。
    pub fn set_prepare_data_list(&mut self, value: Vec<String>) { self.prepare_data_list = value; }
    /// Java `getOnlyOneVariable`。
    #[must_use] pub const fn get_only_one_variable(&self) -> Option<bool> { self.only_one_variable }
    /// Java `setOnlyOneVariable`。
    pub const fn set_only_one_variable(&mut self, value: Option<bool>) { self.only_one_variable = value; }
    /// Java `getCellType` 别名。
    #[must_use] pub const fn get_cell_type(&self) -> WriteTemplateAnalysisCellType { self.cell_type }
    /// Java `setCellType`。
    pub const fn set_cell_type(&mut self, value: WriteTemplateAnalysisCellType) { self.cell_type = value; }
    /// Java `getPrefix`。
    #[must_use] pub fn get_prefix(&self) -> Option<&str> { self.prefix.as_deref() }
    /// Java `setPrefix`。
    pub fn set_prefix(&mut self, value: Option<String>) { self.prefix = value; }
    /// Java `getFirstRow`。
    #[must_use] pub const fn get_first_row(&self) -> Option<bool> { self.first_row }
    /// Java `setFirstRow`。
    pub const fn set_first_row(&mut self, value: Option<bool>) { self.first_row = value; }
}

impl PartialEq for AnalysisCell {
    fn eq(&self, other: &Self) -> bool {
        self.column_index == other.column_index && self.row_index == other.row_index
    }
}
impl Eq for AnalysisCell {}

impl std::hash::Hash for AnalysisCell {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.column_index, state);
        std::hash::Hash::hash(&self.row_index, state);
    }
}
