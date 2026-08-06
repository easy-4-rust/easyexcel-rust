//! 模板填充的标量数据与值类型转换。
//!
//! 对应 Java：`com.alibaba.excel.metadata.template.TemplateData`

use std::collections::BTreeMap;

use crate::core::CellValue;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use num_bigint::BigInt;

include!("template_data/into_template_value.rs");

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

/// 对应 Java：com.alibaba.excel.metadata.template.TemplateData。 Scalar values used to replace `{key}` placeholders in OOXML text nodes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateData {
    pub(crate) values: BTreeMap<String, CellValue>,
}

impl TemplateData {
    /// Creates empty template data.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.template.TemplateData。
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.template.TemplateData。 Adds or replaces a placeholder value.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl IntoTemplateValue) -> Self {
        self.values.insert(key.into(), value.into_template_value());
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.template.TemplateData。 Inserts a placeholder value and returns the previous value.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl IntoTemplateValue,
    ) -> Option<CellValue> {
        self.values.insert(key.into(), value.into_template_value())
    }

    /// Returns all values in deterministic key order.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.template.TemplateData。
    pub const fn values(&self) -> &BTreeMap<String, CellValue> {
        &self.values
    }
}
