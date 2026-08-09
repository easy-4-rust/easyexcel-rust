//! 对应 Java：`com.alibaba.excel.write.metadata.RowData` (interface).

use crate::core::CellValue;

/// 对应 Java：`RowData` interface。
///
/// Java models each cell of a basic-type row through a common interface so
/// `ExcelWriteAddExecutor` can branch on `CollectionRowData`, `MapRowData`,
/// or `JavaBean` row uniformly. Rust achieves the same uniformity by
/// accepting `&[CellValue]` slices from any source, so this trait is a
/// 1:1 API marker without runtime polymorphism.
pub trait RowData {
    /// 返回指定列的数据；越界时返回 `None`。对应 Java `get(int)` 返回 `null`。
    fn get(&self, index: usize) -> Option<&CellValue>;

    /// 返回元素数量。Rust 容器无法超过地址空间，因此无需 Java 的 `Integer.MAX_VALUE` 饱和分支。
    fn size(&self) -> usize;

    /// Returns whether the row carries any value. (Java `isEmpty()`)
    fn is_empty(&self) -> bool;

    /// 保留早期 Rust API 名称，委托给 Java `get(int)` 契约。
    fn get_cell_value(&self, column_index: usize) -> Option<&CellValue> {
        self.get(column_index)
    }
}

impl RowData for [CellValue] {
    fn get(&self, index: usize) -> Option<&CellValue> {
        <[CellValue]>::get(self, index)
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl RowData for Vec<CellValue> {
    fn get(&self, index: usize) -> Option<&CellValue> {
        self.as_slice().get(index)
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_row_data_returns_cell_value_at_index() {
        let cells = vec![CellValue::String("name".to_owned()), CellValue::Float(42.0)];
        let slice: &[CellValue] = &cells;

        assert_eq!(
            slice.get_cell_value(0),
            Some(&CellValue::String("name".to_owned()))
        );
        assert_eq!(slice.get_cell_value(1), Some(&CellValue::Float(42.0)));
        assert_eq!(slice.get_cell_value(2), None);
    }

    #[test]
    fn slice_row_data_is_empty_on_empty_slice() {
        let cells: Vec<CellValue> = Vec::new();
        let slice: &[CellValue] = &cells;
        assert!(slice.is_empty());
    }

    #[test]
    fn slice_row_data_is_not_empty_on_non_empty_slice() {
        let cells = [CellValue::String("a".to_owned())];
        let slice: &[CellValue] = &cells;
        assert!(!slice.is_empty());
    }

    #[test]
    fn vec_row_data_returns_cell_value_at_index() {
        let cells = vec![CellValue::String("name".to_owned()), CellValue::Float(42.0)];

        assert_eq!(
            cells.get_cell_value(0),
            Some(&CellValue::String("name".to_owned()))
        );
        assert_eq!(cells.get_cell_value(1), Some(&CellValue::Float(42.0)));
        assert_eq!(cells.get_cell_value(2), None);
    }

    #[test]
    fn vec_row_data_is_empty_on_empty_vec() {
        let cells: Vec<CellValue> = Vec::new();
        assert!(cells.is_empty());
    }

    #[test]
    fn vec_row_data_is_not_empty_on_non_empty_vec() {
        let cells = [CellValue::String("a".to_owned())];
        assert!(!cells.is_empty());
    }

    #[test]
    fn row_data_get_cell_value_returns_none_for_out_of_bounds() {
        let cells = vec![CellValue::Empty];
        assert_eq!(cells.get_cell_value(5), None);
    }

    #[test]
    fn row_data_handles_empty_cell_value() {
        let cells = vec![CellValue::Empty, CellValue::Bool(true)];
        assert_eq!(cells.get_cell_value(0), Some(&CellValue::Empty));
        assert_eq!(cells.get_cell_value(1), Some(&CellValue::Bool(true)));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::CellValue;

    #[test]
    fn row_data_slice_and_vec_is_empty() {
        let empty_vec: Vec<CellValue> = Vec::new();
        assert!(<Vec<CellValue> as RowData>::is_empty(&empty_vec));
        let empty_slice: &[CellValue] = &[];
        assert!(<[CellValue] as RowData>::is_empty(empty_slice));
        let non_empty = vec![CellValue::String("x".to_owned())];
        assert!(!<Vec<CellValue> as RowData>::is_empty(&non_empty));
        let slice: &[CellValue] = &non_empty;
        assert!(!<[CellValue] as RowData>::is_empty(slice));
    }
}
