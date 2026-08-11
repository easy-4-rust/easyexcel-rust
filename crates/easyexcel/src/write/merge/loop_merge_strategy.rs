//! 对应 Java：`com.alibaba.excel.write.merge.LoopMergeStrategy`.

use crate::core::{
    CellExtra, ExcelError, LoopMergeProperty, Result, WriteCellContext, WriteHandler,
};

use crate::write::merge::abstract_merge_strategy::AbstractMergeStrategy;

/// 对应 Java：`LoopMergeStrategy` (3 constructors + `afterRowDispose`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopMergeStrategy {
    pub(crate) each_rows: u32,
    pub(crate) column_extend: u16,
    pub(crate) column_index: u16,
}

impl LoopMergeStrategy {
    /// 创建单列循环合并策略。对应 Java `LoopMergeStrategy(int eachRow, int columnIndex)`。
    ///
    /// # Errors
    ///
    /// 参数不满足 Java 构造器约束时返回格式错误。
    pub fn with_column(each_rows: i32, column_index: i32) -> Result<Self> {
        Self::from_java_values(each_rows, 1, column_index)
    }

    /// 从注解属性创建策略。对应 Java `LoopMergeStrategy(LoopMergeProperty, Integer)`。
    ///
    /// # Errors
    ///
    /// 列下标为 null、负数或超过后端列范围时返回格式错误。
    pub fn from_property(
        property: LoopMergeProperty,
        column_index: Option<i32>,
    ) -> Result<Self> {
        let column_index = column_index.ok_or_else(|| {
            ExcelError::Format("ColumnIndex must not be null".to_owned())
        })?;
        let each_rows = i32::try_from(property.each_row).map_err(|_| {
            ExcelError::Format("EachRows exceeds Java int range".to_owned())
        })?;
        Self::from_java_values(each_rows, i32::from(property.column_extend), column_index)
    }

    fn from_java_values(each_rows: i32, column_extend: i32, column_index: i32) -> Result<Self> {
        if each_rows < 1 {
            return Err(ExcelError::Format(
                "EachRows must be greater than 1".to_owned(),
            ));
        }
        if column_extend < 1 {
            return Err(ExcelError::Format(
                "ColumnExtend must be greater than 1".to_owned(),
            ));
        }
        if column_extend == 1 && each_rows == 1 {
            return Err(ExcelError::Format(
                "ColumnExtend or eachRows must be greater than 1".to_owned(),
            ));
        }
        if column_index < 0 {
            return Err(ExcelError::Format(
                "ColumnIndex must be greater than 0".to_owned(),
            ));
        }
        let each_rows = u32::try_from(each_rows)
            .map_err(|_| ExcelError::Format("EachRows exceeds backend range".to_owned()))?;
        let column_extend = u16::try_from(column_extend)
            .map_err(|_| ExcelError::Format("ColumnExtend exceeds backend range".to_owned()))?;
        let column_index = u16::try_from(column_index)
            .map_err(|_| ExcelError::Format("ColumnIndex exceeds backend range".to_owned()))?;
        Self::new(each_rows, column_extend, column_index)
    }

    /// 对应 Java：com.alibaba.excel.write.merge.LoopMergeStrategy。 Creates a `LoopMergeStrategy` with the given dimensions. (Java
    /// `LoopMergeStrategy(int eachRow, int columnExtend, int columnIndex)`)
    ///
    /// # Errors
    ///
    /// Returns an error when `each_rows < 1`, `column_extend < 1`, or when
    /// Java's combined constraint `eachRow < 2 && columnExtend < 2` holds.
    pub fn new(each_rows: u32, column_extend: u16, column_index: u16) -> Result<Self> {
        // Java: eachRow < 1 → IllegalArgumentException("EachRows must be greater than 1")
        if each_rows < 1 {
            return Err(ExcelError::Format(
                "EachRows must be greater than 1".to_owned(),
            ));
        }
        // Java: columnExtend < 1 → IllegalArgumentException("ColumnExtend must be greater than 1")
        if column_extend < 1 {
            return Err(ExcelError::Format(
                "ColumnExtend must be greater than 1".to_owned(),
            ));
        }
        // Java: eachRow < 2 && columnExtend < 2 → IllegalArgumentException(
        //   "EachRows or ColumnExtend cannot be less than 2, otherwise they will not be merged")
        if each_rows == 1 && column_extend == 1 {
            return Err(ExcelError::Format(
                "ColumnExtend or eachRows must be greater than 1".to_owned(),
            ));
        }
        Ok(Self {
            each_rows,
            column_extend,
            column_index,
        })
    }

    /// Returns the per-group row count. (Java `getEachRow()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.LoopMergeStrategy。
    pub const fn each_rows(self) -> u32 {
        self.each_rows
    }

    /// Returns the per-group column count. (Java `getColumnExtend()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.LoopMergeStrategy。
    pub const fn column_extend(self) -> u16 {
        self.column_extend
    }

    /// Returns the zero-based column index. (Java `getColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.merge.LoopMergeStrategy。
    pub const fn column_index(self) -> u16 {
        self.column_index
    }
}

impl WriteHandler for LoopMergeStrategy {
    fn order(&self) -> i32 {
        // Java `LoopMergeStrategy` does not override `order()`.
        crate::constant::order_constant::DEFAULT_ORDER
    }

    fn style_loop_merge(&self) -> Option<(LoopMergeProperty, usize)> {
        Some((
            LoopMergeProperty::new(self.each_rows, self.column_extend),
            usize::from(self.column_index),
        ))
    }
}

impl AbstractMergeStrategy for LoopMergeStrategy {
    fn merge(
        &mut self,
        _sheet_name: &str,
        _cell: &WriteCellContext,
        _extra: Option<&CellExtra>,
        _relative_row_index: Option<i32>,
    ) {
        // `rust_xlsxwriter` is told to merge the range at write time via
        // `worksheet.merge_range(...)`. The template fill and main writer
        // paths consult this struct to discover the range; the actual
        // mutation happens in those callers.
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::CellValue;

    #[test]
    fn loop_merge_strategy_merge_default_body_runs() {
        let mut strategy = LoopMergeStrategy::new(2, 1, 0).expect("valid");
        let context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        strategy.merge("Sheet1", &context, None, Some(0));
    }

    #[test]
    fn new_valid_parameters() {
        let s = LoopMergeStrategy::new(3, 2, 0).unwrap();
        assert_eq!(s.each_rows(), 3);
        assert_eq!(s.column_extend(), 2);
        assert_eq!(s.column_index(), 0);
    }

    #[test]
    fn new_rejects_each_rows_zero() {
        assert!(LoopMergeStrategy::new(0, 1, 0).is_err());
    }

    #[test]
    fn new_rejects_column_extend_zero() {
        assert!(LoopMergeStrategy::new(2, 0, 0).is_err());
    }

    #[test]
    fn new_rejects_both_one() {
        assert!(LoopMergeStrategy::new(1, 1, 0).is_err());
    }

    #[test]
    fn new_accepts_each_rows_one_with_column_extend() {
        let s = LoopMergeStrategy::new(1, 3, 0).unwrap();
        assert_eq!(s.each_rows(), 1);
        assert_eq!(s.column_extend(), 3);
    }

    #[test]
    fn with_column_creates_single_column_strategy() {
        let s = LoopMergeStrategy::with_column(5, 2).unwrap();
        assert_eq!(s.each_rows(), 5);
        assert_eq!(s.column_extend(), 1);
        assert_eq!(s.column_index(), 2);
    }

    #[test]
    fn with_column_rejects_negative_params() {
        assert!(LoopMergeStrategy::with_column(-1, 0).is_err());
        assert!(LoopMergeStrategy::with_column(0, 0).is_err());
        assert!(LoopMergeStrategy::with_column(2, -1).is_err());
    }

    #[test]
    fn from_property_creates_strategy() {
        let prop = LoopMergeProperty::new(3, 2);
        let s = LoopMergeStrategy::from_property(prop, Some(1)).unwrap();
        assert_eq!(s.each_rows(), 3);
        assert_eq!(s.column_extend(), 2);
        assert_eq!(s.column_index(), 1);
    }

    #[test]
    fn from_property_rejects_none_column_index() {
        let prop = LoopMergeProperty::new(3, 2);
        assert!(LoopMergeStrategy::from_property(prop, None).is_err());
    }

    #[test]
    fn write_handler_order_returns_default() {
        let s = LoopMergeStrategy::new(2, 1, 0).unwrap();
        assert_eq!(s.order(), crate::constant::order_constant::DEFAULT_ORDER);
    }

    #[test]
    fn style_loop_merge_returns_property_and_column() {
        let s = LoopMergeStrategy::new(3, 2, 5).unwrap();
        let (prop, col) = s.style_loop_merge().unwrap();
        assert_eq!(prop.each_row, 3);
        assert_eq!(prop.column_extend, 2);
        assert_eq!(col, 5);
    }

    #[test]
    fn debug_format_works() {
        let s = LoopMergeStrategy::new(2, 1, 0).unwrap();
        let debug = format!("{:?}", s);
        assert!(debug.contains("LoopMergeStrategy"));
    }

    #[test]
    fn clone_and_eq() {
        let a = LoopMergeStrategy::new(2, 1, 0).unwrap();
        let b = a;
        assert_eq!(a, b);
    }
}
