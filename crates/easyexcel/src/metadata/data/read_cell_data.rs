//! 对应 Java：`com.alibaba.excel.metadata.data.ReadCellData`.

use crate::core::cell_value::CellValue;
use crate::core::formula_data::FormulaData;
use crate::{CellDataType, DataFormatData};
use bigdecimal::BigDecimal;

/// Java-compatible no-model cell metadata.
///
/// 对应 Java：`ReadCellData<T>`: `rowIndex`, `columnIndex`, `numberValue`,
/// `originalNumberValue`, `stringValue`, `booleanValue`, `data`, `type`,
/// `dataFormatData`, `formulaData`. The Rust port preserves the read-side
/// metadata that downstream consumers need.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadCellData {
    row_index: u32,
    column_index: usize,
    declared_type: Option<CellDataType>,
    raw_value: CellValue,
    data: CellValue,
    display_value: String,
    formula: Option<FormulaData>,
    original_number_value: Option<BigDecimal>,
    data_format_data: Option<DataFormatData>,
}

impl ReadCellData {
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。 Internal constructor mirroring Java's `ReadCellData(type, stringValue)`.
    /// Not part of the public API.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        row_index: u32,
        column_index: usize,
        raw_value: CellValue,
        data: CellValue,
        display_value: String,
        formula: Option<FormulaData>,
    ) -> Self {
        Self {
            row_index,
            column_index,
            declared_type: Some(data.data_type()),
            raw_value,
            data,
            display_value,
            formula,
            original_number_value: None,
            data_format_data: None,
        }
    }

    /// 创建空的 Java `ReadCellData`。
    #[must_use]
    pub fn new_empty_instance(row_index: Option<u32>, column_index: Option<usize>) -> Self {
        Self::new(
            row_index.unwrap_or(0),
            column_index.unwrap_or(0),
            CellValue::Empty,
            CellValue::Empty,
            String::new(),
            None,
        )
    }

    /// 从任意后端中立值创建读取单元格。
    #[must_use]
    pub fn new_instance(
        value: impl Into<CellValue>,
        row_index: Option<u32>,
        column_index: Option<usize>,
    ) -> Self {
        let value = value.into();
        let display = value.as_text();
        Self::new(
            row_index.unwrap_or(0),
            column_index.unwrap_or(0),
            value.clone(),
            value,
            display,
            None,
        )
    }

    /// 从原始数字构造并同时保留未格式化 BigDecimal。
    #[must_use]
    pub fn new_instance_original(
        value: BigDecimal,
        row_index: Option<u32>,
        column_index: Option<usize>,
    ) -> Self {
        let mut cell = Self::new_instance(
            CellValue::Decimal(easyexcel_format::EXCEL_MATH_CONTEXT.round_decimal_ref(&value)),
            row_index,
            column_index,
        );
        cell.original_number_value = Some(value);
        cell
    }

    /// 返回 Java `CellDataTypeEnum`。
    #[must_use]
    pub fn cell_type(&self) -> CellDataType {
        self.declared_type.unwrap_or_else(|| self.data.data_type())
    }

    /// 设置 Java `CellData.type`，不伪造对应值。
    pub const fn set_type(&mut self, value: Option<CellDataType>) { self.declared_type = value; }

    /// Java `getType` 兼容别名。
    #[must_use]
    pub const fn get_type(&self) -> Option<CellDataType> { self.declared_type }

    /// 返回原始数字值。
    #[must_use]
    pub const fn original_number_value(&self) -> Option<&BigDecimal> {
        self.original_number_value.as_ref()
    }

    /// 设置原始数字值。
    pub fn set_original_number_value(&mut self, value: Option<BigDecimal>) {
        self.original_number_value = value;
    }

    /// 返回数据格式元数据。
    #[must_use]
    pub const fn data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data.as_ref()
    }

    /// 设置数据格式元数据。
    pub fn set_data_format_data(&mut self, value: Option<DataFormatData>) {
        self.data_format_data = value;
    }

    /// 返回字符串值。
    #[must_use]
    pub fn string_value(&self) -> &str { &self.display_value }

    /// Java `getStringValue` 兼容别名。
    #[must_use] pub fn get_string_value(&self) -> &str { self.string_value() }
    /// 设置字符串负载并同步类型。
    pub fn set_string_value(&mut self, value: impl Into<String>) {
        let value = value.into();
        self.display_value.clone_from(&value);
        self.raw_value = CellValue::String(value.clone());
        self.data = CellValue::String(value);
        if !matches!(self.declared_type, Some(CellDataType::Error | CellDataType::DirectString)) {
            self.declared_type = Some(CellDataType::String);
        }
    }

    /// 返回数字值。
    #[must_use]
    pub fn number_value(&self) -> Option<BigDecimal> {
        match &self.data {
            CellValue::Decimal(value) => Some(value.clone()),
            CellValue::Int(value) => Some(BigDecimal::from(*value)),
            CellValue::Float(value) => value.to_string().parse().ok(),
            _ => None,
        }
    }
    /// Java `getNumberValue` 兼容别名。
    #[must_use] pub fn get_number_value(&self) -> Option<BigDecimal> { self.number_value() }
    /// 设置数字负载并同步类型。
    pub fn set_number_value(&mut self, value: Option<BigDecimal>) {
        match value {
            Some(value) => {
                self.raw_value = CellValue::Decimal(value.clone());
                self.data = CellValue::Decimal(value);
                self.declared_type = Some(CellDataType::Number);
            }
            None => {
                self.raw_value = CellValue::Empty;
                self.data = CellValue::Empty;
            }
        }
    }

    /// 返回布尔值。
    #[must_use]
    pub const fn boolean_value(&self) -> Option<bool> {
        if let CellValue::Bool(value) = &self.data { Some(*value) } else { None }
    }
    /// Java `getBooleanValue` 兼容别名。
    #[must_use] pub const fn get_boolean_value(&self) -> Option<bool> { self.boolean_value() }
    /// 设置布尔负载并同步类型。
    pub fn set_boolean_value(&mut self, value: Option<bool>) {
        match value {
            Some(value) => {
                self.raw_value = CellValue::Bool(value);
                self.data = CellValue::Bool(value);
                self.declared_type = Some(CellDataType::Boolean);
            }
            None => {
                self.raw_value = CellValue::Empty;
                self.data = CellValue::Empty;
            }
        }
    }

    /// 返回公式元数据。
    #[must_use]
    pub const fn formula_data(&self) -> Option<&FormulaData> { self.formula.as_ref() }

    /// 设置公式元数据。
    pub fn set_formula_data(&mut self, value: Option<FormulaData>) { self.formula = value; }

    /// Returns the physical zero-based row index. (Java `getRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }
    /// Java `getRowIndex` 兼容别名。
    #[must_use] pub const fn get_row_index(&self) -> u32 { self.row_index }
    /// Java `setRowIndex`。
    pub const fn set_row_index(&mut self, value: u32) { self.row_index = value; }

    /// Returns the physical zero-based column index. (Java `getColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。
    pub const fn column_index(&self) -> usize {
        self.column_index
    }
    /// Java `getColumnIndex` 兼容别名。
    #[must_use] pub const fn get_column_index(&self) -> usize { self.column_index }
    /// Java `setColumnIndex`。
    pub const fn set_column_index(&mut self, value: usize) { self.column_index = value; }

    /// Returns the original backend-neutral cell value. (Java `CellData.getData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。
    pub const fn raw_value(&self) -> &CellValue {
        &self.raw_value
    }

    /// Returns the Java `ACTUAL_DATA`-equivalent value. (Java `getData()` for non-string)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。
    pub const fn data(&self) -> &CellValue {
        &self.data
    }
    /// Java `getData` 兼容别名。
    #[must_use] pub const fn get_data(&self) -> &CellValue { &self.data }
    /// Java `setData`，数据可独立于单元格显示负载存在。
    pub fn set_data(&mut self, value: CellValue) { self.data = value; }

    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。 Returns the Java-compatible formatted display text. (Java `getStringValue()`)
    #[must_use]
    pub fn display_value(&self) -> &str {
        &self.display_value
    }

    /// Returns formula metadata when the cell contains a formula. (Java `getFormulaData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ReadCellData。
    pub const fn formula(&self) -> Option<&FormulaData> {
        self.formula.as_ref()
    }

    /// Java 无参构造器。
    #[must_use]
    pub fn empty() -> Self { Self::new_empty_instance(None, None) }
    /// Java `ReadCellData(CellDataTypeEnum)` 的显式 Rust 构造器。
    #[must_use]
    pub fn from_type(cell_type: CellDataType) -> Self {
        let mut value = Self::empty();
        value.declared_type = Some(cell_type);
        value
    }
    /// Java `ReadCellData(CellDataTypeEnum, String)`；只允许 STRING/ERROR。
    pub fn from_type_and_string(cell_type: CellDataType, value: impl Into<String>) -> crate::Result<Self> {
        if !matches!(cell_type, CellDataType::String | CellDataType::Error) {
            return Err(crate::ExcelError::Format(
                "Only support CellDataTypeEnum.STRING and CellDataTypeEnum.ERROR".to_owned(),
            ));
        }
        let mut cell = Self::from_type(cell_type);
        cell.set_string_value(value);
        cell.declared_type = Some(cell_type);
        Ok(cell)
    }
    /// Java `ReadCellData(Boolean)`。
    #[must_use] pub fn from_boolean(value: bool) -> Self { Self::new_instance(value, None, None) }
    /// Java `ReadCellData(String)`。
    #[must_use] pub fn from_string(value: impl Into<String>) -> Self { Self::new_instance(value.into(), None, None) }
    /// Java `ReadCellData(BigDecimal)`。
    #[must_use] pub fn from_number(value: BigDecimal) -> Self { Self::new_instance(value, None, None) }
    /// Java `getOriginalNumberValue` 别名。
    #[must_use]
    pub const fn get_original_number_value(&self) -> Option<&BigDecimal> {
        self.original_number_value.as_ref()
    }
    /// Java `getDataFormatData` 别名。
    #[must_use]
    pub const fn get_data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data.as_ref()
    }
    /// Java `clone()` 的显式别名。
    #[must_use]
    pub fn clone_data(&self) -> Self { self.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn empty_creates_default_cell() {
        // 对应 Java：ReadCellData 无参构造器
        let cell = ReadCellData::empty();
        assert_eq!(cell.row_index(), 0);
        assert_eq!(cell.column_index(), 0);
        assert_eq!(cell.cell_type(), CellDataType::Empty);
        assert!(cell.string_value().is_empty());
    }

    #[test]
    fn new_empty_instance_with_indices() {
        // 对应 Java：ReadCellData(rowIndex, columnIndex)
        let cell = ReadCellData::new_empty_instance(Some(5), Some(3));
        assert_eq!(cell.row_index(), 5);
        assert_eq!(cell.column_index(), 3);
    }

    #[test]
    fn new_empty_instance_with_none_indices() {
        // 对应 Java：ReadCellData(null, null) 使用默认 0
        let cell = ReadCellData::new_empty_instance(None, None);
        assert_eq!(cell.row_index(), 0);
        assert_eq!(cell.column_index(), 0);
    }

    #[test]
    fn new_instance_from_string() {
        // 对应 Java：ReadCellData(String)
        let cell = ReadCellData::new_instance("hello".to_owned(), Some(1), Some(2));
        assert_eq!(cell.string_value(), "hello");
        assert_eq!(cell.row_index(), 1);
        assert_eq!(cell.column_index(), 2);
    }

    #[test]
    fn new_instance_from_bool() {
        // 对应 Java：ReadCellData(Boolean)
        let cell = ReadCellData::new_instance(true, None, None);
        assert_eq!(cell.boolean_value(), Some(true));
    }

    #[test]
    fn new_instance_from_int() {
        // 对应 Java：ReadCellData(Integer)
        let cell = ReadCellData::new_instance(42i64, None, None);
        assert_eq!(cell.number_value(), Some(BigDecimal::from(42i64)));
    }

    #[test]
    fn new_instance_original_preserves_original() {
        // 对应 Java：ReadCellData(BigDecimal) with original
        let original = BigDecimal::from(123i64);
        let cell = ReadCellData::new_instance_original(original.clone(), Some(0), Some(0));
        assert!(cell.original_number_value().is_some());
        assert_eq!(cell.original_number_value().unwrap(), &original);
    }

    #[test]
    fn from_type_sets_declared_type() {
        // 对应 Java：ReadCellData(CellDataTypeEnum)
        let cell = ReadCellData::from_type(CellDataType::String);
        assert_eq!(cell.get_type(), Some(CellDataType::String));
    }

    #[test]
    fn from_type_and_string_string_type() {
        // 对应 Java：ReadCellData(CellDataTypeEnum.STRING, value)
        let cell = ReadCellData::from_type_and_string(CellDataType::String, "test").unwrap();
        assert_eq!(cell.string_value(), "test");
        assert_eq!(cell.get_type(), Some(CellDataType::String));
    }

    #[test]
    fn from_type_and_string_error_type() {
        // 对应 Java：ReadCellData(CellDataTypeEnum.ERROR, value)
        let cell = ReadCellData::from_type_and_string(CellDataType::Error, "#N/A").unwrap();
        assert_eq!(cell.string_value(), "#N/A");
        assert_eq!(cell.get_type(), Some(CellDataType::Error));
    }

    #[test]
    fn from_type_and_string_rejects_number() {
        // 对应 Java：ReadCellData(NUMBER, value) 抛异常
        let result = ReadCellData::from_type_and_string(CellDataType::Number, "123");
        assert!(result.is_err());
    }

    #[test]
    fn from_boolean_creates_bool_cell() {
        // 对应 Java：ReadCellData(Boolean)
        let cell = ReadCellData::from_boolean(true);
        assert_eq!(cell.boolean_value(), Some(true));
    }

    #[test]
    fn from_string_creates_string_cell() {
        // 对应 Java：ReadCellData(String)
        let cell = ReadCellData::from_string("abc");
        assert_eq!(cell.string_value(), "abc");
    }

    #[test]
    fn from_number_creates_number_cell() {
        // 对应 Java：ReadCellData(BigDecimal)
        let num = BigDecimal::from(99i64);
        let cell = ReadCellData::from_number(num.clone());
        assert_eq!(cell.number_value(), Some(num));
    }

    #[test]
    fn set_and_get_string_value() {
        // 对应 Java：setStringValue / getStringValue
        let mut cell = ReadCellData::empty();
        cell.set_string_value("new value");
        assert_eq!(cell.get_string_value(), "new value");
        assert_eq!(cell.string_value(), "new value");
    }

    #[test]
    fn set_string_value_updates_type() {
        // 对应 Java：setStringValue 同步更新类型
        let mut cell = ReadCellData::empty();
        cell.set_string_value("text");
        assert_eq!(cell.cell_type(), CellDataType::String);
    }

    #[test]
    fn set_and_get_number_value() {
        // 对应 Java：setNumberValue / getNumberValue
        let mut cell = ReadCellData::empty();
        let num = BigDecimal::from(42i64);
        cell.set_number_value(Some(num.clone()));
        assert_eq!(cell.number_value(), Some(num));
        assert_eq!(cell.get_number_value(), cell.number_value());
    }

    #[test]
    fn set_number_value_none_clears() {
        // 对应 Java：setNumberValue(null)
        let mut cell = ReadCellData::from_number(BigDecimal::from(1i64));
        cell.set_number_value(None);
        assert!(cell.number_value().is_none());
    }

    #[test]
    fn set_and_get_boolean_value() {
        // 对应 Java：setBooleanValue / getBooleanValue
        let mut cell = ReadCellData::empty();
        cell.set_boolean_value(Some(true));
        assert_eq!(cell.boolean_value(), Some(true));
        assert_eq!(cell.get_boolean_value(), Some(true));
    }

    #[test]
    fn set_boolean_value_none_clears() {
        // 对应 Java：setBooleanValue(null)
        let mut cell = ReadCellData::from_boolean(true);
        cell.set_boolean_value(None);
        assert!(cell.boolean_value().is_none());
    }

    #[test]
    fn set_and_get_formula_data() {
        // 对应 Java：setFormulaData / getFormulaData
        let mut cell = ReadCellData::empty();
        assert!(cell.formula_data().is_none());
        cell.set_formula_data(None);
        assert!(cell.formula_data().is_none());
    }

    #[test]
    fn set_and_get_data_format_data() {
        // 对应 Java：setDataFormatData / getDataFormatData
        let mut cell = ReadCellData::empty();
        assert!(cell.get_data_format_data().is_none());
        cell.set_data_format_data(None);
        assert!(cell.data_format_data().is_none());
    }

    #[test]
    fn set_and_get_original_number_value() {
        // 对应 Java：setOriginalNumberValue / getOriginalNumberValue
        let mut cell = ReadCellData::empty();
        assert!(cell.get_original_number_value().is_none());
        let num = BigDecimal::from(100i64);
        cell.set_original_number_value(Some(num.clone()));
        assert_eq!(cell.original_number_value().unwrap(), &num);
        assert_eq!(cell.get_original_number_value().unwrap(), &num);
    }

    #[test]
    fn set_and_get_type() {
        // 对应 Java：setType / getType
        let mut cell = ReadCellData::empty();
        cell.set_type(Some(CellDataType::Boolean));
        assert_eq!(cell.get_type(), Some(CellDataType::Boolean));
        cell.set_type(None);
        assert!(cell.get_type().is_none());
    }

    #[test]
    fn row_index_setter_and_getter() {
        // 对应 Java：setRowIndex / getRowIndex
        let mut cell = ReadCellData::empty();
        cell.set_row_index(10);
        assert_eq!(cell.row_index(), 10);
        assert_eq!(cell.get_row_index(), 10);
    }

    #[test]
    fn column_index_setter_and_getter() {
        // 对应 Java：setColumnIndex / getColumnIndex
        let mut cell = ReadCellData::empty();
        cell.set_column_index(5);
        assert_eq!(cell.column_index(), 5);
        assert_eq!(cell.get_column_index(), 5);
    }

    #[test]
    fn raw_value_and_data_accessors() {
        // 对应 Java：getData / rawData
        let cell = ReadCellData::from_string("test");
        let _raw: &CellValue = cell.raw_value();
        let _data: &CellValue = cell.data();
        let _get_data: &CellValue = cell.get_data();
    }

    #[test]
    fn display_value_returns_formatted_text() {
        // 对应 Java：displayValue
        let cell = ReadCellData::from_string("hello");
        assert_eq!(cell.display_value(), "hello");
    }

    #[test]
    fn clone_data_equals_original() {
        // 对应 Java：clone()
        let cell = ReadCellData::from_string("test");
        let cloned = cell.clone_data();
        assert_eq!(cell, cloned);
    }

    #[test]
    fn number_value_for_int() {
        // 对应 Java：getNumberValue for integer data
        let cell = ReadCellData::new_instance(42i64, None, None);
        assert!(cell.number_value().is_some());
    }

    #[test]
    fn number_value_for_empty_is_none() {
        // 对应 Java：getNumberValue for empty data
        let cell = ReadCellData::empty();
        assert!(cell.number_value().is_none());
    }

    #[test]
    fn boolean_value_for_non_bool_is_none() {
        // 对应 Java：getBooleanValue for non-boolean
        let cell = ReadCellData::from_string("text");
        assert!(cell.boolean_value().is_none());
    }

    #[test]
    fn set_data_overrides_value() {
        // 对应 Java：setData
        let mut cell = ReadCellData::empty();
        cell.set_data(CellValue::Int(99));
        assert_eq!(*cell.data(), CellValue::Int(99));
    }
}
