//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadRowHolder`.

use std::collections::HashMap;

use easyexcel_core::CellValue;

/// 对应 Java：`ReadRowHolder implements Holder`.
#[derive(Debug, Clone)]
pub struct ReadRowHolder {
    /// Mirrors `ReadRowHolder.rowIndex`.
    pub row_index: i32,
    /// Mirrors `ReadRowHolder.cellMap`.
    pub cell_map: HashMap<usize, CellValue>,
}

impl ReadRowHolder {
    /// 对应 Java： constructor.
    #[must_use]
    pub fn new(row_index: i32, cell_map: HashMap<usize, CellValue>) -> Self {
        Self {
            row_index,
            cell_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_holder_new_carries_index_and_cells() {
        // 对应 Java：ReadRowHolder 构造携带 rowIndex 与 cellMap
        let mut cells = HashMap::new();
        cells.insert(1usize, CellValue::String("v".to_owned()));
        let holder = ReadRowHolder::new(3, cells.clone());
        assert_eq!(holder.row_index, 3);
        assert_eq!(holder.cell_map, cells);
    }
}
