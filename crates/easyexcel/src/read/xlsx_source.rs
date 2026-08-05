//! EasyExcel 读取选项到 XLSX 输入引擎的适配。

use std::path::Path;

use crate::core::{ExcelError, Result};
use crate::read::read_helpers::validate_read_options;
use crate::read::read_options::ReadOptions;

pub(crate) use easyexcel_xlsx::xlsx::{XlsxInput, XlsxSource};

pub(crate) fn open_xlsx_source(path: &Path, options: &ReadOptions) -> Result<XlsxSource> {
    validate_read_options(options)?;
    XlsxSource::open(path, options.password.as_deref()).map_err(|error| match error {
        easyexcel_io::Error::PasswordProtected(_) => ExcelError::Unsupported(
            "encrypted OOXML workbook requires a password".to_owned(),
        ),
        other => ExcelError::from(other),
    })
}

#[cfg(test)]
pub(crate) use easyexcel_xlsx::xlsx::is_compound_document;
