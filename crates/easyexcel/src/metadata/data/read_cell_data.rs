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
            CellValue::Decimal(value.clone().with_prec(15)),
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
