//! 对应 Java：`com.alibaba.excel.read.metadata.holder.csv.CsvReadSheetHolder`.

use crate::read::holder::read_sheet_holder::ReadSheetHolder;

/// 对应 Java：`CsvReadSheetHolder extends ReadSheetHolder`.
#[derive(Debug, Clone)]
pub struct CsvReadSheetHolder {
    inner: ReadSheetHolder,
}

impl CsvReadSheetHolder {
    /// 对应 Java： constructor.
    pub fn new(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        Self {
            inner: ReadSheetHolder::new(sheet_no, sheet_name),
        }
    }
    /// Returns the inner holder.
    #[must_use]
    pub const fn inner(&self) -> &ReadSheetHolder {
        &self.inner
    }
}
