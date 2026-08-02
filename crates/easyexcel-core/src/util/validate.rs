//! Mirrors Java com.alibaba.excel.util.Validate.

#![allow(dead_code)]

use crate::excel_error::ExcelError;

/// Mirrors `org.apache.commons.lang3.Validate#isTrue`.
pub fn is_true(expression: bool, message: impl Into<String>) -> Result<(), ExcelError> {
    if expression {
        Ok(())
    } else {
        Err(ExcelError::Unsupported(message.into()))
    }
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
