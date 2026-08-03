//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadWorkbookHolder`.

use crate::reader::context::read_sheet::ReadSheet;

/// 对应 Java：`ReadWorkbookHolder extends AbstractReadHolder`.
///
/// Java carries 17 fields. Rust collapses them into the `ReadOptions`
/// struct that already lives in the reader facade. This struct exists
/// for 1:1 API parity.
#[derive(Debug, Clone)]
pub struct ReadWorkbookHolder {
    /// Mirrors `ReadWorkbookHolder.charset`.
    pub charset: crate::core::CsvCharset,
    /// Mirrors `ReadWorkbookHolder.autoCloseStream`.
    pub auto_close_stream: bool,
    /// Mirrors `ReadWorkbookHolder.ignoreEmptyRow`.
    pub ignore_empty_row: bool,
    /// Mirrors `ReadWorkbookHolder.password`.
    pub password: Option<String>,
    /// Workbooks sheets discovered by the format executor.
    ///
    /// Mirrors `ReadWorkbookHolder.actualSheetDataList`.
    pub actual_sheet_data_list: Option<Vec<ReadSheet>>,
}

impl Default for ReadWorkbookHolder {
    /// Java `ReadWorkbookHolder(ReadWorkbook)`：`autoCloseStream` 未指定时为
    /// `Boolean.TRUE`（`if (readWorkbook.getAutoCloseStream() == null) ... TRUE`），
    /// 因此 Default 与 `new()` 的自动关闭语义保持一致。
    fn default() -> Self {
        Self {
            charset: crate::core::CsvCharset::default(),
            auto_close_stream: true,
            ignore_empty_row: false,
            password: None,
            actual_sheet_data_list: None,
        }
    }
}

impl ReadWorkbookHolder {
    /// Resolves workbook-level holder state from the public read options.
    ///
    /// 对应 Java：`ReadWorkbookHolder(ReadWorkbook, ...)` propagation before
    /// a format-specific context is constructed.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            charset: options.charset.clone(),
            auto_close_stream: true,
            ignore_empty_row: options.ignore_empty_row,
            password: options.password.clone(),
            actual_sheet_data_list: None,
        }
    }

    /// Returns format-discovered sheets in workbook order.
    #[must_use]
    pub fn actual_sheet_data_list(&self) -> Option<&[ReadSheet]> {
        self.actual_sheet_data_list.as_deref()
    }

    /// Stores format-discovered sheets.
    pub fn set_actual_sheet_data_list(&mut self, sheets: Vec<ReadSheet>) {
        self.actual_sheet_data_list = Some(sheets);
    }
}
