//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadSheetHolder`.

use crate::read::holder::read_sheet_holder::ReadSheetHolder;

/// 对应 Java：`XlsxReadSheetHolder extends ReadSheetHolder`.
#[derive(Debug, Clone)]
pub struct XlsxReadSheetHolder {
    inner: ReadSheetHolder,
}

impl XlsxReadSheetHolder {
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
