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
