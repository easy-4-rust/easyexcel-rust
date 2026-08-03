//! 对应 Java：`com.alibaba.excel.read.builder.ExcelReaderTableBuilder`.
//!
//! Java signature (5 members):
//! ```java
//! public class ExcelReaderTableBuilder
//!     extends AbstractExcelReaderParameterBuilder<ExcelReaderTableBuilder, ReadTable> {
//!     private ReadTable readTable;
//!     public ExcelReaderTableBuilder();
//!     public ExcelReaderTableBuilder(ExcelReader excelReader);
//!     public ExcelReaderTableBuilder tableNo(Integer tableNo);
//!     public ReadTable build();
//!     protected ReadTable parameter();
//! }
//! ```

use crate::core::ReadListener;

use crate::reader::excel_reader::ExcelReader;
use crate::reader::metadata::read_table::ReadTable;

/// 对应 Java：`ExcelReaderTableBuilder extends AbstractExcelReaderParameterBuilder`.
///
/// Rust: table-level configuration is sparse in this port because
/// `ReadTable` is an in-memory struct (the Java type itself is
/// minimal). The builder here mostly carries `head_row_number` and
/// `use_scientific_format` for parity with the sheet builder.
#[derive(Debug, Clone, Default)]
pub struct ExcelReaderTableBuilder {
    /// Mirrors `ExcelReaderTableBuilder.tableNo`.
    pub table_no: Option<i32>,
    /// Mirrors `AbstractExcelReaderParameterBuilder.headRowNumber`.
    pub head_row_number: Option<i32>,
    /// Mirrors `AbstractExcelReaderParameterBuilder.useScientificFormat`.
    pub use_scientific_format: Option<bool>,
}

impl ExcelReaderTableBuilder {
    /// Creates an empty table builder. (Java `ExcelReaderTableBuilder()`)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a table builder bound to an [`ExcelReader`].
    /// (Java `ExcelReaderTableBuilder(ExcelReader)`)
    #[must_use]
    pub fn with_excel_reader<T, L>(_excel_reader: &ExcelReader<T, L>) -> Self
    where
        T: crate::core::ExcelRow,
        L: ReadListener<T>,
    {
        Self::default()
    }

    /// Sets the zero-based table index. (Java `tableNo(Integer)`)
    #[must_use]
    pub fn table_no(mut self, table_no: i32) -> Self {
        self.table_no = Some(table_no);
        self
    }

    /// Returns the typed `ReadTable` view used by the reader.
    /// (Java `protected ReadTable parameter()`)
    #[must_use]
    pub fn parameter(&self) -> ReadTable {
        self.build()
    }

    /// Builds the underlying table configuration. (Java `ReadTable build()`)
    ///
    /// Rust port: returns a `ReadTable` carrying the configured
    /// `table_no`. Callers compose this with `ExcelReader::table(...)`.
    #[must_use]
    pub fn build(&self) -> ReadTable {
        ReadTable::with_table_no(self.table_no.unwrap_or(0))
    }

    /// Sets the head row number. (Java
    /// `AbstractExcelReaderParameterBuilder.headRowNumber(Integer)`)
    #[must_use]
    pub fn head_row_number(mut self, head_row_number: i32) -> Self {
        self.head_row_number = Some(head_row_number);
        self
    }

    /// Toggles scientific-format coercion. (Java
    /// `AbstractExcelReaderParameterBuilder.useScientificFormat(Boolean)`)
    #[must_use]
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.use_scientific_format = Some(enabled);
        self
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(builder.head_row_number, None);
        assert_eq!(builder.use_scientific_format, None);

        // 未设置 tableNo 时默认 0（对应 Java：ReadTable 默认 tableNo）
        let table = ExcelReaderTableBuilder::new().build();
        assert_eq!(table.table_no(), 0);
        Ok(())
    }

    #[test]
    fn table_builder_stores_parameter_builder_knobs() {
        // 对应 Java：AbstractExcelReaderParameterBuilder 继承方法
        let builder = ExcelReaderTableBuilder::new()
            .head_row_number(2)
            .use_scientific_format(true);
        assert_eq!(builder.head_row_number, Some(2));
        assert_eq!(builder.use_scientific_format, Some(true));
    }
}
