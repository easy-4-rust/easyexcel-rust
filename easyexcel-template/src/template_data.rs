//! 模板填充的标量数据与值类型转换。
//!
//! 对应 Java：`com.alibaba.excel.metadata.template.TemplateData`

use std::collections::BTreeMap;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_core::CellValue;
use num_bigint::BigInt;

/// Value accepted by [`TemplateData`] placeholder insertion methods.
pub trait IntoTemplateValue {
    /// Converts the value to its typed template representation.
    fn into_template_value(self) -> CellValue;
}

impl IntoTemplateValue for CellValue {
    fn into_template_value(self) -> CellValue {
        self
    }
}

impl IntoTemplateValue for String {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self)
    }
}

impl IntoTemplateValue for &str {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self.to_owned())
    }
}

impl IntoTemplateValue for &String {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self.clone())
    }
}

impl IntoTemplateValue for bool {
    fn into_template_value(self) -> CellValue {
        CellValue::Bool(self)
    }
}

macro_rules! impl_integer_template_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl IntoTemplateValue for $type {
                fn into_template_value(self) -> CellValue {
                    CellValue::Int(i64::from(self))
                }
            }
        )+
    };
}

impl_integer_template_value!(i8, i16, i32, i64, u8, u16, u32);

macro_rules! impl_decimal_integer_template_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl IntoTemplateValue for $type {
                fn into_template_value(self) -> CellValue {
                    CellValue::Decimal(BigDecimal::from(self))
                }
            }
        )+
    };
}

impl_decimal_integer_template_value!(i128, u64, u128);

impl IntoTemplateValue for isize {
    fn into_template_value(self) -> CellValue {
        CellValue::Int(i64::try_from(self).expect("Rust isize is at most 64 bits"))
    }
}

impl IntoTemplateValue for usize {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(BigDecimal::from(
            u64::try_from(self).expect("Rust usize is at most 64 bits"),
        ))
    }
}

impl IntoTemplateValue for BigInt {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(BigDecimal::from(self))
    }
}

impl IntoTemplateValue for f32 {
    fn into_template_value(self) -> CellValue {
        CellValue::Float(f64::from(self))
    }
}

impl IntoTemplateValue for f64 {
    fn into_template_value(self) -> CellValue {
        CellValue::Float(self)
    }
}

impl IntoTemplateValue for BigDecimal {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(self)
    }
}

impl IntoTemplateValue for NaiveDate {
    fn into_template_value(self) -> CellValue {
        CellValue::Date(self)
    }
}

impl IntoTemplateValue for NaiveDateTime {
    fn into_template_value(self) -> CellValue {
        CellValue::DateTime(self)
    }
}

impl<T> IntoTemplateValue for Option<T>
where
    T: IntoTemplateValue,
{
    fn into_template_value(self) -> CellValue {
        self.map_or(CellValue::Empty, IntoTemplateValue::into_template_value)
    }
}

/// Scalar values used to replace `{key}` placeholders in OOXML text nodes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateData {
    pub(crate) values: BTreeMap<String, CellValue>,
}

impl TemplateData {
    /// Creates empty template data.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Adds or replaces a placeholder value.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl IntoTemplateValue) -> Self {
        self.values.insert(key.into(), value.into_template_value());
        self
    }

    /// Inserts a placeholder value and returns the previous value.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl IntoTemplateValue,
    ) -> Option<CellValue> {
        self.values.insert(key.into(), value.into_template_value())
    }

    /// Returns all values in deterministic key order.
    #[must_use]
    pub const fn values(&self) -> &BTreeMap<String, CellValue> {
        &self.values
    }
}
