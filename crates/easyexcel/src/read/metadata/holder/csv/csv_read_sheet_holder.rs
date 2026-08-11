//! 对应 Java：`com.alibaba.excel.read.metadata.holder.csv.CsvReadSheetHolder`.

use crate::read::holder::read_sheet_holder::ReadSheetHolder;
use crate::read::metadata::holder::read_holder::delegate_read_holder_contract;
use std::ops::{Deref, DerefMut};

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
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.csv.CsvReadSheetHolder。
    pub const fn inner(&self) -> &ReadSheetHolder {
        &self.inner
    }

    /// Java `CsvReadSheetHolder(ReadSheet, ReadWorkbookHolder)`。
    #[must_use]
    pub fn from_read_sheet(
        read_sheet: crate::ReadSheet,
        read_workbook_holder: &crate::read::holder::read_workbook_holder::ReadWorkbookHolder,
    ) -> Self {
        Self {
            inner: ReadSheetHolder::from_read_sheet(read_sheet, read_workbook_holder),
        }
    }
}

impl Deref for CsvReadSheetHolder {
    type Target = ReadSheetHolder;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for CsvReadSheetHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

delegate_read_holder_contract!(CsvReadSheetHolder, inner);
