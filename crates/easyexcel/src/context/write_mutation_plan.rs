//! 写处理器修改计划。

use std::collections::BTreeMap;
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

    /// 记录一个在保存前执行的批注删除操作。
    pub(crate) fn remove_comment(
        &self,
        sheet_name: impl Into<String>,
        row_index: u32,
        column_index: u16,
    ) -> Result<()> {
        self.push(WriteMutation::RemoveComment {
            sheet_name: sheet_name.into(),
            row_index,
            column_index,
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

    /// 返回所有批注删除修改，供序列化后的 XLSX OOXML 包执行。
    pub(crate) fn comment_removals(&self) -> Result<Vec<(String, u32, u16)>> {
        let mut actions = BTreeMap::new();
        for mutation in self.snapshot()? {
            match mutation {
                WriteMutation::RemoveComment {
                    sheet_name,
                    row_index,
                    column_index,
                } => {
                    actions.insert((sheet_name, row_index, column_index), true);
                }
                WriteMutation::SetCell {
                    sheet_name,
                    row_index,
                    column_index,
                    value:
                        CellValue::Comment { .. } | CellValue::CommentWithMetadata { .. },
                } => {
                    // 后续 setCellComment 覆盖此前 removeCellComment。
                    actions.insert((sheet_name, row_index, column_index), false);
                }
                _ => {}
            }
        }
        Ok(actions
            .into_iter()
            .filter_map(|(coordinate, remove)| remove.then_some(coordinate))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_plan_is_empty_and_snapshot_returns_empty() {
        let plan = WriteMutationPlan::default();
        assert!(plan.is_empty().unwrap());
        assert!(plan.snapshot().unwrap().is_empty());
    }

    #[test]
    fn set_cell_records_mutation() {
        let plan = WriteMutationPlan::default();
        plan.set_cell("Sheet1", 0, 0, CellValue::String("hello".to_owned()))
            .unwrap();
        assert!(!plan.is_empty().unwrap());
        let snapshot = plan.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn protect_sheet_records_mutation() {
        let plan = WriteMutationPlan::default();
        plan.protect_sheet("Sheet1", "password").unwrap();
        let snapshot = plan.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn add_merge_records_mutation() {
        let plan = WriteMutationPlan::default();
        let range = MergeRange::new(0, 1, 0, 1);
        plan.add_merge("Sheet1", range).unwrap();
        let snapshot = plan.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn remove_comment_records_mutation() {
        let plan = WriteMutationPlan::default();
        plan.remove_comment("Sheet1", 0, 0).unwrap();
        let snapshot = plan.snapshot().unwrap();
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn merge_ranges_extracts_only_merge_mutations() {
        let plan = WriteMutationPlan::default();
        let range = MergeRange::new(0, 1, 0, 1);
        plan.add_merge("Sheet1", range).unwrap();
        plan.set_cell("Sheet1", 0, 0, CellValue::String("x".to_owned()))
            .unwrap();
        plan.add_merge("Sheet2", MergeRange::new(2, 3, 2, 3))
            .unwrap();

        let merges = plan.merge_ranges().unwrap();
        assert_eq!(merges.len(), 2);
        assert_eq!(merges[0].0, "Sheet1");
        assert_eq!(merges[1].0, "Sheet2");
    }

    #[test]
    fn comment_removals_deduplicates_and_filters_overrides() {
        let plan = WriteMutationPlan::default();
        // 删除批注
        plan.remove_comment("Sheet1", 0, 0).unwrap();
        // 后续设置批注覆盖删除
        plan.set_cell(
            "Sheet1",
            0,
            0,
            CellValue::Comment {
                value: Box::new(CellValue::String("x".to_owned())),
                text: "note".to_owned(),
            },
        )
        .unwrap();
        // 另一个删除
        plan.remove_comment("Sheet1", 1, 1).unwrap();

        let removals = plan.comment_removals().unwrap();
        // (0,0) 被 setCellComment 覆盖，只剩 (1,1)
        assert_eq!(removals.len(), 1);
        assert_eq!(removals[0], ("Sheet1".to_owned(), 1, 1));
    }

    #[test]
    fn partial_eq_compares_content() {
        let plan_a = WriteMutationPlan::default();
        let plan_b = WriteMutationPlan::default();
        assert_eq!(plan_a, plan_b);

        plan_a
            .set_cell("S", 0, 0, CellValue::Int(1))
            .unwrap();
        assert_ne!(plan_a, plan_b);
    }

    #[test]
    fn clone_shares_mutation_queue() {
        let plan = WriteMutationPlan::default();
        plan.set_cell("S", 0, 0, CellValue::Int(1)).unwrap();
        let cloned = plan.clone();
        // Arc 共享，内容相同
        assert_eq!(plan, cloned);
    }
}
