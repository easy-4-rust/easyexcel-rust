//! 状态化 OOXML 模板写入器与 XLSX 包读写（fill 生命周期）。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter`（fill 生命周期）

use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use easyexcel_core::{CellValue, ExcelError, Result};
use easyexcel_writer::ExcelOutputStream;
use zip::read::ZipArchive;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::fill_engine::{
    append_rows_to_sheet, attribute_value, replace_collection_fills_in_sheet,
    replace_scalar_cells_in_sheet,
};
use crate::sheet_fill_state::{PendingCollectionFill, PendingSheetFill, ResolvedSheetFill};
use crate::template_entry::TemplateEntry;
use crate::template_output::{ArchiveWriter, ReadSeek, TemplateOutput, WriteSeek};
use crate::{FillConfig, FillWrapper, TemplateData, TemplateSheet};

/// Stateful OOXML template writer matching Java `ExcelWriter.fill` lifecycle.
///
/// Scalar values and collection fills are accumulated against one loaded XLSX
/// package. Repeated collection fills with the same prefix append at the prior
/// fill position instead of reopening the original template.
pub struct ExcelTemplateWriter<'a> {
    pub(crate) output: TemplateOutput<'a>,
    pub(crate) entries: Vec<TemplateEntry>,
    pub(crate) sheets: Vec<PendingSheetFill>,
    pub(crate) next_collection_order: usize,
    pub(crate) finished: bool,
    pub(crate) auto_close_stream: bool,
}

impl std::fmt::Debug for ExcelTemplateWriter<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self.output {
            TemplateOutput::Path(_) => "path",
            TemplateOutput::Borrowed(_) => "borrowed stream",
            TemplateOutput::Owned(_) => "owned stream",
        };
        formatter
            .debug_struct("ExcelTemplateWriter")
            .field("output", &output)
            .field("entries", &self.entries)
            .field("sheets", &self.sheets)
            .field("finished", &self.finished)
            .field("auto_close_stream", &self.auto_close_stream)
            .finish()
    }
}

impl ExcelTemplateWriter<'static> {
    /// Loads a template package for stateful filling.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn new(template: impl AsRef<Path>, output: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self::from_entries(
            TemplateOutput::Path(output.into()),
            load_entries(template.as_ref())?,
        ))
    }

    /// Loads a template from a Java-style input stream and writes to a path.
    ///
    /// The reader is consumed and dropped after its bytes have been copied into
    /// memory, matching Java `EasyExcel`'s default `autoCloseStream(true)` input
    /// lifecycle. Pass `&mut reader` to retain caller ownership.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn from_reader<R>(template: R, output: impl Into<PathBuf>) -> Result<Self>
    where
        R: Read,
    {
        Ok(Self::from_entries(
            TemplateOutput::Path(output.into()),
            load_entries_from_reader(Box::new(template))?,
        ))
    }
}

impl<'a> ExcelTemplateWriter<'a> {
    fn from_entries(output: TemplateOutput<'a>, entries: Vec<TemplateEntry>) -> Self {
        Self {
            output,
            entries,
            sheets: vec![PendingSheetFill::new(TemplateSheet::first())],
            next_collection_order: 0,
            finished: false,
            auto_close_stream: true,
        }
    }

    /// Loads a path template and writes to a caller-owned output stream.
    ///
    /// The borrowed writer is flushed but never closed or dropped by this
    /// object, which is Rust's equivalent of Java `autoCloseStream(false)`.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn to_writer<W>(template: impl AsRef<Path>, output: &'a mut W) -> Result<Self>
    where
        W: Write,
    {
        Ok(Self::from_entries(
            TemplateOutput::Borrowed(output),
            load_entries(template.as_ref())?,
        ))
    }

    /// Loads a stream template and writes to a caller-owned output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn from_reader_to_writer<R, W>(template: R, output: &'a mut W) -> Result<Self>
    where
        R: Read,
        W: Write,
    {
        Ok(Self::from_entries(
            TemplateOutput::Borrowed(output),
            load_entries_from_reader(Box::new(template))?,
        ))
    }

    /// Loads a path template and writes to an explicitly closeable stream.
    ///
    /// Keep a clone of `output` to observe Java-compatible close state after
    /// [`Self::finish`].
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn to_output_stream<W>(
        template: impl AsRef<Path>,
        output: ExcelOutputStream<W>,
    ) -> Result<Self>
    where
        W: Write + 'a,
    {
        Ok(Self::from_entries(
            TemplateOutput::Owned(Box::new(output)),
            load_entries(template.as_ref())?,
        ))
    }

    /// Loads a stream template and writes to an explicitly closeable stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    pub fn from_reader_to_output_stream<R, W>(
        template: R,
        output: ExcelOutputStream<W>,
    ) -> Result<Self>
    where
        R: Read,
        W: Write + 'a,
    {
        Ok(Self::from_entries(
            TemplateOutput::Owned(Box::new(output)),
            load_entries_from_reader(Box::new(template))?,
        ))
    }

    /// Controls whether an owned output stream is closed by [`Self::finish`].
    ///
    /// The default is `true`, matching Java `EasyExcel`. Borrowed writers always
    /// remain caller-owned regardless of this setting.
    #[must_use]
    pub const fn auto_close_stream(mut self, enabled: bool) -> Self {
        self.auto_close_stream = enabled;
        self
    }

    /// Accumulates scalar `{key}` values for this workbook.
    ///
    /// Later fills replace earlier values for the same key, matching Java map
    /// filling before the workbook is finalized.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn fill(&mut self, data: &TemplateData) -> Result<&mut Self> {
        self.fill_on_sheet(&TemplateSheet::first(), data)
    }

    /// Accumulates scalar `{key}` values for one selected worksheet.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn fill_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        data: &TemplateData,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        self.sheet_state_mut(sheet)
            .scalar
            .values
            .extend(data.values.clone());
        Ok(self)
    }

    /// Accumulates a named or unnamed collection fill.
    ///
    /// Repeated calls with the same prefix append rows through Java-compatible
    /// per-prefix cursors. Each call keeps its own configuration, including
    /// direction, `force_new_row`, and `auto_style`.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn fill_list(&mut self, data: &FillWrapper, config: FillConfig) -> Result<&mut Self> {
        self.fill_list_on_sheet(&TemplateSheet::first(), data, config)
    }

    /// Accumulates a collection fill for one selected worksheet.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn fill_list_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        data: &FillWrapper,
        config: FillConfig,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        if data.rows().is_empty() {
            return Ok(self);
        }
        let order = self.next_collection_order;
        self.next_collection_order = self.next_collection_order.saturating_add(1);
        let state = self.sheet_state_mut(sheet);
        state.collections.push(PendingCollectionFill {
            wrapper: data.clone(),
            config,
            order,
        });
        Ok(self)
    }

    /// Queues ordinary rows after the template fill cursor.
    ///
    /// This corresponds to Java's `excelWriter.write(rows, writeSheet)` after
    /// one or more `fill` calls. It is primarily intended for summary rows when
    /// the collection placeholder is the final template row.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn write_rows(
        &mut self,
        rows: impl IntoIterator<Item = Vec<CellValue>>,
    ) -> Result<&mut Self> {
        self.write_rows_on_sheet(&TemplateSheet::first(), rows)
    }

    /// Queues ordinary rows after the fill cursor of one selected worksheet.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    pub fn write_rows_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        rows: impl IntoIterator<Item = Vec<CellValue>>,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        self.sheet_state_mut(sheet).appended_rows.extend(rows);
        Ok(self)
    }

    /// Writes the completed XLSX package. Repeated calls are no-ops.
    ///
    /// # Errors
    ///
    /// Returns an XML, ZIP, or output I/O error.
    pub fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        for sheet in self.resolved_sheet_fills()? {
            replace_collection_fills_in_sheet(
                &mut self.entries,
                &sheet.worksheet,
                &sheet.collections,
            )?;
            replace_scalar_cells_in_sheet(&mut self.entries, &sheet.worksheet, &sheet.scalar)?;
            append_rows_to_sheet(&mut self.entries, &sheet.worksheet, &sheet.appended_rows)?;
        }
        self.finished = true;
        write_entries_to_output(&mut self.output, &self.entries, self.auto_close_stream)
    }

    /// Returns whether [`Self::finish`] has run.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    fn ensure_open(&self) -> Result<()> {
        if self.finished {
            Err(ExcelError::Unsupported(
                "template writer already finished".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    fn sheet_state_mut(&mut self, sheet: &TemplateSheet) -> &mut PendingSheetFill {
        if let Some(index) = self
            .sheets
            .iter()
            .position(|pending| same_sheet(&pending.sheet, sheet))
        {
            return &mut self.sheets[index];
        }
        self.sheets.push(PendingSheetFill::new(sheet.clone()));
        self.sheets.last_mut().expect("sheet state was just pushed")
    }

    pub(crate) fn resolved_sheet_fills(&self) -> Result<Vec<ResolvedSheetFill>> {
        let mut resolved: Vec<ResolvedSheetFill> = Vec::new();
        for pending_sheet in &self.sheets {
            let worksheet = worksheet_path(&self.entries, &pending_sheet.sheet)?;
            if let Some(sheet) = resolved
                .iter_mut()
                .find(|sheet| sheet.worksheet.eq_ignore_ascii_case(&worksheet))
            {
                sheet
                    .scalar
                    .values
                    .extend(pending_sheet.scalar.values.clone());
                sheet
                    .collections
                    .extend(pending_sheet.collections.iter().cloned());
                sheet
                    .appended_rows
                    .extend(pending_sheet.appended_rows.iter().cloned());
            } else {
                resolved.push(ResolvedSheetFill {
                    worksheet,
                    scalar: pending_sheet.scalar.clone(),
                    collections: pending_sheet.collections.clone(),
                    appended_rows: pending_sheet.appended_rows.clone(),
                });
            }
        }
        for sheet in &mut resolved {
            sheet.collections.sort_by_key(|collection| collection.order);
        }
        Ok(resolved)
    }
}

pub(crate) fn same_sheet(left: &TemplateSheet, right: &TemplateSheet) -> bool {
    match (left, right) {
        (
            TemplateSheet::First | TemplateSheet::Index(0),
            TemplateSheet::First | TemplateSheet::Index(0),
        ) => true,
        (TemplateSheet::Index(left), TemplateSheet::Index(right)) => left == right,
        (TemplateSheet::Name(left), TemplateSheet::Name(right)) => left == right,
        _ => false,
    }
}

/// Fills scalar `{key}` placeholders while preserving the XLSX package structure.
///
/// The template and output paths may be identical because the source archive is
/// fully loaded before the destination is opened.
///
/// # Errors
///
/// Returns an I/O or format error for invalid ZIP/OOXML input or output failures.
/// Legacy `.xls` templates are now supported via BIFF8 placeholder replacement.
pub fn fill_xlsx_template(template: &Path, output: &Path, data: &TemplateData) -> Result<()> {
    if template
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xls"))
    {
        return fill_xls_template_scalar(template, output, data);
    }
    let mut writer = ExcelTemplateWriter::new(template, output)?;
    writer.sheets[0].scalar.values.extend(data.values.clone());
    writer.finish()
}

/// Expands Java EasyExcel-style collection placeholders in an XLSX template.
///
/// Unnamed wrappers use `{.field}` while named wrappers use `{name.field}`.
///
/// # Errors
///
/// Returns an I/O or format error when the package cannot be read or written.
pub fn fill_xlsx_template_list(
    template: &Path,
    output: &Path,
    data: &FillWrapper,
    config: FillConfig,
) -> Result<()> {
    if template
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xls"))
    {
        return fill_xls_template_list(template, output, data, config);
    }
    let mut writer = ExcelTemplateWriter::new(template, output)?;
    if !data.rows().is_empty() {
        writer.sheets[0].collections.push(PendingCollectionFill {
            wrapper: data.clone(),
            config,
            order: 0,
        });
    }
    writer.finish()
}

// ---------------------------------------------------------------------------
// BIFF8 (.xls) template fill — Phase 5
// ---------------------------------------------------------------------------

/// Replaces `{key}` placeholders in a legacy BIFF8 `.xls` template with
/// `TemplateData` scalar values. 对应 Java：'s HSSFWorkbook-level fill
/// for XLS workbooks.
fn fill_xls_template_scalar(template: &Path, output: &Path, data: &TemplateData) -> Result<()> {
    let bytes = std::fs::read(template)?;
    let mut pkg = easyexcel_writer::biff8::Biff8TemplatePackage::from_bytes(&bytes)?;
    let placeholders = pkg.scan_placeholders();
    for (sheet_name, row, col, text) in &placeholders {
        let key = text
            .trim_start_matches('{')
            .trim_end_matches('}')
            .to_string();
        if let Some(value) = data.values.get(&key) {
            let replacement = value.as_text();
            pkg.replace_label(sheet_name, *row, *col, &replacement)?;
        }
    }
    pkg.save_to_path(output)
}

/// Replaces list placeholders in a BIFF8 `.xls` template.
fn fill_xls_template_list(
    template: &Path,
    output: &Path,
    data: &FillWrapper,
    _config: FillConfig,
) -> Result<()> {
    let bytes = std::fs::read(template)?;
    let mut pkg = easyexcel_writer::biff8::Biff8TemplatePackage::from_bytes(&bytes)?;
    let placeholders = pkg.scan_placeholders();
    let prefix = data.name().map(|n| format!("{n}.")).unwrap_or_default();
    let is_dot = prefix.is_empty();

    for (sheet_name, row, col, text) in &placeholders {
        let key = if is_dot && text.starts_with("{.") {
            text.trim_start_matches("{.")
                .trim_end_matches('}')
                .to_string()
        } else if !prefix.is_empty() && text.starts_with(&format!("{{{prefix}")) {
            text.trim_start_matches(&format!("{{{prefix}"))
                .trim_end_matches('}')
                .to_string()
        } else if text.starts_with('{') {
            text.trim_start_matches('{')
                .trim_end_matches('}')
                .to_string()
        } else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        for template_row in data.rows() {
            if let Some(value) = template_row.values.get(&key) {
                let replacement = value.as_text();
                pkg.replace_label(sheet_name, *row, *col, &replacement)?;
                break;
            }
        }
    }
    pkg.save_to_path(output)
}

pub(crate) fn load_entries(path: &Path) -> Result<Vec<TemplateEntry>> {
    // Scalar `.xls` fill is handled by [`fill_xlsx_template`] before ZIP load.
    // Stateful ExcelTemplateWriter / collection fill stay OOXML-only.
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xls"))
    {
        return Err(ExcelError::Unsupported(
            // Java: ExcelWriter.fill on HSSFWorkbook. Rust fill is OOXML-only;
            // use with_template + doWrite (Biff8TemplatePackage) for .xls cells.
            "legacy XLS template fill is not supported".to_owned(),
        ));
    }
    load_entries_from(Box::new(File::open(path)?))
}

fn load_entries_from_reader(mut reader: Box<dyn Read + '_>) -> Result<Vec<TemplateEntry>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    load_entries_from(Box::new(Cursor::new(bytes)))
}

pub(crate) fn load_entries_from(reader: Box<dyn ReadSeek>) -> Result<Vec<TemplateEntry>> {
    let mut archive = ZipArchive::new(reader).map_err(format_error)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(format_error)?;
        let mut bytes = Vec::new();
        if !entry.is_dir() {
            entry.read_to_end(&mut bytes)?;
        }
        entries.push(TemplateEntry {
            name: entry.name().to_owned(),
            is_dir: entry.is_dir(),
            compression: entry.compression(),
            unix_mode: entry.unix_mode(),
            bytes,
        });
    }
    Ok(entries)
}

pub(crate) fn worksheet_path(entries: &[TemplateEntry], sheet: &TemplateSheet) -> Result<String> {
    let workbook = entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case("xl/workbook.xml"));
    let relationships = entries.iter().find(|entry| {
        entry
            .name
            .eq_ignore_ascii_case("xl/_rels/workbook.xml.rels")
    });
    if let (Some(workbook), Some(relationships)) = (workbook, relationships) {
        let workbook = std::str::from_utf8(&workbook.bytes)
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        let relationships = std::str::from_utf8(&relationships.bytes)
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        let sheets = workbook_sheets(workbook);
        let selected = match sheet {
            TemplateSheet::First => sheets.first(),
            TemplateSheet::Index(index) => sheets.get(*index),
            TemplateSheet::Name(name) => sheets.iter().find(|(sheet_name, _)| sheet_name == name),
        }
        .ok_or_else(|| ExcelError::SheetNotFound(template_sheet_label(sheet)))?;
        let target = workbook_relationship_target(relationships, &selected.1).ok_or_else(|| {
            ExcelError::Format(format!(
                "workbook relationship {} for sheet {} is missing",
                selected.1, selected.0
            ))
        })?;
        let normalized = normalize_workbook_target(target)?;
        return entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&normalized))
            .map(|entry| entry.name.clone())
            .ok_or_else(|| {
                ExcelError::Format(format!(
                    "worksheet part {normalized} for sheet {} is missing",
                    selected.0
                ))
            });
    }

    let worksheets = entries
        .iter()
        .filter(|entry| {
            entry.name.starts_with("xl/worksheets/")
                && Path::new(&entry.name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        })
        .collect::<Vec<_>>();
    let index = match sheet {
        TemplateSheet::First => 0,
        TemplateSheet::Index(index) => *index,
        TemplateSheet::Name(name) => {
            return Err(ExcelError::SheetNotFound(name.clone()));
        }
    };
    worksheets
        .get(index)
        .map(|entry| entry.name.clone())
        .ok_or_else(|| ExcelError::SheetNotFound(template_sheet_label(sheet)))
}

pub(crate) fn workbook_sheets(xml: &str) -> Vec<(String, String)> {
    xml_elements(xml, "sheet")
        .filter_map(|element| {
            Some((
                attribute_value(element, "name")?.to_owned(),
                attribute_value(element, "r:id")?.to_owned(),
            ))
        })
        .collect()
}

fn workbook_relationship_target<'a>(xml: &'a str, relationship_id: &str) -> Option<&'a str> {
    xml_elements(xml, "Relationship")
        .find(|element| attribute_value(element, "Id") == Some(relationship_id))
        .and_then(|element| attribute_value(element, "Target"))
}

pub(crate) fn xml_elements<'a>(xml: &'a str, name: &'a str) -> impl Iterator<Item = &'a str> {
    let marker = format!("<{name}");
    let mut offset = 0;
    std::iter::from_fn(move || {
        while let Some(relative_start) = xml[offset..].find(&marker) {
            let start = offset + relative_start;
            let after_name = start + marker.len();
            if xml
                .as_bytes()
                .get(after_name)
                .is_some_and(u8::is_ascii_alphanumeric)
            {
                offset = after_name;
                continue;
            }
            let end = start + xml[start..].find('>')? + 1;
            offset = end;
            return Some(&xml[start..end]);
        }
        None
    })
}

pub(crate) fn normalize_workbook_target(target: &str) -> Result<String> {
    let candidate = target
        .strip_prefix('/')
        .map_or_else(|| format!("xl/{target}"), str::to_owned);
    let mut components = Vec::new();
    for component in candidate.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(ExcelError::Format(format!(
                        "worksheet target escapes package root: {target}"
                    )));
                }
            }
            component => components.push(component),
        }
    }
    if components.is_empty() {
        return Err(ExcelError::Format(format!(
            "worksheet target is empty: {target}"
        )));
    }
    Ok(components.join("/"))
}

fn template_sheet_label(sheet: &TemplateSheet) -> String {
    match sheet {
        TemplateSheet::First => "0".to_owned(),
        TemplateSheet::Index(index) => index.to_string(),
        TemplateSheet::Name(name) => name.clone(),
    }
}

pub(crate) fn write_entries(path: &Path, entries: &[TemplateEntry]) -> Result<()> {
    match File::create(path) {
        Ok(writer) => write_file_entries(writer, entries),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn write_entries_to_output(
    output: &mut TemplateOutput<'_>,
    entries: &[TemplateEntry],
    auto_close_stream: bool,
) -> Result<()> {
    match output {
        TemplateOutput::Path(path) => write_entries(path, entries),
        TemplateOutput::Borrowed(writer) => {
            let bytes = encode_entries(entries)?;
            writer.write_all(&bytes)?;
            writer.flush()?;
            Ok(())
        }
        TemplateOutput::Owned(writer) => {
            let write_result = encode_entries(entries).and_then(|bytes| {
                writer
                    .write_all(&bytes)
                    .and_then(|()| writer.flush())
                    .map_err(ExcelError::from)
            });
            let close_result = if auto_close_stream {
                writer.close()
            } else {
                Ok(())
            };
            close_result.map_err(ExcelError::from)?;
            write_result
        }
    }
}

pub(crate) fn encode_entries(entries: &[TemplateEntry]) -> Result<Vec<u8>> {
    let writer = write_entries_to(Box::new(Cursor::new(Vec::new())), entries)?;
    archive_output_bytes(writer)
}

pub(crate) fn archive_output_bytes(writer: Box<dyn WriteSeek>) -> Result<Vec<u8>> {
    writer
        .into_any()
        .downcast::<Cursor<Vec<u8>>>()
        .map(|cursor| cursor.into_inner())
        .map_err(|_| ExcelError::Format("ZIP output buffer type changed".to_owned()))
}

pub(crate) fn write_file_entries(writer: File, entries: &[TemplateEntry]) -> Result<()> {
    let _ = write_entries_to(Box::new(writer), entries)?;
    Ok(())
}

pub(crate) fn write_entries_to(
    writer: Box<dyn WriteSeek>,
    entries: &[TemplateEntry],
) -> Result<Box<dyn WriteSeek>> {
    let mut writer = Some(ZipWriter::new(writer));
    for entry in entries {
        let mut options = SimpleFileOptions::default().compression_method(entry.compression);
        if let Some(mode) = entry.unix_mode {
            options = options.unix_permissions(mode);
        }
        if entry.is_dir {
            let mut operation = |writer: &mut ArchiveWriter| {
                writer
                    .add_directory(&entry.name, options)
                    .map_err(format_error)
            };
            zip_writer_operation(&mut writer, &mut operation)?;
        } else {
            let mut start = |writer: &mut ArchiveWriter| {
                writer
                    .start_file(&entry.name, options)
                    .map_err(format_error)
            };
            zip_writer_operation(&mut writer, &mut start)?;
            let mut write = |writer: &mut ArchiveWriter| {
                writer.write_all(&entry.bytes).map_err(ExcelError::from)
            };
            zip_writer_operation(&mut writer, &mut write)?;
        }
    }
    finish_zip_writer(&mut writer)
}

pub(crate) fn finish_zip_writer(writer: &mut Option<ArchiveWriter>) -> Result<Box<dyn WriteSeek>> {
    let Some(writer) = writer.take() else {
        return Err(ExcelError::Format("ZIP writer is unavailable".to_owned()));
    };
    match catch_unwind(AssertUnwindSafe(|| writer.finish())) {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(error)) => Err(format_error(error)),
        Err(_) => Err(ExcelError::Format(
            "ZIP writer panicked while finalizing output".to_owned(),
        )),
    }
}

pub(crate) fn zip_writer_operation(
    writer: &mut Option<ArchiveWriter>,
    operation: &mut dyn FnMut(&mut ArchiveWriter) -> Result<()>,
) -> Result<()> {
    let Some(active) = writer.as_mut() else {
        return Err(ExcelError::Format("ZIP writer is unavailable".to_owned()));
    };
    match catch_unwind(AssertUnwindSafe(|| operation(active))) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => {
            let damaged = writer.take().expect("active writer exists");
            std::mem::forget(damaged);
            Err(error)
        }
        Err(_) => {
            let damaged = writer.take().expect("active writer exists");
            std::mem::forget(damaged);
            Err(ExcelError::Format(
                "ZIP writer panicked while processing output".to_owned(),
            ))
        }
    }
}

pub(crate) fn format_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use calamine::Reader;
    use std::fs;
    use tempfile::tempdir;
    use zip::CompressionMethod;

    // 错误转换直接复用生产 `format_error`（与 `zip_writer_operation` 一致），
    // 不再保留独立测试副本：`map_err` 闭包只在出错时执行，测试中恒成功，
    // 原 test_error 函数体是死代码，已删除。

    /// 手写 ZIP 包（用于构造缺失/损坏 worksheet 部件的模板）。
    fn write_custom_zip(path: &Path, entries: &[(&str, &[u8])]) -> Result<()> {
        let file = File::create(path).map_err(ExcelError::from)?;
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, bytes) in entries {
            writer
                .start_file(*name, options)
                .map_err(|error| ExcelError::Format(error.to_string()))?;
            writer.write_all(bytes).map_err(ExcelError::from)?;
        }
        writer
            .finish()
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        Ok(())
    }

    /// 用 `rust_xlsxwriter` 生成一个含 `{name}` 占位符的 XLSX 模板。
    fn xlsx_template(directory: &Path, name: &str) -> Result<PathBuf> {
        let template = directory.join(name);
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .write_string(0, 0, "{name}")
            .map_err(format_error)?;
        workbook.save(&template).map_err(format_error)?;
        Ok(template)
    }

    /// 对应 Java：`ExcelWriter.fill` 之后对已损坏 worksheet 部件的错误传播。
    ///
    /// worksheet 部件存在但字节不是合法 UTF-8 时，`finish` 中
    /// `replace_collection_fills_in_sheet` 的 `?` 错误边必须被覆盖。
    #[test]
    fn finish_propagates_collection_fill_failure_for_non_utf8_worksheet() -> Result<()> {
        let directory = tempdir()?;
        let template = directory.path().join("bad-worksheet.xlsx");
        let output = directory.path().join("out.xlsx");
        write_custom_zip(
            &template,
            &[
                (
                    "xl/workbook.xml",
                    br#"<workbook><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
                ),
                (
                    "xl/_rels/workbook.xml.rels",
                    br#"<Relationships><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
                ),
                ("xl/worksheets/sheet1.xml", &[0xff]),
            ],
        )?;
        let mut writer = ExcelTemplateWriter::new(&template, &output)?;
        writer.fill_list(
            &FillWrapper::new([TemplateData::new().with("name", "Alice")]),
            FillConfig::new(),
        )?;
        let error = writer.finish().expect_err("finish must fail");
        assert!(
            matches!(error, ExcelError::Format(ref message) if message.contains("invalid utf-8")),
            "unexpected error: {error}"
        );
        Ok(())
    }

    /// 对应 Java：`fillXlsTemplate` 列表填充（BIFF8 `.xls`）。
    ///
    /// 用 `easyexcel-writer` 的 `Biff8Book` 生成带各类占位符的 `.xls` 模板：
    /// `{.name}`（未命名 `is_dot` 分支）、`{plain}`（普通分支）、
    /// `x{name}`（else 分支）、`{}`（空 key）、`{.}`（点号空 key）、
    /// `{missing}`（未命中 key），以及命名 wrapper 的 `{users.name}`（prefix 分支）。
    #[test]
    fn fill_xlsx_template_list_supports_biff8_xls_placeholders() -> Result<()> {
        let directory = tempdir()?;
        let template = directory.path().join("list-template.xls");
        let unnamed_output = directory.path().join("list-unnamed-output.xls");
        let named_output = directory.path().join("list-named-output.xls");

        let mut book = easyexcel_writer::biff8::Biff8Book::default();
        {
            let sheet = book.sheet_mut("Sheet1");
            let text = |value: &str| {
                easyexcel_writer::biff8::Biff8Cell::general(
                    easyexcel_writer::biff8::Biff8Value::Text(value.to_owned()),
                )
            };
            sheet.set(0, 0, text("{.name}"))?;
            sheet.set(0, 1, text("{plain}"))?;
            sheet.set(0, 2, text("x{name}"))?;
            sheet.set(0, 3, text("{}"))?;
            sheet.set(0, 4, text("{.}"))?;
            sheet.set(0, 5, text("{missing}"))?;
            sheet.set(0, 6, text("{users.name}"))?;
        }
        fs::write(&template, book.to_cfb_bytes()?)?;

        // 未命名 wrapper：覆盖 is_dot / 普通 / else / 空 key / 未命中分支。
        fill_xlsx_template_list(
            &template,
            &unnamed_output,
            &FillWrapper::new([TemplateData::new()
                .with("name", "Alice")
                .with("plain", "Bob")]),
            FillConfig::new(),
        )?;
        let mut workbook: calamine::Xls<_> =
            calamine::open_workbook_from_rs(Cursor::new(fs::read(&unnamed_output)?))
                .map_err(format_error)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 0)),
            Some(&calamine::Data::String("Alice".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 1)),
            Some(&calamine::Data::String("Bob".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 2)),
            Some(&calamine::Data::String("x{name}".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 3)),
            Some(&calamine::Data::String("{}".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 4)),
            Some(&calamine::Data::String("{.}".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 5)),
            Some(&calamine::Data::String("{missing}".to_owned()))
        );
        assert_eq!(
            range.get_value((0, 6)),
            Some(&calamine::Data::String("{users.name}".to_owned()))
        );

        // 命名 wrapper：覆盖 `{prefix.key}` 分支。
        fill_xlsx_template_list(
            &template,
            &named_output,
            &FillWrapper::named("users", [TemplateData::new().with("name", "Carol")]),
            FillConfig::new(),
        )?;
        let mut workbook: calamine::Xls<_> =
            calamine::open_workbook_from_rs(Cursor::new(fs::read(&named_output)?))
                .map_err(format_error)?;
        let range = workbook.worksheet_range("Sheet1").map_err(format_error)?;
        assert_eq!(
            range.get_value((0, 6)),
            Some(&calamine::Data::String("Carol".to_owned()))
        );
        Ok(())
    }

    /// 对应 Java：`loadEntries` 拒绝 legacy `.xls`（OOXML-only 限制）。
    #[test]
    fn load_entries_rejects_legacy_xls_paths() {
        let error = load_entries(Path::new("legacy-template.xls")).expect_err("xls 模板必须被拒绝");
        assert!(
            matches!(error, ExcelError::Unsupported(ref message) if message.contains("not supported")),
            "unexpected error: {error}"
        );
    }

    /// 对应 Java：`ExcelTemplateWriter` 状态机在未 finish 时仍可追加行/标量。
    #[test]
    fn stateful_writer_accepts_rows_and_scalar_before_finish() -> Result<()> {
        let directory = tempdir()?;
        let template = xlsx_template(directory.path(), "stateful-template.xlsx")?;
        let output = directory.path().join("stateful-output.xlsx");
        let mut writer = ExcelTemplateWriter::new(&template, &output)?;
        writer.fill(&TemplateData::new().with("name", "stateful"))?;
        writer.write_rows([vec![CellValue::String("summary".to_owned())]])?;
        assert!(!writer.is_finished());
        writer.finish()?;
        assert!(writer.is_finished());
        assert!(output.exists());
        Ok(())
    }
}
