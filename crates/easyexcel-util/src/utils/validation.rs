//! 与具体表格格式和门面错误类型无关的条件校验。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 要求 `expression` 为真，否则返回调用方提供的错误值。
///
/// 门面层可用该原语映射 Java `Validate.isTrue` 的具体异常类型，基础 crate
/// 不依赖 `easyexcel::ExcelError`。
///
/// # Errors
///
/// 当 `expression` 为 `false` 时返回 `error`。
pub fn ensure<E>(expression: bool, error: E) -> Result<(), E> {
    if expression { Ok(()) } else { Err(error) }
}

#[cfg(test)]
mod tests {
    use super::ensure;

    #[test]
    fn ensure_preserves_the_supplied_error() {
        assert_eq!(ensure(true, "ignored"), Ok(()));
        assert_eq!(ensure(false, "failed"), Err("failed"));
    }
}
