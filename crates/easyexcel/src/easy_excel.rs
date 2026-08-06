//! `EasyExcel` 门面类型与静态工厂。
//!
//! 对应 Java：`com.alibaba.excel.EasyExcel`
//! （Java 中 `EasyExcel extends EasyExcelFactory`，Rust 合并为同一个 `EasyExcel`，
//!  通过类型别名 [`EasyExcelFactory`](crate::EasyExcelFactory) 暴露等价路径）。

use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::{DynamicRow, ExcelRow, ReadListener, Result};
use crate::read::{CompatibleExcelReaderBuilder, CompatibleExcelReaderSheetBuilder};
use crate::template::{
    ExcelTemplateWriter, FillConfig, FillWrapper, TemplateData, fill_xlsx_template,
    fill_xlsx_template_list,
};
use crate::write::{
    CompatibleExcelWriterBuilder, CompatibleExcelWriterOutputStreamBuilder,
    CompatibleExcelWriterSheetBuilder, ExcelOutputStream, WriteSheet,
};

// 显式引用 builder 模块，便于 facade 方法的类型解析。
use crate::excel_reader_builder::ExcelReaderBuilder;
use crate::excel_sync_reader_builder::ExcelSyncReaderBuilder;
use crate::write::builder::excel_writer_builder::ExcelWriterBuilder;

/// 对应 Java：com.alibaba.excel.EasyExcel。 Static factory matching Java `EasyExcel`'s entry point.
pub struct EasyExcel;

impl EasyExcel {
    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's path-independent `read()` builder.
    #[must_use]
    pub fn reader() -> CompatibleExcelReaderBuilder {
        CompatibleExcelReaderBuilder::new()
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's `read(File/String)` builder without a listener.
    #[must_use]
    pub fn reader_from_path(path: impl Into<PathBuf>) -> CompatibleExcelReaderBuilder {
        Self::reader().file(path)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's `read(InputStream)` builder.
    ///
    /// The stream is materialised into an automatically deleted temporary
    /// file so the existing XLSX, XLS, and CSV engines retain random access.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the stream cannot be read or the temporary
    /// file cannot be created.
    pub fn reader_from_input_stream<R>(input: R) -> Result<CompatibleExcelReaderBuilder>
    where
        R: Read,
    {
        Self::reader().input_stream(input)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts an event-driven XLSX, XLS, or CSV read selected from the path extension.
    pub fn read<T, L>(path: impl Into<PathBuf>, listener: L) -> ExcelReaderBuilder<T, L>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        ExcelReaderBuilder::new(path.into(), listener)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts a synchronous read that collects all converted rows.
    pub fn read_sync<T>(path: impl Into<PathBuf>) -> ExcelSyncReaderBuilder<T>
    where
        T: ExcelRow,
    {
        ExcelSyncReaderBuilder::new(path.into())
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts a Java-compatible no-model event read.
    pub fn read_dynamic<L>(
        path: impl Into<PathBuf>,
        listener: L,
    ) -> ExcelReaderBuilder<DynamicRow, L>
    where
        L: ReadListener<DynamicRow>,
    {
        Self::read(path, listener)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts a Java-compatible no-model synchronous read.
    #[must_use]
    pub fn read_dynamic_sync(path: impl Into<PathBuf>) -> ExcelSyncReaderBuilder<DynamicRow> {
        Self::read_sync(path)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts a new XLSX or CSV write, selected from the path extension.
    pub fn write<T>(path: impl Into<PathBuf>) -> ExcelWriterBuilder<T>
    where
        T: ExcelRow,
    {
        ExcelWriterBuilder::new(path.into())
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's path-independent `write()` builder.
    #[must_use]
    pub fn writer() -> CompatibleExcelWriterBuilder {
        CompatibleExcelWriterBuilder::new()
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's `write(File/String)` builder.
    #[must_use]
    pub fn writer_to_path(path: impl Into<PathBuf>) -> CompatibleExcelWriterBuilder {
        Self::writer().file(path)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Starts Java's `write(OutputStream)` builder.
    #[must_use]
    pub fn writer_to_output_stream<W>(
        output: ExcelOutputStream<W>,
    ) -> CompatibleExcelWriterOutputStreamBuilder<W>
    where
        W: Write + Send + 'static,
    {
        Self::writer().output_stream(output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `readSheet()` metadata builder.
    #[must_use]
    pub fn read_sheet() -> CompatibleExcelReaderSheetBuilder {
        CompatibleExcelReaderSheetBuilder::new()
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `readSheet(Integer)` metadata builder.
    #[must_use]
    pub fn read_sheet_index(index: i32) -> CompatibleExcelReaderSheetBuilder {
        Self::read_sheet().sheet_no(index)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `readSheet(String)` metadata builder.
    #[must_use]
    pub fn read_sheet_name(name: impl Into<String>) -> CompatibleExcelReaderSheetBuilder {
        Self::read_sheet().sheet_name(name)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `readSheet(Integer, String)` metadata builder.
    #[must_use]
    pub fn read_sheet_with(
        index: i32,
        name: impl Into<String>,
    ) -> CompatibleExcelReaderSheetBuilder {
        Self::read_sheet().sheet_no(index).sheet_name(name)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Creates typed worksheet metadata for a stateful `ExcelWriter`.
    #[must_use]
    pub fn writer_sheet<T>(name: impl Into<String>) -> WriteSheet<T>
    where
        T: ExcelRow,
    {
        WriteSheet::new(name)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Creates typed worksheet metadata for a Java-style zero-based sheet number.
    #[must_use]
    pub fn writer_sheet_index<T>(index: usize) -> WriteSheet<T>
    where
        T: ExcelRow,
    {
        WriteSheet::new_index(index)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `writerSheet()` metadata builder.
    #[must_use]
    pub fn writer_sheet_builder() -> CompatibleExcelWriterSheetBuilder {
        CompatibleExcelWriterSheetBuilder::new()
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `writerSheet(Integer)` metadata builder.
    #[must_use]
    pub fn writer_sheet_builder_index(index: i32) -> CompatibleExcelWriterSheetBuilder {
        Self::writer_sheet_builder().sheet_no(index)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `writerSheet(String)` metadata builder.
    #[must_use]
    pub fn writer_sheet_builder_name(name: impl Into<String>) -> CompatibleExcelWriterSheetBuilder {
        Self::writer_sheet_builder().sheet_name(name)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `writerSheet(Integer, String)` metadata builder.
    #[must_use]
    pub fn writer_sheet_builder_with(
        index: i32,
        name: impl Into<String>,
    ) -> CompatibleExcelWriterSheetBuilder {
        Self::writer_sheet_builder()
            .sheet_no(index)
            .sheet_name(name)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Creates a `WriteTable` value mirroring Java
    /// `EasyExcelFactory.writerTable(Integer)`. (Java `writerTable(int)`)
    #[must_use]
    pub fn writer_table(table_no: i32) -> crate::write::MirroredWriteTable {
        crate::write::MirroredWriteTable::with_table_no(table_no)
    }

    /// Begins a multi-table write flow that produces an `ExcelWriterTableBuilder`.
    ///
    /// 对应 Java：`ExcelWriterBuilder.table(Integer)` which yields an
    /// `ExcelWriterTableBuilder` for configuring per-table options before
    /// calling `.do_write(rows, sheet, table)`.
    ///
    /// Phase 4 addition: provides the three-arg `write(Collection, WriteSheet, WriteTable)`
    /// overload at the public facade level.
    #[must_use]
    pub fn writer_table_builder(table_no: i32) -> crate::write::ExcelWriterTableBuilder {
        crate::write::ExcelWriterTableBuilder::new().table_no(table_no)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Builds Java's unbound `writerTable()` builder.
    #[must_use]
    pub fn writer_table_builder_default() -> crate::write::ExcelWriterTableBuilder {
        crate::write::ExcelWriterTableBuilder::new()
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Fills scalar `{key}` placeholders in an existing XLSX template.
    ///
    /// Legacy `.xls` templates return typed `ExcelError::Unsupported`
    /// (`legacy XLS template fill is not supported`). Java maps this to
    /// `ExcelWriter.fill` on `HSSFWorkbook`; Rust fill remains OOXML-only.
    /// Use [`Self::write`] / `with_template` for `.xls` cell append instead.
    ///
    /// # Errors
    ///
    /// Returns an I/O, Unsupported, or package format error.
    pub fn fill_template(
        template: impl AsRef<Path>,
        output: impl AsRef<Path>,
        data: &TemplateData,
    ) -> Result<()> {
        fill_xlsx_template(template.as_ref(), output.as_ref(), data)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Expands a collection in an existing XLSX template.
    ///
    /// XLS (`.xls`) collection fill is not supported — returns
    /// `ExcelError::Unsupported` with `legacy XLS template fill is not supported`.
    ///
    /// # Errors
    ///
    /// Returns an I/O, Unsupported, or OOXML package error.
    pub fn fill_template_list(
        template: impl AsRef<Path>,
        output: impl AsRef<Path>,
        data: &FillWrapper,
        config: FillConfig,
    ) -> Result<()> {
        fill_xlsx_template_list(template.as_ref(), output.as_ref(), data, config)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads an XLSX template for repeated Java-style `fill` calls.
    ///
    /// XLS (`.xls`) stateful template writers are rejected with
    /// `legacy XLS template fill is not supported` (use [`Self::fill_template`] for
    /// scalar `.xls` fill).
    ///
    /// # Errors
    ///
    /// Returns an I/O, Unsupported, or OOXML package error when the template cannot be read.
    pub fn template_writer(
        template: impl AsRef<Path>,
        output: impl Into<PathBuf>,
    ) -> Result<ExcelTemplateWriter<'static>> {
        ExcelTemplateWriter::new(template, output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads an XLSX template from a Java-style input stream and writes to a path.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn template_writer_from_reader<R>(
        template: R,
        output: impl Into<PathBuf>,
    ) -> Result<ExcelTemplateWriter<'static>>
    where
        R: Read,
    {
        ExcelTemplateWriter::from_reader(template, output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads a path template and writes to a caller-owned output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn template_writer_to_writer<W>(
        template: impl AsRef<Path>,
        output: &mut W,
    ) -> Result<ExcelTemplateWriter<'_>>
    where
        W: Write,
    {
        ExcelTemplateWriter::to_writer(template, output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads a stream template and writes to a caller-owned output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn template_writer_from_reader_to_writer<R, W>(
        template: R,
        output: &mut W,
    ) -> Result<ExcelTemplateWriter<'_>>
    where
        R: Read,
        W: Write,
    {
        ExcelTemplateWriter::from_reader_to_writer(template, output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads a path template and writes to a closeable output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn template_writer_to_output_stream<'a, W>(
        template: impl AsRef<Path>,
        output: ExcelOutputStream<W>,
    ) -> Result<ExcelTemplateWriter<'a>>
    where
        W: Write + 'a,
    {
        ExcelTemplateWriter::to_output_stream(template, output)
    }

    /// 对应 Java：com.alibaba.excel.EasyExcel。 Loads a stream template and writes to a closeable output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn template_writer_from_reader_to_output_stream<'a, R, W>(
        template: R,
        output: ExcelOutputStream<W>,
    ) -> Result<ExcelTemplateWriter<'a>>
    where
        R: Read,
        W: Write + 'a,
    {
        ExcelTemplateWriter::from_reader_to_output_stream(template, output)
    }
}
