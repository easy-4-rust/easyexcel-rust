//! 对应 Java：`com.alibaba.excel.metadata.data.FormulaData`.

/// Formula metadata associated with a cached cell value while reading.
///
/// 对应 Java：`FormulaData` (`formulaValue` field + `clone()` override).
/// Rust uses `#[derive(Clone)]` so the public `clone()` is automatic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FormulaData {
    formula_value: String,
}

impl FormulaData {
    /// 对应 Java：com.alibaba.excel.metadata.data.FormulaData。 Creates formula metadata from the expression stored in the workbook.
    #[must_use]
    pub fn new(formula_value: impl Into<String>) -> Self {
        Self {
            formula_value: formula_value.into(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.FormulaData。 Returns the formula expression without adding a leading equals sign. (Java `getFormulaValue()`)
    #[must_use]
    pub fn formula_value(&self) -> &str {
        &self.formula_value
    }

    /// Java `getFormulaValue` 别名。
    #[must_use]
    pub fn get_formula_value(&self) -> &str { &self.formula_value }
    /// Java `setFormulaValue`。
    pub fn set_formula_value(&mut self, value: impl Into<String>) {
        self.formula_value = value.into();
    }
    /// Java `clone()` 的显式别名。
    #[must_use]
    pub fn clone_data(&self) -> Self { self.clone() }
}
