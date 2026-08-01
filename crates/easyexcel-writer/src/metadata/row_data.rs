//! Mirrors Java `com.alibaba.excel.write.metadata.RowData` (interface).

use easyexcel_core::CellValue;

/// Mirrors Java `RowData` interface (one method: `getCellValue(int)`).
///
/// Java models each cell of a basic-type row through a common interface so
/// `ExcelWriteAddExecutor` can branch on `CollectionRowData`, `MapRowData`,
/// or JavaBean row uniformly. Rust achieves the same uniformity by
/// accepting `&[CellValue]` slices from any source, so this trait is a
/// 1:1 API marker without runtime polymorphism.
pub trait RowData {
    /// Returns the cell value at the given column index. (Java `getCellValue(int)`)
    fn get_cell_value(&self, column_index: usize) -> Option<&CellValue>;

    /// Returns whether the row carries any value. (Java `isEmpty()`)
    fn is_empty(&self) -> bool;
}

impl RowData for [CellValue] {
    fn get_cell_value(&self, column_index: usize) -> Option<&CellValue> {
        self.get(column_index)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl RowData for Vec<CellValue> {
    fn get_cell_value(&self, column_index: usize) -> Option<&CellValue> {
        self.get(column_index)
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
        let cells = vec![
            CellValue::String("name".to_owned()),
            CellValue::Float(42.0),
        ];
        let slice: &[CellValue] = &cells;

        assert_eq!(
            slice.get_cell_value(0),
            Some(&CellValue::String("name".to_owned()))
        );
        assert_eq!(
            slice.get_cell_value(1),
            Some(&CellValue::Float(42.0))
        );
        assert_eq!(slice.get_cell_value(2), None);
    }

    #[test]
    fn slice_row_data_is_empty_on_empty_slice() {
        let cells: Vec<CellValue> = vec![];
        let slice: &[CellValue] = &cells;
        assert!(slice.is_empty());
    }

    #[test]
    fn slice_row_data_is_not_empty_on_non_empty_slice() {
        let cells = vec![CellValue::String("a".to_owned())];
        let slice: &[CellValue] = &cells;
        assert!(!slice.is_empty());
    }

    #[test]
    fn vec_row_data_returns_cell_value_at_index() {
        let cells = vec![
            CellValue::String("name".to_owned()),
            CellValue::Float(42.0),
        ];

        assert_eq!(
            cells.get_cell_value(0),
            Some(&CellValue::String("name".to_owned()))
        );
        assert_eq!(
            cells.get_cell_value(1),
            Some(&CellValue::Float(42.0))
        );
        assert_eq!(cells.get_cell_value(2), None);
    }

    #[test]
    fn vec_row_data_is_empty_on_empty_vec() {
        let cells: Vec<CellValue> = vec![];
        assert!(cells.is_empty());
    }

    #[test]
    fn vec_row_data_is_not_empty_on_non_empty_vec() {
        let cells = vec![CellValue::String("a".to_owned())];
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

    use easyexcel_core::CellValue;

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
