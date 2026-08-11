use chrono::{NaiveDate, NaiveDateTime};

use easyexcel_io::{Error, Result};

use super::{
    Format, Worksheet, write_blank, write_boolean, write_boolean_with_format,
    write_date_with_format, write_datetime_with_format, write_formula, write_formula_with_format,
    write_integer, write_number, write_number_with_format, write_string, write_string_with_format,
    write_url_with_options,
};

/// 生成式 XLSX 后端可直接写入的中立单元格值。
///
/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。门面先完成 converter、Handler
/// 和 Java 元数据合并，再把最终值交给本类型；具体 `Worksheet` 方法选择只存在于
/// `easyexcel-xlsx`。
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedCellValue {
    /// 空白单元格。
    Blank,
    /// 文本单元格。
    Text(String),
    /// 布尔单元格。
    Bool(bool),
    /// 保留超大整数文本语义的整数单元格。
    Integer(i64),
    /// IEEE-754 数字单元格。
    Number(f64),
    /// 日期单元格。
    Date(NaiveDate),
    /// 日期时间单元格。
    DateTime(NaiveDateTime),
    /// 公式单元格。
    Formula(String),
    /// 带显示文本的超链接。
    Hyperlink {
        /// 已由格式引擎类型规范化的链接目标。
        target: String,
        /// 单元格显示文本。
        text: String,
    },
}

impl GeneratedCellValue {
    /// 借用文本并直接写入，避免 facade 为热路径临时构造拥有所有权的
    /// [`GeneratedCellValue::Text`]。
    ///
    /// # Errors
    ///
    /// 坐标、格式或底层 OOXML 生成失败时返回错误。
    pub fn write_text(
        worksheet: &mut Worksheet,
        row: u32,
        column: u16,
        value: &str,
        format: Option<&Format>,
    ) -> Result<()> {
        match format {
            Some(format) => write_string_with_format(worksheet, row, column, value, format),
            None => write_string(worksheet, row, column, value),
        }
    }

    /// 借用公式并直接写入，避免为只读公式表达式复制 `String`。
    ///
    /// # Errors
    ///
    /// 坐标、格式或底层 OOXML 生成失败时返回错误。
    pub fn write_formula_value(
        worksheet: &mut Worksheet,
        row: u32,
        column: u16,
        formula: &str,
        format: Option<&Format>,
    ) -> Result<()> {
        match format {
            Some(format) => write_formula_with_format(worksheet, row, column, formula, format),
            None => write_formula(worksheet, row, column, formula),
        }
    }

    /// 借用已规范化的链接目标和显示文本并直接写入。
    ///
    /// # Errors
    ///
    /// 坐标、格式或底层 OOXML 生成失败时返回错误。
    pub fn write_hyperlink(
        worksheet: &mut Worksheet,
        row: u32,
        column: u16,
        target: &str,
        text: &str,
        format: &Format,
    ) -> Result<()> {
        write_url_with_options(worksheet, row, column, target, text, format)
    }

    /// 写入指定工作表坐标；`format=None` 启用无样式快速路径。
    ///
    /// # Errors
    ///
    /// 坐标、格式或底层 OOXML 生成失败时返回错误。空白、日期、日期时间与超链接
    /// 依赖显式格式，调用方未提供时 fail-closed。
    pub fn write(
        &self,
        worksheet: &mut Worksheet,
        row: u32,
        column: u16,
        format: Option<&Format>,
    ) -> Result<()> {
        match (self, format) {
            (Self::Blank, Some(format)) => write_blank(worksheet, row, column, format),
            (Self::Text(value), format) => Self::write_text(worksheet, row, column, value, format),
            (Self::Bool(value), Some(format)) => {
                write_boolean_with_format(worksheet, row, column, *value, format)
            }
            (Self::Bool(value), None) => write_boolean(worksheet, row, column, *value),
            (Self::Integer(value), format) => write_integer(worksheet, row, column, *value, format),
            (Self::Number(value), Some(format)) => {
                write_number_with_format(worksheet, row, column, *value, format)
            }
            (Self::Number(value), None) => write_number(worksheet, row, column, *value),
            (Self::Date(value), Some(format)) => {
                write_date_with_format(worksheet, row, column, *value, format)
            }
            (Self::DateTime(value), Some(format)) => {
                write_datetime_with_format(worksheet, row, column, *value, format)
            }
            (Self::Formula(value), format) => {
                Self::write_formula_value(worksheet, row, column, value, format)
            }
            (Self::Hyperlink { target, text }, Some(format)) => {
                Self::write_hyperlink(worksheet, row, column, target, text, format)
            }
            (Self::Blank | Self::Date(_) | Self::DateTime(_) | Self::Hyperlink { .. }, None) => {
                Err(Error::Xlsx(
                    "generated blank/date/hyperlink cell requires an explicit format".to_owned(),
                ))
            }
        }
    }
}
