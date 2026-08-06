//! 对应 Java： com.alibaba.excel.util.PoiUtils.

use crate::WriteRowContext;

/// 对应 Java：com.alibaba.excel.util.PoiUtils。 Mirrors `com.alibaba.excel.util.PoiUtils#customHeight`.
///
/// Java 版本反射读取 `XSSFRow` / `HSSFRow` 的 `customHeight` 属性；Rust
/// 门面从后端无关的行写入上下文读取显式高度请求，物理格式写入仍由引擎负责。
#[must_use]
pub fn custom_height(row_context: &WriteRowContext) -> bool {
    row_context.row().requested_height().is_some()
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn custom_height_reflects_explicit_row_height() {
        let row_context = WriteRowContext::new("Sheet1", 0, None, false);
        assert!(!custom_height(&row_context));
        row_context.row().set_height(27);
        assert!(custom_height(&row_context));
    }
}
