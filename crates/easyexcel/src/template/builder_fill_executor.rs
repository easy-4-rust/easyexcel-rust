//! Bridges [`WriteFillExecutor`] to [`ExcelTemplateWriter`].
//!
//! Keeps template fill logic out of `easyexcel-writer` while letting
//! `ExcelBuilderImpl.fill` delegate to the same engine as
//! `EasyExcel::template_writer`.

use std::any::Any;
use std::path::{Path, PathBuf};

use crate::core::{
    CellValue, ExcelError, Result, WriteDirection, WriteFillConfig, WriteFillExecutor,
    WriteFillSheet,
};

use super::template_output::TemplateOutput;
use super::template_writer::write_template_bytes_to_output;
use crate::{
    ExcelTemplateWriter, FillConfig, FillDirection, FillWrapper, MergeRange, TemplateData,
    TemplateSheet,
};

/// Stateful template fill executor for [`crate::write::ExcelBuilderImpl`].
///
/// 对应 Java：`ExcelWriteFillExecutor` backed by the same loaded XLSX
/// package as [`ExcelTemplateWriter`].
pub struct BuilderFillExecutor {
    inner: Option<ExcelTemplateWriter<'static>>,
    xls: Option<crate::write::xls_adapter::Biff8TemplatePackage>,
    output: Option<TemplateOutput<'static>>,
    auto_close_stream: bool,
    password: Option<String>,
    write_excel_on_exception: bool,
    biff8_macro_policy: crate::Biff8MacroPolicy,
    finished: bool,
}

/// 由类型注解和写 handler 编译出的模板集合填充样式。
pub(crate) struct CompiledTemplateFillStyles {
    pub(crate) workbook: Vec<u8>,
    pub(crate) columns: Vec<usize>,
}

impl BuilderFillExecutor {
    /// 对应 Java：ExcelWriteFillExecutor。 Loads a template from path or bytes and prepares fill against `output`.
    ///
    /// # Errors
    ///
    /// Returns I/O or OOXML package errors when the template cannot be read.
    pub fn new(
        template_file: Option<PathBuf>,
        template_bytes: Option<Vec<u8>>,
        output: PathBuf,
    ) -> Result<Self> {
        Self::new_with_password(
            template_file,
            template_bytes,
            output,
            None,
            false,
            crate::Biff8MacroPolicy::Preserve,
        )
    }

    pub(crate) fn new_with_password(
        template_file: Option<PathBuf>,
        template_bytes: Option<Vec<u8>>,
        output: PathBuf,
        password: Option<String>,
        write_excel_on_exception: bool,
        biff8_macro_policy: crate::Biff8MacroPolicy,
    ) -> Result<Self> {
        let mut bytes = if let Some(path) = template_file {
            std::fs::read(path)?
        } else if let Some(bytes) = template_bytes {
            bytes
        } else {
            return Err(ExcelError::Unsupported(
                "with_template requires a template file or template bytes".to_owned(),
            ));
        };
        if easyexcel_xlsx::is_encrypted_ooxml(&bytes) {
            let password = password.as_deref().ok_or_else(|| {
                ExcelError::Unsupported(
                    "encrypted OOXML template requires a workbook password".to_owned(),
                )
            })?;
            bytes = easyexcel_xlsx::decrypt_package(&bytes, password).map_err(ExcelError::from)?;
        }
        if crate::write::xls_adapter::looks_like_xls(&bytes) {
            let xls = crate::write::xls_adapter::Biff8TemplatePackage::from_bytes_with_password(
                &bytes,
                password.as_deref(),
            )?;
            return Ok(Self {
                inner: None,
                xls: Some(xls),
                output: Some(TemplateOutput::Path(output)),
                auto_close_stream: true,
                password,
                write_excel_on_exception,
                biff8_macro_policy,
                finished: false,
            });
        }
        let mut inner =
            ExcelTemplateWriter::from_template_bytes(TemplateOutput::Path(output), &bytes)?;
        inner.set_package_password(password.clone());
        Ok(Self {
            inner: Some(inner),
            xls: None,
            output: None,
            auto_close_stream: true,
            password,
            write_excel_on_exception,
            biff8_macro_policy,
            finished: false,
        })
    }

    pub(crate) fn with_compiled_styles(
        template_file: Option<PathBuf>,
        template_bytes: Option<Vec<u8>>,
        output: PathBuf,
        styles: Option<CompiledTemplateFillStyles>,
        password: Option<String>,
        write_excel_on_exception: bool,
        biff8_macro_policy: crate::Biff8MacroPolicy,
    ) -> Result<Self> {
        let mut executor = Self::new_with_password(
            template_file,
            template_bytes,
            output,
            password,
            write_excel_on_exception,
            biff8_macro_policy,
        )?;
        if let (Some(styles), Some(inner)) = (styles, executor.inner.as_mut()) {
            inner.import_collection_styles(&styles.workbook, &styles.columns)?;
        }
        Ok(executor)
    }

    /// 对应 Java：ExcelWriteFillExecutor。 Loads a template file and writes to an existing path.
    ///
    /// # Errors
    ///
    /// Returns I/O or OOXML package errors when the template cannot be read.
    pub fn from_template_path(
        template: impl AsRef<Path>,
        output: impl Into<PathBuf>,
    ) -> Result<Self> {
        Self::new(Some(template.as_ref().to_path_buf()), None, output.into())
    }

    /// 将统一 `ExcelWriter` 的真实输出目标交给已完成模板解析的 executor。
    pub(crate) fn redirect_output(
        &mut self,
        output: TemplateOutput<'static>,
        auto_close_stream: bool,
    ) {
        self.auto_close_stream = auto_close_stream;
        if let Some(inner) = self.inner.as_mut() {
            inner.redirect_output(output, auto_close_stream);
        } else {
            self.output = Some(output);
        }
    }
}

/// 对应 Java：ExcelWriteFillExecutor。 Creates a boxed fill executor for facade wiring into [`ExcelBuilderImpl`](crate::write::ExcelBuilderImpl).
///
/// # Errors
///
/// Returns I/O or OOXML package errors when the template cannot be read.
pub fn create_builder_fill_executor(
    template_file: Option<PathBuf>,
    template_bytes: Option<Vec<u8>>,
    output: PathBuf,
) -> Result<Box<dyn WriteFillExecutor>> {
    Ok(Box::new(BuilderFillExecutor::new(
        template_file,
        template_bytes,
        output,
    )?))
}

pub(crate) fn create_builder_fill_executor_with_styles(
    template_file: Option<PathBuf>,
    template_bytes: Option<Vec<u8>>,
    output: PathBuf,
    styles: Option<CompiledTemplateFillStyles>,
    password: Option<String>,
    write_excel_on_exception: bool,
    biff8_macro_policy: crate::Biff8MacroPolicy,
) -> Result<Box<dyn WriteFillExecutor>> {
    Ok(Box::new(BuilderFillExecutor::with_compiled_styles(
        template_file,
        template_bytes,
        output,
        styles,
        password,
        write_excel_on_exception,
        biff8_macro_policy,
    )?))
}

impl WriteFillExecutor for BuilderFillExecutor {
    fn fill(
        &mut self,
        data: &dyn Any,
        fill_config: WriteFillConfig,
        sheet: WriteFillSheet,
    ) -> Result<()> {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "template writer already finished".to_owned(),
            ));
        }
        if let Some(xls) = self.xls.as_mut() {
            if let Some(scalar) = data.downcast_ref::<TemplateData>() {
                let sheet_name = resolve_xls_sheet_name(xls, &sheet)?;
                xls.replace_scalar_cell_values_on_sheet(Some(&sheet_name), scalar.values())?;
                return Ok(());
            }
            if let Some(collection) = data.downcast_ref::<FillWrapper>() {
                let rows = collection
                    .rows()
                    .iter()
                    .map(|row| row.values().clone())
                    .collect::<Vec<_>>();
                let sheet_name = if let Some(index) = sheet.sheet_index {
                    xls.sheet_names().get(index).cloned().ok_or_else(|| {
                        ExcelError::Format(format!("sheet index {index} does not exist"))
                    })?
                } else {
                    sheet.sheet_name.clone()
                };
                xls.fill_collection_cell_values(
                    Some(&sheet_name),
                    collection.name(),
                    &rows,
                    matches!(fill_config.direction, Some(WriteDirection::Horizontal)),
                    fill_config.force_new_row,
                    fill_config.auto_style,
                )?;
                return Ok(());
            }
            return Err(ExcelError::Format(format!(
                "fill data must be TemplateData or FillWrapper, got {}",
                std::any::type_name_of_val(data)
            )));
        }
        let inner = self.inner.as_mut().ok_or_else(|| {
            ExcelError::Format("template fill executor has no active backend".to_owned())
        })?;
        let template_sheet = to_template_sheet(&sheet);
        if let Some(scalar) = data.downcast_ref::<TemplateData>() {
            inner.fill_on_sheet(&template_sheet, scalar)?;
            return Ok(());
        }
        if let Some(collection) = data.downcast_ref::<FillWrapper>() {
            inner.fill_list_on_sheet(
                &template_sheet,
                collection,
                to_template_fill_config(fill_config),
            )?;
            return Ok(());
        }
        Err(ExcelError::Format(format!(
            "fill data must be TemplateData or FillWrapper, got {}",
            std::any::type_name_of_val(data)
        )))
    }

    fn write_rows(&mut self, rows: Vec<Vec<CellValue>>, sheet: WriteFillSheet) -> Result<()> {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "template writer already finished".to_owned(),
            ));
        }
        if let Some(xls) = self.xls.as_mut() {
            let sheet_name = resolve_xls_sheet_name(xls, &sheet)?;
            let sparse_rows = rows
                .into_iter()
                .map(|row| row.into_iter().enumerate().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            xls.append_rows(&sheet_name, &sparse_rows)?;
            return Ok(());
        }
        self.inner
            .as_mut()
            .ok_or_else(|| ExcelError::Format("template fill executor has no backend".to_owned()))?
            .write_rows_on_sheet(&to_template_sheet(&sheet), rows)?;
        Ok(())
    }

    fn add_merge(&mut self, range: MergeRange, sheet: WriteFillSheet) -> Result<()> {
        if self.finished {
            return Err(ExcelError::Unsupported(
                "template writer already finished".to_owned(),
            ));
        }
        if let Some(xls) = self.xls.as_mut() {
            let sheet_name = resolve_xls_sheet_name(xls, &sheet)?;
            xls.add_merge_range(
                &sheet_name,
                easyexcel_xls::biff8::Biff8Merge {
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
            return Ok(());
        }
        self.inner
            .as_mut()
            .ok_or_else(|| ExcelError::Format("template fill executor has no backend".to_owned()))?
            .add_merge_on_sheet(&to_template_sheet(&sheet), range)?;
        Ok(())
    }

    fn finish(&mut self, on_exception: bool) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        if on_exception && !self.write_excel_on_exception {
            if let Some(inner) = self.inner.as_mut() {
                inner.discard_output()?;
            } else {
                let output = self.output.as_mut().ok_or_else(|| {
                    ExcelError::Format("template fill executor has no output target".to_owned())
                })?;
                super::template_writer::discard_template_output(output, self.auto_close_stream)?;
            }
            self.finished = true;
            return Ok(());
        }
        if let Some(xls) = self.xls.as_ref() {
            let bytes = xls.to_bytes_with_password_and_macro_policy(
                self.password.as_deref(),
                &self.biff8_macro_policy,
            )?;
            let output = self.output.as_mut().ok_or_else(|| {
                ExcelError::Format("template fill executor has no output target".to_owned())
            })?;
            write_template_bytes_to_output(output, &bytes, self.auto_close_stream)?;
            self.finished = true;
            return Ok(());
        }
        self.inner
            .as_mut()
            .ok_or_else(|| ExcelError::Format("template fill executor has no backend".to_owned()))?
            .finish()?;
        self.finished = true;
        Ok(())
    }
}

fn resolve_xls_sheet_name(
    xls: &crate::write::xls_adapter::Biff8TemplatePackage,
    sheet: &WriteFillSheet,
) -> Result<String> {
    if let Some(index) = sheet.sheet_index {
        return xls
            .sheet_names()
            .get(index)
            .cloned()
            .ok_or_else(|| ExcelError::Format(format!("sheet index {index} does not exist")));
    }
    Ok(sheet.sheet_name.clone())
}

fn to_template_sheet(sheet: &WriteFillSheet) -> TemplateSheet {
    if let Some(index) = sheet.sheet_index {
        TemplateSheet::index(index)
    } else if sheet.sheet_name.chars().all(|ch| ch.is_ascii_digit())
        && let Ok(index) = sheet.sheet_name.parse::<usize>()
    {
        TemplateSheet::index(index)
    } else {
        TemplateSheet::name(sheet.sheet_name.clone())
    }
}

fn to_template_fill_config(config: WriteFillConfig) -> FillConfig {
    let mut fill_config = FillConfig::new()
        .force_new_row(config.force_new_row)
        .auto_style(config.auto_style);
    if let Some(direction) = config.direction {
        fill_config = fill_config.direction(match direction {
            WriteDirection::Vertical => FillDirection::Vertical,
            WriteDirection::Horizontal => FillDirection::Horizontal,
        });
    }
    fill_config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_fill_config_propagates_direction_force_row_and_auto_style() {
        let config = to_template_fill_config(WriteFillConfig {
            force_new_row: true,
            direction: Some(WriteDirection::Horizontal),
            auto_style: false,
        });

        assert_eq!(config.effective_direction(), FillDirection::Horizontal);
        assert!(config.effective_force_new_row());
        assert!(!config.effective_auto_style());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::core::WriteFillSheet;
    use tempfile::tempdir;

    /// 对应 Java：生成含 `{name}` 占位符的 XLSX 模板文件。
    fn write_template(directory: &Path, name: &str) -> Result<PathBuf> {
        let template = directory.join(name);
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .write_string(0, 0, "{name}")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        workbook
            .save(&template)
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        Ok(template)
    }

    fn sheet(name: &str) -> WriteFillSheet {
        WriteFillSheet {
            sheet_name: name.to_owned(),
            sheet_index: None,
        }
    }

    /// 对应 Java：`new ExcelWriteFillExecutor(templateInputStream, output)` 的字节入口。
    #[test]
    fn builder_fill_executor_new_accepts_template_bytes_and_rejects_invalid_inputs() -> Result<()> {
        let directory = tempdir()?;
        let output = directory.path().join("bytes-output.xlsx");
        let template = write_template(directory.path(), "bytes-template.xlsx")?;
        let bytes = std::fs::read(&template)?;

        // 字节模板：成功路径（覆盖 from_reader 的 Ok 边）。
        let mut executor = create_builder_fill_executor(None, Some(bytes), output.clone())?;
        executor.fill(
            &TemplateData::new().with("name", "bytes-fill"),
            WriteFillConfig::new(),
            sheet("Sheet1"),
        )?;
        executor.finish(false)?;
        assert!(output.exists());

        // 文件与字节都不提供：Java 语义上抛 Unsupported。
        assert!(matches!(
            BuilderFillExecutor::new(None, None, output.clone()),
            Err(ExcelError::Unsupported(_))
        ));

        // 非法字节：from_reader 的 Err 边。
        assert!(BuilderFillExecutor::new(None, Some(vec![0, 1, 2, 3]), output.clone()).is_err());
        assert!(create_builder_fill_executor(None, Some(vec![0xde, 0xad]), output).is_err());
        Ok(())
    }

    /// 对应 Java：`ExcelWriter.fill` 走 `from_template_path` 的列表填充生命周期。
    #[test]
    fn builder_fill_executor_from_template_path_fills_list_and_finishes() -> Result<()> {
        let directory = tempdir()?;
        let template = write_template(directory.path(), "list-template.xlsx")?;
        let output = directory.path().join("list-output.xlsx");
        let mut executor = BuilderFillExecutor::from_template_path(&template, &output)?;
        executor.fill(
            &FillWrapper::new([
                TemplateData::new().with("name", "A"),
                TemplateData::new().with("name", "B"),
            ]),
            WriteFillConfig {
                auto_style: false,
                ..WriteFillConfig::new()
            },
            sheet("Sheet1"),
        )?;
        executor.finish(false)?;
        assert!(output.exists());

        // finish 之后继续 fill：对应 Java fill 后关闭工作簿抛异常。
        let error = executor
            .fill(
                &FillWrapper::new([TemplateData::new().with("name", "C")]),
                WriteFillConfig::new(),
                sheet("Sheet1"),
            )
            .expect_err("finish 后 fill 必须失败");
        assert!(matches!(error, ExcelError::Unsupported(_)));

        // 模板文件不存在：from_template_path 的 Err 边。
        assert!(
            BuilderFillExecutor::from_template_path(
                directory.path().join("missing.xlsx"),
                directory.path().join("never.xlsx"),
            )
            .is_err()
        );
        Ok(())
    }

    /// 对应 Java：fill 载荷必须是 `TemplateData` 或 `FillWrapper`。
    #[test]
    fn builder_fill_executor_rejects_unsupported_fill_payload() -> Result<()> {
        let directory = tempdir()?;
        let template = write_template(directory.path(), "payload-template.xlsx")?;
        let mut executor = BuilderFillExecutor::from_template_path(
            &template,
            directory.path().join("payload-output.xlsx"),
        )?;
        let error = executor
            .fill(
                &"not-a-fill-payload".to_owned(),
                WriteFillConfig::new(),
                sheet("Sheet1"),
            )
            .expect_err("不支持的 fill 载荷必须失败");
        assert!(
            matches!(error, ExcelError::Format(ref message) if message.contains("TemplateData or FillWrapper")),
            "unexpected error: {error}"
        );
        Ok(())
    }

    /// 对应 Java：`WriteSheet` 的 sheetIndex / 数字名 / 普通名选择。
    #[test]
    fn builder_fill_executor_sheet_selection_maps_index_numeric_name_and_name() {
        let by_index = to_template_sheet(&WriteFillSheet {
            sheet_name: "Ignored".to_owned(),
            sheet_index: Some(2),
        });
        assert!(matches!(by_index, TemplateSheet::Index(2)));

        let by_numeric_name = to_template_sheet(&WriteFillSheet {
            sheet_name: "123".to_owned(),
            sheet_index: None,
        });
        assert!(matches!(by_numeric_name, TemplateSheet::Index(123)));

        let by_name = to_template_sheet(&WriteFillSheet {
            sheet_name: "Sheet1".to_owned(),
            sheet_index: None,
        });
        assert!(matches!(by_name, TemplateSheet::Name(name) if name == "Sheet1"));

        // 空字符串全为数字（真空成立）但解析失败 → 回退到按名称。
        let by_empty_name = to_template_sheet(&WriteFillSheet {
            sheet_name: String::new(),
            sheet_index: None,
        });
        assert!(matches!(by_empty_name, TemplateSheet::Name(name) if name.is_empty()));
    }

    /// 对应 Java：`FillConfig` 未指定方向时保持默认（垂直）。
    #[test]
    fn builder_fill_config_defaults_to_vertical_direction() {
        let config = to_template_fill_config(WriteFillConfig::new());
        assert_eq!(config.effective_direction(), FillDirection::Vertical);
        assert!(!config.effective_force_new_row());
        assert!(config.effective_auto_style());
    }
}
