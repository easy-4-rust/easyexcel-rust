//! Mirrors Java `com.alibaba.excel.write.merge.AbstractMergeStrategy`.

use easyexcel_core::{CellExtra, WriteCellContext, WriteHandler};

/// Mirrors Java `AbstractMergeStrategy implements CellWriteHandler`.
///
/// The Java side overrides `afterCellDispose` and calls the abstract
/// `merge(Sheet, Cell, Head, Integer relativeRowIndex)`. Rust mirrors the
/// structure so the strategy classes can override the hook.
pub trait AbstractMergeStrategy: WriteHandler {
    /// Called once per non-head cell. (Java `afterCellDispose`)
    fn after_cell_dispose(&mut self, context: &WriteCellContext) {
        let _ = context; // no-op default
    }

    /// Applies the merge to the worksheet. (Java `merge(Sheet, Cell, Head, Integer)`)
    fn merge(
        &mut self,
        sheet_name: &str,
        cell: &WriteCellContext,
        _extra: Option<&CellExtra>,
        _relative_row_index: Option<i32>,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMergeStrategy {
        merge_called: bool,
        last_sheet_name: Option<String>,
    }

    impl TestMergeStrategy {
        fn new() -> Self {
            Self {
                merge_called: false,
                last_sheet_name: None,
            }
        }
    }

    impl WriteHandler for TestMergeStrategy {}

    impl AbstractMergeStrategy for TestMergeStrategy {
        fn merge(
            &mut self,
            sheet_name: &str,
            _cell: &WriteCellContext,
            _extra: Option<&CellExtra>,
            _relative_row_index: Option<i32>,
        ) {
            self.merge_called = true;
            self.last_sheet_name = Some(sheet_name.to_owned());
        }
    }

    #[test]
    fn abstract_merge_strategy_merge_is_called() {
        let mut strategy = TestMergeStrategy::new();

        // Create a minimal context for testing
        let context = WriteCellContext {
            sheet_name: "TestSheet".to_owned(),
            row_index: 0,
            column_index: 0,
            field: None,
            column: None,
            head_name: None,
            is_head: false,
            relative_row_index: None,
            original_value: None,
            original_field_type: None,
            pending_original_value: None,
            pending_original_field_type: None,
            cell_data: None,
            cell: None,
            skip: false,
        };

        strategy.merge("Sheet1", &context, None, Some(0));

        assert!(strategy.merge_called);
        assert_eq!(strategy.last_sheet_name, Some("Sheet1".to_owned()));
    }

    #[test]
    fn abstract_merge_strategy_merge_with_none_relative_row_index() {
        let mut strategy = TestMergeStrategy::new();

        let context = WriteCellContext {
            sheet_name: "TestSheet".to_owned(),
            row_index: 0,
            column_index: 0,
            field: None,
            column: None,
            head_name: None,
            is_head: false,
            relative_row_index: None,
            original_value: None,
            original_field_type: None,
            pending_original_value: None,
            pending_original_field_type: None,
            cell_data: None,
            cell: None,
            skip: false,
        };

        strategy.merge("Sheet3", &context, None, None);

        assert!(strategy.merge_called);
        assert_eq!(strategy.last_sheet_name, Some("Sheet3".to_owned()));
    }
}
