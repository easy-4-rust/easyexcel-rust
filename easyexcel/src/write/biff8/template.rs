//! EasyExcel 单元格模型到 BIFF8 模板包的适配层。
//!
//! OLE/CFB 打开、BIFF 记录保留、偏移修复和序列化均由 `easyexcel-xls`
//! 实现；本模块只保留 Java EasyExcel `CellValue` 语义转换与兼容错误。

use std::io::Write;
use std::path::Path;

use crate::core::{CellValue, ExcelError, Result};

use super::{Biff8Cell, Biff8Merge, Biff8Value};

/// 保留原 EasyExcel 路径的 BIFF8 模板包门面。
#[derive(Debug, Clone)]
pub struct Biff8TemplatePackage {
    inner: easyexcel_xls::biff8::Biff8TemplatePackage,
}

impl Biff8TemplatePackage {
    /// 从 OLE `.xls` 字节加载模板。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_bytes(bytes)
            .map(|inner| Self { inner })
            .map_err(map_engine_error)
    }

    /// 从文件加载模板。
    pub fn from_path(path: &Path) -> Result<Self> {
        easyexcel_xls::biff8::Biff8TemplatePackage::from_path(path)
            .map(|inner| Self { inner })
            .map_err(map_engine_error)
    }

    /// 返回工作表名称。
    #[must_use]
    pub fn sheet_names(&self) -> Vec<String> {
        self.inner.sheet_names()
    }

    /// 返回工作表下一可追加行。
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        self.inner
            .next_row_for_sheet(sheet_name)
            .map_err(map_engine_error)
    }

    /// 写入已转换的 BIFF8 单元格。
    pub fn set_cell(
        &mut self,
        sheet_name: &str,
        row: u32,
        col: usize,
        cell: &Biff8Cell,
    ) -> Result<()> {
        self.inner
            .set_cell(sheet_name, row, col, cell)
            .map_err(map_engine_error)
    }

    /// 写入 EasyExcel 单元格值。
    pub fn set_cell_value(
        &mut self,
        sheet_name: &str,
        row: u32,
        col: usize,
        value: &CellValue,
    ) -> Result<()> {
        let cell = cell_value_to_template_cell(value)?;
        self.set_cell(sheet_name, row, col, &cell)
    }

    /// 从当前最后一行后追加稀疏行。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
    ) -> Result<u32> {
        let mut next = self.next_row_for_sheet(sheet_name)?;
        for row_values in rows {
            for (col, value) in row_values {
                self.set_cell_value(sheet_name, next, *col, value)?;
            }
            next = next.saturating_add(1);
        }
        Ok(next)
    }

    /// 添加合并区域。
    pub fn add_merge_range(&mut self, sheet_name: &str, range: Biff8Merge) -> Result<()> {
        self.inner
            .add_merge_range(sheet_name, range)
            .map_err(map_engine_error)
    }

    /// 序列化为 OLE/CFB 字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.inner.to_bytes().map_err(map_engine_error)
    }

    /// 扫描工作簿内的文本占位符。
    #[must_use]
    pub fn scan_placeholders(&self) -> Vec<(String, u16, u8, String)> {
        self.inner.scan_placeholders()
    }

    /// 替换指定位置的 LABEL/LABELSST 文本。
    pub fn replace_label(
        &mut self,
        sheet_name: &str,
        row: u16,
        col: u8,
        replacement: &str,
    ) -> Result<()> {
        self.inner
            .replace_label(sheet_name, row, col, replacement)
            .map_err(map_engine_error)
    }

    /// 保存到文件。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.inner.save_to_path(path).map_err(map_engine_error)
    }

    /// 保存到输出流。
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        self.inner.save_to_writer(output).map_err(map_engine_error)
    }
}

/// 判断字节是否为 OLE `.xls` 容器。
#[must_use]
pub fn looks_like_xls(bytes: &[u8]) -> bool {
    easyexcel_xls::biff8::looks_like_xls(bytes)
}

fn cell_value_to_template_cell(value: &CellValue) -> Result<Biff8Cell> {
    let mapped = match value {
        CellValue::Empty => Biff8Value::Blank,
        CellValue::String(text)
        | CellValue::Error(text)
        | CellValue::Hyperlink { text, .. }
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

fn map_engine_error(error: easyexcel_io::Error) -> ExcelError {
    match error {
        easyexcel_io::Error::Xls(message) => {
            message.strip_prefix("worksheet not found: ").map_or_else(
                || ExcelError::Format(message.clone()),
                |name| ExcelError::SheetNotFound(name.to_owned()),
            )
        }
        other => ExcelError::from(other),
    }
}
