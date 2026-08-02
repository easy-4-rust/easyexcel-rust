//! Excel 内部单元格数据模型。
//!
//! 对应 Java：`com.alibaba.excel.metadata.data.CellData`
//! 原文件：`easyexcel-core/.../metadata/data/CellData.java`
//!
//! 读路径既有 [`crate::ReadCellData`]，写路径既有 [`crate::WriteCellData`]；
//! 本结构保留 Java 泛型 `CellData<T>` 的字段面，便于 1:1 迁移与测试对照。

use bigdecimal::BigDecimal;

use crate::CellDataType;
use crate::FormulaData;

/// Excel 内部单元格数据，对齐 Java `CellData<T>`。
///
/// # Java 对应字段
/// - `type` → [`Self::cell_type`]
/// - `numberValue` → [`Self::number_value`]
/// - `stringValue` → [`Self::string_value`]
/// - `booleanValue` → [`Self::boolean_value`]
/// - `data` → [`Self::data`]
/// - `formulaData` → [`Self::formula_data`]
#[derive(Debug, Clone, PartialEq)]
pub struct CellData<T = ()> {
    /// 单元格类型。Java `type` / `getType()` / `setType`
    pub cell_type: Option<CellDataType>,
    /// 数值。Java `numberValue`
    pub number_value: Option<BigDecimal>,
    /// 字符串或错误文本。Java `stringValue`
    pub string_value: Option<String>,
    /// 布尔值。Java `booleanValue`
    pub boolean_value: Option<bool>,
    /// 转换后的业务数据。Java `data`
    pub data: Option<T>,
    /// 公式。Java `formulaData`
    pub formula_data: Option<FormulaData>,
    /// 行号。来自 Java `AbstractCell.rowIndex`
    pub row_index: Option<usize>,
    /// 列号。来自 Java `AbstractCell.columnIndex`
    pub column_index: Option<usize>,
}

impl<T> Default for CellData<T> {
    fn default() -> Self {
        Self {
            cell_type: None,
            number_value: None,
            string_value: None,
            boolean_value: None,
            data: None,
            formula_data: None,
            row_index: None,
            column_index: None,
        }
    }
}

impl<T> CellData<T> {
    /// 创建空单元格。对应 Java 默认构造。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 确保类型非空并按值收缩 EMPTY。对应 Java `checkEmpty()`。
    pub fn check_empty(&mut self) {
        if self.cell_type.is_none() {
            self.cell_type = Some(CellDataType::Empty);
        }
        match self.cell_type {
            Some(CellDataType::String | CellDataType::DirectString | CellDataType::Error) => {
                if self.string_value.as_ref().is_none_or(String::is_empty) {
                    self.cell_type = Some(CellDataType::Empty);
                }
            }
            Some(CellDataType::Number) => {
                if self.number_value.is_none() {
                    self.cell_type = Some(CellDataType::Empty);
                }
            }
            Some(CellDataType::Boolean) => {
                if self.boolean_value.is_none() {
                    self.cell_type = Some(CellDataType::Empty);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::CellDataType;
    use crate::FormulaData;

    #[test]
    fn default_and_new_produce_empty_cell() {
        // 对应 Java：CellData 默认构造所有字段为空
        let default = CellData::<i32>::default();
        assert_eq!(default.cell_type, None);
        assert_eq!(default.number_value, None);
        assert_eq!(default.string_value, None);
        assert_eq!(default.boolean_value, None);
        assert_eq!(default.data, None);
        assert_eq!(default.formula_data, None);
        assert_eq!(default.row_index, None);
        assert_eq!(default.column_index, None);
        assert_eq!(CellData::<i32>::new(), default);
    }

    #[test]
    fn check_empty_fills_missing_type_with_empty() {
        // 对应 Java：checkEmpty() 无类型时补 EMPTY
        let mut cell = CellData::<()>::default();
        cell.check_empty();
        assert_eq!(cell.cell_type, Some(CellDataType::Empty));
    }

    #[test]
    fn check_empty_collapses_string_without_content() {
        // 对应 Java：字符串为空或缺失时收缩为 EMPTY
        let mut empty_string = CellData::<()> {
            cell_type: Some(CellDataType::String),
            string_value: Some(String::new()),
            ..CellData::default()
        };
        empty_string.check_empty();
        assert_eq!(empty_string.cell_type, Some(CellDataType::Empty));

        let mut none_string = CellData::<()> {
            cell_type: Some(CellDataType::DirectString),
            string_value: None,
            ..CellData::default()
        };
        none_string.check_empty();
        assert_eq!(none_string.cell_type, Some(CellDataType::Empty));

        let mut error = CellData::<()> {
            cell_type: Some(CellDataType::Error),
            string_value: Some(String::new()),
            ..CellData::default()
        };
        error.check_empty();
        assert_eq!(error.cell_type, Some(CellDataType::Empty));

        // 有内容的字符串保留类型
        let mut kept = CellData::<()> {
            cell_type: Some(CellDataType::String),
            string_value: Some("x".to_string()),
            ..CellData::default()
        };
        kept.check_empty();
        assert_eq!(kept.cell_type, Some(CellDataType::String));
    }

    #[test]
    fn check_empty_collapses_number_and_boolean_without_value() {
        // 对应 Java：数值/布尔缺失时收缩为 EMPTY
        let mut number = CellData::<()> {
            cell_type: Some(CellDataType::Number),
            number_value: None,
            ..CellData::default()
        };
        number.check_empty();
        assert_eq!(number.cell_type, Some(CellDataType::Empty));

        let mut boolean = CellData::<()> {
            cell_type: Some(CellDataType::Boolean),
            boolean_value: None,
            ..CellData::default()
        };
        boolean.check_empty();
        assert_eq!(boolean.cell_type, Some(CellDataType::Empty));
    }

    #[test]
    fn check_empty_keeps_other_types_untouched() {
        // 对应 Java：其他类型（EMPTY/DATE/公式等）不收缩
        let mut date = CellData::<()> {
            cell_type: Some(CellDataType::Date),
            ..CellData::default()
        };
        date.check_empty();
        assert_eq!(date.cell_type, Some(CellDataType::Date));

        let mut formula = CellData::<()> {
            cell_type: Some(CellDataType::Formula),
            formula_data: Some(FormulaData::new("=SUM(A1)")),
            ..CellData::default()
        };
        formula.check_empty();
        assert_eq!(formula.cell_type, Some(CellDataType::Formula));
    }
}
