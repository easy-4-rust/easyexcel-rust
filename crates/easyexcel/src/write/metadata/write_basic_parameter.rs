//! 对应 Java：`com.alibaba.excel.write.metadata.WriteBasicParameter`.

use crate::core::ConverterRegistry;
use crate::metadata::BasicParameter;

/// 对应 Java：`WriteBasicParameter extends BasicParameter`.
///
/// Java carries 9 fields (`relativeHeadRowIndex`, `needHead`,
/// `customWriteHandlerList`, `useDefaultStyle`, `automaticMergeHead`,
/// `excludeColumnIndexes`, `excludeColumnFieldNames`,
/// `includeColumnIndexes`, `includeColumnFieldNames`,
/// `orderByIncludeColumn`). Rust reuses `WriteOptions` for the same
/// data, and uses this struct as a thin handle so the 1:1 API name is
/// preserved.
#[derive(Debug, Clone, Default)]
pub struct WriteBasicParameter {
    /// Java 父类 `BasicParameter` 的完整字段。
    pub basic_parameter: BasicParameter,
    /// Mirrors `WriteBasicParameter.relativeHeadRowIndex`.
    pub relative_head_row_index: Option<i32>,
    /// Mirrors `WriteBasicParameter.needHead`.
    pub need_head: Option<bool>,
    /// Mirrors `WriteBasicParameter.useDefaultStyle`.
    pub use_default_style: Option<bool>,
    /// Mirrors `WriteBasicParameter.automaticMergeHead`.
    pub automatic_merge_head: Option<bool>,
    /// Mirrors `WriteBasicParameter.excludeColumnIndexes`.
    pub exclude_column_indexes: Option<Vec<usize>>,
    /// Mirrors `WriteBasicParameter.excludeColumnFieldNames`.
    pub exclude_column_field_names: Option<Vec<String>>,
    /// Mirrors `WriteBasicParameter.includeColumnIndexes`.
    pub include_column_indexes: Option<Vec<usize>>,
    /// Mirrors `WriteBasicParameter.includeColumnFieldNames`.
    pub include_column_field_names: Option<Vec<String>>,
    /// Mirrors `WriteBasicParameter.orderByIncludeColumn`.
    pub order_by_include_column: Option<bool>,
    /// Mirrors `WriteBasicParameter.converters` (custom-registered converters).
    pub converters: ConverterRegistry,
    /// Handler 类型/注册键列表；实际可变实例由 writer 的共享处理链持有。
    pub custom_write_handler_list: Vec<String>,
}

// Java Lombok `@EqualsAndHashCode` 默认 `callSuper = false`，只比较本类声明字段。
impl PartialEq for WriteBasicParameter {
    fn eq(&self, other: &Self) -> bool {
        self.relative_head_row_index == other.relative_head_row_index
            && self.need_head == other.need_head
            && self.custom_write_handler_list == other.custom_write_handler_list
            && self.use_default_style == other.use_default_style
            && self.automatic_merge_head == other.automatic_merge_head
            && self.exclude_column_indexes == other.exclude_column_indexes
            && self.exclude_column_field_names == other.exclude_column_field_names
            && self.include_column_indexes == other.include_column_indexes
            && self.include_column_field_names == other.include_column_field_names
            && self.order_by_include_column == other.order_by_include_column
    }
}

impl Eq for WriteBasicParameter {}

impl std::hash::Hash for WriteBasicParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.relative_head_row_index, state);
        std::hash::Hash::hash(&self.need_head, state);
        std::hash::Hash::hash(&self.custom_write_handler_list, state);
        std::hash::Hash::hash(&self.use_default_style, state);
        std::hash::Hash::hash(&self.automatic_merge_head, state);
        std::hash::Hash::hash(&self.exclude_column_indexes, state);
        std::hash::Hash::hash(&self.exclude_column_field_names, state);
        std::hash::Hash::hash(&self.include_column_indexes, state);
        std::hash::Hash::hash(&self.include_column_field_names, state);
        std::hash::Hash::hash(&self.order_by_include_column, state);
    }
}

impl WriteBasicParameter {
    /// 从工作簿/Sheet 共用的写入选项构造 Java 参数对象。
    ///
    /// 对应 Java：`WriteWorkbook extends WriteBasicParameter`。所有已经在
    /// [`crate::WriteOptions`] 中解析完成的父类字段都必须进入 Holder，不能只保存
    /// `WriteWorkbook` 外壳而让 Handler 看到一组默认值。
    #[must_use]
    pub fn from_options(options: &crate::WriteOptions) -> Self {
        Self {
            basic_parameter: BasicParameter {
                head: options.dynamic_head.clone(),
                auto_trim: Some(options.auto_trim),
                use1904windowing: Some(options.use_1904_windowing),
                locale: Some(options.locale.clone()),
                use_scientific_format: Some(options.use_scientific_format),
                filed_cache_location: Some(options.filed_cache_location),
                ..BasicParameter::default()
            },
            relative_head_row_index: Some(options.relative_head_row_index),
            need_head: Some(options.need_head),
            use_default_style: Some(options.use_default_style),
            automatic_merge_head: Some(options.automatic_merge_head),
            exclude_column_indexes: Some(options.exclude_column_indexes.clone()),
            exclude_column_field_names: Some(options.exclude_column_field_names.clone()),
            include_column_indexes: options.include_column_indexes.clone(),
            include_column_field_names: options.include_column_field_names.clone(),
            order_by_include_column: Some(options.order_by_include_column),
            converters: options.converters.clone(),
            custom_write_handler_list: Vec::new(),
        }
    }

    /// 返回自定义 Handler 注册键。对应 Java `getCustomWriteHandlerList()`。
    #[must_use]
    pub fn get_custom_write_handler_list(&self) -> &[String] {
        &self.custom_write_handler_list
    }

    /// 替换自定义 Handler 注册键。对应 Java `setCustomWriteHandlerList(...)`。
    pub fn set_custom_write_handler_list(&mut self, value: Vec<String>) {
        self.custom_write_handler_list = value;
    }
    /// 返回 Java 父类参数。
    #[must_use] pub const fn get_basic_parameter(&self) -> &BasicParameter { &self.basic_parameter }
    /// 返回可变 Java 父类参数。
    pub const fn get_basic_parameter_mut(&mut self) -> &mut BasicParameter { &mut self.basic_parameter }
    /// Returns whether a header row is required. (Java `getNeedHead()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteBasicParameter。
    pub const fn get_need_head(&self) -> Option<bool> {
        self.need_head
    }

    /// Returns the relative head row index. (Java `getRelativeHeadRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteBasicParameter。
    pub const fn get_relative_head_row_index(&self) -> Option<i32> {
        self.relative_head_row_index
    }

    /// Returns whether headers are auto-merged. (Java `getAutomaticMergeHead()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteBasicParameter。
    pub const fn get_automatic_merge_head(&self) -> Option<bool> {
        self.automatic_merge_head
    }

    /// Returns whether to use default style. (Java `getUseDefaultStyle()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteBasicParameter。
    pub const fn get_use_default_style(&self) -> Option<bool> {
        self.use_default_style
    }

    /// Returns whether to order by include column. (Java `getOrderByIncludeColumn()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteBasicParameter。
    pub const fn get_order_by_include_column(&self) -> Option<bool> {
        self.order_by_include_column
    }

    pub const fn set_need_head(&mut self, value: Option<bool>) { self.need_head = value; }
    pub const fn set_relative_head_row_index(&mut self, value: Option<i32>) {
        self.relative_head_row_index = value;
    }
    pub const fn set_automatic_merge_head(&mut self, value: Option<bool>) {
        self.automatic_merge_head = value;
    }
    pub const fn set_use_default_style(&mut self, value: Option<bool>) {
        self.use_default_style = value;
    }
    pub const fn set_order_by_include_column(&mut self, value: Option<bool>) {
        self.order_by_include_column = value;
    }
    #[must_use] pub fn get_exclude_column_indexes(&self) -> Option<&[usize]> {
        self.exclude_column_indexes.as_deref()
    }
    pub fn set_exclude_column_indexes(&mut self, value: Option<Vec<usize>>) {
        self.exclude_column_indexes = value;
    }
    #[must_use] pub fn get_exclude_column_field_names(&self) -> Option<&[String]> {
        self.exclude_column_field_names.as_deref()
    }
    pub fn set_exclude_column_field_names(&mut self, value: Option<Vec<String>>) {
        self.exclude_column_field_names = value;
    }
    #[must_use] pub fn get_include_column_indexes(&self) -> Option<&[usize]> {
        self.include_column_indexes.as_deref()
    }
    pub fn set_include_column_indexes(&mut self, value: Option<Vec<usize>>) {
        self.include_column_indexes = value;
    }
    #[must_use] pub fn get_include_column_field_names(&self) -> Option<&[String]> {
        self.include_column_field_names.as_deref()
    }
    pub fn set_include_column_field_names(&mut self, value: Option<Vec<String>>) {
        self.include_column_field_names = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constructor_creates_empty_parameter() {
        let param = WriteBasicParameter::default();
        assert_eq!(param.get_need_head(), None);
        assert_eq!(param.get_relative_head_row_index(), None);
        assert_eq!(param.get_automatic_merge_head(), None);
        assert_eq!(param.get_use_default_style(), None);
        assert_eq!(param.get_order_by_include_column(), None);
        assert!(param.exclude_column_indexes.is_none());
        assert!(param.exclude_column_field_names.is_none());
        assert!(param.include_column_indexes.is_none());
        assert!(param.include_column_field_names.is_none());
    }

    #[test]
    fn getters_return_configured_values() {
        let param = WriteBasicParameter {
            need_head: Some(true),
            relative_head_row_index: Some(2),
            automatic_merge_head: Some(true),
            use_default_style: Some(false),
            order_by_include_column: Some(true),
            ..WriteBasicParameter::default()
        };

        assert_eq!(param.get_need_head(), Some(true));
        assert_eq!(param.get_relative_head_row_index(), Some(2));
        assert_eq!(param.get_automatic_merge_head(), Some(true));
        assert_eq!(param.get_use_default_style(), Some(false));
        assert_eq!(param.get_order_by_include_column(), Some(true));
    }

    #[test]
    fn column_filter_fields_store_values() {
        let param = WriteBasicParameter {
            exclude_column_indexes: Some(vec![0, 2]),
            exclude_column_field_names: Some(vec!["id".to_owned()]),
            include_column_indexes: Some(vec![1, 3]),
            include_column_field_names: Some(vec!["name".to_owned(), "age".to_owned()]),
            ..WriteBasicParameter::default()
        };

        assert_eq!(param.exclude_column_indexes, Some(vec![0, 2]));
        assert_eq!(
            param.exclude_column_field_names,
            Some(vec!["id".to_owned()])
        );
        assert_eq!(param.include_column_indexes, Some(vec![1, 3]));
        assert_eq!(
            param.include_column_field_names,
            Some(vec!["name".to_owned(), "age".to_owned()])
        );
    }

    #[test]
    fn clone_preserves_all_fields() {
        let param = WriteBasicParameter {
            need_head: Some(true),
            relative_head_row_index: Some(1),
            automatic_merge_head: Some(true),
            use_default_style: Some(false),
            order_by_include_column: Some(true),
            include_column_indexes: Some(vec![0, 1]),
            ..WriteBasicParameter::default()
        };
        let cloned = param.clone();

        assert_eq!(param, cloned);
    }

    #[test]
    fn partial_eq_compares_all_fields() {
        let a = WriteBasicParameter {
            need_head: Some(true),
            ..WriteBasicParameter::default()
        };
        let b = WriteBasicParameter {
            need_head: Some(true),
            ..WriteBasicParameter::default()
        };
        let c = WriteBasicParameter {
            need_head: Some(false),
            ..WriteBasicParameter::default()
        };

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn debug_format_includes_fields() {
        let param = WriteBasicParameter {
            need_head: Some(true),
            relative_head_row_index: Some(2),
            ..WriteBasicParameter::default()
        };
        let debug = format!("{param:?}");
        assert!(debug.contains("need_head"));
        assert!(debug.contains("relative_head_row_index"));
    }
}
