//! 对应 Java：`com.alibaba.excel.converters.WriteConverterContext`.

use crate::core::convert_context::ConvertContext;
use crate::core::excel_column::ExcelColumn;

/// Context supplied to a custom Rust-to-cell converter.
///
/// 对应 Java：`WriteConverterContext<T>(value, contentProperty,
/// writeContext)`. Rust drops the heavy `WriteContext` reference and uses
/// the lightweight `ConvertContext`.
#[derive(Debug, Clone, Copy)]
pub struct WriteConverterContext<'a, T> {
    value: &'a T,
    column: &'a ExcelColumn,
    context: &'a ConvertContext,
}

impl<'a, T> WriteConverterContext<'a, T> {
    /// 替换待转换值。对应 Java Lombok `setValue`。
    ///
    /// Rust 要求新引用与上下文具有相同生命周期，避免 Java 无参构造后可能出现的
    /// 临时非法状态。
    pub const fn set_value(&mut self, value: &'a T) {
        self.value = value;
    }

    /// 替换字段内容属性。对应 Java Lombok setter。
    pub const fn set_content_property(&mut self, value: &'a ExcelColumn) { self.column = value; }

    /// 替换写入上下文。对应 Java Lombok `setWriteContext`。
    pub const fn set_write_context(&mut self, value: &'a ConvertContext) {
        self.context = value;
    }
    /// Creates a write conversion context. (Java `@AllArgsConstructor`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.WriteConverterContext。
    pub const fn new(value: &'a T, column: &'a ExcelColumn, context: &'a ConvertContext) -> Self {
        Self {
            value,
            column,
            context,
        }
    }

    /// Returns the Rust field value. (Java `getValue()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.WriteConverterContext。
    pub const fn value(&self) -> &'a T {
        self.value
    }

    /// Java `getValue()` 兼容别名。
    #[must_use]
    pub const fn get_value(&self) -> &'a T { self.value() }

    /// Returns the field's static column metadata. (Java `getContentProperty()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.WriteConverterContext。
    pub const fn column(&self) -> &'a ExcelColumn {
        self.column
    }

    /// 返回字段内容属性。对应 Java：`getContentProperty()`。
    #[must_use]
    pub const fn get_content_property(&self) -> &'a ExcelColumn { self.column() }

    /// Returns the target row, column, field, and format information. (Java `getWriteContext()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.WriteConverterContext。
    pub const fn convert_context(&self) -> &'a ConvertContext {
        self.context
    }

    /// 返回写上下文的轻量等价物。对应 Java：`getWriteContext()`。
    #[must_use]
    pub const fn get_write_context(&self) -> &'a ConvertContext { self.convert_context() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConvertContext;
    use crate::ExcelColumn;

    fn sample_column() -> ExcelColumn {
        ExcelColumn::new("test", "Test", None, 0, None)
    }

    fn sample_context() -> ConvertContext {
        ConvertContext {
            sheet_name: "S".to_owned(),
            row_index: 0,
            column_index: None,
            field: "",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn new_and_getters() {
        let value = 42_i32;
        let column = sample_column();
        let context = sample_context();
        let ctx = WriteConverterContext::new(&value, &column, &context);
        assert_eq!(*ctx.value(), 42);
        assert_eq!(*ctx.get_value(), 42);
        assert!(std::ptr::eq(ctx.column(), &column));
        assert!(std::ptr::eq(ctx.get_content_property(), &column));
        assert!(std::ptr::eq(ctx.convert_context(), &context));
        assert!(std::ptr::eq(ctx.get_write_context(), &context));
    }

    #[test]
    fn setters() {
        let value_a = 1_i32;
        let value_b = 2_i32;
        let column_a = sample_column();
        let column_b = sample_column();
        let context_a = sample_context();
        let context_b = sample_context();
        let mut ctx = WriteConverterContext::new(&value_a, &column_a, &context_a);
        ctx.set_value(&value_b);
        assert_eq!(*ctx.value(), 2);
        ctx.set_content_property(&column_b);
        assert!(std::ptr::eq(ctx.column(), &column_b));
        ctx.set_write_context(&context_b);
        assert!(std::ptr::eq(ctx.convert_context(), &context_b));
    }
}
