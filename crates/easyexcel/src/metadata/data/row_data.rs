//! Mirrors the union of `ReadRowHolder.cellMap` / `CellExtra` /
//! `currentRowAnalysisResult` aggregated into one row payload.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bigdecimal::BigDecimal;

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::dynamic_value::DynamicValue;
use crate::core::enum_read_default_return::ReadDefaultReturn;
use crate::core::excel_column::ExcelColumn;
use crate::core::formula_data::FormulaData;

/// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 A physical row plus resolved header positions.
///
/// Java distributes these across `ReadRowHolder` (current row), `CellData`
/// (per-cell scalars), and `AnalysisContext` (current sheet / row index).
/// Rust fuses them into a single value that travels from `XlsxSaxAnalyser`
/// through `T::from_row_with_converters` and into listener callbacks.
#[derive(Debug, Clone)]
pub struct RowData {
    sheet_name: String,
    row_index: u32,
    cells: Vec<CellValue>,
    headers: Arc<HashMap<String, usize>>,
    formulas: Option<HashMap<usize, FormulaData>>,
    display_values: Option<HashMap<usize, String>>,
    decimal_values: Option<HashMap<usize, BigDecimal>>,
    present_columns: Option<HashSet<usize>>,
    read_default_return: ReadDefaultReturn,
    use_1904_windowing: bool,
}

impl RowData {
    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Creates row data. (Java `ReadRowHolder(rowIndex, rowType, globalConfiguration, cellMap)` subset)
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        row_index: u32,
        cells: Vec<CellValue>,
        headers: Arc<HashMap<String, usize>>,
    ) -> Self {
        let present_columns = Some((0..cells.len()).collect());
        Self {
            sheet_name: sheet_name.into(),
            row_index,
            cells,
            headers,
            formulas: None,
            display_values: None,
            decimal_values: None,
            present_columns,
            read_default_return: ReadDefaultReturn::default(),
            use_1904_windowing: false,
        }
    }

    /// 使用事件读取器已经收集的全部组成部分直接构造一行。
    ///
    /// 对应 Java：`ReadRowHolder` 到 `ModelBuildEventListener` 的内部交接。
    /// 该入口避免 [`Self::new`] 先构造一份连续列集合、随后又被真实
    /// `present_columns` 覆盖的重复分配。
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_stream_parts(
        sheet_name: impl Into<String>,
        row_index: u32,
        cells: Vec<CellValue>,
        headers: Arc<HashMap<String, usize>>,
        formulas: Option<HashMap<usize, FormulaData>>,
        display_values: Option<HashMap<usize, String>>,
        decimal_values: Option<HashMap<usize, BigDecimal>>,
        present_columns: Option<HashSet<usize>>,
        read_default_return: ReadDefaultReturn,
        use_1904_windowing: bool,
    ) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            row_index,
            cells,
            headers,
            formulas,
            display_values,
            decimal_values,
            present_columns,
            read_default_return,
            use_1904_windowing,
        }
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Attaches formula metadata indexed by zero-based physical column. (Java `CellData.formulaData`)
    #[must_use]
    pub fn with_formulas(mut self, formulas: HashMap<usize, FormulaData>) -> Self {
        self.formulas = Some(formulas);
        self
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Attaches Java-compatible formatted display text by physical column index. (Java `CellData.stringValue`)
    #[must_use]
    pub fn with_display_values(mut self, display_values: HashMap<usize, String>) -> Self {
        self.display_values = Some(display_values);
        self
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Attaches exact OOXML decimal values by physical column index. (Java `CellData.numberValue`)
    #[must_use]
    pub fn with_decimal_values(mut self, decimal_values: HashMap<usize, BigDecimal>) -> Self {
        self.decimal_values = Some(decimal_values);
        self
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Attaches the physical columns that were explicitly present in the source.
    #[must_use]
    pub fn with_present_columns(mut self, present_columns: HashSet<usize>) -> Self {
        self.present_columns = Some(present_columns);
        self
    }

    /// Selects the Java-compatible no-model return mode. (Java `ReadDefaultReturnEnum`)
    #[must_use]
    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。
    pub const fn with_read_default_return(mut self, mode: ReadDefaultReturn) -> Self {
        self.read_default_return = mode;
        self
    }

    /// Selects Excel's 1904 numeric date system for field conversion.
    #[must_use]
    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。
    pub const fn with_use_1904_windowing(mut self, enabled: bool) -> Self {
        self.use_1904_windowing = enabled;
        self
    }

    /// Returns the physical zero-based row index. (Java `ReadRowHolder.getRowIndex()`)
    #[must_use]
    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Returns the sheet name. (Java `ReadRowHolder.sheetName`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    /// Resolves a cell using Java `EasyExcel`'s index-before-name priority.
    ///
    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)` semantics
    /// in the `ModelBuildEventListener` (`buildUserModel`).
    #[must_use]
    pub fn cell(&self, column: &ExcelColumn) -> Option<&CellValue> {
        let index = column
            .index
            .or_else(|| self.headers.get(column.leaf_name()).copied())?;
        self.cells.get(index)
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Resolves formula metadata using the same index-before-name priority as [`Self::cell`].
    #[must_use]
    pub fn formula(&self, column: &ExcelColumn) -> Option<&FormulaData> {
        let index = column
            .index
            .or_else(|| self.headers.get(column.leaf_name()).copied())?;
        self.formulas.as_ref().and_then(|m| m.get(&index))
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Returns POI-compatible display text retained for a numeric source cell.
    #[must_use]
    pub fn display_value(&self, column: &ExcelColumn) -> Option<&str> {
        let index = column
            .index
            .or_else(|| self.headers.get(column.leaf_name()).copied())?;
        self.display_values
            .as_ref()
            .and_then(|m| m.get(&index).map(String::as_str))
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Returns the exact decimal token retained from OOXML for a numeric cell.
    #[must_use]
    pub fn decimal_value(&self, column: &ExcelColumn) -> Option<&BigDecimal> {
        let index = column
            .index
            .or_else(|| self.headers.get(column.leaf_name()).copied())?;
        self.decimal_values.as_ref().and_then(|m| m.get(&index))
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Dynamic-row support: maximum physical column touched by either headers or cells.
    pub(crate) fn dynamic_width(&self) -> usize {
        let head_width = self
            .headers
            .values()
            .copied()
            .max()
            .map_or(0, |index| index.saturating_add(1));
        self.cells.len().max(head_width)
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Dynamic-row support: produce a `DynamicValue` for a column.
    pub(crate) fn dynamic_cell(&self, column_index: usize) -> DynamicValue {
        if !self
            .present_columns
            .as_ref()
            .is_some_and(|s| s.contains(&column_index))
        {
            return DynamicValue::Null;
        }
        let raw_value = self
            .cells
            .get(column_index)
            .cloned()
            .unwrap_or(CellValue::Empty);
        let raw_value = if matches!(raw_value, CellValue::Int(_) | CellValue::Float(_)) {
            self.decimal_values
                .as_ref()
                .and_then(|m| m.get(&column_index).cloned())
                .map_or(raw_value, CellValue::Decimal)
        } else {
            raw_value
        };
        let data = actual_cell_value(&raw_value);
        let display_value = self
            .display_values
            .as_ref()
            .and_then(|m| m.get(&column_index).cloned())
            .unwrap_or_else(|| raw_value.as_text());
        match self.read_default_return {
            ReadDefaultReturn::String => DynamicValue::String(display_value),
            ReadDefaultReturn::ActualData => DynamicValue::ActualData(data),
            ReadDefaultReturn::ReadCellData => {
                let formula = self
                    .formulas
                    .as_ref()
                    .and_then(|m| m.get(&column_index).cloned());
                DynamicValue::ReadCellData(crate::core::read_cell_data::ReadCellData::new(
                    self.row_index,
                    column_index,
                    raw_value,
                    data,
                    display_value,
                    formula,
                ))
            }
        }
    }

    /// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。 Creates a conversion context for a column. (Java `ReadConverterContext` constructor)
    #[must_use]
    pub fn convert_context(&self, column: &ExcelColumn) -> ConvertContext {
        let mut context = self.convert_context_base();
        self.configure_convert_context(&mut context, column);
        context
    }

    /// 创建可供 derive 在同一行内复用的转换上下文基线。
    ///
    /// 对应 Java：`AnalysisContext` 在一行的多个字段转换期间复用 Sheet 与行号状态。
    #[doc(hidden)]
    #[must_use]
    pub fn convert_context_base(&self) -> ConvertContext {
        ConvertContext {
            sheet_name: self.sheet_name.clone(),
            row_index: self.row_index,
            column_index: None,
            field: "",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: self.use_1904_windowing,
        }
    }

    /// 将复用的行上下文切换到指定字段，避免为每个字段重新分配 Sheet 名称。
    ///
    /// 对应 Java：`AnalysisContext.currentReadHolder` 与 `ExcelContentProperty` 的逐字段切换。
    #[doc(hidden)]
    pub fn configure_convert_context(&self, context: &mut ConvertContext, column: &ExcelColumn) {
        context.column_index = column
            .index
            .or_else(|| self.headers.get(column.leaf_name()).copied());
        context.field = column.field;
        context.format = column.format;
        context.date_time_format = column.date_time_format;
        context.number_format = column.number_format;
        context.use_1904_windowing = column.use_1904_windowing.unwrap_or(self.use_1904_windowing);
    }
}
/// 对应 Java：`AnalysisContext.readRowHolder().getCell(column)`。
pub(crate) fn actual_cell_value(value: &CellValue) -> CellValue {
    match value {
        CellValue::Empty => CellValue::String(String::new()),
        CellValue::Error(value) => CellValue::String(value.clone()),
        value => value.clone(),
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn row_index_and_sheet_name_accessors() {
        // 对应 Java：ReadRowHolder.getRowIndex / sheetName
        let row = RowData::new(
            "Sheet1",
            7,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        );
        assert_eq!(row.row_index(), 7);
        assert_eq!(row.sheet_name(), "Sheet1");
    }
}
