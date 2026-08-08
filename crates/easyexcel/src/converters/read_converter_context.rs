//! 对应 Java：`com.alibaba.excel.converters.ReadConverterContext`.

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_column::ExcelColumn;
use crate::core::formula_data::FormulaData;
use bigdecimal::BigDecimal;

/// Context supplied to a custom cell-to-Rust converter.
///
/// 对应 Java：`ReadConverterContext<T>(readCellData, contentProperty,
/// analysisContext)`. Rust drops the `ReadCellData` wrapper and stores
/// `&CellValue` directly to avoid cloning the entire cell envelope.
#[derive(Debug, Clone, Copy)]
pub struct ReadConverterContext<'a> {
    cell: Option<&'a CellValue>,
    formula: Option<&'a FormulaData>,
    display_value: Option<&'a str>,
    decimal_value: Option<&'a BigDecimal>,
    column: &'a ExcelColumn,
    context: &'a ConvertContext,
}

impl<'a> ReadConverterContext<'a> {
    /// 替换读取单元格数据。对应 Java Lombok setter。
    pub const fn set_read_cell_data(&mut self, value: Option<&'a CellValue>) { self.cell = value; }
    /// 替换字段内容属性。对应 Java Lombok setter。
    pub const fn set_content_property(&mut self, value: &'a ExcelColumn) { self.column = value; }
    /// 替换分析上下文。对应 Java Lombok setter。
    pub const fn set_analysis_context(&mut self, value: &'a ConvertContext) { self.context = value; }
    /// Creates a read conversion context. (Java `@AllArgsConstructor`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn new(
        cell: Option<&'a CellValue>,
        column: &'a ExcelColumn,
        context: &'a ConvertContext,
    ) -> Self {
        Self {
            cell,
            formula: None,
            display_value: None,
            decimal_value: None,
            column,
            context,
        }
    }

    /// Creates a read conversion context with optional formula metadata.
    /// 对应 Java：'s ability to expose `formulaData` from `ReadCellData`.
    #[must_use]
    pub const fn with_formula(
        cell: Option<&'a CellValue>,
        formula: Option<&'a FormulaData>,
        column: &'a ExcelColumn,
        context: &'a ConvertContext,
    ) -> Self {
        Self {
            cell,
            formula,
            display_value: None,
            decimal_value: None,
            column,
            context,
        }
    }

    /// Creates a context with the full scalar metadata retained by Java `ReadCellData`.
    ///
    /// `display_value` mirrors `ReadCellData.stringValue` after POI
    /// `DataFormatter`; `decimal_value` mirrors the exact
    /// `ReadCellData.numberValue` parsed from OOXML rather than its `f64`
    /// transport representation.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn with_cell_metadata(
        cell: Option<&'a CellValue>,
        formula: Option<&'a FormulaData>,
        display_value: Option<&'a str>,
        decimal_value: Option<&'a BigDecimal>,
        column: &'a ExcelColumn,
        context: &'a ConvertContext,
    ) -> Self {
        Self {
            cell,
            formula,
            display_value,
            decimal_value,
            column,
            context,
        }
    }

    /// Returns the source cell, or `None` when it is physically absent. (Java `getReadCellData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn cell(&self) -> Option<&'a CellValue> {
        self.cell
    }

    /// 返回读取单元格数据。对应 Java：`getReadCellData()`。
    #[must_use]
    pub const fn get_read_cell_data(&self) -> Option<&'a CellValue> {
        self.cell()
    }

    /// Returns formula metadata when the source cell contains a formula. (Java `ReadCellData.getFormulaData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn formula(&self) -> Option<&'a FormulaData> {
        self.formula
    }

    /// Returns the Excel/POI-compatible rendered text when the reader retained it.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn display_value(&self) -> Option<&'a str> {
        self.display_value
    }

    /// Returns the exact decimal token retained from the source workbook.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn decimal_value(&self) -> Option<&'a BigDecimal> {
        self.decimal_value
    }

    /// Returns the field's static column metadata. (Java `getContentProperty()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn column(&self) -> &'a ExcelColumn {
        self.column
    }

    /// 返回字段内容属性。对应 Java：`getContentProperty()`。
    #[must_use]
    pub const fn get_content_property(&self) -> &'a ExcelColumn {
        self.column()
    }

    /// Returns the resolved row, column, field, and format information. (Java `getAnalysisContext()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.ReadConverterContext。
    pub const fn convert_context(&self) -> &'a ConvertContext {
        self.context
    }

    /// 返回分析上下文的轻量等价物。对应 Java：`getAnalysisContext()`。
    #[must_use]
    pub const fn get_analysis_context(&self) -> &'a ConvertContext {
        self.convert_context()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    fn sample_column() -> ExcelColumn {
        ExcelColumn::new("value", "Value", Some(0), 0, None)
    }

    fn sample_context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn with_formula_carries_formula_metadata() {
        // 对应 Java：ReadConverterContext.withFormula 携带公式信息
        let column = sample_column();
        let context = sample_context();
        let cell = CellValue::Int(3);
        let formula = FormulaData::new("=1+2");
        let read_context =
            ReadConverterContext::with_formula(Some(&cell), Some(&formula), &column, &context);
        assert_eq!(read_context.cell(), Some(&cell));
        assert_eq!(
            read_context.formula().map(FormulaData::formula_value),
            Some("=1+2")
        );
        assert_eq!(read_context.display_value(), None);
        assert_eq!(read_context.decimal_value(), None);
        assert_eq!(read_context.column().field, "value");
        assert_eq!(read_context.convert_context().sheet_name, "Sheet1");

        // 无公式与无单元格
        let bare = ReadConverterContext::with_formula(None, None, &column, &context);
        assert_eq!(bare.cell(), None);
        assert_eq!(bare.formula(), None);
    }
}
