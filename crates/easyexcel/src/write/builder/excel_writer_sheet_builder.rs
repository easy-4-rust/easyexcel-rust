//! Rust implementation of Java
//! `com.alibaba.excel.write.builder.ExcelWriterSheetBuilder`.

use crate::core::{ExcelError, ExcelRow, Result, WriteHandler};

use crate::write::builder::excel_writer_table_builder::ExcelWriterTableBuilder;
use crate::write::metadata::write_sheet::WriteSheet as WriteSheetMetadata;
use crate::{ExcelWriter, WriteOptions, WriteSheet};

/// 对应 Java：`ExcelWriterSheetBuilder.table()`。 A sheet builder optionally owning the writer that will execute it.
pub struct ExcelWriterSheetBuilder {
    excel_writer: Option<ExcelWriter>,
    write_sheet: WriteSheetMetadata,
    handlers: Vec<Box<dyn WriteHandler>>,
}

impl ExcelWriterSheetBuilder {
    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Creates an unbound metadata builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            excel_writer: None,
            write_sheet: WriteSheetMetadata::new(),
            handlers: Vec::new(),
        }
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Creates a sheet builder owning its stateful writer.
    #[must_use]
    pub fn with_excel_writer(excel_writer: ExcelWriter) -> Self {
        Self {
            excel_writer: Some(excel_writer),
            write_sheet: WriteSheetMetadata::new(),
            handlers: Vec::new(),
        }
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Creates a sheet builder with effective workbook options already
    /// inherited, while retaining nullable sheet-level overrides separately.
    #[must_use]
    pub fn with_excel_writer_and_options(
        excel_writer: ExcelWriter,
        inherited_options: WriteOptions,
    ) -> Self {
        let mut write_sheet = WriteSheetMetadata::new();
        write_sheet.options = inherited_options;
        // 语义敏感：sheet_index 是 Java `Integer` 语义（i32），值域受工作表
        // 数量约束，不可能超出 i32 范围；保留 as 转换。
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let sheet_no = write_sheet.options.sheet_index.unwrap_or(0) as i32;
        write_sheet.sheet_no = sheet_no;
        write_sheet
            .sheet_name
            .clone_from(&write_sheet.options.sheet_name);
        Self {
            excel_writer: Some(excel_writer),
            write_sheet,
            handlers: Vec::new(),
        }
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Sets the zero-based sheet number.
    #[must_use]
    pub fn sheet_no(mut self, sheet_no: i32) -> Self {
        self.write_sheet.set_sheet_no(sheet_no);
        // 语义敏感：`sheet_no.max(0)` 保证非负，i32->usize 不会丢符号。
        #[allow(clippy::cast_sign_loss)]
        let sheet_index = sheet_no.max(0) as usize;
        self.write_sheet.options.sheet_index = Some(sheet_index);
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Sets the sheet name.
    #[must_use]
    pub fn sheet_name(mut self, sheet_name: impl Into<String>) -> Self {
        let sheet_name = sheet_name.into();
        self.write_sheet.set_sheet_name(sheet_name.clone());
        self.write_sheet.options.sheet_name = sheet_name;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Sets the number of rows before the header.
    #[must_use]
    pub fn relative_head_row_index(mut self, index: i32) -> Self {
        self.write_sheet.parameter.relative_head_row_index = Some(index);
        self.write_sheet.options.relative_head_row_index = index;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Controls header output.
    #[must_use]
    pub fn need_head(mut self, enabled: bool) -> Self {
        self.write_sheet.parameter.need_head = Some(enabled);
        self.write_sheet.options.need_head = enabled;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Enables or disables Java's default bold header style.
    #[must_use]
    pub fn use_default_style(mut self, enabled: bool) -> Self {
        self.write_sheet.parameter.use_default_style = Some(enabled);
        self.write_sheet.options.use_default_style = enabled;
        self.write_sheet.options.head_style = if enabled {
            crate::CellStyle::new().bold(true)
        } else {
            crate::CellStyle::new()
        };
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Controls automatic multi-level header merging.
    #[must_use]
    pub fn automatic_merge_head(mut self, enabled: bool) -> Self {
        self.write_sheet.parameter.automatic_merge_head = Some(enabled);
        self.write_sheet.options.automatic_merge_head = enabled;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Includes only the supplied physical columns.
    #[must_use]
    pub fn include_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        let indexes = indexes.into_iter().collect::<Vec<_>>();
        self.write_sheet.parameter.include_column_indexes = Some(indexes.clone());
        self.write_sheet.options.include_column_indexes = Some(indexes);
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Includes only the supplied Rust field names.
    #[must_use]
    pub fn include_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        let names = names.into_iter().map(Into::into).collect::<Vec<_>>();
        self.write_sheet.parameter.include_column_field_names = Some(names.clone());
        self.write_sheet.options.include_column_field_names = Some(names);
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Excludes physical columns.
    #[must_use]
    pub fn exclude_column_indexes(mut self, indexes: impl IntoIterator<Item = usize>) -> Self {
        let indexes = indexes.into_iter().collect::<Vec<_>>();
        self.write_sheet.parameter.exclude_column_indexes = Some(indexes.clone());
        self.write_sheet.options.exclude_column_indexes = indexes;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Excludes Rust field names.
    #[must_use]
    pub fn exclude_column_field_names<S>(mut self, names: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        let names = names.into_iter().map(Into::into).collect::<Vec<_>>();
        self.write_sheet.parameter.exclude_column_field_names = Some(names.clone());
        self.write_sheet.options.exclude_column_field_names = names;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Orders output by the include-list order.
    #[must_use]
    pub fn order_by_include_column(mut self, enabled: bool) -> Self {
        self.write_sheet.parameter.order_by_include_column = Some(enabled);
        self.write_sheet.options.order_by_include_column = enabled;
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Stores a handler owned by the sheet holder.
    #[must_use]
    pub fn register_write_handler(mut self, handler: impl WriteHandler + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Builds the untyped Java-compatible sheet metadata.
    #[must_use]
    pub fn build(&self) -> WriteSheetMetadata {
        self.write_sheet.clone()
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Writes the supplied rows and finishes the owned writer.
    ///
    /// This mirrors Java `ExcelWriterSheetBuilder.doWrite(Collection)`.
    ///
    /// # Errors
    ///
    /// Returns a format error when the builder owns no writer, and
    /// propagates writer errors from `write_with_sheet_handlers` / `finish`.
    pub fn do_write<T, I>(mut self, rows: I) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
    {
        let mut writer = self.excel_writer.take().ok_or_else(|| {
            ExcelError::Format("Must use ExcelWriterBuilder.sheet() to call do_write()".to_owned())
        })?;
        let sheet = WriteSheet::<T>::from_options(self.write_sheet.options.clone());
        writer.write_with_sheet_handlers(rows, &sheet, self.handlers)?;
        writer.finish()
    }

    /// 对应 Java：`ExcelWriterSheetBuilder.table()`。 Resolves rows lazily, then delegates to [`Self::do_write`].
    ///
    /// # Errors
    ///
    /// See [`Self::do_write`].
    pub fn do_write_with<T, I, F>(self, supplier: F) -> Result<()>
    where
        T: ExcelRow,
        I: IntoIterator<Item = T>,
        F: FnOnce() -> I,
    {
        self.do_write(supplier())
    }

    /// Creates a table builder bound to this writer and sheet.
    ///
    /// 对应 Java：`ExcelWriterSheetBuilder.table()`.
    #[must_use]
    pub fn table(mut self) -> ExcelWriterTableBuilder {
        match self.excel_writer.take() {
            Some(writer) => {
                ExcelWriterTableBuilder::with_excel_writer(writer, self.write_sheet, self.handlers)
            }
            None => ExcelWriterTableBuilder::new(),
        }
    }

    /// Creates a numbered table builder.
    ///
    /// 对应 Java：`ExcelWriterSheetBuilder.table(Integer)`.
    #[must_use]
    pub fn table_no(self, table_no: i32) -> ExcelWriterTableBuilder {
        self.table().table_no(table_no)
    }
}

impl Default for ExcelWriterSheetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{DynamicRow, DynamicValue, WriteHandler};
    use tempfile::tempdir;

    struct NoopHandler;
    impl WriteHandler for NoopHandler {
        fn order(&self) -> i32 {
            0
        }
    }

    #[test]
    fn excel_writer_sheet_builder_new() {
        let builder = ExcelWriterSheetBuilder::new();
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_default() {
        let builder = ExcelWriterSheetBuilder::default();
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_sheet_no() {
        let builder = ExcelWriterSheetBuilder::new().sheet_no(2);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_sheet_name() {
        let builder = ExcelWriterSheetBuilder::new().sheet_name("MySheet");
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_relative_head_row_index() {
        let builder = ExcelWriterSheetBuilder::new().relative_head_row_index(1);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_need_head() {
        let builder = ExcelWriterSheetBuilder::new().need_head(true);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_use_default_style() {
        let builder = ExcelWriterSheetBuilder::new().use_default_style(true);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_automatic_merge_head() {
        let builder = ExcelWriterSheetBuilder::new().automatic_merge_head(true);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_include_column_indexes() {
        let builder = ExcelWriterSheetBuilder::new().include_column_indexes([0, 1, 2]);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_include_column_field_names() {
        let builder = ExcelWriterSheetBuilder::new().include_column_field_names(["a", "b"]);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_exclude_column_indexes() {
        let builder = ExcelWriterSheetBuilder::new().exclude_column_indexes([0]);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_exclude_column_field_names() {
        let builder = ExcelWriterSheetBuilder::new().exclude_column_field_names(["a"]);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_order_by_include_column() {
        let builder = ExcelWriterSheetBuilder::new().order_by_include_column(true);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_register_write_handler() {
        let builder = ExcelWriterSheetBuilder::new().register_write_handler(NoopHandler);
        let _ = builder.build();
    }

    #[test]
    fn excel_writer_sheet_builder_with_excel_writer_and_options() {
        let options = WriteOptions::default();
        let _builder = ExcelWriterSheetBuilder::with_excel_writer_and_options(
            // Can't easily create ExcelWriter here without a real file
            // Use a manual test
            crate::write::excel_writer_core::ExcelWriter::new("/tmp/test.xlsx"),
            options,
        );
    }

    #[test]
    fn excel_writer_sheet_builder_table() {
        let builder = ExcelWriterSheetBuilder::new();
        let _table_builder = builder.table();
    }

    #[test]
    fn excel_writer_sheet_builder_use_default_style_false() {
        let builder = ExcelWriterSheetBuilder::new().use_default_style(false);
        let _ = builder.build();
    }

    #[test]
    fn noop_handler_order_returns_zero() {
        assert_eq!(NoopHandler.order(), 0);
    }

    fn dynamic_row(value: &str) -> DynamicRow {
        let mut values = std::collections::BTreeMap::new();
        values.insert(0, DynamicValue::String(value.to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn with_excel_writer_owns_writer_and_writes() -> crate::core::Result<()> {
        use calamine::{DataType, Reader, Xlsx, open_workbook};
        let directory = tempdir()?;
        let output = directory.path().join("sheet-builder-owner.xlsx");
        let writer = ExcelWriter::new(&output);

        ExcelWriterSheetBuilder::with_excel_writer(writer)
            .need_head(false)
            .do_write(vec![dynamic_row("alice")])?;

        let mut workbook: Xlsx<_> = open_workbook(&output)
            .map_err(|error: calamine::XlsxError| ExcelError::Format(error.to_string()))?;
        let range = workbook
            .worksheet_range("Sheet1")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        assert_eq!(
            range.get_value((0, 0)).and_then(|cell| cell.get_string()),
            Some("alice")
        );
        Ok(())
    }

    #[test]
    fn do_write_with_resolves_rows_lazily() -> crate::core::Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("sheet-builder-lazy.xlsx");
        let writer = ExcelWriter::new(&output);

        ExcelWriterSheetBuilder::with_excel_writer(writer)
            .need_head(false)
            .do_write_with(|| vec![dynamic_row("bob")])?;
        assert!(output.exists());
        Ok(())
    }

    #[test]
    fn do_write_without_owned_writer_returns_format_error() {
        let error = ExcelWriterSheetBuilder::new()
            .do_write(vec![dynamic_row("x")])
            .expect_err("builder without a writer must fail");
        assert!(matches!(error, ExcelError::Format(_)));
    }
}
