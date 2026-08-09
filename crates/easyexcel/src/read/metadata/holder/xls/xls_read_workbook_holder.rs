//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder`.

use crate::read::holder::read_workbook_holder::ReadWorkbookHolder;
use crate::read::metadata::holder::read_holder::delegate_read_holder_contract;
use crate::ReadSheet;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`XlsReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct XlsReadWorkbookHolder {
    inner: ReadWorkbookHolder,
    need_read_sheet: bool,
    bound_sheet_record_list: Vec<ReadSheet>,
    current_sheet_stopped: bool,
    ignore_record: bool,
    read_sheet_index: i32,
    hssf_workbook: Option<Vec<u8>>,
    poifs_file_system: Option<Vec<u8>>,
    format_tracking_listener: bool,
}

impl XlsReadWorkbookHolder {
    /// 对应 Java： constructor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ReadWorkbookHolder::default(),
            need_read_sheet: true,
            bound_sheet_record_list: Vec::new(),
            current_sheet_stopped: false,
            ignore_record: false,
            read_sheet_index: -1,
            hssf_workbook: None,
            poifs_file_system: None,
            format_tracking_listener: false,
        }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder。 Creates the format-specific holder from resolved workbook options.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            inner: ReadWorkbookHolder::from_options(options),
            need_read_sheet: true,
            bound_sheet_record_list: Vec::new(),
            current_sheet_stopped: false,
            ignore_record: false,
            read_sheet_index: -1,
            hssf_workbook: None,
            poifs_file_system: None,
            format_tracking_listener: false,
        }
    }

    /// Java `XlsReadWorkbookHolder(ReadWorkbook)`。
    #[must_use]
    pub fn from_read_workbook(value: crate::ReadWorkbook) -> Self {
        let mut holder = Self::new();
        holder.inner = ReadWorkbookHolder::from_read_workbook(value);
        holder
    }

    /// Returns the inner holder.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder。
    pub const fn inner(&self) -> &ReadWorkbookHolder {
        &self.inner
    }

    /// Returns mutable common workbook state.
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder。
    pub const fn inner_mut(&mut self) -> &mut ReadWorkbookHolder {
        &mut self.inner
    }

    /// Returns whether the main record pass should process worksheet data.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder。
    pub const fn need_read_sheet(&self) -> bool {
        self.need_read_sheet
    }

    /// Controls worksheet-data processing.
    ///
    /// Java `XlsListSheetListener` disables it during its metadata-only pass.
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder。
    pub const fn set_need_read_sheet(&mut self, need_read_sheet: bool) {
        self.need_read_sheet = need_read_sheet;
    }
    #[must_use] pub const fn get_need_read_sheet(&self) -> bool { self.need_read_sheet() }
    #[must_use] pub fn get_bound_sheet_record_list(&self) -> &[ReadSheet] { &self.bound_sheet_record_list }
    pub fn set_bound_sheet_record_list(&mut self, value: Vec<ReadSheet>) { self.bound_sheet_record_list = value; }
    #[must_use] pub const fn get_current_sheet_stopped(&self) -> bool { self.current_sheet_stopped }
    pub const fn set_current_sheet_stopped(&mut self, value: bool) { self.current_sheet_stopped = value; }
    #[must_use] pub const fn get_ignore_record(&self) -> bool { self.ignore_record }
    pub const fn set_ignore_record(&mut self, value: bool) { self.ignore_record = value; }
    #[must_use] pub const fn get_read_sheet_index(&self) -> i32 { self.read_sheet_index }
    pub const fn set_read_sheet_index(&mut self, value: i32) { self.read_sheet_index = value; }
    #[must_use] pub fn get_hssf_workbook(&self) -> Option<&[u8]> { self.hssf_workbook.as_deref() }
    pub fn set_hssf_workbook(&mut self, value: Option<Vec<u8>>) { self.hssf_workbook = value; }
    #[must_use] pub fn get_poifs_file_system(&self) -> Option<&[u8]> { self.poifs_file_system.as_deref() }
    pub fn set_poifs_file_system(&mut self, value: Option<Vec<u8>>) { self.poifs_file_system = value; }
    #[must_use] pub const fn get_format_tracking_hssf_listener(&self) -> bool { self.format_tracking_listener }
    pub const fn set_format_tracking_hssf_listener(&mut self, value: bool) { self.format_tracking_listener = value; }
    /// Java `getFormatTrackingHSSFListener()` 原始缩写兼容入口。
    #[must_use] pub const fn get_format_tracking_hssflistener(&self) -> bool { self.format_tracking_listener }
    /// Java `setFormatTrackingHSSFListener()` 原始缩写兼容入口。
    pub const fn set_format_tracking_hssflistener(&mut self, value: bool) { self.format_tracking_listener = value; }
}

impl Deref for XlsReadWorkbookHolder {
    type Target = ReadWorkbookHolder;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for XlsReadWorkbookHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

delegate_read_holder_contract!(XlsReadWorkbookHolder, inner);

impl Default for XlsReadWorkbookHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xls_holder_constructors_and_need_read_sheet() {
        // 对应 Java：XlsReadWorkbookHolder 构造与 needReadSheet 开关
        let mut holder = XlsReadWorkbookHolder::new();
        assert!(holder.need_read_sheet());
        holder.set_need_read_sheet(false);
        assert!(!holder.need_read_sheet());
        assert!(
            !holder.inner().ignore_empty_row,
            "derive Default 初始为 false"
        );

        let options = crate::ReadOptions {
            ignore_empty_row: false,
            ..crate::ReadOptions::default()
        };
        let from_options = XlsReadWorkbookHolder::from_options(&options);
        assert!(!from_options.inner().ignore_empty_row);
        assert_eq!(from_options.inner().charset, options.charset);
        let default_from_options =
            XlsReadWorkbookHolder::from_options(&crate::ReadOptions::default());
        assert!(default_from_options.inner().ignore_empty_row);

        let mut mut_holder = XlsReadWorkbookHolder::default();
        mut_holder.inner_mut().ignore_empty_row = false;
        assert!(!mut_holder.inner().ignore_empty_row);
    }
}
