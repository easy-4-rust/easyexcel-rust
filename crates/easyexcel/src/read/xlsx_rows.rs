//! Java `EasyExcel` XLSX 事件读取适配层。
//!
//! OOXML ZIP/XML、共享字符串、样式、显示格式以及 merge/hyperlink/comment
//! 解析均由 `easyexcel-xlsx` 实现；本模块只映射到 `EasyExcel` metadata 类型。

use std::collections::HashSet;
use std::io::{Read, Seek};

use bigdecimal::BigDecimal;
use easyexcel_format::SpreadsheetLocale;
use easyexcel_xlsx::xlsx::{
    ReadSeek, XlsxCellEventReader, XlsxCellValue, XlsxDisplayOptions, XlsxEventMetadata,
    XlsxExtraKind,
};

use crate::ReadOptions;
use crate::core::{CellExtra, CellExtraType, CellValue, ExcelError, FormulaData, Result};
use crate::read::read_cache::create_cache;

include!("xlsx_rows/xlsx_row_metadata.rs");

include!("xlsx_rows/xlsx_display_cell.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 把中立 XLSX 单元格事件转换为 Java `EasyExcel` metadata 的游标。
pub(crate) struct XlsxDisplayCellReader<'a> {
    inner: XlsxCellEventReader<'a>,
    use_1904_windowing: bool,
}

impl XlsxDisplayCellReader<'_> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn next_cell(&mut self) -> Result<Option<XlsxDisplayCell>> {
        let Some(cell) = self.inner.next_cell().map_err(ExcelError::from)? else {
            return Ok(None);
        };
        let value = match cell.value {
            XlsxCellValue::Empty => CellValue::Empty,
            XlsxCellValue::String(value) => CellValue::String(value),
            XlsxCellValue::Bool(value) => CellValue::Bool(value),
            XlsxCellValue::Error(value) => CellValue::Error(value),
            XlsxCellValue::Number(value) if cell.date_formatted => {
                crate::read::cell_conversion::excel_serial_datetime_cell(
                    value,
                    self.use_1904_windowing,
                )
            }
            XlsxCellValue::Number(value) => CellValue::Float(value),
        };
        Ok(Some(XlsxDisplayCell {
            position: cell.position,
            value,
            formula: cell.formula.map(FormulaData::new),
            display_value: cell.display_value,
            decimal_value: cell.decimal_value,
        }))
    }
}
