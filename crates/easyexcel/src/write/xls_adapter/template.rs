//! `EasyExcel` 单元格模型到 BIFF8 模板包的适配层。
//!
//! OLE/CFB 打开、BIFF 记录保留、偏移修复和序列化均由 `easyexcel-xls`
//! 实现；本模块只保留 Java `EasyExcel` `CellValue` 语义转换与兼容错误。

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::core::{CellValue, CoordinateData, ExcelError, HyperlinkType, Result};

use super::{
    Biff8Cell, Biff8Comment, Biff8HyperlinkKind, Biff8Merge, Biff8StyleRequest,
    Biff8StyleTable, GeneratedBiff8CellValue, apply_write_font,
};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 保留原 `EasyExcel` 路径的 BIFF8 模板包门面。
#[derive(Debug, Clone)]
pub(crate) struct Biff8TemplatePackage {
    inner: easyexcel_xls::biff8::Biff8TemplatePackage,
    rich_text_styles: Biff8StyleTable,
    emitted_rich_text_fonts: usize,
    rich_text_font_index_offset: u16,
}

impl Biff8TemplatePackage {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从 OLE `.xls` 字节加载模板。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_bytes(bytes)
            .map(Self::from_inner)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：`HSSFWorkbook(templateStream)` + 调用级 BIFF8 密码。 从字节加载模板。
    pub fn from_bytes_with_password(bytes: &[u8], password: Option<&str>) -> Result<Self> {
        let Some(password) = password else {
            return Self::from_bytes(bytes);
        };
        easyexcel_xls::biff8::Biff8TemplatePackage::from_bytes_with_password(bytes, Some(password))
            .map(Self::from_inner)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从文件加载模板。
    pub fn from_path(path: &Path) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_path(path)
            .map(Self::from_inner)
            .map_err(ExcelError::from)
    }

    fn from_inner(inner: easyexcel_xls::biff8::Biff8TemplatePackage) -> Self {
        let rich_text_font_index_offset = inner.next_custom_font_index().saturating_sub(6);
        Self {
            inner,
            rich_text_styles: Biff8StyleTable::default(),
            emitted_rich_text_fonts: 0,
            rich_text_font_index_offset,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表名称。
    #[must_use]
    pub fn sheet_names(&self) -> Vec<String> {
        self.inner.sheet_names()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表下一可追加行。
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        self.inner
            .next_row_for_sheet(sheet_name)
            .map_err(ExcelError::from)
    }

    /// Ensures a worksheet exists; creates an empty one when the name is new.
    ///
    /// # Errors
    ///
    /// Returns a format error when the workbook metadata cannot be updated.
    pub fn ensure_sheet(&mut self, sheet_name: &str) -> Result<()> {
        self.inner
            .ensure_sheet(sheet_name)
            .map(|_| ())
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从当前最后一行后追加稀疏行。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
    ) -> Result<u32> {
        let first_row = self.next_row_for_sheet(sheet_name)?;
        let mut mapped_rows = Vec::with_capacity(rows.len());
        let mut decorations = Vec::new();
        for (row_offset, row) in rows.iter().enumerate() {
            let row_index = first_row.saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX));
            let mut mapped = Vec::with_capacity(row.len());
            for (column, value) in row {
                mapped.push((*column, self.template_cell(value)?));
                let formatting_runs = self.comment_formatting_runs(value)?;
                collect_template_decorations(
                    &mut decorations,
                    row_index,
                    *column,
                    value,
                    formatting_runs,
                )?;
            }
            mapped_rows.push(mapped);
        }
        self.flush_rich_text_fonts()?;
        let next_row = self.inner
            .append_rows(sheet_name, &mapped_rows)
            .map_err(ExcelError::from)?;
        let mut comments = Vec::new();
        for decoration in decorations {
            match decoration {
                TemplateDecoration::Hyperlink {
                    first_row,
                    last_row,
                    first_col,
                    last_col,
                    address,
                    label,
                    kind,
                } => self
                    .inner
                    .add_hyperlink_range(
                        sheet_name, first_row, last_row, first_col, last_col, address, label, kind,
                    )
                    .map_err(ExcelError::from)?,
                TemplateDecoration::Comment(comment) => comments.push(comment),
            }
        }
        self.inner
            .add_comments(sheet_name, &comments)
            .map_err(ExcelError::from)?;
        Ok(next_row)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 添加合并区域。
    pub fn add_merge_range(&mut self, sheet_name: &str, range: Biff8Merge) -> Result<()> {
        self.inner
            .add_merge_range(sheet_name, range)
            .map_err(ExcelError::from)
    }

    /// 将 Handler 提交的后端中立修改应用到原位 BIFF8 模板。
    ///
    /// `SetCell` 与 `AddMerge` 直接更新保留记录模型；装饰型单元格继续写出
    /// HLINK/comment 元数据；图表、保护和对象表修改由 BIFF8 模板包原位完成。
    pub fn apply_mutations(
        &mut self,
        plan: &crate::context::write_mutation_plan::WriteMutationPlan,
    ) -> Result<()> {
        for mutation in plan.snapshot()? {
            match mutation {
                crate::context::write_mutation::WriteMutation::SetCell {
                    sheet_name,
                    row_index,
                    column_index,
                    value,
                } => {
                    let cell = self.template_cell(&value)?;
                    self.inner
                        .set_cell(&sheet_name, row_index, usize::from(column_index), &cell)
                        .map_err(ExcelError::from)?;
                    let mut decorations = Vec::new();
                    let formatting_runs = self.comment_formatting_runs(&value)?;
                    collect_template_decorations(
                        &mut decorations,
                        row_index,
                        usize::from(column_index),
                        &value,
                        formatting_runs,
                    )?;
                    self.flush_rich_text_fonts()?;
                    let mut comments = Vec::new();
                    for decoration in decorations {
                        match decoration {
                            TemplateDecoration::Hyperlink {
                                first_row,
                                last_row,
                                first_col,
                                last_col,
                                address,
                                label,
                                kind,
                            } => self
                                .inner
                                .add_hyperlink_range(
                                    &sheet_name,
                                    first_row,
                                    last_row,
                                    first_col,
                                    last_col,
                                    address,
                                    label,
                                    kind,
                                )
                                .map_err(ExcelError::from)?,
                            TemplateDecoration::Comment(comment) => comments.push(comment),
                        }
                    }
                    self.inner
                        .add_comments(&sheet_name, &comments)
                        .map_err(ExcelError::from)?;
                }
                crate::context::write_mutation::WriteMutation::AddMerge { sheet_name, range } => {
                    self.add_merge_range(
                        &sheet_name,
                        Biff8Merge {
                            first_row: u16::try_from(range.first_row).map_err(|_| {
                                ExcelError::Format("BIFF8 merge row exceeds 65535".to_owned())
                            })?,
                            last_row: u16::try_from(range.last_row).map_err(|_| {
                                ExcelError::Format("BIFF8 merge row exceeds 65535".to_owned())
                            })?,
                            first_col: u8::try_from(range.first_column).map_err(|_| {
                                ExcelError::Format("BIFF8 merge column exceeds 255".to_owned())
                            })?,
                            last_col: u8::try_from(range.last_column).map_err(|_| {
                                ExcelError::Format("BIFF8 merge column exceeds 255".to_owned())
                            })?,
                        },
                    )?;
                }
                crate::context::write_mutation::WriteMutation::AddChart(chart) => {
                    let sheet_names = self.sheet_names();
                    let mut book = super::Biff8Book::default();
                    book.sheets.extend(
                        sheet_names
                            .into_iter()
                            .map(super::Biff8Sheet::new),
                    );
                    crate::write::excel_writer_core::add_biff8_chart(
                        &mut book,
                        &chart,
                    )?;
                    let target = book
                        .sheets
                        .iter_mut()
                        .find(|sheet| sheet.name == chart.sheet_name)
                        .ok_or_else(|| ExcelError::SheetNotFound(chart.sheet_name.clone()))?;
                    self.inner
                        .add_charts(&chart.sheet_name, &std::mem::take(&mut target.charts))
                        .map_err(ExcelError::from)?;
                }
                crate::context::write_mutation::WriteMutation::ProtectSheet {
                    sheet_name,
                    password,
                } => self
                    .inner
                    .protect_sheet(&sheet_name, &password)
                    .map_err(ExcelError::from)?,
                crate::context::write_mutation::WriteMutation::RemoveComment {
                    sheet_name,
                    row_index,
                    column_index,
                } => {
                    self.inner
                        .remove_comment(&sheet_name, row_index, usize::from(column_index))
                        .map_err(ExcelError::from)?;
                }
            }
        }
        Ok(())
    }

    /// 使用完整 `CellValue` 语义替换 BIFF8 标量占位符。
    ///
    /// 占位符定位和样式保留由 `easyexcel-xls` 完成；本适配层只转换值，并在
    /// 引擎返回的最终坐标上补写 HLINK、NOTE/TXO 等独立 BIFF8 记录。
    pub fn replace_scalar_cell_values_on_sheet(
        &mut self,
        sheet_name: Option<&str>,
        values: &BTreeMap<String, CellValue>,
    ) -> Result<usize> {
        let snapshot = self.clone();
        let result = (|| -> Result<usize> {
            let mut cells = BTreeMap::new();
            for (key, value) in values {
                cells.insert(key.clone(), self.template_cell(value)?);
            }
            self.flush_rich_text_fonts()?;
            let placements = self
                .inner
                .replace_scalar_cells_on_sheet(sheet_name, &cells)
                .map_err(ExcelError::from)?;
            let mut decorations = Vec::new();
            for (physical_sheet, row, column, key) in &placements {
                let Some(value) = values.get(key) else {
                    continue;
                };
                let formatting_runs = self.comment_formatting_runs(value)?;
                let mut cell_decorations = Vec::new();
                collect_template_decorations(
                    &mut cell_decorations,
                    u32::from(*row),
                    usize::from(*column),
                    value,
                    formatting_runs,
                )?;
                decorations.extend(
                    cell_decorations
                        .into_iter()
                        .map(|decoration| (physical_sheet.clone(), decoration)),
                );
            }
            self.flush_rich_text_fonts()?;
            self.apply_template_decorations(decorations)?;
            Ok(placements.len())
        })();
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    /// 对应 Java：`HSSFWorkbook#write`。序列化为 OLE/BIFF8 字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.inner.to_bytes().map_err(ExcelError::from)
    }

    /// 按密码与 VBA 策略序列化，供 facade 将现有 BIFF8 引擎输出到任意目标。
    pub(crate) fn to_bytes_with_password_and_macro_policy(
        &self,
        password: Option<&str>,
        policy: &crate::Biff8MacroPolicy,
    ) -> Result<Vec<u8>> {
        self.inner
            .to_bytes_with_password_and_macro_policy(password, policy)
            .map_err(ExcelError::from)
    }

    /// 使用完整 `CellValue` 语义执行 BIFF8 集合占位符填充。
    pub fn fill_collection_cell_values(
        &mut self,
        sheet_name: Option<&str>,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, CellValue>],
        horizontal: bool,
        force_new_row: bool,
        auto_style: bool,
    ) -> Result<usize> {
        let snapshot = self.clone();
        let result = (|| -> Result<usize> {
            let mut cells = Vec::with_capacity(rows.len());
            for row in rows {
                let mut mapped = BTreeMap::new();
                for (key, value) in row {
                    mapped.insert(key.clone(), self.template_cell(value)?);
                }
                cells.push(mapped);
            }
            self.flush_rich_text_fonts()?;
            let placements = self
                .inner
                .fill_collection_cells(
                    sheet_name,
                    collection_name,
                    &cells,
                    horizontal,
                    force_new_row,
                    auto_style,
                )
                .map_err(ExcelError::from)?;
            let mut decorations = Vec::new();
            for (physical_sheet, row, column, input_row, key) in &placements {
                let Some(value) = rows.get(*input_row).and_then(|values| values.get(key)) else {
                    continue;
                };
                let formatting_runs = self.comment_formatting_runs(value)?;
                let mut cell_decorations = Vec::new();
                collect_template_decorations(
                    &mut cell_decorations,
                    u32::from(*row),
                    usize::from(*column),
                    value,
                    formatting_runs,
                )?;
                decorations.extend(
                    cell_decorations
                        .into_iter()
                        .map(|decoration| (physical_sheet.clone(), decoration)),
                );
            }
            self.flush_rich_text_fonts()?;
            self.apply_template_decorations(decorations)?;
            Ok(placements.len())
        })();
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 保存到文件。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.inner.save_to_path(path).map_err(ExcelError::from)
    }

    /// 对应 Java：`HSSFWorkbook#write` + BIFF8 密码。 保存到文件。
    pub fn save_to_path_with_password(&self, path: &Path, password: Option<&str>) -> Result<()> {
        self.inner
            .save_to_path_with_password(path, password)
            .map_err(ExcelError::from)
    }

    /// 按密码与 VBA 策略保存 BIFF8 模板。
    pub fn save_to_path_with_password_and_macro_policy(
        &self,
        path: &Path,
        password: Option<&str>,
        policy: &crate::Biff8MacroPolicy,
    ) -> Result<()> {
        let bytes = self
            .inner
            .to_bytes_with_password_and_macro_policy(password, policy)
            .map_err(ExcelError::from)?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes).map_err(ExcelError::from)
    }

    /// 按密码与 VBA 策略保存到调用方输出流。
    pub fn save_to_writer_with_password_and_macro_policy(
        &self,
        output: &mut dyn Write,
        password: Option<&str>,
        policy: &crate::Biff8MacroPolicy,
    ) -> Result<()> {
        let bytes = self
            .inner
            .to_bytes_with_password_and_macro_policy(password, policy)
            .map_err(ExcelError::from)?;
        output.write_all(&bytes)?;
        output.flush()?;
        Ok(())
    }

    fn template_cell(&mut self, value: &CellValue) -> Result<Biff8Cell> {
        if let CellValue::Comment { value, .. }
        | CellValue::CommentWithMetadata { value, .. } = value
        {
            return self.template_cell(value);
        }
        if matches!(value, CellValue::Images { .. } | CellValue::Image(_)) {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support images until BIFF8 Workbook drawing records are implemented"
                    .to_owned(),
            ));
        }
        let CellValue::RichText(rich) = value else {
            return cell_value_to_template_cell(value);
        };
        let runs = self.resolve_rich_text_runs(rich)?;
        self.flush_rich_text_fonts()?;
        Ok(GeneratedBiff8CellValue::RichText {
            text: rich.text_string().to_owned(),
            runs,
        }
        .into_cell())
    }

    fn comment_formatting_runs(&mut self, value: &CellValue) -> Result<Option<Vec<(u16, u16)>>> {
        let CellValue::CommentWithMetadata { comment, .. } = value else {
            return Ok(None);
        };
        let runs = comment
            .get_rich_text_string_data()
            .map(|rich| self.resolve_rich_text_runs(rich))
            .transpose()?;
        if runs.as_ref().is_some_and(|runs| runs.len() > 8_190) {
            return Err(ExcelError::Unsupported(
                "legacy XLS comment rich text exceeds the TXO 65535-byte formatting limit"
                    .to_owned(),
            ));
        }
        Ok(runs)
    }

    fn resolve_rich_text_runs(
        &mut self,
        rich: &crate::RichTextStringData,
    ) -> Result<Vec<(u16, u16)>> {
        let intervals = rich
            .interval_fonts()
            .iter()
            .map(|interval| (interval.start_index(), interval.end_index()))
            .collect::<Vec<_>>();
        let segments = easyexcel_model::segment_utf16_text(rich.text_string(), &intervals)
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        let mut runs = Vec::new();
        let mut utf16_start = 0usize;
        let mut previous_font = None;
        for segment in segments {
            let font = segment.interval_index.map_or(rich.write_font(), |index| {
                Some(rich.interval_fonts()[index].write_font())
            });
            let font_index = font.map_or(0, |font| {
                let mut request = Biff8StyleRequest::default();
                apply_write_font(&mut request, font);
                let allocated = self.rich_text_styles.resolve_font_index(&request);
                if allocated == 0 {
                    0
                } else {
                    allocated.saturating_add(self.rich_text_font_index_offset)
                }
            });
            if previous_font != Some(font_index) {
                runs.push((
                    u16::try_from(utf16_start).map_err(|_| {
                        ExcelError::Format(
                            "BIFF8 rich text exceeds 65535 UTF-16 units".to_owned(),
                        )
                    })?,
                    font_index,
                ));
                previous_font = Some(font_index);
            }
            utf16_start = utf16_start.saturating_add(segment.text.encode_utf16().count());
        }
        Ok(runs)
    }

    fn flush_rich_text_fonts(&mut self) -> Result<()> {
        let fonts = self.rich_text_styles.custom_fonts();
        let new_fonts = fonts
            .get(self.emitted_rich_text_fonts..)
            .ok_or_else(|| ExcelError::Format("BIFF8 rich-text font allocator regressed".to_owned()))?;
        self.inner
            .append_custom_fonts(new_fonts)
            .map_err(ExcelError::from)?;
        self.emitted_rich_text_fonts = fonts.len();
        Ok(())
    }

    fn apply_template_decorations(
        &mut self,
        decorations: Vec<(String, TemplateDecoration)>,
    ) -> Result<()> {
        let mut comments = BTreeMap::<String, Vec<Biff8Comment>>::new();
        for (sheet_name, decoration) in decorations {
            match decoration {
                TemplateDecoration::Hyperlink {
                    first_row,
                    last_row,
                    first_col,
                    last_col,
                    address,
                    label,
                    kind,
                } => self
                    .inner
                    .add_hyperlink_range(
                        &sheet_name,
                        first_row,
                        last_row,
                        first_col,
                        last_col,
                        address,
                        label,
                        kind,
                    )
                    .map_err(ExcelError::from)?,
                TemplateDecoration::Comment(comment) => {
                    comments.entry(sheet_name).or_default().push(comment);
                }
            }
        }
        for (sheet_name, comments) in comments {
            self.inner
                .add_comments(&sheet_name, &comments)
                .map_err(ExcelError::from)?;
        }
        Ok(())
    }
}

fn cell_value_to_template_cell(value: &CellValue) -> Result<Biff8Cell> {
    let mapped = match value {
        CellValue::Empty => GeneratedBiff8CellValue::Blank,
        CellValue::String(text)
        | CellValue::Error(text)
        | CellValue::Hyperlink { text, .. }
        | CellValue::HyperlinkWithMetadata { text, .. } => GeneratedBiff8CellValue::Text(text.clone()),
        CellValue::Formula(text) => GeneratedBiff8CellValue::Formula(text.clone()),
        CellValue::RichText(rich) => GeneratedBiff8CellValue::Text(rich.text_string().to_owned()),
        CellValue::Bool(flag) => GeneratedBiff8CellValue::Bool(*flag),
        CellValue::Int(number) => GeneratedBiff8CellValue::Number(
            #[allow(clippy::cast_precision_loss)]
            {
                *number as f64
            },
        ),
        CellValue::Float(number) => GeneratedBiff8CellValue::Number(*number),
        CellValue::Decimal(number) => {
            let numeric = crate::write::finite_decimal_f64(number, "BIFF8")?;
            if crate::write::decimal_integer_requires_text(number)? {
                GeneratedBiff8CellValue::Text(number.to_plain_string())
            } else {
                GeneratedBiff8CellValue::Number(numeric)
            }
        }
        CellValue::Date(date) => GeneratedBiff8CellValue::DateSerial(super::date_to_excel_serial(*date)),
        CellValue::DateTime(datetime) => GeneratedBiff8CellValue::DateTimeSerial(super::datetime_to_excel_serial(*datetime)),
        CellValue::Comment { value, .. }
        | CellValue::CommentWithMetadata { value, .. } => {
            return cell_value_to_template_cell(value);
        }
        CellValue::Images { .. } | CellValue::Image(_) => {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support images until BIFF8 Workbook drawing records are implemented"
                    .to_owned(),
            ));
        }
    };
    Ok(mapped.into_cell())
}

enum TemplateDecoration {
    Hyperlink {
        first_row: u32,
        last_row: u32,
        first_col: usize,
        last_col: usize,
        address: String,
        label: String,
        kind: Biff8HyperlinkKind,
    },
    Comment(Biff8Comment),
}

fn collect_template_decorations(
    target: &mut Vec<TemplateDecoration>,
    row: u32,
    column: usize,
    value: &CellValue,
    formatting_runs: Option<Vec<(u16, u16)>>,
) -> Result<()> {
    match value {
        CellValue::Hyperlink { url, text } => target.push(TemplateDecoration::Hyperlink {
            first_row: row,
            last_row: row,
            first_col: column,
            last_col: column,
            address: url.clone(),
            label: text.clone(),
            kind: Biff8HyperlinkKind::Url,
        }),
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            coordinates,
        } => {
            if let Some(kind) = template_hyperlink_kind(*hyperlink_type) {
                let (first_row, last_row, first_col, last_col) =
                    resolve_template_hyperlink_range(row, column, *coordinates)?;
                let address = kind.normalized_target(address);
                target.push(TemplateDecoration::Hyperlink {
                    first_row,
                    last_row,
                    first_col,
                    last_col,
                    address,
                    label: text.clone(),
                    kind,
                });
            }
        }
        CellValue::Comment { value, text } => {
            let row = u16::try_from(row)
                .map_err(|_| ExcelError::Format("BIFF8 comment row exceeds 65535".to_owned()))?;
            let column = u8::try_from(column)
                .map_err(|_| ExcelError::Format("BIFF8 comment column exceeds 255".to_owned()))?;
            if text.contains('\0') || text.encode_utf16().count() > usize::from(u16::MAX) {
                return Err(ExcelError::Format("BIFF8 comment text is invalid or too long".to_owned()));
            }
            target.push(TemplateDecoration::Comment(Biff8Comment::new(
                row,
                column,
                text.clone(),
                "easyexcel-rust",
            )));
            collect_template_decorations(
                target,
                u32::from(row),
                usize::from(column),
                value,
                None,
            )?;
        }
        CellValue::CommentWithMetadata { value, comment } => {
            let cell_row = u16::try_from(row)
                .map_err(|_| ExcelError::Format("BIFF8 comment row exceeds 65535".to_owned()))?;
            let cell_column = u8::try_from(column)
                .map_err(|_| ExcelError::Format("BIFF8 comment column exceeds 255".to_owned()))?;
            let text = comment.note_text();
            let author = comment.get_author().unwrap_or("").to_owned();
            let visible = comment.get_visible();
            if text.contains('\0')
                || author.contains('\0')
                || text.encode_utf16().count() > usize::from(u16::MAX)
                || author.encode_utf16().count() > usize::from(u16::MAX)
            {
                return Err(ExcelError::Format(
                    "BIFF8 comment text or author is invalid or too long".to_owned(),
                ));
            }
            let anchor = comment.get_anchor();
            let (first_row, last_row, first_col, last_col) =
                resolve_template_hyperlink_range(row, column, anchor.get_coordinates())?;
            let mut biff8_comment = Biff8Comment::new(cell_row, cell_column, text, author).with_anchor(
                u16::try_from(first_row).map_err(|_| {
                    ExcelError::Format("BIFF8 comment first row exceeds 65535".to_owned())
                })?,
                u8::try_from(first_col).map_err(|_| {
                    ExcelError::Format("BIFF8 comment first column exceeds 255".to_owned())
                })?,
                u16::try_from(last_row.saturating_add(1)).map_err(|_| {
                    ExcelError::Format("BIFF8 comment last row exceeds 65535".to_owned())
                })?,
                u8::try_from(last_col.saturating_add(1)).map_err(|_| {
                    ExcelError::Format("BIFF8 comment last column exceeds 255".to_owned())
                })?,
                anchor.get_top().map(template_comment_offset),
                anchor.get_right().map(template_comment_offset),
                anchor.get_bottom().map(template_comment_offset),
                anchor.get_left().map(template_comment_offset),
            );
            if let Some(formatting_runs) = formatting_runs {
                biff8_comment = biff8_comment.with_formatting_runs(formatting_runs);
            }
            if let Some(visible) = visible {
                biff8_comment = biff8_comment.with_visible(visible);
            }
            target.push(TemplateDecoration::Comment(biff8_comment));
            collect_template_decorations(
                target,
                u32::from(cell_row),
                usize::from(cell_column),
                value,
                None,
            )?;
        }
        CellValue::Images { .. } | CellValue::Image(_) => {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support images until BIFF8 Workbook drawing records are implemented"
                    .to_owned(),
            ));
        }
        _ => {}
    }
    Ok(())
}

const fn template_comment_offset(pixels: u32) -> u16 {
    let emu = pixels.saturating_mul(9_525);
    if emu > u16::MAX as u32 { u16::MAX } else { emu as u16 }
}

const fn template_hyperlink_kind(value: HyperlinkType) -> Option<Biff8HyperlinkKind> {
    match value {
        HyperlinkType::None => None,
        HyperlinkType::Url => Some(Biff8HyperlinkKind::Url),
        HyperlinkType::Document => Some(Biff8HyperlinkKind::Document),
        HyperlinkType::Email => Some(Biff8HyperlinkKind::Email),
        HyperlinkType::File => Some(Biff8HyperlinkKind::File),
    }
}

fn resolve_template_hyperlink_range(
    row: u32,
    column: usize,
    coordinates: CoordinateData,
) -> Result<(u32, u32, usize, usize)> {
    let column = u32::try_from(column)
        .map_err(|_| ExcelError::Format("BIFF8 hyperlink column exceeds u32".to_owned()))?;
    let resolve = |current: u32, absolute: Option<u32>, relative: Option<i32>| -> Result<u32> {
        let current = i64::from(current);
        let value = absolute.map_or(current + i64::from(relative.unwrap_or(0)), i64::from);
        u32::try_from(value)
            .map_err(|_| ExcelError::Format("BIFF8 hyperlink coordinate is negative".to_owned()))
    };
    let first_row = resolve(
        row,
        coordinates.get_first_row_index(),
        coordinates.get_relative_first_row_index(),
    )?;
    let last_row = resolve(
        row,
        coordinates.get_last_row_index(),
        coordinates.get_relative_last_row_index(),
    )?;
    let first_col = resolve(
        column,
        coordinates.get_first_column_index().map(u32::from),
        coordinates.get_relative_first_column_index(),
    )?;
    let last_col = resolve(
        column,
        coordinates.get_last_column_index().map(u32::from),
        coordinates.get_relative_last_column_index(),
    )?;
    Ok((
        first_row,
        last_row,
        usize::try_from(first_col).unwrap_or(usize::MAX),
        usize::try_from(last_col).unwrap_or(usize::MAX),
    ))
}
