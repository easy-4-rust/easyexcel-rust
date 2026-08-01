//! Mirrors Java `com.alibaba.excel.write.builder.AbstractExcelWriterParameterBuilder`.

use crate::CellStyle;
use easyexcel_core::WriteHandler;

use crate::metadata::WriteBasicParameter;

/// Mirrors Java `AbstractExcelWriterParameterBuilder<T, C>`.
///
/// The Java side chains 12 setter methods (`needHead`, `useDefaultStyle`,
/// `automaticMergeHead`, `excludeColumnIndexes`, `excludeColumnFieldNames`,
/// `includeColumnIndexes`, `includeColumnFieldNames`,
/// `orderByIncludeColumn`, `relativeHeadRowIndex`, `registerWriteHandler`,
/// `excludeColumnFiledNames` (typo'd alias), and `head(List)`).
///
/// In Rust, the same surface lives on the chain-returning
/// [`crate::EasyExcel::write`]-based builder. This trait preserves the
/// 1:1 names so Java-aware code can still find the canonical setters.
pub trait AbstractExcelWriterParameterBuilder {
    /// Returns the parameter being mutated. (Java `parameter()`)
    fn parameter(&mut self) -> &mut WriteBasicParameter;

    /// Sets whether a header row is written. (Java `needHead(Boolean)`)
    fn need_head(&mut self, need_head: bool) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().need_head = Some(need_head);
        self
    }

    /// Sets the default style flag. (Java `useDefaultStyle(Boolean)`)
    fn use_default_style(&mut self, use_default_style: bool) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().use_default_style = Some(use_default_style);
        self
    }

    /// Sets automatic header merging. (Java `automaticMergeHead(Boolean)`)
    fn automatic_merge_head(&mut self, automatic_merge_head: bool) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().automatic_merge_head = Some(automatic_merge_head);
        self
    }

    /// Sets the relative head row index. (Java `relativeHeadRowIndex(Integer)`)
    fn relative_head_row_index(&mut self, index: i32) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().relative_head_row_index = Some(index);
        self
    }

    /// Sets the include-order flag. (Java `orderByIncludeColumn(Boolean)`)
    fn order_by_include_column(&mut self, enabled: bool) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().order_by_include_column = Some(enabled);
        self
    }

    /// Replaces inherited excluded physical columns.
    /// (Java `excludeColumnIndexes(Collection<Integer>)`)
    fn exclude_column_indexes(&mut self, indexes: Vec<usize>) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().exclude_column_indexes = Some(indexes);
        self
    }

    /// Replaces inherited excluded field names.
    /// (Java `excludeColumnFieldNames(Collection<String>)`)
    fn exclude_column_field_names(&mut self, names: Vec<String>) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().exclude_column_field_names = Some(names);
        self
    }

    /// Deprecated Java spelling retained for migration compatibility.
    fn exclude_column_filed_names(&mut self, names: Vec<String>) -> &mut Self
    where
        Self: Sized,
    {
        self.exclude_column_field_names(names)
    }

    /// Replaces inherited included physical columns.
    /// (Java `includeColumnIndexes(Collection<Integer>)`)
    fn include_column_indexes(&mut self, indexes: Vec<usize>) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().include_column_indexes = Some(indexes);
        self
    }

    /// Replaces inherited included field names.
    /// (Java `includeColumnFieldNames(Collection<String>)`)
    fn include_column_field_names(&mut self, names: Vec<String>) -> &mut Self
    where
        Self: Sized,
    {
        self.parameter().include_column_field_names = Some(names);
        self
    }

    /// Deprecated Java spelling retained for migration compatibility.
    fn include_column_filed_names(&mut self, names: Vec<String>) -> &mut Self
    where
        Self: Sized,
    {
        self.include_column_field_names(names)
    }

    /// Appends a write handler. (Java `registerWriteHandler(WriteHandler)`)
    fn register_write_handler(&mut self, handler: Box<dyn WriteHandler>) -> &mut Self
    where
        Self: Sized;

    /// Convenience setter that returns a `CellStyle` to builder methods. The
    /// Java side exposes typed setters on `ExcelWriterBuilder.head_style`; the
    /// trait accepts the value object directly.
    fn head_style_slot(&self) -> Option<CellStyle> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_core::WriteHandler;

    /// Concrete test impl of the parameter-builder trait.
    struct TestParamBuilder {
        param: WriteBasicParameter,
        handlers: Vec<Box<dyn WriteHandler>>,
    }

    impl AbstractExcelWriterParameterBuilder for TestParamBuilder {
        fn parameter(&mut self) -> &mut WriteBasicParameter {
            &mut self.param
        }

        fn register_write_handler(&mut self, handler: Box<dyn WriteHandler>) -> &mut Self {
            self.handlers.push(handler);
            self
        }
    }

    #[test]
    fn abstract_writer_parameter_builder_need_head() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.need_head(false);
        assert_eq!(b.param.need_head, Some(false));
    }

    #[test]
    fn abstract_writer_parameter_builder_use_default_style() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.use_default_style(true);
        assert_eq!(b.param.use_default_style, Some(true));
    }

    #[test]
    fn abstract_writer_parameter_builder_automatic_merge_head() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.automatic_merge_head(true);
        assert_eq!(b.param.automatic_merge_head, Some(true));
    }

    #[test]
    fn abstract_writer_parameter_builder_relative_head_row_index() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.relative_head_row_index(5);
        assert_eq!(b.param.relative_head_row_index, Some(5));
    }

    #[test]
    fn abstract_writer_parameter_builder_order_by_include_column() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.order_by_include_column(true);
        assert_eq!(b.param.order_by_include_column, Some(true));
    }

    #[test]
    fn abstract_writer_parameter_builder_exclude_column_indexes() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.exclude_column_indexes(vec![1, 2, 3]);
        assert_eq!(b.param.exclude_column_indexes, Some(vec![1, 2, 3]));
    }

    #[test]
    fn abstract_writer_parameter_builder_exclude_column_field_names() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.exclude_column_field_names(vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(
            b.param.exclude_column_field_names,
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
    }

    #[test]
    fn abstract_writer_parameter_builder_exclude_column_filed_names_alias() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.exclude_column_filed_names(vec!["c".to_owned()]);
        assert_eq!(
            b.param.exclude_column_field_names,
            Some(vec!["c".to_owned()])
        );
    }

    #[test]
    fn abstract_writer_parameter_builder_include_column_indexes() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.include_column_indexes(vec![0]);
        assert_eq!(b.param.include_column_indexes, Some(vec![0]));
    }

    #[test]
    fn abstract_writer_parameter_builder_include_column_field_names() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.include_column_field_names(vec!["x".to_owned()]);
        assert_eq!(
            b.param.include_column_field_names,
            Some(vec!["x".to_owned()])
        );
    }

    #[test]
    fn abstract_writer_parameter_builder_include_column_filed_names_alias() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        b.include_column_filed_names(vec!["y".to_owned()]);
        assert_eq!(
            b.param.include_column_field_names,
            Some(vec!["y".to_owned()])
        );
    }

    #[test]
    fn abstract_writer_parameter_builder_register_write_handler() {
        /// Minimal no-op WriteHandler used to test handler registration.
        struct NoopHandler;
        impl WriteHandler for NoopHandler {
            fn order(&self) -> i32 {
                0
            }
        }
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        let handler: Box<dyn WriteHandler> = Box::new(NoopHandler);
        b.register_write_handler(handler);
        assert_eq!(b.handlers.len(), 1);
        assert_eq!(NoopHandler.order(), 0);
    }

    #[test]
    fn abstract_writer_parameter_builder_head_style_slot() {
        let b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        let _ = b.head_style_slot();
    }

    #[test]
    fn abstract_writer_parameter_builder_parameter_accessor() {
        let mut b = TestParamBuilder {
            param: WriteBasicParameter::default(),
            handlers: vec![],
        };
        let _ = b.parameter();
    }
}
