//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xls.XlsReadWorkbookHolder`.

use crate::holder::read_workbook_holder::ReadWorkbookHolder;

/// 对应 Java：`XlsReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct XlsReadWorkbookHolder {
    inner: ReadWorkbookHolder,
    need_read_sheet: bool,
}

impl XlsReadWorkbookHolder {
    /// 对应 Java： constructor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ReadWorkbookHolder::default(),
            need_read_sheet: true,
        }
    }

    /// Creates the format-specific holder from resolved workbook options.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            inner: ReadWorkbookHolder::from_options(options),
            need_read_sheet: true,
        }
    }

    /// Returns the inner holder.
    #[must_use]
    pub const fn inner(&self) -> &ReadWorkbookHolder {
        &self.inner
    }

    /// Returns mutable common workbook state.
    pub const fn inner_mut(&mut self) -> &mut ReadWorkbookHolder {
        &mut self.inner
    }

    /// Returns whether the main record pass should process worksheet data.
    #[must_use]
    pub const fn need_read_sheet(&self) -> bool {
        self.need_read_sheet
    }

    /// Controls worksheet-data processing.
    ///
    /// Java `XlsListSheetListener` disables it during its metadata-only pass.
    pub const fn set_need_read_sheet(&mut self, need_read_sheet: bool) {
        self.need_read_sheet = need_read_sheet;
    }
}

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
