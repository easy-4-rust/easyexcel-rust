//! 对应 Java：`com.alibaba.excel.write.builder.ExcelWriterTableBuilder`.

use crate::CellStyle;
use crate::WriteHandler;
use crate::WriteOptions;

use crate::write::builder::abstract_excel_writer_parameter_builder::AbstractExcelWriterParameterBuilder;
use crate::write::holder::write_table_holder::WriteTableHolder;
use crate::write::metadata::write_sheet::WriteSheet as WriteSheetMetadata;
use crate::write::metadata::{WriteBasicParameter, WriteTable};
use crate::{ExcelWriter, WriteSheet};

/// 对应 Java：`ExcelWriterTableBuilder extends AbstractExcelWriterParameterBuilder`.
///
/// Java carries a `WriteTable` and a back-reference to the parent
/// `WriteSheet`; Rust mirrors the data on the parameter struct and
/// exposes the same builder surface.
pub struct ExcelWriterTableBuilder {
    parameter: WriteBasicParameter,
    table: WriteTable,
    own_handlers: Vec<Box<dyn WriteHandler>>,
    parent_handlers: Vec<Box<dyn WriteHandler>>,
    excel_writer: Option<ExcelWriter>,
    write_sheet: Option<WriteSheetMetadata>,
}

impl ExcelWriterTableBuilder {
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Creates a table builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameter: WriteBasicParameter::default(),
            table: WriteTable::new(),
            own_handlers: Vec::new(),
            parent_handlers: Vec::new(),
            excel_writer: None,
            write_sheet: None,
        }
    }

    /// Creates a table builder bound to its parent writer and sheet.
    ///
    /// 对应 Java：`ExcelWriterTableBuilder(ExcelWriter, WriteSheet)`.
    #[must_use]
    pub fn with_excel_writer(
        excel_writer: ExcelWriter,
        write_sheet: WriteSheetMetadata,
        parent_handlers: Vec<Box<dyn WriteHandler>>,
    ) -> Self {
        Self {
            parameter: WriteBasicParameter::default(),
            table: WriteTable::new(),
            own_handlers: Vec::new(),
            parent_handlers,
            excel_writer: Some(excel_writer),
            write_sheet: Some(write_sheet),
        }
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Sets the zero-based table index. (Java `tableNo(Integer)`)
    #[must_use]
    pub fn table_no(mut self, table_no: i32) -> Self {
        self.table.table_no = table_no;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Sets whether a header row is written. (Java `needHead(Boolean)`)
    #[must_use]
    pub fn need_head(mut self, need_head: bool) -> Self {
        self.parameter.need_head = Some(need_head);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Sets the relative head row index. (Java `relativeHeadRowIndex(Integer)`)
    #[must_use]
    pub fn relative_head_row_index(mut self, index: i32) -> Self {
        self.parameter.relative_head_row_index = Some(index);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Sets automatic header merging. (Java `automaticMergeHead(Boolean)`)
    #[must_use]
    pub fn automatic_merge_head(mut self, automatic_merge_head: bool) -> Self {
        self.parameter.automatic_merge_head = Some(automatic_merge_head);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Sets the include-order flag. (Java `orderByIncludeColumn(Boolean)`)
    #[must_use]
    pub fn order_by_include_column(mut self, enabled: bool) -> Self {
        self.parameter.order_by_include_column = Some(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Controls whether Java's default header style is enabled.
    #[must_use]
    pub fn use_default_style(mut self, enabled: bool) -> Self {
        self.parameter.use_default_style = Some(enabled);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Replaces inherited included physical columns.
    #[must_use]
    pub fn include_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.parameter.include_column_indexes = Some(indexes.into_iter().collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Replaces inherited included field names.
    #[must_use]
    pub fn include_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.parameter.include_column_field_names =
            Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Replaces inherited excluded physical columns.
    #[must_use]
    pub fn exclude_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        self.parameter.exclude_column_indexes = Some(indexes.into_iter().collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Replaces inherited excluded field names.
    #[must_use]
    pub fn exclude_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        self.parameter.exclude_column_field_names =
            Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Registers a write handler. (Java `registerWriteHandler(WriteHandler)`)
    #[must_use]
    pub fn register_write_handler(mut self, handler: Box<dyn WriteHandler>) -> Self {
        self.own_handlers.push(handler);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Applies a head style to the built table options.
    #[must_use]
    pub fn head_style(mut self, style: CellStyle) -> Self {
        self.table.options.head_style = style;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Builds the `WriteTable` value. (Java `build()`)
    #[must_use]
    pub fn build(&self) -> WriteTable {
        let mut table = self.table.clone();
        table.parameter = self.parameter.clone();
        apply_explicit_parameter(&mut table.options, &self.parameter);
        table.options.converters = self.parameter.converters.clone();
        table
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Builds a table holder for handler contexts. (Java `WriteTableHolder`)
    #[must_use]
    pub fn build_table_holder<'a>(&self, parent_sheet: &'a str) -> WriteTableHolder<'a> {
        let mut holder = WriteTableHolder::new(self.table.table_no);
        holder.set_parent_sheet(parent_sheet);
        holder
    }

    /// Returns a reference to the inner `WriteTable` for inspection.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。
    pub const fn table(&self) -> &WriteTable {
        &self.table
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Returns the number of currently registered handlers. Useful for tests.
    #[must_use]
    pub fn handler_count(&self) -> usize {
        self.own_handlers.len() + self.parent_handlers.len()
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Returns the registered handler list. (Java `getCustomWriteHandlerList()`)
    #[must_use]
    pub fn handlers(&self) -> &[Box<dyn WriteHandler>] {
        &self.own_handlers
    }

    /// Writes the supplied rows through the parent sheet/table and finishes.
    ///
    /// 对应 Java：`ExcelWriterTableBuilder.doWrite(Collection)`.
    ///
    /// # Errors
    ///
    /// Returns a format error when the builder was already consumed or the
    /// parent write sheet is missing, and propagates writer errors from
    /// [`crate::ExcelWriter::write_with_table_handlers`] / `finish`.
    pub fn do_write<T, I>(mut self, rows: I) -> crate::core::Result<()>
    where
        T: crate::core::ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let mut writer = self.excel_writer.take().ok_or_else(|| {
            crate::core::ExcelError::Format(
                "Must use ExcelWriterBuilder.sheet().table() to call do_write()".to_owned(),
            )
        })?;
        let write_sheet = self.write_sheet.take().ok_or_else(|| {
            crate::core::ExcelError::Format(
                "table builder is missing its parent write sheet".to_owned(),
            )
        })?;
        let table = self.build();
        let typed_sheet = WriteSheet::<T>::from_options(write_sheet.options);
        writer.write_with_table_handlers(
            rows,
            &typed_sheet,
            &table,
            self.parent_handlers,
            self.own_handlers,
        )?;
        writer.finish()
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Resolves rows lazily, then delegates to [`Self::do_write`].
    ///
    /// # Errors
    ///
    /// See [`Self::do_write`].
    pub fn do_write_with<T, I, F>(self, supplier: F) -> crate::core::Result<()>
    where
        T: crate::core::ExcelRow,
        I: IntoIterator<Item = T>,
        F: FnOnce() -> I,
    {
        self.do_write(supplier())
    }

    /// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Mutably records a head style for callers using Java-style setters.
    pub fn head_style_record(&mut self, style: CellStyle) -> &mut Self {
        self.table.options.head_style = style;
        self
    }
}

impl Default for ExcelWriterTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractExcelWriterParameterBuilder for ExcelWriterTableBuilder {
    fn parameter(&mut self) -> &mut WriteBasicParameter {
        &mut self.parameter
    }

    fn register_write_handler(&mut self, handler: Box<dyn WriteHandler>) -> &mut Self {
        self.own_handlers.push(handler);
        self
    }
}

/// 对应 Java：com.alibaba.excel.write.builder.ExcelWriterTableBuilder。 Merges [`WriteTable`] overrides into worksheet [`WriteOptions`].
///
/// Table-level head settings override the parent sheet options, matching Java
/// `WriteTableHolder` inheritance from `WriteSheetHolder`.
#[must_use]
pub fn merge_table_options(sheet_options: &WriteOptions, table: &WriteTable) -> WriteOptions {
    let mut merged = sheet_options.clone();
    let defaults = WriteOptions::default();
    let parameter = &table.parameter;

    if let Some(value) = parameter.relative_head_row_index {
        merged.relative_head_row_index = value;
    } else if table.options.relative_head_row_index != defaults.relative_head_row_index {
        merged.relative_head_row_index = table.options.relative_head_row_index;
    }
    if let Some(value) = parameter.need_head {
        merged.need_head = value;
    } else if table.options.need_head != defaults.need_head {
        merged.need_head = table.options.need_head;
    }
    if let Some(value) = parameter.automatic_merge_head {
        merged.automatic_merge_head = value;
    } else if table.options.automatic_merge_head != defaults.automatic_merge_head {
        merged.automatic_merge_head = table.options.automatic_merge_head;
    }
    if let Some(value) = parameter.order_by_include_column {
        merged.order_by_include_column = value;
    } else if table.options.order_by_include_column != defaults.order_by_include_column {
        merged.order_by_include_column = table.options.order_by_include_column;
    }
    if let Some(indexes) = &parameter.include_column_indexes {
        merged.include_column_indexes = Some(indexes.clone());
    } else if table.options.include_column_indexes != defaults.include_column_indexes {
        merged
            .include_column_indexes
            .clone_from(&table.options.include_column_indexes);
    }
    if let Some(names) = &parameter.include_column_field_names {
        merged.include_column_field_names = Some(names.clone());
    } else if table.options.include_column_field_names != defaults.include_column_field_names {
        merged
            .include_column_field_names
            .clone_from(&table.options.include_column_field_names);
    }
    if let Some(indexes) = &parameter.exclude_column_indexes {
        merged.exclude_column_indexes.clone_from(indexes);
    } else if table.options.exclude_column_indexes != defaults.exclude_column_indexes {
        merged
            .exclude_column_indexes
            .clone_from(&table.options.exclude_column_indexes);
    }
    if let Some(names) = &parameter.exclude_column_field_names {
        merged.exclude_column_field_names.clone_from(names);
    } else if table.options.exclude_column_field_names != defaults.exclude_column_field_names {
        merged
            .exclude_column_field_names
            .clone_from(&table.options.exclude_column_field_names);
    }
    if let Some(use_default_style) = parameter.use_default_style {
        merged.use_default_style = use_default_style;
        merged.head_style = if use_default_style {
            crate::CellStyle::new().bold(true)
        } else {
            crate::CellStyle::new()
        };
    } else if table.options.head_style != defaults.head_style {
        merged.head_style.clone_from(&table.options.head_style);
    }
    if !table.options.converters.is_empty() {
        merged.converters = merged.converters.merged_with(&table.options.converters);
    }
    merged
}

fn apply_explicit_parameter(options: &mut WriteOptions, parameter: &WriteBasicParameter) {
    if let Some(value) = parameter.relative_head_row_index {
        options.relative_head_row_index = value;
    }
    if let Some(value) = parameter.need_head {
        options.need_head = value;
    }
    if let Some(value) = parameter.automatic_merge_head {
        options.automatic_merge_head = value;
    }
    if let Some(value) = parameter.order_by_include_column {
        options.order_by_include_column = value;
    }
    if let Some(indexes) = &parameter.include_column_indexes {
        options.include_column_indexes = Some(indexes.clone());
    }
    if let Some(names) = &parameter.include_column_field_names {
        options.include_column_field_names = Some(names.clone());
    }
    if let Some(indexes) = &parameter.exclude_column_indexes {
        options.exclude_column_indexes.clone_from(indexes);
    }
    if let Some(names) = &parameter.exclude_column_field_names {
        options.exclude_column_field_names.clone_from(names);
    }
    if let Some(use_default_style) = parameter.use_default_style {
        options.use_default_style = use_default_style;
        options.head_style = if use_default_style {
            crate::CellStyle::new().bold(true)
        } else {
            crate::CellStyle::new()
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{CellValue, ExcelColumn, ExcelRow, RowData};
    use calamine::{DataType, Reader, Xlsx, open_workbook};
    use tempfile::tempdir;

    use super::*;

    /// 打开输出文件并统一包装 calamine 错误。
    ///
    /// 损坏/缺失文件时 `open_workbook` 失败走 `map_err` 分支；正常文件走 Ok 分支。
    /// 所有测试共用这一个闭包体，保证两条路径都被覆盖。
    fn open_output_workbook(
        path: &std::path::Path,
    ) -> crate::core::Result<Xlsx<std::io::BufReader<std::fs::File>>> {
        open_workbook(path).map_err(|error: calamine::XlsxError| {
            crate::core::ExcelError::Format(error.to_string())
        })
    }

    struct TableRow(&'static str);

    impl ExcelRow for TableRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &RowData) -> crate::core::Result<Self> {
            Ok(Self(""))
        }

        fn to_row(&self) -> crate::core::Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(self.0.to_owned())])
        }
    }

    #[test]
    fn table_builder_builds_write_table_and_holder() {
        let table = ExcelWriterTableBuilder::new()
            .table_no(3)
            .need_head(false)
            .relative_head_row_index(2)
            .build();
        assert_eq!(table.table_no(), 3);
        assert!(!table.options().need_head);
        assert_eq!(table.options().relative_head_row_index, 2);

        let holder = ExcelWriterTableBuilder::new()
            .table_no(3)
            .build_table_holder("Sheet1");
        assert_eq!(holder.table_no(), 3);
        assert_eq!(holder.parent_sheet(), Some("Sheet1"));
    }

    #[test]
    fn merge_table_options_overrides_head_settings() {
        let sheet = WriteOptions::default();
        let table = WriteTable {
            options: WriteOptions {
                need_head: false,
                relative_head_row_index: 4,
                ..WriteOptions::default()
            },
            ..WriteTable::new()
        };
        let merged = merge_table_options(&sheet, &table);
        assert!(!merged.need_head);
        assert_eq!(merged.relative_head_row_index, 4);
        assert_eq!(merged.sheet_name, sheet.sheet_name);
    }

    #[test]
    fn unset_table_parameter_inherits_every_nullable_parent_value() {
        let sheet = WriteOptions {
            need_head: false,
            relative_head_row_index: 3,
            automatic_merge_head: false,
            order_by_include_column: true,
            include_column_indexes: Some(vec![1]),
            include_column_field_names: Some(vec!["second".to_owned()]),
            exclude_column_indexes: vec![0],
            exclude_column_field_names: vec!["first".to_owned()],
            head_style: CellStyle::new(),
            ..WriteOptions::default()
        };

        let table = ExcelWriterTableBuilder::new().build();
        let merged = merge_table_options(&sheet, &table);

        assert!(!merged.need_head);
        assert_eq!(merged.relative_head_row_index, 3);
        assert!(!merged.automatic_merge_head);
        assert!(merged.order_by_include_column);
        assert_eq!(merged.include_column_indexes, Some(vec![1]));
        assert_eq!(
            merged.include_column_field_names,
            Some(vec!["second".to_owned()])
        );
        assert_eq!(merged.exclude_column_indexes, vec![0]);
        assert_eq!(merged.exclude_column_field_names, vec!["first".to_owned()]);
        assert_eq!(merged.head_style, CellStyle::new());
    }

    #[test]
    fn explicit_table_defaults_override_non_default_parent_values() {
        let sheet = WriteOptions {
            need_head: false,
            relative_head_row_index: 3,
            automatic_merge_head: false,
            order_by_include_column: true,
            include_column_indexes: Some(vec![1]),
            exclude_column_indexes: vec![1],
            head_style: CellStyle::new(),
            ..WriteOptions::default()
        };

        let table = ExcelWriterTableBuilder::new()
            .need_head(true)
            .relative_head_row_index(0)
            .automatic_merge_head(true)
            .order_by_include_column(false)
            .include_column_indexes([0])
            .exclude_column_indexes(Vec::<usize>::new())
            .use_default_style(true)
            .build();
        let merged = merge_table_options(&sheet, &table);

        assert!(merged.need_head);
        assert_eq!(merged.relative_head_row_index, 0);
        assert!(merged.automatic_merge_head);
        assert!(!merged.order_by_include_column);
        assert_eq!(merged.include_column_indexes, Some(vec![0]));
        assert!(merged.exclude_column_indexes.is_empty());
        assert_eq!(merged.head_style, CellStyle::new().bold(true));
    }

    #[test]
    fn bound_table_builder_writes_through_parent_sheet_and_finishes() -> crate::core::Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("table.xlsx");
        let writer = ExcelWriter::new(&output);
        let mut sheet = WriteSheetMetadata::new();
        sheet.set_sheet_name("Users");
        sheet.options.sheet_name = "Users".to_owned();

        ExcelWriterTableBuilder::with_excel_writer(writer, sheet, Vec::new())
            .table_no(2)
            .need_head(false)
            .do_write(vec![TableRow("alice")])?;

        #[rustfmt::skip]
        let mut workbook = open_output_workbook(&output)?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| crate::core::ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

    struct NoopHandler;

    impl WriteHandler for NoopHandler {}

    #[test]
    fn table_builder_extra_setters_and_accessors() {
        let builder = ExcelWriterTableBuilder::new()
            .exclude_column_field_names(["skip"])
            .head_style(CellStyle::new().bold(true));
        assert_eq!(
            builder.table().options.head_style,
            CellStyle::new().bold(true)
        );
        assert_eq!(builder.handler_count(), 0);
        assert!(builder.handlers().is_empty());

        let mut builder = builder.register_write_handler(Box::new(NoopHandler));
        assert_eq!(builder.handler_count(), 1);
        assert_eq!(builder.handlers().len(), 1);
        builder.head_style_record(CellStyle::new());
        builder.parameter().order_by_include_column = Some(true);
        AbstractExcelWriterParameterBuilder::register_write_handler(
            &mut builder,
            Box::new(NoopHandler),
        );
        assert_eq!(builder.handler_count(), 2);
        let built = builder.build();
        assert!(built.options().order_by_include_column);
        assert_eq!(built.options().head_style, CellStyle::new());
        assert_eq!(
            built.options().exclude_column_field_names,
            vec!["skip".to_owned()]
        );
    }

    #[test]
    fn table_builder_default_matches_new() {
        assert_eq!(ExcelWriterTableBuilder::default().table().table_no(), 0);
        assert_eq!(ExcelWriterTableBuilder::new().table().table_no(), 0);
        assert_eq!(ExcelWriterTableBuilder::default().handler_count(), 0);
    }

    #[test]
    fn unbound_table_builder_do_write_returns_format_error() {
        let error = ExcelWriterTableBuilder::new()
            .do_write(vec![TableRow("alice")])
            .expect_err("unbound table builder must fail");
        assert!(matches!(error, crate::core::ExcelError::Format(_)));
    }

    #[test]
    fn missing_parent_sheet_do_write_returns_format_error() {
        let directory = tempdir().expect("tempdir must succeed");
        let output = directory.path().join("missing-sheet.xlsx");
        let builder = ExcelWriterTableBuilder {
            parameter: WriteBasicParameter::default(),
            table: WriteTable::new(),
            own_handlers: Vec::new(),
            parent_handlers: Vec::new(),
            excel_writer: Some(ExcelWriter::new(&output)),
            write_sheet: None,
        };
        let error = builder
            .do_write(vec![TableRow("alice")])
            .expect_err("missing parent sheet must fail");
        assert!(matches!(error, crate::core::ExcelError::Format(_)));
    }

    #[test]
    fn finished_parent_writer_do_write_propagates_write_error() {
        let directory = tempdir().expect("tempdir must succeed");
        let output = directory.path().join("finished-writer.xlsx");
        let mut writer = ExcelWriter::new(&output);
        writer
            .finish()
            .expect("finishing an unused writer must succeed");
        let mut sheet = WriteSheetMetadata::new();
        sheet.set_sheet_name("Users");
        sheet.options.sheet_name = "Users".to_owned();

        let builder = ExcelWriterTableBuilder {
            parameter: WriteBasicParameter::default(),
            table: WriteTable::new(),
            own_handlers: Vec::new(),
            parent_handlers: Vec::new(),
            excel_writer: Some(writer),
            write_sheet: Some(sheet),
        };
        let error = builder
            .do_write(vec![TableRow("alice")])
            .expect_err("writing through a finished writer must fail");
        assert!(matches!(error, crate::core::ExcelError::Unsupported(_)));
    }

    #[test]
    fn merge_table_options_applies_parameter_field_names_and_plain_style() {
        let sheet = WriteOptions::default();
        let mut table = WriteTable::new();
        table.parameter.exclude_column_field_names = Some(vec!["first".to_owned()]);
        table.parameter.use_default_style = Some(false);
        let merged = merge_table_options(&sheet, &table);
        assert_eq!(merged.exclude_column_field_names, vec!["first".to_owned()]);
        assert!(!merged.use_default_style);
        assert_eq!(merged.head_style, CellStyle::new());
    }

    #[test]
    fn bound_table_builder_do_write_with_resolves_lazily() -> crate::core::Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("table-with.xlsx");
        let writer = ExcelWriter::new(&output);
        let mut sheet = WriteSheetMetadata::new();
        sheet.set_sheet_name("Users");
        sheet.options.sheet_name = "Users".to_owned();

        ExcelWriterTableBuilder::with_excel_writer(writer, sheet, Vec::new())
            .need_head(false)
            .do_write_with(|| vec![TableRow("bob")])?;

        let mut workbook = open_output_workbook(&output)?;
        let range = workbook
            .worksheet_range("Users")
            .map_err(|error| crate::core::ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("bob")
        );
        Ok(())
    }

    /// 输出文件损坏时 `open_workbook` 必须报错（`map_err` 格式化分支）。
    ///
    /// 对应 Java：写出的 xlsx 无法被重新解析时（文件被截断/覆盖），
    /// 读取侧打开失败并按 `ExcelError::Format` 包装。
    #[test]
    fn bound_table_builder_corrupt_output_errors_on_open() {
        let directory = tempdir().expect("tempdir");
        let output = directory.path().join("corrupt.xlsx");
        std::fs::write(&output, b"not an xlsx package").expect("write corrupt bytes");
        let result = open_output_workbook(&output);
        assert!(result.is_err());
    }

    #[test]
    fn merge_table_options_falls_back_to_explicit_table_options() {
        let sheet = WriteOptions::default();
        let mut table = WriteTable::new();
        table.options.relative_head_row_index = 4;
        table.options.need_head = false;
        table.options.automatic_merge_head = false;
        table.options.order_by_include_column = true;
        table.options.include_column_indexes = Some(vec![1]);
        table.options.include_column_field_names = Some(vec!["second".to_owned()]);
        table.options.exclude_column_indexes = vec![0];
        table.options.exclude_column_field_names = vec!["first".to_owned()];
        table.options.head_style = CellStyle::new();

        let merged = merge_table_options(&sheet, &table);
        assert_eq!(merged.relative_head_row_index, 4);
        assert!(!merged.need_head);
        assert!(!merged.automatic_merge_head);
        assert!(merged.order_by_include_column);
        assert_eq!(merged.include_column_indexes, Some(vec![1]));
        assert_eq!(
            merged.include_column_field_names,
            Some(vec!["second".to_owned()])
        );
        assert_eq!(merged.exclude_column_indexes, vec![0]);
        assert_eq!(merged.exclude_column_field_names, vec!["first".to_owned()]);
        assert_eq!(merged.head_style, CellStyle::new());
    }

    #[test]
    fn build_applies_explicit_field_name_and_style_overrides() {
        let table = ExcelWriterTableBuilder::new()
            .exclude_column_field_names(["skip"])
            .use_default_style(false)
            .build();
        assert_eq!(
            table.options().exclude_column_field_names,
            vec!["skip".to_owned()]
        );
        assert!(!table.options().use_default_style);
        assert_eq!(table.options().head_style, CellStyle::new());
    }

    #[test]
    fn table_row_helper_supports_from_row() -> crate::core::Result<()> {
        let headers = std::sync::Arc::new(std::collections::HashMap::<String, usize>::new());
        let row = RowData::new(
            "Users",
            0,
            vec![CellValue::String("alice".to_owned())],
            headers,
        );
        assert_eq!(TableRow::from_row(&row)?.0, "");
        Ok(())
    }
}
