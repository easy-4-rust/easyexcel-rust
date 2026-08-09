//! 对应 Java：`com.alibaba.excel.write.metadata.MapRowData`.

use std::collections::BTreeMap;

use crate::core::CellValue;

/// 对应 Java：`MapRowData implements RowData`.
///
/// Java wraps a `Map<Integer, ?>` and its `RowData` adapter reports
/// `map.size()` then calls `map.get(0..size)`. Rust preserves that exact,
/// occasionally surprising contiguous-key contract. Use
/// [`crate::core::DynamicRow`] when sparse physical-column semantics are
/// desired instead.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapRowData(pub BTreeMap<usize, CellValue>);

impl MapRowData {
    /// 对应 Java：com.alibaba.excel.write.metadata.MapRowData。 Creates a `MapRowData` from a column-indexed map.
    #[must_use]
    pub fn new(values: BTreeMap<usize, CellValue>) -> Self {
        Self(values)
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.MapRowData。 Returns the underlying map. (Java `getMap()` equivalent)
    #[must_use]
    pub fn values(&self) -> &BTreeMap<usize, CellValue> {
        &self.0
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.MapRowData。 Returns whether the row is empty. (Java `RowData.isEmpty()`)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 按 Java 的连续整数键契约返回数据。对应 `MapRowData#get(int)`。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&CellValue> {
        self.0.get(&index)
    }

    /// 返回 Map 条目数量。对应 Java `MapRowData#size()`。
    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl super::row_data::RowData for MapRowData {
    fn get(&self, index: usize) -> Option<&CellValue> {
        self.get(index)
    }

    fn size(&self) -> usize {
        self.size()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl crate::core::ExcelRow for MapRowData {
    fn schema() -> &'static [crate::core::ExcelColumn] {
        &[]
    }

    fn from_row(row: &crate::core::RowData) -> crate::core::Result<Self> {
        let actual_row = row
            .clone()
            .with_read_default_return(crate::core::ReadDefaultReturn::ActualData);
        let dynamic = <crate::core::DynamicRow as crate::core::ExcelRow>::from_row(&actual_row)?;
        let values = <crate::core::DynamicRow as crate::core::ExcelRow>::to_row(&dynamic)?
            .into_iter()
            .enumerate()
            .collect();
        Ok(Self(values))
    }

    fn to_row(&self) -> crate::core::Result<Vec<CellValue>> {
        Ok((0..self.0.len())
            .map(|index| self.0.get(&index).cloned().unwrap_or(CellValue::Empty))
            .collect())
    }
}
