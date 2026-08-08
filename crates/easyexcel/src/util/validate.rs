//! 对应 Java： com.alibaba.excel.util.Validate.

#![allow(dead_code)]

use crate::core::excel_error::ExcelError;

/// 对应 Java：com.alibaba.excel.util.Validate。 Mirrors `org.apache.commons.lang3.Validate#isTrue`.
///
/// # Errors
///
/// 当 `expression` 为 `false` 时返回 [`ExcelError::Unsupported`]，错误消息为 `message`。
pub fn is_true(expression: bool, message: impl Into<String>) -> Result<(), ExcelError> {
    easyexcel_utils::validation::ensure(expression, message.into()).map_err(ExcelError::Unsupported)
}

/// 对应 Java：`Validate.isTrue(boolean)`。
///
/// # Errors
///
/// 表达式为 `false` 时返回 Java 默认消息。
pub fn is_true_default(expression: bool) -> Result<(), ExcelError> {
    is_true(expression, "The validated expression is false")
}

/// 对应 Java：`Validate.notNull(T)`。
///
/// # Errors
///
/// 值为 `None` 时返回 Java 默认消息。
pub fn not_null<T>(value: Option<T>) -> Result<T, ExcelError> {
    not_null_with_message(value, "The validated object is null")
}

/// 对应 Java：`Validate.notNull(T, String, Object...)`。
///
/// # Errors
///
/// 值为 `None` 时返回指定消息。
pub fn not_null_with_message<T>(
    value: Option<T>,
    message: impl Into<String>,
) -> Result<T, ExcelError> {
    value.ok_or_else(|| ExcelError::Unsupported(message.into()))
}

/// 对应 Java：`Validate.checkNotNull(T)`。
///
/// # Errors
///
/// 值为 `None` 时返回 Java 默认消息。
pub fn check_not_null<T>(value: Option<T>) -> Result<T, ExcelError> {
    not_null(value)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn is_true_ok_and_error_paths() {
        // 对应 Java：Validate.isTrue 校验成功与失败
        is_true(true, "must be true").expect("ok");
        let error = is_true(false, "expression failed").expect_err("fails");
        assert!(error.to_string().contains("expression failed"));
    }
}
