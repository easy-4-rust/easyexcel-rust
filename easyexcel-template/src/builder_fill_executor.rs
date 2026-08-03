//! Bridges [`WriteFillExecutor`] to [`ExcelTemplateWriter`].
//!
//! Keeps template fill logic out of `easyexcel-writer` while letting
//! `ExcelBuilderImpl.fill` delegate to the same engine as
//! `EasyExcel::template_writer`.

use std::any::Any;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use easyexcel_core::{
    ExcelError, Result, WriteDirection, WriteFillConfig, WriteFillExecutor, WriteFillSheet,
};

use crate::{
    ExcelTemplateWriter, FillConfig, FillDirection, FillWrapper, TemplateData, TemplateSheet,
};

/// Stateful template fill executor for [`easyexcel_writer::ExcelBuilderImpl`].
///
/// 对应 Java：`ExcelWriteFillExecutor` backed by the same loaded XLSX
/// package as [`ExcelTemplateWriter`].
pub struct BuilderFillExecutor {
    inner: ExcelTemplateWriter<'static>,
}

impl BuilderFillExecutor {
    /// Loads a template from path or bytes and prepares fill against `output`.
    ///
    /// # Errors
    ///
    /// Returns I/O or OOXML package errors when the template cannot be read.
    pub fn new(
        template_file: Option<PathBuf>,
        template_bytes: Option<Vec<u8>>,
        output: PathBuf,
    ) -> Result<Self> {
        let inner = if let Some(path) = template_file {
            ExcelTemplateWriter::new(path, output)?
        } else if let Some(bytes) = template_bytes {
            ExcelTemplateWriter::from_reader(Cursor::new(bytes), output)?
        } else {
            return Err(ExcelError::Unsupported(
                "with_template requires a template file or template bytes".to_owned(),
            ));
        };
        Ok(Self { inner })
    }

    /// Loads a template file and writes to an existing path.
    ///
    /// # Errors
    ///
    /// Returns I/O or OOXML package errors when the template cannot be read.
    pub fn from_template_path(
        template: impl AsRef<Path>,
        output: impl Into<PathBuf>,
    ) -> Result<Self> {
        Ok(Self {
            inner: ExcelTemplateWriter::new(template, output)?,
        })
    }
}

/// Creates a boxed fill executor for facade wiring into [`ExcelBuilderImpl`](easyexcel_writer::ExcelBuilderImpl).
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

impl WriteFillExecutor for BuilderFillExecutor {
    fn fill(
        &mut self,
        data: &dyn Any,
        fill_config: WriteFillConfig,
        sheet: WriteFillSheet,
    ) -> Result<()> {
        let template_sheet = to_template_sheet(&sheet);
        if let Some(scalar) = data.downcast_ref::<TemplateData>() {
            self.inner.fill_on_sheet(&template_sheet, scalar)?;
            return Ok(());
        }
        if let Some(collection) = data.downcast_ref::<FillWrapper>() {
            self.inner.fill_list_on_sheet(
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

    fn finish(&mut self, _on_exception: bool) -> Result<()> {
        self.inner.finish()
    }
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

        assert_eq!(config.get_direction(), FillDirection::Horizontal);
        assert!(config.get_force_new_row());
        assert!(!config.get_auto_style());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use easyexcel_core::WriteFillSheet;
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

    /// 对应 Java：fill 载荷必须是 TemplateData 或 FillWrapper。
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
        assert_eq!(config.get_direction(), FillDirection::Vertical);
        assert!(!config.get_force_new_row());
        assert!(config.get_auto_style());
    }
}
