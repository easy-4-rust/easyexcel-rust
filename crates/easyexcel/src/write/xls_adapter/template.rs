//! `EasyExcel` 单元格模型到 BIFF8 模板包的适配层。
//!
//! OLE/CFB 打开、BIFF 记录保留、偏移修复和序列化均由 `easyexcel-xls`
//! 实现；本模块只保留 Java `EasyExcel` `CellValue` 语义转换与兼容错误。

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::core::{CellValue, ExcelError, Result};

use super::{Biff8Cell, Biff8Merge, Biff8Value};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 保留原 `EasyExcel` 路径的 BIFF8 模板包门面。
#[derive(Debug, Clone)]
pub(crate) struct Biff8TemplatePackage {
    inner: easyexcel_xls::biff8::Biff8TemplatePackage,
}

impl Biff8TemplatePackage {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从 OLE `.xls` 字节加载模板。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_bytes(bytes)
            .map(|inner| Self { inner })
            .map_err(ExcelError::from)
    }

    /// 对应 Java：`HSSFWorkbook(templateStream)` + 调用级 BIFF8 密码。 从字节加载模板。
    pub fn from_bytes_with_password(bytes: &[u8], password: Option<&str>) -> Result<Self> {
        let Some(password) = password else {
            return Self::from_bytes(bytes);
        };
        easyexcel_xls::biff8::Biff8TemplatePackage::from_bytes_with_password(bytes, Some(password))
            .map(|inner| Self { inner })
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从文件加载模板。
    pub fn from_path(path: &Path) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_path(path)
            .map(|inner| Self { inner })
            .map_err(ExcelError::from)
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从当前最后一行后追加稀疏行。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
    ) -> Result<u32> {
        let rows = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(column, value)| {
                        cell_value_to_template_cell(value).map(|cell| (*column, cell))
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        self.inner
            .append_rows(sheet_name, &rows)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 添加合并区域。
    pub fn add_merge_range(&mut self, sheet_name: &str, range: Biff8Merge) -> Result<()> {
        self.inner
            .add_merge_range(sheet_name, range)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用中立文本数据替换 BIFF8 标量占位符。
    pub fn replace_scalar_placeholders(
        &mut self,
        values: &BTreeMap<String, String>,
    ) -> Result<usize> {
        self.inner
            .replace_scalar_placeholders(values)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：`ExcelWriteFillExecutor#doFill`。 按工作表和 `FillConfig` 扩展集合占位符。
    pub fn fill_collection_placeholders(
        &mut self,
        sheet_name: Option<&str>,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, String>],
        horizontal: bool,
        force_new_row: bool,
        auto_style: bool,
    ) -> Result<usize> {
        self.inner
            .fill_collection_placeholders(
                sheet_name,
                collection_name,
                rows,
                horizontal,
                force_new_row,
                auto_style,
            )
            .map_err(ExcelError::from)
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
}

fn cell_value_to_template_cell(value: &CellValue) -> Result<Biff8Cell> {
    let mapped = match value {
        CellValue::Empty => Biff8Value::Blank,
        CellValue::String(text)
        | CellValue::Error(text)
        | CellValue::Hyperlink { text, .. }
        | CellValue::HyperlinkWithMetadata { text, .. }
        | CellValue::Formula(text) => Biff8Value::Text(text.clone()),
        CellValue::RichText(rich) => Biff8Value::Text(rich.text_string().to_owned()),
        CellValue::Bool(flag) => Biff8Value::Bool(*flag),
        CellValue::Int(number) => Biff8Value::Number(
            #[allow(clippy::cast_precision_loss)]
            {
                *number as f64
            },
        ),
        CellValue::Float(number) => Biff8Value::Number(*number),
        CellValue::Decimal(number) => {
            let numeric = crate::write::finite_decimal_f64(number, "BIFF8")?;
            if crate::write::decimal_integer_requires_text(number)? {
                Biff8Value::Text(number.to_plain_string())
            } else {
                Biff8Value::Number(numeric)
            }
        }
        CellValue::Date(date) => {
            return Ok(Biff8Cell::date_serial(super::date_to_excel_serial(*date)));
        }
        CellValue::DateTime(datetime) => {
            return Ok(Biff8Cell::datetime_serial(super::datetime_to_excel_serial(
                *datetime,
            )));
        }
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => {
            return cell_value_to_template_cell(value);
        }
        CellValue::Image(_) => {
            return Err(ExcelError::Unsupported(
                "legacy XLS writing does not support images".to_owned(),
            ));
        }
    };
    Ok(Biff8Cell::general(mapped))
}
