//! 对应 Java：`com.alibaba.excel.exception.ExcelWriteDataConvertException`。

use super::ExcelDataConvertException;
use std::hash::{Hash, Hasher};

/// 写入转换异常，额外保留发生错误时的完整 Cell Handler 上下文。
#[derive(Debug, Clone)]
pub struct ExcelWriteDataConvertException {
    inner: ExcelDataConvertException,
    cell_write_handler_context: crate::WriteCellContext,
}

// Java Lombok 只比较本类声明的 `cellWriteHandlerContext`，不比较父异常。
impl PartialEq for ExcelWriteDataConvertException {
    fn eq(&self, other: &Self) -> bool {
        self.cell_write_handler_context == other.cell_write_handler_context
    }
}

// 相等上下文必然具有相同物理位置与生命周期标志；这些稳定字段足以满足 Hash 契约，
// 而无需把动态 Holder/handler 对象强制变成可哈希对象。
impl Hash for ExcelWriteDataConvertException {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let context = &self.cell_write_handler_context;
        context.sheet_name.hash(state);
        context.row_index.hash(state);
        context.column_index.hash(state);
        context.is_head.hash(state);
        context.relative_row_index.hash(state);
        context.original_field_type.hash(state);
        context.ignore_fill_style.hash(state);
        context.skip.hash(state);
    }
}
impl ExcelWriteDataConvertException {
    /// Java 双参数构造器。
    #[must_use]
    pub fn new(context: crate::WriteCellContext, message: impl Into<String>) -> Self {
        let inner = data_convert_exception(&context, message, None::<String>);
        Self {
            inner,
            cell_write_handler_context: context,
        }
    }
    /// Java 带 cause 构造器。
    #[must_use]
    pub fn with_cause(
        context: crate::WriteCellContext,
        message: impl Into<String>,
        cause: impl ToString,
    ) -> Self {
        let inner = data_convert_exception(&context, message, Some(cause.to_string()));
        Self {
            inner,
            cell_write_handler_context: context,
        }
    }
    #[must_use]
    pub const fn get_cell_write_handler_context(&self) -> &crate::WriteCellContext {
        &self.cell_write_handler_context
    }
    pub fn set_cell_write_handler_context(&mut self, value: crate::WriteCellContext) {
        self.cell_write_handler_context = value;
    }
    #[must_use]
    pub const fn data_convert_exception(&self) -> &ExcelDataConvertException {
        &self.inner
    }
}

fn data_convert_exception(
    context: &crate::WriteCellContext,
    message: impl Into<String>,
    cause: Option<String>,
) -> ExcelDataConvertException {
    let first = context
        .get_first_cell_data()
        .cloned()
        .unwrap_or(crate::CellValue::Empty);
    let mut cell_data = crate::CellData::new();
    cell_data.set_type(Some(first.data_type()));
    cell_data.set_data(Some(first));
    match cause {
        Some(cause) => ExcelDataConvertException::with_cause(
            usize::try_from(context.get_row_index()).unwrap_or(usize::MAX),
            usize::from(context.get_column_index()),
            cell_data,
            context.get_excel_content_property().cloned(),
            message,
            cause,
        ),
        None => ExcelDataConvertException::new(
            usize::try_from(context.get_row_index()).unwrap_or(usize::MAX),
            usize::from(context.get_column_index()),
            cell_data,
            context.get_excel_content_property().cloned(),
            message,
        ),
    }
}
impl std::fmt::Display for ExcelWriteDataConvertException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}
impl std::error::Error for ExcelWriteDataConvertException {}
impl From<ExcelWriteDataConvertException> for crate::ExcelError {
    fn from(value: ExcelWriteDataConvertException) -> Self {
        value.inner.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的 `WriteCellContext`。
    fn test_context() -> crate::WriteCellContext {
        crate::WriteCellContext::new("Sheet1", 0, 0, crate::CellValue::String("hello".to_owned()))
    }

    #[test]
    fn new_creates_exception() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "bad value");
        assert!(!exc.to_string().is_empty());
    }

    #[test]
    fn with_cause_creates_exception() {
        let ctx = test_context();
        let exc =
            ExcelWriteDataConvertException::with_cause(ctx, "conversion failed", "type mismatch");
        let msg = exc.to_string();
        assert!(msg.contains("conversion failed"), "msg: {msg}");
    }

    #[test]
    fn get_cell_write_handler_context_returns_ref() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "err");
        let returned = exc.get_cell_write_handler_context();
        assert_eq!(returned.sheet_name, "Sheet1");
        assert_eq!(returned.row_index, 0);
        assert_eq!(returned.column_index, 0);
    }

    #[test]
    fn set_cell_write_handler_context_updates() {
        let ctx1 = test_context();
        let mut exc = ExcelWriteDataConvertException::new(ctx1, "err");
        let ctx2 = crate::WriteCellContext::new("Sheet2", 5, 3, crate::CellValue::Int(42));
        exc.set_cell_write_handler_context(ctx2);
        assert_eq!(exc.get_cell_write_handler_context().sheet_name, "Sheet2");
        assert_eq!(exc.get_cell_write_handler_context().row_index, 5);
    }

    #[test]
    fn data_convert_exception_returns_inner() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "inner error");
        let inner = exc.data_convert_exception();
        let msg = format!("{inner}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn display_delegates_to_inner() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "display test");
        let display = exc.to_string();
        assert!(!display.is_empty());
    }

    #[test]
    fn debug_is_implemented() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "debug test");
        let dbg = format!("{exc:?}");
        assert!(dbg.contains("ExcelWriteDataConvertException"));
    }

    #[test]
    fn error_trait_is_implemented() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "error trait");
        let err: &dyn std::error::Error = &exc;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn partial_eq_uses_context_only() {
        let ctx_a = test_context();
        let ctx_b = test_context();
        let a = ExcelWriteDataConvertException::new(ctx_a, "msg a");
        let b = ExcelWriteDataConvertException::new(ctx_b, "msg b");
        assert_eq!(a, b);
    }

    #[test]
    fn partial_eq_different_context() {
        let ctx_a = crate::WriteCellContext::new("Sheet1", 0, 0, crate::CellValue::Int(1));
        let ctx_b = crate::WriteCellContext::new("Sheet2", 0, 0, crate::CellValue::Int(1));
        let a = ExcelWriteDataConvertException::new(ctx_a, "msg");
        let b = ExcelWriteDataConvertException::new(ctx_b, "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_equal_contexts_produce_same_hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let a = ExcelWriteDataConvertException::new(test_context(), "a");
        let b = ExcelWriteDataConvertException::new(test_context(), "b");
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn clone_produces_equal_copy() {
        let a = ExcelWriteDataConvertException::new(test_context(), "clone test");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn into_excel_error() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "convert");
        let err: crate::ExcelError = exc.into();
        let msg = format!("{err}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn no_cause_sets_none() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::new(ctx, "no cause");
        let inner = exc.data_convert_exception();
        let msg = format!("{inner}");
        assert!(msg.contains("no cause"), "msg: {msg}");
    }

    #[test]
    fn with_cause_preserves_cause_info() {
        let ctx = test_context();
        let exc = ExcelWriteDataConvertException::with_cause(ctx, "main", "root cause");
        let msg = format!("{exc}");
        assert!(msg.contains("main"), "msg should contain main: {msg}");
    }
}
