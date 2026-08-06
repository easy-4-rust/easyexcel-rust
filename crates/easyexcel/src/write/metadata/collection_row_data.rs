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
