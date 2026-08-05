//! Java EasyExcel XLSX 事件读取适配层。
//!
//! OOXML ZIP/XML、共享字符串、样式、显示格式以及 merge/hyperlink/comment
//! 解析均由 `easyexcel-xlsx` 实现；本模块只映射到 EasyExcel metadata 类型。

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

/// EasyExcel 门面持有的 XLSX 工作簿事件元数据。
pub(crate) struct XlsxRowMetadata {
    inner: XlsxEventMetadata<Box<dyn ReadSeek>>,
}

/// EasyExcel listener 消费的显示单元格。
pub(crate) struct XlsxDisplayCell {
    pub(crate) position: (u32, usize),
    pub(crate) value: CellValue,
    pub(crate) formula: Option<FormulaData>,
    pub(crate) display_value: Option<String>,
    pub(crate) decimal_value: Option<BigDecimal>,
}

/// 把中立 XLSX 单元格事件转换为 Java EasyExcel metadata 的游标。
pub(crate) struct XlsxDisplayCellReader<'a> {
    inner: XlsxCellEventReader<'a>,
    use_1904_windowing: bool,
}

impl XlsxRowMetadata {
    #[cfg(test)]
    pub(crate) fn new(input: impl Read + Seek + 'static) -> Result<Self> {
        Self::new_with_cache(input, &ReadOptions::default())
    }

    pub(crate) fn new_with_cache(
        input: impl Read + Seek + 'static,
        options: &ReadOptions,
    ) -> Result<Self> {
        let mode = options.read_cache;
        let selector = options
            .read_cache_selector
            .as_ref()
            .map(|stored| stored as &dyn crate::cache::ReadCacheSelector);
        let inner = XlsxEventMetadata::new_with_cache_factory(
            Box::new(input) as Box<dyn ReadSeek>,
            |xml_size| {
                selector.map_or_else(
                    || create_cache(mode, xml_size),
                    |selector| selector.create_cache(xml_size),
                )
            },
        )
        .map_err(ExcelError::from)?;
        Ok(Self { inner })
    }

    pub(crate) fn sheet_names(&self) -> Vec<String> {
        self.inner.sheet_names().to_vec()
    }

    pub(crate) fn display_cells(
        &mut self,
        sheet_name: &str,
        use_1904_windowing: bool,
        use_scientific_format: bool,
        locale: SpreadsheetLocale,
    ) -> Result<XlsxDisplayCellReader<'_>> {
        let inner = self
            .inner
            .cells(
                sheet_name,
                XlsxDisplayOptions {
                    date_1904: use_1904_windowing,
                    use_scientific_format,
                    locale,
                },
            )
            .map_err(ExcelError::from)?;
        Ok(XlsxDisplayCellReader {
            inner,
            use_1904_windowing,
        })
    }

    pub(crate) fn last_explicit_row(&mut self, sheet_name: &str) -> Result<Option<u32>> {
        self.inner
            .last_explicit_row(sheet_name)
            .map_err(ExcelError::from)
    }

    pub(crate) fn extras(
        &mut self,
        sheet_name: &str,
        enabled: &HashSet<CellExtraType>,
    ) -> Result<Vec<CellExtra>> {
        let engine_enabled = enabled
            .iter()
            .map(|kind| match kind {
                CellExtraType::Merge => XlsxExtraKind::Merge,
                CellExtraType::Hyperlink => XlsxExtraKind::Hyperlink,
                CellExtraType::Comment => XlsxExtraKind::Comment,
            })
            .collect();
        self.inner
            .extras(sheet_name, &engine_enabled)
            .map(|extras| {
                extras
                    .into_iter()
                    .map(|extra| {
                        let kind = match extra.kind {
                            XlsxExtraKind::Merge => CellExtraType::Merge,
                            XlsxExtraKind::Hyperlink => CellExtraType::Hyperlink,
                            XlsxExtraKind::Comment => CellExtraType::Comment,
                        };
                        CellExtra::new(
                            kind,
                            extra.text,
                            extra.first_row,
                            extra.last_row,
                            extra.first_column,
                            extra.last_column,
                        )
                    })
                    .collect()
            })
            .map_err(ExcelError::from)
    }
}

impl XlsxDisplayCellReader<'_> {
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
