//! 对应 Java：`com.alibaba.excel.write.metadata.CollectionRowData`.

/// 对应 Java：`CollectionRowData implements RowData`.
///
/// Java wraps a `Collection<?>` of raw values for a no-model row. The Rust
/// port is a tuple newtype that holds the same `Vec<CellValue>`. It implements
/// [`crate::core::ExcelRow`], so it can enter both the public writer facade
/// and `ExcelWriteAddExecutor`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollectionRowData(pub Vec<crate::core::CellValue>);

impl CollectionRowData {
    /// 对应 Java：com.alibaba.excel.write.metadata.CollectionRowData。 Creates a `CollectionRowData` mirroring Java's constructor.
    #[must_use]
    pub fn new(values: Vec<crate::core::CellValue>) -> Self {
        Self(values)
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.CollectionRowData。 Returns the underlying values. (Java `getCollection()` equivalent)
    #[must_use]
    pub fn values(&self) -> &[crate::core::CellValue] {
        &self.0
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.CollectionRowData。 Returns whether the row is empty. (Java `RowData.isEmpty()`)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 返回指定下标的数据。对应 Java `CollectionRowData#get(int)`。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&crate::core::CellValue> {
        self.0.get(index)
    }

    /// 返回集合大小。对应 Java `CollectionRowData#size()`。
    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl super::row_data::RowData for CollectionRowData {
    fn get(&self, index: usize) -> Option<&crate::core::CellValue> {
        self.get(index)
    }

    fn size(&self) -> usize {
        self.size()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl crate::core::ExcelRow for CollectionRowData {
    fn schema() -> &'static [crate::core::ExcelColumn] {
        &[]
    }

    fn from_row(row: &crate::core::RowData) -> crate::core::Result<Self> {
        let actual_row = row
            .clone()
            .with_read_default_return(crate::core::ReadDefaultReturn::ActualData);
        let dynamic = <crate::core::DynamicRow as crate::core::ExcelRow>::from_row(&actual_row)?;
        Ok(Self(
            <crate::core::DynamicRow as crate::core::ExcelRow>::to_row(&dynamic)?,
        ))
    }

    fn to_row(&self) -> crate::core::Result<Vec<crate::core::CellValue>> {
        Ok(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CellValue;

    #[test]
    fn new_and_values() {
        let data = CollectionRowData::new(vec![
            CellValue::String("hello".to_owned()),
            CellValue::Int(42),
        ]);
        assert_eq!(data.values().len(), 2);
        assert_eq!(data.size(), 2);
        assert!(!data.is_empty());
    }

    #[test]
    fn empty_collection() {
        let data = CollectionRowData::new(vec![]);
        assert!(data.is_empty());
        assert_eq!(data.size(), 0);
    }

    #[test]
    fn get_by_index() {
        let data = CollectionRowData::new(vec![
            CellValue::Int(10),
            CellValue::Int(20),
        ]);
        assert!(data.get(0).is_some());
        assert!(data.get(2).is_none());
    }

    #[test]
    fn default_is_empty() {
        let data = CollectionRowData::default();
        assert!(data.is_empty());
    }

    #[test]
    fn clone_and_eq() {
        let a = CollectionRowData::new(vec![CellValue::Bool(true)]);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_fmt() {
        let data = CollectionRowData::new(vec![]);
        assert!(format!("{:?}", data).contains("CollectionRowData"));
    }

    #[test]
    fn row_data_trait_impl() {
        use super::super::row_data::RowData;
        let data = CollectionRowData::new(vec![CellValue::Float(1.5)]);
        let rd: &dyn RowData = &data;
        assert_eq!(rd.size(), 1);
        assert!(!rd.is_empty());
        assert!(rd.get(0).is_some());
    }

    #[test]
    fn excel_row_to_row_roundtrip() {
        use crate::core::ExcelRow;
        let data = CollectionRowData::new(vec![CellValue::String("x".to_owned())]);
        let row = data.to_row().expect("to_row");
        assert_eq!(row.len(), 1);
    }
}
