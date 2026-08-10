//! 状态化 OOXML 模板写入器与 XLSX 包读写（fill 生命周期）。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter`（fill 生命周期）

use std::collections::BTreeMap;
#[cfg(test)]
use std::io::Cursor;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::core::{CellValue, ExcelError, Result};
use crate::write::ExcelOutputStream;

use crate::template::fill_engine::{
    append_rows_to_sheet_with_decorations, replace_collection_fills_in_sheet_with_decorations,
    replace_scalar_cells_in_sheet_with_decorations,
};
use crate::template::sheet_fill_state::{
    PendingCollectionFill, PendingSheetFill, ResolvedSheetFill,
};
use crate::template::template_entry::TemplateEntry;
use crate::template::template_output::TemplateOutput;
#[cfg(test)]
use crate::template::template_output::WriteSeek;
use crate::{FillConfig, FillDirection, FillWrapper, MergeRange, TemplateData, TemplateSheet};

/// Stateful OOXML template writer matching Java `ExcelWriter.fill` lifecycle.
///
/// Scalar values and collection fills are accumulated against one loaded XLSX
/// package. Repeated collection fills with the same prefix append at the prior
/// fill position instead of reopening the original template.
/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub struct ExcelTemplateWriter<'a> {
    pub(crate) output: TemplateOutput<'a>,
    pub(crate) entries: Vec<TemplateEntry>,
    pub(crate) sheets: Vec<PendingSheetFill>,
    pub(crate) next_collection_order: usize,
    pub(crate) finished: bool,
    pub(crate) auto_close_stream: bool,
    /// OOXML package output password. Encryption itself remains owned by
    /// `easyexcel-xlsx`; BIFF8 templates use their dedicated backend.
    pub(crate) package_password: Option<String>,
    pub(crate) collection_column_styles: BTreeMap<usize, u32>,
    /// Stateful BIFF8 backend when the template is an OLE `.xls` workbook.
    pub(crate) xls: Option<crate::write::xls_adapter::Biff8TemplatePackage>,
}

impl std::fmt::Debug for ExcelTemplateWriter<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self.output {
            TemplateOutput::Path(_) => "path",
            TemplateOutput::Borrowed(_) => "borrowed stream",
            TemplateOutput::Owned(_) => "owned stream",
            TemplateOutput::Managed { .. } => "managed writer stream",
        };
        formatter
            .debug_struct("ExcelTemplateWriter")
            .field("output", &output)
            .field("entries", &self.entries)
            .field("sheets", &self.sheets)
            .field("finished", &self.finished)
            .field("auto_close_stream", &self.auto_close_stream)
            .field("xls", &self.xls.is_some())
            .finish()
    }
}

impl ExcelTemplateWriter<'static> {
    /// Loads a template package for stateful filling.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn new(template: impl AsRef<Path>, output: impl Into<PathBuf>) -> Result<Self> {
        Self::from_template_path(TemplateOutput::Path(output.into()), template.as_ref())
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
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn from_reader<R>(mut template: R, output: impl Into<PathBuf>) -> Result<Self>
    where
        R: Read,
    {
        let mut bytes = Vec::new();
        template.read_to_end(&mut bytes)?;
        Self::from_template_bytes(TemplateOutput::Path(output.into()), &bytes)
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
            package_password: None,
            collection_column_styles: BTreeMap::new(),
            xls: None,
        }
    }

    fn from_xls(
        output: TemplateOutput<'a>,
        xls: crate::write::xls_adapter::Biff8TemplatePackage,
    ) -> Self {
        Self {
            output,
            entries: Vec::new(),
            sheets: Vec::new(),
            next_collection_order: 0,
            finished: false,
            auto_close_stream: true,
            package_password: None,
            collection_column_styles: BTreeMap::new(),
            xls: Some(xls),
        }
    }

    fn from_template_path(output: TemplateOutput<'a>, template: &Path) -> Result<Self> {
        if easyexcel_io::Format::detect_path(template)? == easyexcel_io::Format::Xls {
            return Ok(Self::from_xls(
                output,
                crate::write::xls_adapter::Biff8TemplatePackage::from_path(template)?,
            ));
        }
        Ok(Self::from_entries(output, load_entries(template)?))
    }

    pub(crate) fn from_template_bytes(output: TemplateOutput<'a>, bytes: &[u8]) -> Result<Self> {
        match easyexcel_io::Format::from_magic(bytes) {
            easyexcel_io::Format::Xls => Ok(Self::from_xls(
                output,
                crate::write::xls_adapter::Biff8TemplatePackage::from_bytes(bytes)?,
            )),
            easyexcel_io::Format::Xlsx => Ok(Self::from_entries(
                output,
                load_entries_from_reader(Box::new(std::io::Cursor::new(bytes.to_vec())))?,
            )),
            _ => Err(ExcelError::Unsupported(
                "template must be an XLSX or BIFF8 XLS workbook".to_owned(),
            )),
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
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn to_writer<W>(template: impl AsRef<Path>, output: &'a mut W) -> Result<Self>
    where
        W: Write,
    {
        Self::from_template_path(TemplateOutput::Borrowed(output), template.as_ref())
    }

    /// Loads a stream template and writes to a caller-owned output stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn from_reader_to_writer<R, W>(mut template: R, output: &'a mut W) -> Result<Self>
    where
        R: Read,
        W: Write,
    {
        let mut bytes = Vec::new();
        template.read_to_end(&mut bytes)?;
        Self::from_template_bytes(TemplateOutput::Borrowed(output), &bytes)
    }

    /// Loads a path template and writes to an explicitly closeable stream.
    ///
    /// Keep a clone of `output` to observe Java-compatible close state after
    /// [`Self::finish`].
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn to_output_stream<W>(
        template: impl AsRef<Path>,
        output: ExcelOutputStream<W>,
    ) -> Result<Self>
    where
        W: Write + 'a,
    {
        Self::from_template_path(TemplateOutput::Owned(Box::new(output)), template.as_ref())
    }

    /// Loads a stream template and writes to an explicitly closeable stream.
    ///
    /// # Errors
    ///
    /// Returns an I/O or OOXML package error when the template cannot be read.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn from_reader_to_output_stream<R, W>(
        mut template: R,
        output: ExcelOutputStream<W>,
    ) -> Result<Self>
    where
        R: Read,
        W: Write + 'a,
    {
        let mut bytes = Vec::new();
        template.read_to_end(&mut bytes)?;
        Self::from_template_bytes(TemplateOutput::Owned(Box::new(output)), &bytes)
    }

    /// Controls whether an owned output stream is closed by [`Self::finish`].
    ///
    /// The default is `true`, matching Java `EasyExcel`. Borrowed writers always
    /// remain caller-owned regardless of this setting.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub const fn auto_close_stream(mut self, enabled: bool) -> Self {
        self.auto_close_stream = enabled;
        self
    }

    /// 配置 OOXML 模板产物的工作簿密码。
    ///
    /// 对应 Java：`WriteWorkbook#getPassword()`；仅由统一 builder wiring
    /// 调用，具体 ECMA-376 加密由 `easyexcel-xlsx` 承载。
    pub(crate) fn set_package_password(&mut self, password: Option<String>) {
        self.package_password = password;
    }

    /// 将 facade writer 已持有的真实输出目标移交给模板引擎。
    ///
    /// 模板解析完成后才调用，避免解析失败时提前夺走调用方输出流。
    pub(crate) fn redirect_output(
        &mut self,
        output: TemplateOutput<'a>,
        auto_close_stream: bool,
    ) {
        self.output = output;
        self.auto_close_stream = auto_close_stream;
    }

    /// 异常结束且禁止输出工作簿时，只完成目标生命周期，不写模板内容。
    pub(crate) fn discard_output(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        discard_template_output(&mut self.output, self.auto_close_stream)?;
        self.finished = true;
        Ok(())
    }

    /// Accumulates scalar `{key}` values for this workbook.
    ///
    /// Later fills replace earlier values for the same key, matching Java map
    /// filling before the workbook is finalized.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn fill(&mut self, data: &TemplateData) -> Result<&mut Self> {
        self.fill_on_sheet(&TemplateSheet::first(), data)
    }

    /// Accumulates scalar `{key}` values for one selected worksheet.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn fill_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        data: &TemplateData,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        if self.xls.is_some() {
            let sheet_name = self.resolve_xls_sheet_name(sheet)?;
            self.xls
                .as_mut()
                .expect("XLS backend checked")
                .replace_scalar_cell_values_on_sheet(Some(&sheet_name), data.values())?;
            return Ok(self);
        }
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
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn fill_list(&mut self, data: &FillWrapper, config: FillConfig) -> Result<&mut Self> {
        self.fill_list_on_sheet(&TemplateSheet::first(), data, config)
    }

    /// Accumulates a collection fill for one selected worksheet.
    ///
    /// # Errors
    ///
    /// Returns an error after the writer has finished.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
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
        if self.xls.is_some() {
            let sheet_name = self.resolve_xls_sheet_name(sheet)?;
            let rows = data
                .rows()
                .iter()
                .map(|row| row.values().clone())
                .collect::<Vec<_>>();
            self.xls
                .as_mut()
                .expect("XLS backend checked")
                .fill_collection_cell_values(
                    Some(&sheet_name),
                    data.name(),
                    &rows,
                    matches!(config.effective_direction(), FillDirection::Horizontal),
                    config.effective_force_new_row(),
                    config.effective_auto_style(),
                )?;
            return Ok(self);
        }
        let order = self.next_collection_order;
        self.next_collection_order = self.next_collection_order.saturating_add(1);
        let column_styles = self.collection_column_styles.clone();
        let state = self.sheet_state_mut(sheet);
        state.collections.push(PendingCollectionFill {
            wrapper: data.clone(),
            config,
            order,
            column_styles,
        });
        Ok(self)
    }

    /// 合并由写注解和 handler 编译出的样式，并将其应用到后续集合填充。
    pub(crate) fn import_collection_styles(
        &mut self,
        compiled_xlsx: &[u8],
        columns: &[usize],
    ) -> Result<()> {
        if self.xls.is_some() {
            return Ok(());
        }
        if columns.is_empty() {
            return Ok(());
        }
        let worksheet = worksheet_path(&self.entries, &TemplateSheet::first())?;
        let base_by_column =
            easyexcel_xlsx::collection_column_style_indexes(&self.entries, &worksheet);
        let base_indexes = columns
            .iter()
            .map(|column| base_by_column.get(column).copied().unwrap_or(0))
            .collect::<Vec<_>>();
        // 样式表和 worksheet 引用必须原子提交；映射失败时保留可重试的模板包。
        let package = easyexcel_xlsx::OoxmlPackage::from_entries(self.entries.clone());
        let mut package = easyexcel_xlsx::OoxmlTemplatePackage::from_package(package);
        let mapped = package
            .import_compiled_styles_onto(compiled_xlsx, &base_indexes)
            .map_err(ExcelError::from)?;
        self.entries = package.into_package().into_entries();
        self.collection_column_styles = columns.iter().copied().zip(mapped).collect();
        Ok(())
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
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
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
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn write_rows_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        rows: impl IntoIterator<Item = Vec<CellValue>>,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        if self.xls.is_some() {
            let sheet_name = self.resolve_xls_sheet_name(sheet)?;
            let rows = rows
                .into_iter()
                .map(|row| row.into_iter().enumerate().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            self.xls
                .as_mut()
                .expect("XLS backend checked")
                .append_rows(&sheet_name, &rows)?;
            return Ok(self);
        }
        self.sheet_state_mut(sheet).appended_rows.extend(rows);
        Ok(self)
    }

    /// 在模板包的指定工作表上立即增加一个绝对合并区域。
    ///
    /// 该修改在集合填充之前进入工作表 XML，因此后续 `forceNewRow`
    /// 引发的行迁移会像 Java POI `shiftRows` 一样同步更新合并引用。
    /// 对应 Java：`ExcelBuilderImpl#merge`。
    ///
    /// # Errors
    ///
    /// 工作表不存在、坐标非法或 OOXML 更新失败时返回错误。
    pub(crate) fn add_merge_on_sheet(
        &mut self,
        sheet: &TemplateSheet,
        range: MergeRange,
    ) -> Result<&mut Self> {
        self.ensure_open()?;
        if self.xls.is_some() {
            let sheet_name = self.resolve_xls_sheet_name(sheet)?;
            self.xls
                .as_mut()
                .expect("XLS backend checked")
                .add_merge_range(
                    &sheet_name,
                    crate::write::xls_adapter::Biff8Merge {
                        first_row: u16::try_from(range.first_row).map_err(|_| {
                            ExcelError::Format("BIFF8 supports at most 65536 rows".to_owned())
                        })?,
                        last_row: u16::try_from(range.last_row).map_err(|_| {
                            ExcelError::Format("BIFF8 supports at most 65536 rows".to_owned())
                        })?,
                        first_col: u8::try_from(range.first_column).map_err(|_| {
                            ExcelError::Format("BIFF8 supports at most 256 columns".to_owned())
                        })?,
                        last_col: u8::try_from(range.last_column).map_err(|_| {
                            ExcelError::Format("BIFF8 supports at most 256 columns".to_owned())
                        })?,
                    },
                )?;
            return Ok(self);
        }
        // 在快照上修改，只有工作表解析和布局更新均成功后才替换原包。
        let package = easyexcel_xlsx::OoxmlPackage::from_entries(self.entries.clone());
        let mut package = easyexcel_xlsx::OoxmlTemplatePackage::from_package(package);
        {
            let names = package.sheet_names().map_err(ExcelError::from)?;
            let name = match sheet {
                TemplateSheet::First => names.first(),
                TemplateSheet::Index(index) => names.get(*index),
                TemplateSheet::Name(name) => names.iter().find(|candidate| *candidate == name),
            }
            .ok_or_else(|| ExcelError::SheetNotFound(format!("template sheet {sheet:?}")))?;
            package
                .apply_sheet_layout(
                    name,
                    &[],
                    &[easyexcel_xlsx::xlsx::template_xml::TemplateMergeRange {
                        first_row: range.first_row,
                        last_row: range.last_row,
                        first_column: range.first_column,
                        last_column: range.last_column,
                    }],
                )
                .map_err(ExcelError::from)?;
        }
        self.entries = package.into_package().into_entries();
        Ok(self)
    }

    /// Writes the completed XLSX package. Repeated calls are no-ops.
    ///
    /// # Errors
    ///
    /// Returns an XML, ZIP, or output I/O error.
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
    pub fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        if let Some(xls) = self.xls.as_ref() {
            let bytes = xls.to_bytes()?;
            write_template_bytes_to_output(
                &mut self.output,
                &bytes,
                self.auto_close_stream,
            )?;
            self.finished = true;
            return Ok(());
        }
        // finish 可能在 XML、关系或批注部件合并阶段失败。所有模板替换都在
        // 工作副本上完成，成功后一次提交，失败时允许调用方修正后重试。
        let mut entries = self.entries.clone();
        let mut decorations = Vec::new();
        for sheet in self.resolved_sheet_fills()? {
            decorations.extend(
                replace_collection_fills_in_sheet_with_decorations(
                    &mut entries,
                    &sheet.worksheet,
                    &sheet.collections,
                )?
                .into_iter()
                .map(|placement| (sheet.worksheet.clone(), placement)),
            );
            decorations.extend(
                replace_scalar_cells_in_sheet_with_decorations(
                    &mut entries,
                    &sheet.worksheet,
                    &sheet.scalar,
                )?
                .into_iter()
                .map(|placement| (sheet.worksheet.clone(), placement)),
            );
            decorations.extend(
                append_rows_to_sheet_with_decorations(
                    &mut entries,
                    &sheet.worksheet,
                    &sheet.appended_rows,
                )?
                .into_iter()
                .map(|placement| (sheet.worksheet.clone(), placement)),
            );
        }
        if !decorations.is_empty() {
            let package = easyexcel_xlsx::OoxmlPackage::from_entries(entries);
            let mut package = easyexcel_xlsx::OoxmlTemplatePackage::from_package(package);
            let mut sheet_names: BTreeMap<String, String> = BTreeMap::new();
            for (worksheet, placement) in decorations {
                let sheet_name = if let Some(sheet_name) = sheet_names.get(&worksheet) {
                    sheet_name.clone()
                } else {
                    let sheet_name = package.sheet_name_by_worksheet_path(&worksheet)?;
                    sheet_names.insert(worksheet.clone(), sheet_name.clone());
                    sheet_name
                };
                match placement.decoration {
                    easyexcel_xlsx::TemplateDecoration::Comment(comment) => {
                        package.set_template_comment(
                            &sheet_name,
                            placement.row,
                            placement.column,
                            &comment,
                        )?;
                    }
                    easyexcel_xlsx::TemplateDecoration::Hyperlink(hyperlink) => {
                        package.set_template_hyperlink(
                            &sheet_name,
                            placement.row,
                            placement.column,
                            &hyperlink,
                        )?;
                    }
                    easyexcel_xlsx::TemplateDecoration::Image(image) => {
                        package.set_template_image(
                            &sheet_name,
                            placement.row,
                            placement.column,
                            &image,
                        )?;
                    }
                }
            }
            entries = package.into_package().into_entries();
        }
        write_entries_to_output_with_password(
            &mut self.output,
            &entries,
            self.auto_close_stream,
            self.package_password.as_deref(),
        )?;
        self.entries = entries;
        self.finished = true;
        Ok(())
    }

    /// Returns whether [`Self::finish`] has run.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.ExcelWriter。
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

    fn resolve_xls_sheet_name(&self, sheet: &TemplateSheet) -> Result<String> {
        let names = self
            .xls
            .as_ref()
            .ok_or_else(|| ExcelError::Format("template writer has no XLS backend".to_owned()))?
            .sheet_names();
        match sheet {
            TemplateSheet::First => names.first().cloned(),
            TemplateSheet::Index(index) => names.get(*index).cloned(),
            TemplateSheet::Name(name) => names.iter().find(|candidate| *candidate == name).cloned(),
        }
        .ok_or_else(|| ExcelError::SheetNotFound(format!("template sheet {sheet:?}")))
    }

    /// 对应 Java：com.alibaba.excel.ExcelWriter。
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

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn same_sheet(left: &TemplateSheet, right: &TemplateSheet) -> bool {
    left.as_engine_selector()
        .equivalent(right.as_engine_selector())
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
/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub fn fill_xlsx_template(template: &Path, output: &Path, data: &TemplateData) -> Result<()> {
    if easyexcel_io::Format::from_path(template) == Some(easyexcel_io::Format::Xls) {
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
/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub fn fill_xlsx_template_list(
    template: &Path,
    output: &Path,
    data: &FillWrapper,
    config: FillConfig,
) -> Result<()> {
    if easyexcel_io::Format::from_path(template) == Some(easyexcel_io::Format::Xls) {
        return fill_xls_template_list(template, output, data, config);
    }
    let mut writer = ExcelTemplateWriter::new(template, output)?;
    if !data.rows().is_empty() {
        writer.sheets[0].collections.push(PendingCollectionFill {
            wrapper: data.clone(),
            config,
            order: 0,
            column_styles: BTreeMap::new(),
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
    let mut pkg = crate::write::xls_adapter::Biff8TemplatePackage::from_path(template)?;
    pkg.replace_scalar_cell_values_on_sheet(None, data.values())?;
    pkg.save_to_path(output)
}

/// Replaces list placeholders in a BIFF8 `.xls` template.
fn fill_xls_template_list(
    template: &Path,
    output: &Path,
    data: &FillWrapper,
    config: FillConfig,
) -> Result<()> {
    let mut pkg = crate::write::xls_adapter::Biff8TemplatePackage::from_path(template)?;
    let rows = data
        .rows()
        .iter()
        .map(|row| row.values().clone())
        .collect::<Vec<_>>();
    let first_sheet = pkg.sheet_names().into_iter().next();
    pkg.fill_collection_cell_values(
        first_sheet.as_deref(),
        data.name(),
        &rows,
        matches!(config.effective_direction(), FillDirection::Horizontal),
        config.effective_force_new_row(),
        config.effective_auto_style(),
    )?;
    pkg.save_to_path(output)
}

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn load_entries(path: &Path) -> Result<Vec<TemplateEntry>> {
    // 这是 OOXML entry 级内部入口；公共 ExcelTemplateWriter 会在此之前
    // 选择 BIFF8 backend，因此不能把 XLS 能力误报为“尚未实现”。
    if easyexcel_io::Format::from_path(path) == Some(easyexcel_io::Format::Xls) {
        return Err(ExcelError::Format(
            "BIFF8 templates must be opened through ExcelTemplateWriter".to_owned(),
        ));
    }
    Ok(easyexcel_xlsx::OoxmlPackage::from_path(path)?.into_entries())
}

fn load_entries_from_reader(reader: Box<dyn Read + '_>) -> Result<Vec<TemplateEntry>> {
    Ok(easyexcel_xlsx::OoxmlPackage::from_stream(reader)?.into_entries())
}

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn worksheet_path(entries: &[TemplateEntry], sheet: &TemplateSheet) -> Result<String> {
    easyexcel_xlsx::worksheet_path(entries, sheet.as_engine_selector()).map_err(ExcelError::from)
}

#[cfg(test)]
pub(crate) use easyexcel_xlsx::{normalize_workbook_target, workbook_sheets, xml_elements};

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn write_entries(path: &Path, entries: &[TemplateEntry]) -> Result<()> {
    Ok(easyexcel_xlsx::OoxmlPackage::from_entries(entries.to_vec()).save_to_path(path)?)
}

pub(crate) fn write_template_bytes_to_output(
    output: &mut TemplateOutput<'_>,
    bytes: &[u8],
    auto_close_stream: bool,
) -> Result<()> {
    match output {
        TemplateOutput::Path(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, bytes).map_err(ExcelError::from)
        }
        TemplateOutput::Borrowed(writer) => {
            easyexcel_io::write_all_and_flush(*writer, bytes).map_err(ExcelError::from)
        }
        TemplateOutput::Owned(writer) => {
            easyexcel_io::write_all_and_flush(writer.as_mut(), bytes).map_err(ExcelError::from)?;
            if auto_close_stream {
                writer.close().map_err(ExcelError::from)?;
            }
            Ok(())
        }
        TemplateOutput::Managed { writer, close } => {
            easyexcel_io::write_all_and_flush(writer.as_mut(), bytes).map_err(ExcelError::from)?;
            if auto_close_stream
                && let Some(close) = close.take()
            {
                close().map_err(ExcelError::from)?;
            }
            Ok(())
        }
    }
}

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn write_entries_to_output(
    output: &mut TemplateOutput<'_>,
    entries: &[TemplateEntry],
    auto_close_stream: bool,
) -> Result<()> {
    match output {
        TemplateOutput::Path(path) => write_entries(path, entries),
        TemplateOutput::Borrowed(writer) => {
            let bytes = encode_entries(entries)?;
            easyexcel_io::write_all_and_flush(*writer, &bytes)?;
            Ok(())
        }
        TemplateOutput::Owned(writer) => {
            let write_result = encode_entries(entries).and_then(|bytes| {
                easyexcel_io::write_all_and_flush(writer.as_mut(), &bytes).map_err(ExcelError::from)
            });
            let close_result = if auto_close_stream {
                writer.close()
            } else {
                Ok(())
            };
            close_result.map_err(ExcelError::from)?;
            write_result
        }
        TemplateOutput::Managed { writer, close } => {
            let write_result = encode_entries(entries).and_then(|bytes| {
                easyexcel_io::write_all_and_flush(writer.as_mut(), &bytes).map_err(ExcelError::from)
            });
            let close_result = if auto_close_stream {
                close.take().map_or(Ok(()), |close| close())
            } else {
                Ok(())
            };
            close_result.map_err(ExcelError::from)?;
            write_result
        }
    }
}

pub(crate) fn discard_template_output(
    output: &mut TemplateOutput<'_>,
    auto_close_stream: bool,
) -> Result<()> {
    match output {
        TemplateOutput::Path(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::File::create(path)?;
            Ok(())
        }
        TemplateOutput::Borrowed(_) => Ok(()),
        TemplateOutput::Owned(writer) => {
            if auto_close_stream {
                writer.close().map_err(ExcelError::from)?;
            }
            Ok(())
        }
        TemplateOutput::Managed { close, .. } => {
            if auto_close_stream
                && let Some(close) = close.take()
            {
                close().map_err(ExcelError::from)?;
            }
            Ok(())
        }
    }
}

/// 将 OOXML entries 写入目标，并在请求时交由 XLSX 引擎包装为 ECMA-376
/// Agile Encryption CFB 容器。
fn write_entries_to_output_with_password(
    output: &mut TemplateOutput<'_>,
    entries: &[TemplateEntry],
    auto_close_stream: bool,
    password: Option<&str>,
) -> Result<()> {
    let Some(password) = password else {
        return write_entries_to_output(output, entries, auto_close_stream);
    };
    let plaintext = encode_entries(entries)?;
    let mut encrypted = std::io::Cursor::new(Vec::new());
    easyexcel_xlsx::encrypt_package_to(&plaintext, password, &mut encrypted)
        .map_err(ExcelError::from)?;
    write_template_bytes_to_output(output, encrypted.get_ref(), auto_close_stream)
}

/// 对应 Java：com.alibaba.excel.ExcelWriter。
pub(crate) fn encode_entries(entries: &[TemplateEntry]) -> Result<Vec<u8>> {
    Ok(easyexcel_xlsx::OoxmlPackage::from_entries(entries.to_vec()).to_bytes()?)
}

#[cfg(test)]
pub(crate) fn archive_output_bytes(writer: Box<dyn WriteSeek>) -> Result<Vec<u8>> {
    writer
        .into_any()
        .downcast::<Cursor<Vec<u8>>>()
        .map(|cursor| cursor.into_inner())
        .map_err(|_| ExcelError::Format("ZIP output buffer type changed".to_owned()))
}

#[cfg(test)]
pub(crate) fn write_entries_to(
    writer: Box<dyn WriteSeek>,
    entries: &[TemplateEntry],
) -> Result<Box<dyn WriteSeek>> {
    Ok(easyexcel_xlsx::OoxmlPackage::from_entries(entries.to_vec()).write_to(writer)?)
}

#[cfg(test)]
pub(crate) fn format_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use calamine::Reader;
    use std::fs::{self, File};
    use tempfile::tempdir;
    use zip::CompressionMethod;
    use zip::write::{SimpleFileOptions, ZipWriter};

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
    /// 用 `easyexcel-xls` 的 `Biff8Book` 生成带各类占位符的 `.xls` 模板：
    /// `{.name}`（未命名 `is_dot` 分支）、`{plain}`（普通分支）、
    /// `x{name}`（else 分支）、`{}`（空 key）、`{.}`（点号空 key）、
    /// `{missing}`（未命中 key），以及命名 wrapper 的 `{users.name}`（prefix 分支）。
    #[test]
    fn fill_xlsx_template_list_supports_biff8_xls_placeholders() -> Result<()> {
        let directory = tempdir()?;
        let template = directory.path().join("list-template.xls");
        let unnamed_output = directory.path().join("list-unnamed-output.xls");
        let named_output = directory.path().join("list-named-output.xls");

        let mut book = crate::write::xls_adapter::Biff8Book::default();
        {
            let sheet = book.sheet_mut("Sheet1");
            let text = |value: &str| {
                crate::write::xls_adapter::Biff8Cell::general(
                    crate::write::xls_adapter::Biff8Value::Text(value.to_owned()),
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
