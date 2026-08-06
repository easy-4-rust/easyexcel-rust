//! 对应 Java：`com.alibaba.excel.context.WriteContext` (interface).

use crate::ConverterRegistry;
use crate::ExcelWriteHeadProperty;
use crate::Holder;
use crate::WriteSheetContext;
use crate::WriteWorkbookContext;
use crate::core::excel_error::ExcelError;
use std::path::Path;
/// 对应 Java：`WriteContext` (110-line interface).
///
/// Java exposes a single `currentWriteHolder()` accessor plus the
/// `finish(boolean onException)` lifecycle. Rust collapses the
/// interface to a marker struct so dependents can take a `&WriteContext`
/// reference without depending on `rust_xlsxwriter` types.
pub trait WriteContext {
    /// Returns the active write holder. (Java `WriteContext.currentWriteHolder()`)
    fn current_write_holder(&self) -> &dyn WriteContextHolder;
}
include!("write_context/write_context_lifecycle.rs");
include!("write_context/write_context_holder.rs");
include!("write_context/write_context_holder_state.rs");

/// 对应 Java：com.alibaba.excel.context.WriteContext。 Executes Java `WriteContext.finish(boolean onException)` semantics.
///
/// This function performs real dynamic dispatch to a resource-owning context;
/// it is not available for metadata-only [`WriteContextImpl`] values.
///
/// # Errors
///
/// Returns the concrete writer's output, handler, finalization, or close error.
pub fn finish_write_context(
    context: &mut dyn WriteContextLifecycle,
    on_exception: bool,
) -> Result<(), ExcelError> {
    context.finish_context(on_exception)
}

pub use crate::context::write_context_impl::WriteContextImpl;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_context_impl_exposes_workbook_sheet_and_table_accessors() {
        let mut context = WriteContextImpl::new("out.xlsx");
        context.set_sheet_context("Sheet1");
        context.set_table_no(Some(2));

        let holder = context.current_write_holder();
        assert_eq!(holder.path(), Path::new("out.xlsx"));
        assert_eq!(
            holder.workbook_context().map(WriteWorkbookContext::path),
            Some(Path::new("out.xlsx"))
        );
        assert_eq!(
            holder.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Sheet1")
        );
        assert_eq!(holder.table_no(), Some(2));
        assert_eq!(holder.holder_type(), Holder::Table);
    }

    #[derive(Default)]
    struct LifecycleProbe {
        on_exception: Option<bool>,
    }

    impl WriteContext for LifecycleProbe {
        fn current_write_holder(&self) -> &dyn WriteContextHolder {
            self
        }
    }

    impl WriteContextHolder for LifecycleProbe {
        fn path(&self) -> &Path {
            Path::new("probe.xlsx")
        }

        fn holder_type(&self) -> Holder {
            Holder::Workbook
        }

        fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
            static PROPERTY: std::sync::OnceLock<ExcelWriteHeadProperty> =
                std::sync::OnceLock::new();
            PROPERTY.get_or_init(ExcelWriteHeadProperty::new)
        }

        fn converter_map(&self) -> &ConverterRegistry {
            static REGISTRY: std::sync::OnceLock<ConverterRegistry> = std::sync::OnceLock::new();
            REGISTRY.get_or_init(ConverterRegistry::default)
        }

        fn need_head(&self) -> bool {
            true
        }

        fn automatic_merge_head(&self) -> bool {
            true
        }

        fn relative_head_row_index(&self) -> i32 {
            0
        }

        fn order_by_include_column(&self) -> bool {
            false
        }

        fn include_column_indexes(&self) -> Option<&[usize]> {
            None
        }

        fn include_column_field_names(&self) -> Option<&[String]> {
            None
        }

        fn exclude_column_indexes(&self) -> &[usize] {
            &[]
        }

        fn exclude_column_field_names(&self) -> &[String] {
            &[]
        }
    }

    impl WriteContextLifecycle for LifecycleProbe {
        fn finish_context(&mut self, on_exception: bool) -> Result<(), ExcelError> {
            self.on_exception = Some(on_exception);
            Ok(())
        }
    }

    #[test]
    fn finish_write_context_dispatches_to_resource_owner() {
        let mut context = LifecycleProbe::default();
        finish_write_context(&mut context, true).expect("lifecycle should run");
        assert_eq!(context.on_exception, Some(true));
    }

    #[test]
    fn lifecycle_probe_exercises_every_holder_accessor() {
        // 对应 Java：WriteContextHolder 的默认实现与 probe 各访问器
        let probe = LifecycleProbe::default();
        let holder: &dyn WriteContextHolder = probe.current_write_holder();
        assert!(holder.workbook_context().is_none());
        assert!(holder.sheet_context().is_none());
        assert!(holder.table_no().is_none());
        assert!(holder.sheet_name().is_none());
        assert!(holder.sheet_no().is_none());
        assert!(holder.last_row_index().is_none());
        assert!(!holder.has_data());
        assert_eq!(holder.path(), Path::new("probe.xlsx"));
        assert_eq!(holder.holder_type(), Holder::Workbook);
        let _ = holder.excel_write_head_property();
        let _ = holder.converter_map();
        assert!(holder.need_head());
        assert!(holder.automatic_merge_head());
        assert_eq!(holder.relative_head_row_index(), 0);
        assert!(!holder.order_by_include_column());
        assert!(holder.include_column_indexes().is_none());
        assert!(holder.include_column_field_names().is_none());
        assert!(holder.exclude_column_indexes().is_empty());
        assert!(holder.exclude_column_field_names().is_empty());
        assert!(!probe.current_write_holder().has_data());
    }
}
#[cfg(test)]
mod tests_extra {
    use super::*;

    /// 对应 Java：仅实现必需方法的 holder，用于验证 trait 默认实现。
    #[derive(Default)]
    struct MinimalHolder;

    impl WriteContext for MinimalHolder {
        fn current_write_holder(&self) -> &dyn WriteContextHolder {
            self
        }
    }

    impl WriteContextHolder for MinimalHolder {
        fn path(&self) -> &Path {
            Path::new("minimal.xlsx")
        }

        fn holder_type(&self) -> Holder {
            Holder::Workbook
        }

        fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
            static PROPERTY: std::sync::OnceLock<ExcelWriteHeadProperty> =
                std::sync::OnceLock::new();
            PROPERTY.get_or_init(ExcelWriteHeadProperty::new)
        }

        fn converter_map(&self) -> &ConverterRegistry {
            static REGISTRY: std::sync::OnceLock<ConverterRegistry> = std::sync::OnceLock::new();
            REGISTRY.get_or_init(ConverterRegistry::default)
        }

        fn need_head(&self) -> bool {
            true
        }

        fn automatic_merge_head(&self) -> bool {
            true
        }

        fn relative_head_row_index(&self) -> i32 {
            0
        }

        fn order_by_include_column(&self) -> bool {
            false
        }

        fn include_column_indexes(&self) -> Option<&[usize]> {
            None
        }

        fn include_column_field_names(&self) -> Option<&[String]> {
            None
        }

        fn exclude_column_indexes(&self) -> &[usize] {
            &[]
        }

        fn exclude_column_field_names(&self) -> &[String] {
            &[]
        }
    }

    #[test]
    fn holder_default_trait_methods_return_none() {
        // 对应 Java：WriteContextHolder 的 workbook/sheet/table 默认均为空
        let probe = MinimalHolder;
        let holder: &dyn WriteContextHolder = probe.current_write_holder();
        assert!(holder.workbook_context().is_none());
        assert!(holder.sheet_context().is_none());
        assert!(holder.table_no().is_none());
        assert!(holder.sheet_name().is_none());
        assert!(holder.sheet_no().is_none());
        assert!(holder.last_row_index().is_none());
        assert!(!holder.has_data());
    }

    #[test]
    fn write_context_impl_inherent_accessors_and_holder_switch() {
        // 对应 Java：WriteContextImpl 固有访问器与 holder 切换
        let mut context = WriteContextImpl::new("out.xlsx");
        assert_eq!(context.path(), Path::new("out.xlsx"));
        assert_eq!(context.workbook_context().path(), Path::new("out.xlsx"));
        assert!(context.sheet_context().is_none());
        assert_eq!(context.table_no(), None);
        assert_eq!(context.current_holder_state().holder_type, Holder::Workbook);

        // 无 sheet 时 set_table_no(None) 回到 Workbook
        context.set_table_no(None);
        assert_eq!(context.current_holder_state().holder_type, Holder::Workbook);

        // 有 sheet 时 set_table_no(None) 保持 Sheet
        context.set_sheet_context("Sheet1");
        context.set_table_no(None);
        assert_eq!(context.current_holder_state().holder_type, Holder::Sheet);
        assert_eq!(
            context.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Sheet1")
        );

        // set_table_no(Some) 切换到 Table
        context.set_table_no(Some(3));
        assert_eq!(context.current_holder_state().holder_type, Holder::Table);
    }

    #[test]
    fn write_context_impl_holder_trait_remaining_methods() {
        // 对应 Java：WriteContextImpl 的 converterMap/include/exclude 访问器
        let context = WriteContextImpl::new("out.xlsx");
        let holder: &dyn WriteContextHolder = &context;
        let _ = holder.converter_map();
        assert!(holder.include_column_indexes().is_none());
        assert!(holder.include_column_field_names().is_none());
        assert!(holder.exclude_column_indexes().is_empty());
        assert!(holder.exclude_column_field_names().is_empty());
    }

    #[test]
    fn default_holder_state_fields_and_from_holder() {
        // 对应 Java：WriteContextHolderState 默认值及从 holder 拷贝
        let state = WriteContextHolderState::default();
        assert_eq!(state.holder_type, Holder::Workbook);
        assert!(state.need_head);
        assert!(state.automatic_merge_head);
        assert_eq!(state.relative_head_row_index, 0);
        assert!(!state.order_by_include_column);
        assert!(state.include_column_indexes.is_none());
        assert!(state.include_column_field_names.is_none());
        assert!(state.exclude_column_indexes.is_empty());
        assert!(state.exclude_column_field_names.is_empty());

        let probe = MinimalHolder;
        let copied = WriteContextHolderState::from_holder(&probe);
        assert_eq!(copied, state);
    }

    #[test]
    fn minimal_holder_path_returns_configured_path() {
        // 对应 Java：WriteContextHolder.path() 返回上下文输出路径
        let probe = MinimalHolder;
        assert_eq!(probe.path(), Path::new("minimal.xlsx"));
    }
}
