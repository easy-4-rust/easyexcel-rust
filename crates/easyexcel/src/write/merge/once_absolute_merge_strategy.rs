//! 对应 Java：`com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy`.

use crate::core::{CellExtra, OnceAbsoluteMergeProperty, WriteCellContext, WriteHandler};

use crate::write::merge::abstract_merge_strategy::AbstractMergeStrategy;

/// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。 对应 Java：`OnceAbsoluteMergeStrategy implements SheetWriteHandler`.
///
/// Registered instances are consumed by the XLSX write path via
/// [`WriteHandler::style_once_absolute_merge`] (in addition to type-level
/// `@OnceAbsoluteMerge` metadata).
// 语义敏感：字段名与 Java `OnceAbsoluteMergeStrategy(int firstRowIndex,
// int lastRowIndex, int firstColumnIndex, int lastColumnIndex)` 一一对应。
#[allow(clippy::struct_field_names)]
/// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
pub struct OnceAbsoluteMergeStrategy {
    first_row_index: i32,
    last_row_index: i32,
    first_column_index: i32,
    last_column_index: i32,
}

impl OnceAbsoluteMergeStrategy {
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。 Creates the strategy. (Java
    /// `OnceAbsoluteMergeStrategy(int, int, int, int)`)
    ///
    /// Java throws when any index is negative; Rust returns a typed error at
    /// construction time.
    ///
    /// # Errors
    ///
    /// Returns [`crate::core::ExcelError::Format`] when any index is
    /// negative.
    pub fn new(
        first_row_index: i32,
        last_row_index: i32,
        first_column_index: i32,
        last_column_index: i32,
    ) -> crate::core::Result<Self> {
        if first_row_index < 0
            || last_row_index < 0
            || first_column_index < 0
            || last_column_index < 0
        {
            return Err(crate::core::ExcelError::Format(
                "all once-absolute merge indexes must be non-negative".to_owned(),
            ));
        }
        Ok(Self {
            first_row_index,
            last_row_index,
            first_column_index,
            last_column_index,
        })
    }

    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。 Creates from annotation/runtime property.
    /// (Java `OnceAbsoluteMergeStrategy(OnceAbsoluteMergeProperty)`)
    ///
    /// # Errors
    ///
    /// Returns [`crate::core::ExcelError::Format`] when any property index
    /// is negative.
    pub fn from_property(property: OnceAbsoluteMergeProperty) -> crate::core::Result<Self> {
        Self::new(
            property.first_row_index,
            property.last_row_index,
            property.first_column_index,
            property.last_column_index,
        )
    }

    /// Returns the merge region as a property. (Java getters)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
    pub const fn to_property(&self) -> OnceAbsoluteMergeProperty {
        OnceAbsoluteMergeProperty::new(
            self.first_row_index,
            self.last_row_index,
            self.first_column_index,
            self.last_column_index,
        )
    }

    /// Returns the first row index. (Java `getFirstRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
    pub const fn first_row_index(&self) -> i32 {
        self.first_row_index
    }

    /// Returns the last row index. (Java `getLastRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
    pub const fn last_row_index(&self) -> i32 {
        self.last_row_index
    }

    /// Returns the first column index. (Java `getFirstColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
    pub const fn first_column_index(&self) -> i32 {
        self.first_column_index
    }

    /// Returns the last column index. (Java `getLastColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.OnceAbsoluteMergeStrategy。
    pub const fn last_column_index(&self) -> i32 {
        self.last_column_index
    }
}

impl WriteHandler for OnceAbsoluteMergeStrategy {
    fn order(&self) -> i32 {
        -60_000
    }

    fn style_once_absolute_merge(&self) -> Option<OnceAbsoluteMergeProperty> {
        // Java `afterSheetCreate` → `addMergedRegionUnsafe`
        Some(self.to_property())
    }
}

impl AbstractMergeStrategy for OnceAbsoluteMergeStrategy {
    fn merge(
        &mut self,
        _sheet_name: &str,
        _cell: &WriteCellContext,
        _extra: Option<&CellExtra>,
        _relative_row_index: Option<i32>,
    ) {
        // Absolute merges run once at sheet create via
        // `WriteHandler::style_once_absolute_merge`, not per cell.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_absolute_merge_strategy_new_ok() {
        let s = OnceAbsoluteMergeStrategy::new(0, 1, 0, 1).unwrap();
        assert_eq!(s.first_row_index(), 0);
        assert_eq!(s.last_row_index(), 1);
        assert_eq!(s.first_column_index(), 0);
        assert_eq!(s.last_column_index(), 1);
    }

    #[test]
    fn once_absolute_merge_strategy_new_negative_error() {
        assert!(OnceAbsoluteMergeStrategy::new(-1, 1, 0, 1).is_err());
        assert!(OnceAbsoluteMergeStrategy::new(0, -1, 0, 1).is_err());
        assert!(OnceAbsoluteMergeStrategy::new(0, 1, -1, 1).is_err());
        assert!(OnceAbsoluteMergeStrategy::new(0, 1, 0, -1).is_err());
    }

    #[test]
    fn once_absolute_merge_strategy_from_property() {
        let prop = OnceAbsoluteMergeProperty::new(0, 5, 0, 3);
        let s = OnceAbsoluteMergeStrategy::from_property(prop).unwrap();
        assert_eq!(s.first_row_index(), 0);
    }

    #[test]
    fn once_absolute_merge_strategy_to_property() {
        let s = OnceAbsoluteMergeStrategy::new(0, 2, 1, 3).unwrap();
        let prop = s.to_property();
        assert_eq!(prop.first_row_index, 0);
        assert_eq!(prop.last_row_index, 2);
        assert_eq!(prop.first_column_index, 1);
        assert_eq!(prop.last_column_index, 3);
    }

    #[test]
    fn once_absolute_merge_strategy_order() {
        let s = OnceAbsoluteMergeStrategy::new(0, 1, 0, 1).unwrap();
        assert_eq!(s.order(), -60_000);
    }

    #[test]
    fn once_absolute_merge_strategy_style_once_absolute_merge() {
        let s = OnceAbsoluteMergeStrategy::new(0, 1, 0, 1).unwrap();
        let prop = s.style_once_absolute_merge().unwrap();
        assert_eq!(prop.first_row_index, 0);
    }

    #[test]
    fn once_absolute_merge_strategy_accessors() {
        let s = OnceAbsoluteMergeStrategy::new(1, 5, 2, 7).unwrap();
        assert_eq!(s.first_row_index(), 1);
        assert_eq!(s.last_row_index(), 5);
        assert_eq!(s.first_column_index(), 2);
        assert_eq!(s.last_column_index(), 7);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::CellValue;

    #[test]
    fn once_absolute_merge_strategy_merge_default_body_runs() {
        let mut strategy = OnceAbsoluteMergeStrategy::new(0, 1, 0, 1).expect("valid");
        let context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        strategy.merge("Sheet1", &context, None, Some(0));
    }
}
