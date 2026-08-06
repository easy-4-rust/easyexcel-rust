//! Rust-native counterpart of Java `java.util.Date`.

use chrono::NaiveDateTime;

use crate::{CellValue, ConvertContext, ExcelError, FromExcelCell, IntoExcelCell};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Distinct workbook-local date-time type for Java `java.util.Date` fields.
///
/// `chrono::NaiveDateTime` remains the counterpart of Java
/// `java.time.LocalDateTime`. Keeping `JavaDate` as a transparent newtype gives
/// the converter registry two distinct `TypeId` keys, exactly as Java has two
/// distinct `Class<?>` keys, while retaining ergonomic conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct JavaDate(NaiveDateTime);

impl JavaDate {
    /// Creates a Java-date equivalent from workbook-local date and time.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(value: NaiveDateTime) -> Self {
        Self(value)
    }

    /// Returns the workbook-local date and time.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn naive_local(self) -> NaiveDateTime {
        self.0
    }
}

impl From<NaiveDateTime> for JavaDate {
    fn from(value: NaiveDateTime) -> Self {
        Self::new(value)
    }
}

impl From<JavaDate> for NaiveDateTime {
    fn from(value: JavaDate) -> Self {
        value.naive_local()
    }
}

impl FromExcelCell for JavaDate {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        NaiveDateTime::from_excel_cell(value, context).map(Self::new)
    }
}

impl IntoExcelCell for JavaDate {
    fn to_excel_cell(&self, context: &ConvertContext) -> Result<CellValue, ExcelError> {
        self.0.to_excel_cell(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn java_date_has_a_distinct_type_key_and_lossless_naive_conversion() {
        let value = NaiveDate::from_ymd_opt(2026, 7, 24)
            .unwrap()
            .and_hms_milli_opt(12, 34, 56, 789)
            .unwrap();
        let java_date = JavaDate::from(value);
        assert_eq!(java_date.naive_local(), value);
        assert_ne!(
            std::any::TypeId::of::<JavaDate>(),
            std::any::TypeId::of::<NaiveDateTime>()
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn from_java_date_to_naive_datetime() {
        // 对应 Java：java.util.Date 与 LocalDateTime 互转
        let value = NaiveDate::from_ymd_opt(2024, 5, 6)
            .unwrap()
            .and_hms_opt(7, 8, 9)
            .unwrap();
        let java_date = JavaDate::new(value);
        let back: NaiveDateTime = java_date.into();
        assert_eq!(back, value);
        assert_eq!(JavaDate::from(value).naive_local(), value);
    }
}
