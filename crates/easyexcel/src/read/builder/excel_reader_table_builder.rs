//! Pre-4.x `ExcelReaderTableBuilder` source-compatibility metadata.
//!
//! This class is absent from the pinned Java `EasyExcel` 4.0.3 source tree and
//! therefore does not count toward the 4.0.3 parity inventory.

use crate::core::ReadListener;

use crate::read::excel_reader::ExcelReader;
use crate::read::metadata::read_table::ReadTable;

/// Pre-4.x compatibility metadata; this type is absent from Java `EasyExcel`
/// 4.0.3 and is not part of the 4.0.3 parity surface.
///
/// It remains available for downstream source compatibility, but it does not
/// bind or execute an [`ExcelReader`]. Use [`super::excel_reader_sheet_builder::ExcelReaderSheetBuilder`]
/// for executable per-sheet configuration.
#[deprecated(note = "absent from EasyExcel 4.0.3; use ExcelReaderSheetBuilder")]
#[derive(Debug, Clone, Default)]
pub struct ExcelReaderTableBuilder {
    /// Mirrors `ExcelReaderTableBuilder.tableNo`.
    pub table_no: Option<i32>,
}

#[allow(deprecated)]
impl ExcelReaderTableBuilder {
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderTableBuilder。 Creates an empty table builder. (Java `ExcelReaderTableBuilder()`)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderTableBuilder。 Creates a table builder bound to an [`ExcelReader`].
    /// (Java `ExcelReaderTableBuilder(ExcelReader)`)
    #[must_use]
    pub fn with_excel_reader<T, L>(_excel_reader: &ExcelReader<T, L>) -> Self
    where
        T: crate::core::ExcelRow,
        L: ReadListener<T>,
    {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderTableBuilder。 Sets the zero-based table index. (Java `tableNo(Integer)`)
    #[must_use]
    pub fn table_no(mut self, table_no: i32) -> Self {
        self.table_no = Some(table_no);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderTableBuilder。 Returns the typed `ReadTable` view used by the reader.
    /// (Java `protected ReadTable parameter()`)
    #[must_use]
    pub fn parameter(&self) -> ReadTable {
        self.build()
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderTableBuilder。 Builds the underlying table configuration. (Java `ReadTable build()`)
    ///
    /// Rust port: returns a `ReadTable` carrying the configured
    /// `table_no`. Callers compose this with `ExcelReader::table(...)`.
    #[must_use]
    pub fn build(&self) -> ReadTable {
        ReadTable::with_table_no(self.table_no.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    use std::io::Write;

    use crate::core::{AnalysisContext, DynamicRow, ReadListener, Result};
    use tempfile::NamedTempFile;

    use super::*;
    use crate::ReadOptions;

    #[derive(Default)]
    struct CollectListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for CollectListener {
        fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    #[test]
    fn table_builder_parameter_and_build_round_trip() {
        // 对应 Java：ExcelReaderTableBuilder.tableNo/build()/parameter()
        let builder = ExcelReaderTableBuilder::new().table_no(3);
        assert_eq!(builder.table_no, Some(3));
        let table = builder.build();
        assert_eq!(table.table_no(), 3);
        assert_eq!(builder.parameter(), table);
    }

    #[test]
    fn table_builder_with_excel_reader_starts_defaulted() -> Result<()> {
        // 对应 Java：ExcelReaderTableBuilder(ExcelReader) 构造
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "alice,30")?;
        let mut reader = ExcelReader::new(
            file.path(),
            ReadOptions::default(),
            CollectListener::default(),
        )?;

        // 接线 CollectListener：真实读取触发 invoke，收集数据行
        let mut observed = CollectListener::default();
        reader.read_all_with_additional_listener(&mut observed)?;
        assert_eq!(observed.rows.len(), 1);
        assert_eq!(
            observed.rows[0].get(0),
            Some(&crate::core::DynamicValue::String("alice".to_owned()))
        );

        let builder = ExcelReaderTableBuilder::with_excel_reader(&reader);
        assert_eq!(builder.table_no, None);

        // 未设置 tableNo 时默认 0（对应 Java：ReadTable 默认 tableNo）
        let table = ExcelReaderTableBuilder::new().build();
        assert_eq!(table.table_no(), 0);
        Ok(())
    }
}
