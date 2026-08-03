//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadWorkbookHolder`.

use crate::holder::read_workbook_holder::ReadWorkbookHolder;

/// 对应 Java：`XlsxReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct XlsxReadWorkbookHolder {
    inner: ReadWorkbookHolder,
}

impl XlsxReadWorkbookHolder {
    /// 对应 Java：`XlsxReadWorkbookHolder(ReadWorkbook)`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ReadWorkbookHolder::default(),
        }
    }

    /// Creates the format-specific holder from resolved workbook options.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            inner: ReadWorkbookHolder::from_options(options),
        }
    }

    /// Returns the inner holder.
    #[must_use]
    pub const fn inner(&self) -> &ReadWorkbookHolder {
        &self.inner
    }
}

impl Default for XlsxReadWorkbookHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xlsx_holder_constructors_and_inner_access() {
        // 对应 Java：XlsxReadWorkbookHolder 构造与 inner 访问器
        let holder = XlsxReadWorkbookHolder::new();
        assert!(
            !holder.inner().ignore_empty_row,
            "derive Default 初始为 false"
        );

        let options = crate::ReadOptions {
            ignore_empty_row: false,
            ..crate::ReadOptions::default()
        };
        let from_options = XlsxReadWorkbookHolder::from_options(&options);
        assert!(!from_options.inner().ignore_empty_row);
        assert_eq!(from_options.inner().charset, options.charset);
        let default_from_options =
            XlsxReadWorkbookHolder::from_options(&crate::ReadOptions::default());
        assert!(default_from_options.inner().ignore_empty_row);

        let defaulted = XlsxReadWorkbookHolder::default();
        assert!(defaulted.inner().auto_close_stream);
    }
}
