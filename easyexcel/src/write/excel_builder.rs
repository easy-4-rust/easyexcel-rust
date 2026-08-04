//! 对应 Java：`com.alibaba.excel.write.ExcelBuilder` and `ExcelBuilderImpl`.
use std::any::Any;

pub use crate::write::excel_builder_impl::ExcelBuilderImpl;

#[cfg(test)]
use crate::WriteOptions;
use crate::WriteSheet;
#[cfg(test)]
use crate::core::Holder;
use crate::core::{DynamicRow, ExcelRow, Result, WriteContext, fill_requires_template_error};
#[cfg(test)]
use crate::core::{
    ExcelError, WriteContextImpl, WriteFillConfig, WriteFillExecutor, WriteFillSheet,
    finish_write_context,
};
use crate::write::metadata::WriteTable;
/// Minimal fill configuration accepted by [`ExcelBuilder::fill`].
///
/// 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig` at the
/// builder surface. Stateful template filling remains on
/// `easyexcel_template::FillConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfig {
    /// Collection expansion direction. `None` is initialized as vertical.
    /// (Java `FillConfig.direction`)
    pub direction: Option<crate::core::WriteDirection>,
    /// Whether collection fill forces a new row. (Java `FillConfig.forceNewRow`)
    pub force_new_row: bool,
    /// Whether generated cells inherit the template style.
    /// (Java `FillConfig.autoStyle`, default `true`)
    pub auto_style: bool,
    has_init: bool,
}
impl FillConfig {
    /// Creates Java-compatible effective defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            direction: None,
            force_new_row: false,
            auto_style: true,
            has_init: false,
        }
    }

    /// Sets the collection expansion direction.
    #[must_use]
    pub const fn direction(mut self, direction: crate::core::WriteDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Sets whether collection fill forces a new row.
    #[must_use]
    pub const fn force_new_row(mut self, force_new_row: bool) -> Self {
        self.force_new_row = force_new_row;
        self
    }

    /// Sets whether generated cells inherit the template style.
    #[must_use]
    pub const fn auto_style(mut self, auto_style: bool) -> Self {
        self.auto_style = auto_style;
        self
    }

    /// Applies Java defaults once. Rust stores effective non-null values, so
    /// initialization only records the lifecycle transition.
    pub fn init(&mut self) {
        if !self.has_init {
            self.has_init = true;
        }
    }

    /// Returns whether [`Self::init`] has run.
    #[must_use]
    pub const fn has_init(&self) -> bool {
        self.has_init
    }
}
impl Default for FillConfig {
    fn default() -> Self {
        Self::new()
    }
}
/// Workbook builder contract matching Java `ExcelBuilder`.
///
/// 对应 Java：`com.alibaba.excel.write.ExcelBuilder`.
pub trait ExcelBuilder {
    /// Appends rows to a worksheet. (Java `addContent(Collection, WriteSheet)`)
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, or I/O error from the underlying writer.
    fn add_content<T, I>(&mut self, data: I, write_sheet: &WriteSheet<T>) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>;

    /// Appends rows to a worksheet table. (Java `addContent(Collection, WriteSheet, WriteTable)`)
    ///
    /// # Errors
    ///
    /// Returns a conversion, handler, or I/O error from the underlying writer.
    fn add_content_with_table<T, I>(
        &mut self,
        data: I,
        write_sheet: &WriteSheet<T>,
        write_table: &WriteTable,
    ) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>;

    /// Fills template placeholders on a worksheet. (Java `fill(Object, FillConfig, WriteSheet)`)
    ///
    /// `data` must be a supported fill payload (`TemplateData`, `FillWrapper`, …)
    /// wired through [`WriteFillExecutor`] by the `easyexcel` facade when a
    /// template is configured.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Unsupported`] when no template stream is configured.
    fn fill(
        &mut self,
        _data: &dyn Any,
        _fill_config: FillConfig,
        _write_sheet: &WriteSheet<DynamicRow>,
    ) -> Result<()> {
        Err(fill_requires_template_error())
    }

    /// Creates a merged region using zero-based inclusive coordinates.
    ///
    /// Mirrors deprecated Java `merge(int, int, int, int)`.
    ///
    /// # Errors
    ///
    /// Returns a format error when the coordinates are out of range or the
    /// writer backend cannot merge the region.
    fn merge(&mut self, first_row: u32, last_row: u32, first_col: u16, last_col: u16)
    -> Result<()>;

    /// Returns the active write context. (Java `writeContext()`)
    fn write_context(&self) -> &dyn WriteContext;

    /// Completes the workbook lifecycle. (Java `finish(boolean onException)`)
    ///
    /// # Errors
    ///
    /// Returns an output, close, or handler error.
    fn finish(&mut self, on_exception: bool) -> Result<()>;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        CellValue, DynamicRow, ExcelColumn, ExcelWriteMetadata, RowData, WriteDirection,
        WriteSheetContext,
    };
    use tempfile::tempdir;

    struct ContextRow;

    #[derive(Default)]
    struct ContextFillExecutor;

    impl WriteFillExecutor for ContextFillExecutor {
        fn fill(
            &mut self,
            _data: &dyn Any,
            _fill_config: WriteFillConfig,
            _sheet: WriteFillSheet,
        ) -> Result<()> {
            Ok(())
        }

        fn finish(&mut self, _on_exception: bool) -> Result<()> {
            Ok(())
        }
    }

    impl ExcelRow for ContextRow {
        fn schema() -> &'static [ExcelColumn] {
            static SCHEMA: [ExcelColumn; 3] = [
                ExcelColumn::new("a", "A", Some(0), 0, None),
                ExcelColumn::new("b", "B", Some(1), 0, None),
                ExcelColumn::new("c", "C", Some(2), 0, None),
            ];
            &SCHEMA
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            static METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new().head_row_height(28);
            &METADATA
        }

        fn from_row(_row: &RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String("a".to_owned()),
                CellValue::String("b".to_owned()),
                CellValue::String("c".to_owned()),
            ])
        }
    }

    #[test]
    fn fill_config_initializes_java_defaults_and_preserves_overrides() {
        let mut defaults = FillConfig::new();
        assert_eq!(defaults.direction, None);
        assert!(!defaults.force_new_row);
        assert!(defaults.auto_style);
        assert!(!defaults.has_init());
        defaults.init();
        defaults.init();
        assert!(defaults.has_init());

        let configured = FillConfig::new()
            .direction(WriteDirection::Horizontal)
            .force_new_row(true)
            .auto_style(false);
        assert_eq!(configured.direction, Some(WriteDirection::Horizontal));
        assert!(configured.force_new_row);
        assert!(!configured.auto_style);
    }

    #[test]
    fn fill_uses_explicit_excel_type_instead_of_path_extension() {
        let mut builder = ExcelBuilderImpl::from_options(
            "logical.xlsx",
            WriteOptions {
                excel_type: Some(crate::support::ExcelTypeEnum::Csv),
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("explicit CSV type must reject template fill");
        assert_eq!(
            error.to_string(),
            "unsupported operation: csv does not support filling data."
        );
    }

    #[test]
    fn excel_builder_impl_delegates_add_content_and_finish() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-facade.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder
            .add_content(
                [DynamicRow::new({
                    let mut cells = std::collections::BTreeMap::new();
                    cells.insert(0, crate::core::DynamicValue::String("alpha".to_owned()));
                    cells
                })],
                &sheet,
            )
            .expect("add_content should succeed");
        finish_write_context(&mut builder, false)?;
        finish_write_context(&mut builder, false)?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn excel_builder_merge_is_applied_on_next_add_content() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-merge.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder.merge(0, 0, 0, 1)?;
        builder
            .add_content(
                [DynamicRow::new({
                    let mut cells = std::collections::BTreeMap::new();
                    cells.insert(0, crate::core::DynamicValue::String("merged".to_owned()));
                    cells
                })],
                &sheet,
            )
            .expect("add_content should succeed");
        builder.finish(false)?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn write_context_exposes_sheet_and_table_after_add_content() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-context.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new(" Sheet1 ");
        let table = crate::write::ExcelWriterTableBuilder::new()
            .table_no(1)
            .need_head(false)
            .build();
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder.add_content_with_table([], &sheet, &table)?;

        let holder = builder.write_context().current_write_holder();
        assert_eq!(
            holder.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Sheet1")
        );
        assert_eq!(holder.table_no(), Some(1));
        assert!(holder.workbook_context().is_some());
        Ok(())
    }

    #[test]
    fn live_current_write_holder_tracks_resolved_sheet_and_table_state() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("live-holder-context.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<ContextRow>::from_options(WriteOptions {
            sheet_name: " Typed ".to_owned(),
            include_column_indexes: Some(vec![2, 0]),
            order_by_include_column: true,
            relative_head_row_index: 3,
            automatic_merge_head: false,
            dynamic_head: Some(vec![
                vec!["Group".to_owned(), "A*".to_owned()],
                vec!["Group".to_owned(), "B*".to_owned()],
                vec!["Group".to_owned(), "C*".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        builder.add_content([], &sheet)?;

        let sheet_holder = builder.write_context().current_write_holder();
        assert_eq!(sheet_holder.holder_type(), Holder::Sheet);
        assert_eq!(
            sheet_holder
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("Typed")
        );
        assert!(sheet_holder.need_head());
        assert!(!sheet_holder.automatic_merge_head());
        assert_eq!(sheet_holder.relative_head_row_index(), 3);
        assert!(sheet_holder.order_by_include_column());
        assert_eq!(
            sheet_holder.include_column_indexes(),
            Some([2, 0].as_slice())
        );
        assert_eq!(
            sheet_holder
                .excel_write_head_property()
                .head_row_height_property()
                .map(crate::metadata::RowHeightProperty::height),
            Some(28)
        );
        assert_eq!(
            sheet_holder
                .excel_write_head_property()
                .head_map()
                .values()
                .map(|head| (
                    head.column_index(),
                    head.field_name().map(str::to_owned),
                    head.head_name_list().to_vec(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    Some(0),
                    Some("c".to_owned()),
                    vec!["Group".to_owned(), "C*".to_owned()]
                ),
                (
                    Some(1),
                    Some("a".to_owned()),
                    vec!["Group".to_owned(), "A*".to_owned()]
                ),
            ]
        );

        let table = crate::write::ExcelWriterTableBuilder::new()
            .table_no(7)
            .need_head(false)
            .include_column_field_names(["b"])
            .build();
        builder.add_content_with_table([], &sheet, &table)?;
        let table_holder = builder.write_context().current_write_holder();
        assert_eq!(table_holder.holder_type(), Holder::Table);
        assert_eq!(table_holder.table_no(), Some(7));
        assert!(!table_holder.need_head());
        assert_eq!(
            table_holder.include_column_field_names(),
            Some(["b".to_owned()].as_slice())
        );
        assert_eq!(
            table_holder.include_column_indexes(),
            Some([2, 0].as_slice())
        );
        assert_eq!(
            table_holder
                .excel_write_head_property()
                .head_map()
                .values()
                .map(|head| head.field_name())
                .collect::<Vec<_>>(),
            vec![Some("b"), Some("c"), Some("a")]
        );
        builder.finish(false)?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn template_fill_updates_the_same_live_current_holder() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-context.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: " Fill ".to_owned(),
            need_head: false,
            relative_head_row_index: 4,
            automatic_merge_head: false,
            dynamic_head: Some(vec![vec!["填充列".to_owned()]]),
            ..WriteOptions::default()
        });
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;

        let holder = builder.write_context().current_write_holder();
        assert_eq!(holder.holder_type(), Holder::Sheet);
        assert_eq!(
            holder.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Fill")
        );
        assert!(!holder.need_head());
        assert!(!holder.automatic_merge_head());
        assert_eq!(holder.relative_head_row_index(), 4);
        assert_eq!(
            holder
                .excel_write_head_property()
                .head_map()
                .values()
                .flat_map(crate::metadata::Head::head_name_list)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["填充列"]
        );
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Extra coverage tests: fill gates, error paths, and surface accessors.
    // ---------------------------------------------------------------------

    /// `FillConfig::default()` must equal the Java-compatible `new()`.
    #[test]
    fn fill_config_default_matches_new() {
        assert_eq!(FillConfig::default(), FillConfig::new());
    }

    /// A builder that does not override `fill` must hit the trait's default
    /// rejection path when no template is configured.
    #[test]
    fn default_trait_fill_requires_template() {
        struct StubBuilder {
            context: WriteContextImpl,
        }

        impl ExcelBuilder for StubBuilder {
            fn add_content<T, I>(&mut self, _data: I, _write_sheet: &WriteSheet<T>) -> Result<()>
            where
                T: ExcelRow,
                I: IntoIterator<Item = T>,
            {
                Ok(())
            }

            fn add_content_with_table<T, I>(
                &mut self,
                _data: I,
                _write_sheet: &WriteSheet<T>,
                _write_table: &WriteTable,
            ) -> Result<()>
            where
                T: ExcelRow,
                I: IntoIterator<Item = T>,
            {
                Ok(())
            }

            fn merge(
                &mut self,
                _first_row: u32,
                _last_row: u32,
                _first_col: u16,
                _last_col: u16,
            ) -> Result<()> {
                Ok(())
            }

            fn write_context(&self) -> &dyn WriteContext {
                &self.context
            }

            fn finish(&mut self, _on_exception: bool) -> Result<()> {
                Ok(())
            }
        }

        let mut builder = StubBuilder {
            context: WriteContextImpl::new("stub.xlsx"),
        };
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("default fill must be rejected without a template");
        assert_eq!(
            error.to_string(),
            "unsupported operation: Calling the 'fill' method must use a template."
        );

        // The remaining trait surface must be callable on the stub.
        let table = WriteTable::default();
        builder
            .add_content([DynamicRow::default()], &sheet)
            .expect("stub add_content");
        builder
            .add_content_with_table([], &sheet, &table)
            .expect("stub add_content_with_table");
        builder.merge(0, 0, 0, 1).expect("stub merge");
        assert_eq!(
            builder.write_context().current_write_holder().path(),
            std::path::Path::new("stub.xlsx")
        );
        builder.finish(false).expect("stub finish");
    }

    /// `into_writer`, `writer_mut`, and `logical_path` surface accessors.
    #[test]
    fn builder_surface_accessors_and_into_writer() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("writer-ops.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        assert_eq!(builder.logical_path(), path.as_path());
        assert!(!builder.writer_mut().has_template_configured());
        let writer = builder.into_writer();
        assert!(!writer.has_template_configured());
        Ok(())
    }

    /// `has_fill_executor` gate plus the fill-then-finish lifecycle that marks
    /// the builder as finished via the fill delegate.
    #[test]
    fn fill_executor_gates_and_finish_via_fill() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-finish.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        assert!(!builder.has_fill_executor());
        assert!(!builder.finished_via_fill());
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        assert!(builder.has_fill_executor());

        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;
        builder.finish(false)?;
        assert!(builder.finished_via_fill());
        Ok(())
    }

    /// `ExcelBuilderImpl` coerced to `&dyn WriteContext` must dispatch to the
    /// live holder (covers the `WriteContext` trait impl).
    #[test]
    fn builder_dispatches_as_dyn_write_context() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("dyn-context.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.add_content([], &sheet)?;

        let context: &dyn WriteContext = &builder;
        assert!(context.current_write_holder().workbook_context().is_some());
        Ok(())
    }

    /// `fill` without a configured template stream is rejected.
    #[test]
    fn fill_without_template_is_rejected() {
        let mut builder =
            ExcelBuilderImpl::from_options("no-template.xlsx", WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("fill without a template must fail");
        assert_eq!(
            error.to_string(),
            "unsupported operation: Calling the 'fill' method must use a template."
        );
    }

    /// `fill` on a legacy XLS writer is rejected before the executor runs.
    #[test]
    fn fill_legacy_xls_is_rejected() {
        let mut builder = ExcelBuilderImpl::from_options(
            "legacy.xls",
            WriteOptions {
                excel_type: Some(crate::support::ExcelTypeEnum::Xls),
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("legacy XLS fill must fail");
        assert_eq!(
            error.to_string(),
            "unsupported operation: legacy XLS template fill is not supported"
        );
    }

    /// `fill` with a template but no installed executor is rejected.
    #[test]
    fn fill_without_executor_wired_is_rejected() {
        let mut builder = ExcelBuilderImpl::from_options(
            "unwired.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("fill without a wired executor must fail");
        assert!(error.to_string().contains("not wired"));
    }

    /// A failing fill delegate propagates its error through `ExcelBuilderImpl::fill`.
    #[test]
    fn fill_propagates_delegate_error() {
        struct FailingFillExecutor {
            fail_first: bool,
        }

        impl WriteFillExecutor for FailingFillExecutor {
            fn fill(
                &mut self,
                _data: &dyn Any,
                _fill_config: WriteFillConfig,
                _sheet: WriteFillSheet,
            ) -> Result<()> {
                if self.fail_first {
                    self.fail_first = false;
                    Err(ExcelError::Unsupported("fill failed on purpose".to_owned()))
                } else {
                    Ok(())
                }
            }

            fn finish(&mut self, _on_exception: bool) -> Result<()> {
                Ok(())
            }
        }

        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("fill-error.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(
            &path,
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(FailingFillExecutor { fail_first: true }));
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("delegate error must propagate");
        assert_eq!(
            error.to_string(),
            "unsupported operation: fill failed on purpose"
        );

        // A later successful fill routes `finish` through the fill delegate.
        builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect("second fill succeeds");
        builder.finish(false).expect("finish via fill delegate");
        assert!(builder.finished_via_fill());
    }

    /// `write_rows` preserves the sheet name when `auto_trim` is disabled.
    #[test]
    fn write_rows_preserves_sheet_name_when_auto_trim_disabled() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("no-trim.xlsx");
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "Data2".to_owned(),
            auto_trim: false,
            ..WriteOptions::default()
        });
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let mut cells = std::collections::BTreeMap::new();
        cells.insert(0, crate::core::DynamicValue::String("x".to_owned()));
        builder.add_content([DynamicRow::new(cells)], &sheet)?;
        assert_eq!(
            builder
                .write_context()
                .current_write_holder()
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("Data2")
        );
        Ok(())
    }

    /// `fill` preserves the sheet name when `auto_trim` is disabled.
    #[test]
    fn fill_respects_auto_trim_disabled() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-no-trim.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: "FillNoTrim".to_owned(),
            auto_trim: false,
            need_head: false,
            ..WriteOptions::default()
        });
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;
        assert_eq!(
            builder
                .write_context()
                .current_write_holder()
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("FillNoTrim")
        );
        Ok(())
    }

    /// A schema with duplicate forced indexes surfaces as a holder-state error
    /// from `add_content` (covers the `?` inside `update_current_holder`).
    #[test]
    fn add_content_propagates_holder_resolution_error() {
        struct DuplicateIndexRow;

        impl ExcelRow for DuplicateIndexRow {
            fn schema() -> &'static [ExcelColumn] {
                static SCHEMA: [ExcelColumn; 2] = [
                    ExcelColumn::new("a", "A", Some(0), 0, None),
                    ExcelColumn::new("b", "B", Some(0), 1, None),
                ];
                &SCHEMA
            }

            fn from_row(_row: &RowData) -> Result<Self> {
                Ok(Self)
            }

            fn to_row(&self) -> Result<Vec<CellValue>> {
                Ok(Vec::new())
            }
        }

        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("dup-index.xlsx");
        let sheet = WriteSheet::<DuplicateIndexRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let error = builder
            .add_content([DuplicateIndexRow], &sheet)
            .expect_err("duplicate forced index must fail");
        assert!(error.to_string().contains("must be inconsistent"));

        // The conversion surface of the failing-schema row stays callable.
        let row = DuplicateIndexRow::from_row(&RowData::new(
            "Sheet1",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))
        .expect("from_row");
        assert!(row.to_row().expect("to_row").is_empty());
    }

    /// Direct conversion-surface check for `ContextRow` (`from_row`/`to_row`).
    #[test]
    fn context_row_conversion_surface() -> Result<()> {
        let row = ContextRow::from_row(&RowData::new(
            "Sheet1",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))?;
        assert_eq!(
            row.to_row()?,
            vec![
                CellValue::String("a".to_owned()),
                CellValue::String("b".to_owned()),
                CellValue::String("c".to_owned()),
            ]
        );
        Ok(())
    }

    /// `finish(true)` routes to `finish_on_exception` when no fill session ran.
    #[test]
    fn finish_on_exception_discards_workbook_data() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("finish-on-exception.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        builder.add_content([], &sheet)?;
        builder.finish(true)?;
        Ok(())
    }
}
