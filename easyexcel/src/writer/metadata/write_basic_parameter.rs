//! 对应 Java：`com.alibaba.excel.write.metadata.WriteBasicParameter`.

use crate::core::ConverterRegistry;

/// 对应 Java：`WriteBasicParameter extends BasicParameter`.
///
/// Java carries 9 fields (`relativeHeadRowIndex`, `needHead`,
/// `customWriteHandlerList`, `useDefaultStyle`, `automaticMergeHead`,
/// `excludeColumnIndexes`, `excludeColumnFieldNames`,
/// `includeColumnIndexes`, `includeColumnFieldNames`,
/// `orderByIncludeColumn`). Rust reuses [`WriteOptions`] for the same
/// data, and uses this struct as a thin handle so the 1:1 API name is
/// preserved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WriteBasicParameter {
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
}

impl WriteBasicParameter {
    /// Returns whether a header row is required. (Java `getNeedHead()`)
    #[must_use]
    pub const fn get_need_head(&self) -> Option<bool> {
        self.need_head
    }

    /// Returns the relative head row index. (Java `getRelativeHeadRowIndex()`)
    #[must_use]
    pub const fn get_relative_head_row_index(&self) -> Option<i32> {
        self.relative_head_row_index
    }

    /// Returns whether headers are auto-merged. (Java `getAutomaticMergeHead()`)
    #[must_use]
    pub const fn get_automatic_merge_head(&self) -> Option<bool> {
        self.automatic_merge_head
    }

    /// Returns whether to use default style. (Java `getUseDefaultStyle()`)
    #[must_use]
    pub const fn get_use_default_style(&self) -> Option<bool> {
        self.use_default_style
    }

    /// Returns whether to order by include column. (Java `getOrderByIncludeColumn()`)
    #[must_use]
    pub const fn get_order_by_include_column(&self) -> Option<bool> {
        self.order_by_include_column
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
