//! 对应 Java：`com.alibaba.excel.read.metadata.holder.csv.CsvReadWorkbookHolder`.

use crate::holder::read_workbook_holder::ReadWorkbookHolder;

/// 对应 Java：`CsvReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct CsvReadWorkbookHolder {
    inner: ReadWorkbookHolder,
}

impl CsvReadWorkbookHolder {
    /// 对应 Java： constructor.
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

impl Default for CsvReadWorkbookHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_holder_constructors_and_inner_access() {
        // 对应 Java：CsvReadWorkbookHolder 构造与 inner 访问器
        let holder = CsvReadWorkbookHolder::new();
        assert!(
            !holder.inner().ignore_empty_row,
            "derive Default 初始为 false"
        );

        let options = crate::ReadOptions {
            ignore_empty_row: false,
            ..crate::ReadOptions::default()
        };
        let from_options = CsvReadWorkbookHolder::from_options(&options);
        assert!(!from_options.inner().ignore_empty_row);
        assert_eq!(from_options.inner().charset, options.charset);
        assert_eq!(from_options.inner().password, options.password);
        let default_from_options =
            CsvReadWorkbookHolder::from_options(&crate::ReadOptions::default());
        assert!(default_from_options.inner().ignore_empty_row);

        let defaulted = CsvReadWorkbookHolder::default();
        assert!(defaulted.inner().auto_close_stream);
    }
}
