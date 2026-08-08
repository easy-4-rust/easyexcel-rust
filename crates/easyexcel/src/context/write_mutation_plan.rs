//! 写处理器修改计划。

use std::sync::{Arc, Mutex};

use crate::{CellValue, ChartMutation, ExcelError, MergeRange, Result};

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

    pub(crate) fn add_chart(&self, chart: ChartMutation) -> Result<()> {
        self.push(WriteMutation::AddChart(chart))
    }

    /// 记录一个在保存前应用的工作表合并区域。
    pub(crate) fn add_merge(&self, sheet_name: impl Into<String>, range: MergeRange) -> Result<()> {
        self.push(WriteMutation::AddMerge {
            sheet_name: sheet_name.into(),
            range,
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

    /// 返回所有工作表合并修改，供 XLSX 在序列化后以 OOXML 元数据方式应用。
    pub(crate) fn merge_ranges(&self) -> Result<Vec<(String, MergeRange)>> {
        Ok(self
            .snapshot()?
            .into_iter()
            .filter_map(|mutation| match mutation {
                WriteMutation::AddMerge { sheet_name, range } => Some((sheet_name, range)),
                _ => None,
            })
            .collect())
    }

    fn push(&self, mutation: WriteMutation) -> Result<()> {
        self.mutations
            .lock()
            .map_err(|_| ExcelError::Format("write mutation plan lock poisoned".to_owned()))?
            .push(mutation);
        Ok(())
    }
}
