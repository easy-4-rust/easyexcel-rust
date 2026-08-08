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
    /// 替换字段内容属性。对应 Java Lombok setter。
    pub const fn set_content_property(&mut self, value: &'a ExcelColumn) { self.column = value; }
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
