//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadRowHolder`.

use indexmap::IndexMap;

use crate::core::CellValue;
use crate::{CustomReadObject, GlobalConfiguration, HolderEnum, RowTypeEnum};

/// 对应 Java：`ReadRowHolder implements Holder`.
#[derive(Debug, Clone)]
pub struct ReadRowHolder {
    /// Mirrors `ReadRowHolder.rowIndex`.
    pub row_index: i32,
    /// Mirrors `ReadRowHolder.cellMap`.
    pub cell_map: IndexMap<usize, CellValue>,
    row_type: RowTypeEnum,
    global_configuration: GlobalConfiguration,
    current_row_analysis_result: Option<CustomReadObject>,
}

impl ReadRowHolder {
    /// 对应 Java： constructor.
    #[must_use]
    pub fn new(
        row_index: i32,
        cell_map: impl IntoIterator<Item = (usize, CellValue)>,
    ) -> Self {
        Self {
            row_index,
            cell_map: cell_map.into_iter().collect(),
            row_type: RowTypeEnum::Data,
            global_configuration: GlobalConfiguration::default(),
            current_row_analysis_result: None,
        }
    }

    /// Java 完整构造器。
    #[must_use]
    pub fn new_with_metadata(
        row_index: i32,
        row_type: RowTypeEnum,
        global_configuration: GlobalConfiguration,
        cell_map: impl IntoIterator<Item = (usize, CellValue)>,
    ) -> Self {
        Self {
            row_index,
            cell_map: cell_map.into_iter().collect(),
            row_type,
            global_configuration,
            current_row_analysis_result: None,
        }
    }

    #[must_use] pub const fn get_row_index(&self) -> i32 { self.row_index }
    pub const fn set_row_index(&mut self, value: i32) { self.row_index = value; }
    #[must_use] pub const fn get_row_type(&self) -> RowTypeEnum { self.row_type }
    pub const fn set_row_type(&mut self, value: RowTypeEnum) { self.row_type = value; }
    #[must_use] pub const fn get_cell_map(&self) -> &IndexMap<usize, CellValue> { &self.cell_map }
    pub fn set_cell_map(
        &mut self,
        value: impl IntoIterator<Item = (usize, CellValue)>,
    ) {
        self.cell_map = value.into_iter().collect();
    }
    #[must_use] pub const fn get_global_configuration(&self) -> &GlobalConfiguration {
        &self.global_configuration
    }
    pub fn set_global_configuration(&mut self, value: GlobalConfiguration) {
        self.global_configuration = value;
    }
    #[must_use] pub const fn get_current_row_analysis_result(&self) -> Option<&CustomReadObject> {
        self.current_row_analysis_result.as_ref()
    }
    pub fn set_current_row_analysis_result(&mut self, value: Option<CustomReadObject>) {
        self.current_row_analysis_result = value;
    }
    #[must_use] pub const fn holder_type(&self) -> HolderEnum { HolderEnum::Row }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_holder_new_carries_index_and_cells() {
        // 对应 Java：ReadRowHolder 构造携带 rowIndex 与 cellMap
        let mut cells = IndexMap::new();
        cells.insert(1usize, CellValue::String("v".to_owned()));
        let holder = ReadRowHolder::new(3, cells.clone());
        assert_eq!(holder.row_index, 3);
        assert_eq!(holder.cell_map, cells);
    }

    #[test]
    fn new_with_metadata_sets_row_type() {
        // 对应 Java：ReadRowHolder 完整构造器
        let holder = ReadRowHolder::new_with_metadata(
            5,
            RowTypeEnum::Data,
            GlobalConfiguration::default(),
            vec![(0, CellValue::Int(42))],
        );
        assert_eq!(holder.get_row_index(), 5);
        assert_eq!(holder.get_row_type(), RowTypeEnum::Data);
    }

    #[test]
    fn set_row_index_and_get() {
        // 对应 Java：setRowIndex / getRowIndex
        let mut holder = ReadRowHolder::new(0, vec![]);
        holder.set_row_index(10);
        assert_eq!(holder.get_row_index(), 10);
    }

    #[test]
    fn set_row_type_and_get() {
        // 对应 Java：setRowType / getRowType
        let mut holder = ReadRowHolder::new(0, vec![]);
        holder.set_row_type(RowTypeEnum::Empty);
        assert_eq!(holder.get_row_type(), RowTypeEnum::Empty);
    }

    #[test]
    fn set_cell_map_and_get() {
        // 对应 Java：setCellMap / getCellMap
        let mut holder = ReadRowHolder::new(0, vec![]);
        holder.set_cell_map(vec![
            (0, CellValue::String("a".to_owned())),
            (1, CellValue::Int(1)),
        ]);
        assert_eq!(holder.get_cell_map().len(), 2);
    }

    #[test]
    fn global_configuration_accessor() {
        // 对应 Java：globalConfiguration getter/setter
        let mut holder = ReadRowHolder::new(0, vec![]);
        let _gc: &GlobalConfiguration = holder.get_global_configuration();
        holder.set_global_configuration(GlobalConfiguration::default());
    }

    #[test]
    fn current_row_analysis_result_accessor() {
        // 对应 Java：currentRowAnalysisResult getter/setter
        let mut holder = ReadRowHolder::new(0, vec![]);
        assert!(holder.get_current_row_analysis_result().is_none());
        holder.set_current_row_analysis_result(None);
        assert!(holder.get_current_row_analysis_result().is_none());
    }

    #[test]
    fn holder_type_is_row() {
        // 对应 Java：HolderEnum.Row
        let holder = ReadRowHolder::new(0, vec![]);
        assert_eq!(holder.holder_type(), HolderEnum::Row);
    }

    #[test]
    fn clone_produces_equal() {
        // 对应 Java：clone
        let holder = ReadRowHolder::new(1, vec![(0, CellValue::Bool(true))]);
        let cloned = holder.clone();
        assert_eq!(holder.row_index, cloned.row_index);
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString
        let holder = ReadRowHolder::new(0, vec![]);
        let _debug = format!("{holder:?}");
    }
}
