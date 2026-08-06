//! 写处理器修改计划。

use std::sync::{Arc, Mutex};

use crate::{CellValue, ExcelError, Result};

use super::write_mutation::WriteMutation;

/// 在线程安全的共享队列中记录 handler 请求，并在工作簿保存前统一执行。
///
/// 对应 Java：handler 通过 POI 活跃对象立即产生的工作簿修改。
#[derive(Debug, Clone, Default)]
pub(crate) struct WriteMutationPlan {
    mutations: Arc<Mutex<Vec<WriteMutation>>>,
}

impl PartialEq for WriteMutationPlan {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.mutations, &other.mutations) {
            return true;
        }
        match (self.mutations.lock(), other.mutations.lock()) {
            (Ok(left), Ok(right)) => *left == *right,
            _ => false,
        }
    }
}

impl WriteMutationPlan {
    pub(crate) fn set_cell(
        &self,
        sheet_name: impl Into<String>,
        row_index: u32,
        column_index: u16,
        value: CellValue,
    ) -> Result<()> {
        self.push(WriteMutation::SetCell {
            sheet_name: sheet_name.into(),
            row_index,
            column_index,
            value,
        })
    }

    pub(crate) fn protect_sheet(
        &self,
        sheet_name: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<()> {
        self.push(WriteMutation::ProtectSheet {
            sheet_name: sheet_name.into(),
            password: password.into(),
        })
    }

    pub(crate) fn snapshot(&self) -> Result<Vec<WriteMutation>> {
        self.mutations
            .lock()
            .map(|mutations| mutations.clone())
            .map_err(|_| ExcelError::Format("write mutation plan lock poisoned".to_owned()))
    }

    pub(crate) fn is_empty(&self) -> Result<bool> {
        self.mutations
            .lock()
            .map(|mutations| mutations.is_empty())
            .map_err(|_| ExcelError::Format("write mutation plan lock poisoned".to_owned()))
    }

    fn push(&self, mutation: WriteMutation) -> Result<()> {
        self.mutations
            .lock()
            .map_err(|_| ExcelError::Format("write mutation plan lock poisoned".to_owned()))?
            .push(mutation);
        Ok(())
    }
}
