//! 对应 Java：`com.alibaba.excel.write.merge.AbstractMergeStrategy`.

use easyexcel_core::{CellExtra, WriteCellContext, WriteHandler};

/// 对应 Java：`AbstractMergeStrategy implements CellWriteHandler`.
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
    use easyexcel_core::CellValue;

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
    fn abstract_merge_strategy_trait_compiles() {
        let strategy = TestMergeStrategy::new();
        assert!(!strategy.merge_called);
        assert!(strategy.last_sheet_name.is_none());
    }

    #[test]
    fn abstract_merge_strategy_merge_updates_state() {
        let mut strategy = TestMergeStrategy::new();
        assert!(!strategy.merge_called);

        let context = WriteCellContext::new("TestSheet", 0, 0, CellValue::Empty);

        strategy.merge("Sheet1", &context, None, Some(0));

        assert!(strategy.merge_called);
        assert_eq!(strategy.last_sheet_name, Some("Sheet1".to_owned()));
    }

    #[test]
    fn abstract_merge_strategy_merge_with_none_relative_row_index() {
        let mut strategy = TestMergeStrategy::new();
        let context = WriteCellContext::new("TestSheet", 0, 0, CellValue::Empty);

        strategy.merge("Sheet3", &context, None, None);

        assert!(strategy.merge_called);
        assert_eq!(strategy.last_sheet_name, Some("Sheet3".to_owned()));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use easyexcel_core::{CellValue, WriteCellContext, WriteHandler};

    struct ExtraMergeStrategy {
        merge_called: bool,
    }

    impl WriteHandler for ExtraMergeStrategy {}

    impl AbstractMergeStrategy for ExtraMergeStrategy {
        fn merge(
            &mut self,
            _sheet_name: &str,
            _cell: &WriteCellContext,
            _extra: Option<&CellExtra>,
            _relative_row_index: Option<i32>,
        ) {
            self.merge_called = true;
        }
    }

    #[test]
    fn abstract_merge_strategy_after_cell_dispose_default_is_noop() {
        let mut strategy = ExtraMergeStrategy {
            merge_called: false,
        };
        let context = WriteCellContext::new("TestSheet", 0, 0, CellValue::Empty);
        AbstractMergeStrategy::after_cell_dispose(&mut strategy, &context);
        assert!(!strategy.merge_called);
        strategy.merge("Sheet1", &context, None, Some(0));
        assert!(strategy.merge_called);
    }
}
