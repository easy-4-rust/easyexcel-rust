//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteHolder` (interface).

use std::collections::HashSet;

use crate::{ConfigurationHolder, ExcelWriteHeadProperty};

/// 对应 Java：`WriteHolder extends ConfigurationHolder`.
pub trait WriteHolder: ConfigurationHolder {
    /// Returns the resolved `ExcelWriteHeadProperty` for the holder. (Java `excelWriteHeadProperty()`)
    fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty;

    /// Returns whether a field is ignored for the holder. (Java `ignore(fieldName, columnIndex)`)
    fn ignore(&self, field_name: Option<&str>, column_index: Option<usize>) -> bool;

    /// Returns whether a header is required. (Java `needHead()`)
    fn need_head(&self) -> bool;

    /// Returns the relative head row index. (Java `relativeHeadRowIndex()`)
    fn relative_head_row_index(&self) -> i32;

    /// Returns whether headers are auto-merged. (Java `automaticMergeHead()`)
    fn automatic_merge_head(&self) -> bool;

    /// Returns whether output columns follow include-list order.
    /// (Java `orderByIncludeColumn()`)
    fn order_by_include_column(&self) -> bool;

    /// Returns included physical column indexes. (Java `includeColumnIndexes()`)
    fn include_column_indexes(&self) -> Option<&HashSet<usize>>;

    /// Returns included field names. (Java `includeColumnFieldNames()`)
    fn include_column_field_names(&self) -> Option<&HashSet<String>>;

    /// Returns excluded physical column indexes. (Java `excludeColumnIndexes()`)
    fn exclude_column_indexes(&self) -> Option<&HashSet<usize>>;

    /// Returns excluded field names. (Java `excludeColumnFieldNames()`)
    fn exclude_column_field_names(&self) -> Option<&HashSet<String>>;
}

/// 为 Java 中继承 `AbstractWriteHolder` 的具体 Holder 生成显式接口委托。
///
/// 每个具体 Holder 在自己的文件中声明委托，以保留逐类型证据；这里仅集中 Rust
/// trait 的重复转发逻辑，不承载工作簿格式能力。
macro_rules! delegate_write_holder_contract {
    ($holder:ident<$lifetime:lifetime>, $parent:ident) => {
        impl<$lifetime> $crate::metadata::MetadataHolder for $holder<$lifetime> {
            fn holder_type(&self) -> $crate::HolderEnum {
                $crate::metadata::MetadataHolder::holder_type(self.$parent())
            }
        }

        impl<$lifetime> $crate::metadata::ConfigurationHolder for $holder<$lifetime> {
            fn is_new(&self) -> bool {
                $crate::metadata::ConfigurationHolder::is_new(self.$parent())
            }

            fn global_configuration(&self) -> &$crate::GlobalConfiguration {
                $crate::metadata::ConfigurationHolder::global_configuration(self.$parent())
            }

            fn converter_map(&self) -> &$crate::ConverterRegistry {
                $crate::metadata::ConfigurationHolder::converter_map(self.$parent())
            }
        }

        impl<$lifetime> $crate::write::metadata::holder::write_holder::WriteHolder
            for $holder<$lifetime>
        {
            fn excel_write_head_property(&self) -> &$crate::ExcelWriteHeadProperty {
                $crate::write::metadata::holder::write_holder::WriteHolder::excel_write_head_property(
                    self.$parent(),
                )
            }

            fn ignore(&self, field_name: Option<&str>, column_index: Option<usize>) -> bool {
                $crate::write::metadata::holder::write_holder::WriteHolder::ignore(
                    self.$parent(),
                    field_name,
                    column_index,
                )
            }

            fn need_head(&self) -> bool {
                $crate::write::metadata::holder::write_holder::WriteHolder::need_head(self.$parent())
            }

            fn relative_head_row_index(&self) -> i32 {
                $crate::write::metadata::holder::write_holder::WriteHolder::relative_head_row_index(
                    self.$parent(),
                )
            }

            fn automatic_merge_head(&self) -> bool {
                $crate::write::metadata::holder::write_holder::WriteHolder::automatic_merge_head(
                    self.$parent(),
                )
            }

            fn order_by_include_column(&self) -> bool {
                $crate::write::metadata::holder::write_holder::WriteHolder::order_by_include_column(
                    self.$parent(),
                )
            }

            fn include_column_indexes(&self) -> Option<&std::collections::HashSet<usize>> {
                $crate::write::metadata::holder::write_holder::WriteHolder::include_column_indexes(
                    self.$parent(),
                )
            }

            fn include_column_field_names(
                &self,
            ) -> Option<&std::collections::HashSet<String>> {
                $crate::write::metadata::holder::write_holder::WriteHolder::include_column_field_names(
                    self.$parent(),
                )
            }

            fn exclude_column_indexes(&self) -> Option<&std::collections::HashSet<usize>> {
                $crate::write::metadata::holder::write_holder::WriteHolder::exclude_column_indexes(
                    self.$parent(),
                )
            }

            fn exclude_column_field_names(
                &self,
            ) -> Option<&std::collections::HashSet<String>> {
                $crate::write::metadata::holder::write_holder::WriteHolder::exclude_column_field_names(
                    self.$parent(),
                )
            }
        }
    };
}

pub(crate) use delegate_write_holder_contract;
