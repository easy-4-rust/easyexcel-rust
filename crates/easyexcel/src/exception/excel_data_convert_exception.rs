//! 对应 Java：`com.alibaba.excel.exception.ExcelDataConvertException`。

use super::ExcelRuntimeException;
use std::hash::{Hash, Hasher};

/// 携带精确单元格位置与字段配置的数据转换异常。
#[derive(Debug, Clone)]
pub struct ExcelDataConvertException {
    inner: ExcelRuntimeException,
    row_index: usize,
    column_index: usize,
    cell_data: crate::CellData<crate::CellValue>,
    excel_content_property: Option<crate::ExcelContentProperty>,
}

// Lombok `@EqualsAndHashCode` 的默认 `callSuper=false`：message/cause 不参与值相等。
impl PartialEq for ExcelDataConvertException {
    fn eq(&self, other: &Self) -> bool {
        self.row_index == other.row_index
            && self.column_index == other.column_index
            && self.cell_data == other.cell_data
            && self.excel_content_property == other.excel_content_property
    }
}

impl Hash for ExcelDataConvertException {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // `CellValue::Float` 保留 IEEE PartialEq（NaN 不自等），因此不能虚假声明 Eq。
        // 物理位置是 Java 声明字段的稳定子集；相等对象必有相同位置，满足 Hash 契约。
        self.row_index.hash(state);
        self.column_index.hash(state);
    }
}
impl ExcelDataConvertException {
    /// Java 五参数构造器。
    #[must_use]
    pub fn new(
        row_index: usize,
        column_index: usize,
        cell_data: crate::CellData<crate::CellValue>,
        excel_content_property: Option<crate::ExcelContentProperty>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            inner: ExcelRuntimeException::with_message(message),
            row_index,
            column_index,
            cell_data,
            excel_content_property,
        }
    }
    /// Java 带 cause 构造器。
    #[must_use]
    pub fn with_cause(
        row_index: usize,
        column_index: usize,
        cell_data: crate::CellData<crate::CellValue>,
        excel_content_property: Option<crate::ExcelContentProperty>,
        message: impl Into<String>,
        cause: impl ToString,
    ) -> Self {
        Self {
            inner: ExcelRuntimeException::with_message_and_cause(message, cause),
            row_index,
            column_index,
            cell_data,
            excel_content_property,
        }
    }
    #[must_use]
    pub const fn get_row_index(&self) -> usize {
        self.row_index
    }
    pub const fn set_row_index(&mut self, value: usize) {
        self.row_index = value;
    }
    #[must_use]
    pub const fn get_column_index(&self) -> usize {
        self.column_index
    }
    pub const fn set_column_index(&mut self, value: usize) {
        self.column_index = value;
    }
    #[must_use]
    pub const fn get_cell_data(&self) -> &crate::CellData<crate::CellValue> {
        &self.cell_data
    }
    pub fn set_cell_data(&mut self, value: crate::CellData<crate::CellValue>) {
        self.cell_data = value;
    }
    #[must_use]
    pub const fn get_excel_content_property(&self) -> Option<&crate::ExcelContentProperty> {
        self.excel_content_property.as_ref()
    }
    pub fn set_excel_content_property(&mut self, value: Option<crate::ExcelContentProperty>) {
        self.excel_content_property = value;
    }
    #[must_use]
    pub const fn runtime_exception(&self) -> &ExcelRuntimeException {
        &self.inner
    }
}
impl std::fmt::Display for ExcelDataConvertException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelDataConvertException {}
impl From<ExcelDataConvertException> for crate::ExcelError {
    fn from(value: ExcelDataConvertException) -> Self {
        crate::ExcelError::Data {
            sheet: String::new(),
            row: u32::try_from(value.row_index).unwrap_or(u32::MAX),
            column: Some(value.column_index),
            field: "",
            value: value.cell_data.get_string_value().unwrap_or("").to_owned(),
            message: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CellData;
    use crate::CellValue;

    fn sample_cell_data() -> CellData<CellValue> {
        let mut data: CellData<CellValue> = CellData::new();
        data.string_value = Some("test".to_owned());
        data
    }

    #[test]
    fn new_constructor_and_getters() {
        let ex =
            ExcelDataConvertException::new(5, 3, sample_cell_data(), None, "conversion failed");
        assert_eq!(ex.get_row_index(), 5);
        assert_eq!(ex.get_column_index(), 3);
        assert_eq!(ex.get_cell_data().get_string_value(), Some("test"));
        assert!(ex.get_excel_content_property().is_none());
        assert!(ex.to_string().contains("conversion failed"));
    }

    #[test]
    fn with_cause_constructor() {
        let ex = ExcelDataConvertException::with_cause(
            1,
            2,
            sample_cell_data(),
            None,
            "bad value",
            "parse error",
        );
        assert_eq!(ex.get_row_index(), 1);
        assert_eq!(ex.get_column_index(), 2);
    }

    #[test]
    fn setters() {
        let mut ex = ExcelDataConvertException::new(0, 0, sample_cell_data(), None, "err");
        ex.set_row_index(10);
        assert_eq!(ex.get_row_index(), 10);
        ex.set_column_index(20);
        assert_eq!(ex.get_column_index(), 20);
        let new_data: CellData<CellValue> = CellData::new();
        ex.set_cell_data(new_data);
        assert!(ex.get_cell_data().get_string_value().is_none());
        ex.set_excel_content_property(None);
        assert!(ex.get_excel_content_property().is_none());
    }

    #[test]
    fn partial_eq_ignores_message() {
        let a = ExcelDataConvertException::new(1, 2, sample_cell_data(), None, "msg A");
        let b = ExcelDataConvertException::new(1, 2, sample_cell_data(), None, "msg B");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_consistent_with_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let a = ExcelDataConvertException::new(1, 2, sample_cell_data(), None, "A");
        let b = ExcelDataConvertException::new(1, 2, sample_cell_data(), None, "B");
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn display_trait() {
        let ex = ExcelDataConvertException::new(0, 0, sample_cell_data(), None, "hello");
        assert_eq!(format!("{}", ex), "hello");
    }

    #[test]
    fn error_trait() {
        let ex = ExcelDataConvertException::new(0, 0, sample_cell_data(), None, "err");
        let err: &dyn std::error::Error = &ex;
        assert!(err.to_string().contains("err"));
    }

    #[test]
    fn from_converts_to_excel_error() {
        let ex = ExcelDataConvertException::new(3, 5, sample_cell_data(), None, "bad");
        let err: crate::ExcelError = ex.into();
        match &err {
            crate::ExcelError::Data {
                row,
                column,
                message,
                ..
            } => {
                assert_eq!(*row, 3);
                assert_eq!(*column, Some(5));
                assert!(message.contains("bad"));
            }
            other => panic!("expected Data variant, got {:?}", other),
        }
    }

    #[test]
    fn runtime_exception_ref() {
        let ex = ExcelDataConvertException::new(0, 0, sample_cell_data(), None, "inner");
        let _inner = ex.runtime_exception();
    }
}
