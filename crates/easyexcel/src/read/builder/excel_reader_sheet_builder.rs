//! 对应 Java：`com.alibaba.excel.read.builder.ExcelReaderSheetBuilder`.
//!
//! Java signature (10 methods + `AbstractExcelReaderParameterBuilder`
//! inherited methods):
//! ```java
//! public class ExcelReaderSheetBuilder
//!     extends AbstractExcelReaderParameterBuilder<ExcelReaderSheetBuilder, ReadSheet> {
//!     public ExcelReaderSheetBuilder();
//!     public ExcelReaderSheetBuilder(ExcelReader excelReader);
//!     public ExcelReaderSheetBuilder sheetNo(Integer sheetNo);
//!     public ExcelReaderSheetBuilder sheetName(String sheetName);
//!     public ReadSheet build();
//!     public void doRead();
//!     public <T> List<T> doReadSync();
//!     protected ReadSheet parameter();
//!     // inherited from AbstractExcelReaderParameterBuilder:
//!     public T headRowNumber(Integer headRowNumber);
//!     public T useScientificFormat(Boolean useScientificFormat);
//!     public T registerReadListener(ReadListener<?> readListener);
//! }
//! ```

use crate::core::{AnalysisContext, ExcelRow, ReadListener, Result};

use crate::context::read_sheet::ReadSheet;
use crate::read::excel_reader::ExcelReader;

/// 对应 Java：`ExcelReaderSheetBuilder extends AbstractExcelReaderParameterBuilder`.
///
/// The unbound form mirrors Java's `EasyExcelFactory.readSheet(...)` and only
/// carries metadata. [`Self::with_excel_reader`] returns a borrowed bound
/// builder that provides Java-compatible `do_read` and `do_read_sync`
/// lifecycle methods without storing an unsafe self-reference.
#[derive(Debug, Clone, Default)]
pub struct ExcelReaderSheetBuilder {
    /// Mirrors `ExcelReaderSheetBuilder.sheetNo`.
    pub sheet_no: Option<i32>,
    /// Mirrors `ExcelReaderSheetBuilder.sheetName`.
    pub sheet_name: Option<String>,
    /// Mirrors `AbstractExcelReaderParameterBuilder.headRowNumber`.
    pub head_row_number: Option<i32>,
    /// Mirrors `AbstractExcelReaderParameterBuilder.useScientificFormat`.
    pub use_scientific_format: Option<bool>,
}

impl ExcelReaderSheetBuilder {
    /// Creates an empty sheet builder. (Java `ExcelReaderSheetBuilder()`)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sheet builder bound to an [`ExcelReader`].
    /// (Java `ExcelReaderSheetBuilder(ExcelReader excelReader)`)
    ///
    #[must_use]
    pub fn with_excel_reader<T, L>(
        excel_reader: &mut ExcelReader<T, L>,
    ) -> BoundExcelReaderSheetBuilder<'_, T, L>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        BoundExcelReaderSheetBuilder {
            excel_reader,
            sheet_builder: Self::default(),
        }
    }

    /// Sets the zero-based sheet index. (Java `sheetNo(Integer)`)
    #[must_use]
    pub fn sheet_no(mut self, sheet_no: i32) -> Self {
        self.sheet_no = Some(sheet_no);
        self
    }

    /// Sets the sheet name. (Java `sheetName(String)`)
    #[must_use]
    pub fn sheet_name(mut self, sheet_name: impl Into<String>) -> Self {
        self.sheet_name = Some(sheet_name.into());
        self
    }

    /// Returns the typed `ReadSheet` view used by the reader.
    /// (Java `protected ReadSheet parameter()`)
    #[must_use]
    pub fn parameter(&self) -> ReadSheet {
        self.build()
    }

    /// Returns the finalised [`ReadSheet`] for use with the
    /// [`ExcelReader`]. (Java `public ReadSheet build()`)
    ///
    /// `Sheet_no` defaults to 0 (matching Java's `new ReadSheet()`).
    #[must_use]
    // 对应 Java：`Math.max(sheetNo/headRowNumber, 0)` 保证非负后转 usize/u32，
    // 符号位必然为 0 且值不超目标类型上限，`as` 转换不丢失符号或截断。
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn build(&self) -> ReadSheet {
        let mut sheet = match (self.sheet_no, &self.sheet_name) {
            (Some(no), Some(name)) => ReadSheet::with_name(no.max(0) as usize, name.clone()),
            (Some(no), None) => ReadSheet::new(no.max(0) as usize),
            (None, Some(name)) => ReadSheet::named(name.clone()),
            (None, None) => ReadSheet::default_construction(),
        };
        if let Some(head_row_number) = self.head_row_number {
            sheet.set_head_row_number(head_row_number.max(0) as u32);
        }
        if let Some(enabled) = self.use_scientific_format {
            sheet.set_use_scientific_format(enabled);
        }
        sheet
    }

    /// Sets the head row number (zero-based). (Java
    /// `AbstractExcelReaderParameterBuilder.headRowNumber(Integer)`)
    #[must_use]
    pub fn head_row_number(mut self, head_row_number: i32) -> Self {
        self.head_row_number = Some(head_row_number);
        self
    }

    /// Toggles scientific-format coercion for numeric cells.
    /// (Java `AbstractExcelReaderParameterBuilder.useScientificFormat(Boolean)`)
    #[must_use]
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.use_scientific_format = Some(enabled);
        self
    }
}

/// A sheet builder borrowing the reader that will execute it.
///
/// Java stores an `ExcelReader` field directly on `ExcelReaderSheetBuilder`.
/// Rust expresses the same ownership relation with an exclusive borrow, which
/// prevents the reader from being used concurrently while sheet options are
/// being assembled and executed.
pub struct BoundExcelReaderSheetBuilder<'a, T, L> {
    excel_reader: &'a mut ExcelReader<T, L>,
    sheet_builder: ExcelReaderSheetBuilder,
}

impl<T, L> BoundExcelReaderSheetBuilder<'_, T, L>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    /// Sets the zero-based sheet index.
    #[must_use]
    pub fn sheet_no(mut self, sheet_no: i32) -> Self {
        self.sheet_builder = self.sheet_builder.sheet_no(sheet_no);
        self
    }

    /// Sets the sheet name.
    #[must_use]
    pub fn sheet_name(mut self, sheet_name: impl Into<String>) -> Self {
        self.sheet_builder = self.sheet_builder.sheet_name(sheet_name);
        self
    }

    /// Sets the number of header rows for this sheet.
    #[must_use]
    pub fn head_row_number(mut self, head_row_number: i32) -> Self {
        self.sheet_builder = self.sheet_builder.head_row_number(head_row_number);
        self
    }

    /// Controls scientific formatting for this sheet.
    #[must_use]
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.sheet_builder = self.sheet_builder.use_scientific_format(enabled);
        self
    }

    /// Builds the sheet metadata without executing it.
    #[must_use]
    pub fn build(&self) -> ReadSheet {
        self.sheet_builder.build()
    }

    /// Reads the configured sheet, then finishes the bound reader.
    ///
    /// This is the Rust equivalent of Java
    /// `ExcelReaderSheetBuilder.doRead()`.
    ///
    /// # Errors
    ///
    /// 当工作表解析失败时返回 [`ExcelError`]。
    pub fn do_read(self) -> Result<()> {
        let sheet = self.sheet_builder.build();
        self.excel_reader.read(&[sheet])?;
        self.excel_reader.finish();
        Ok(())
    }

    /// Reads the configured sheet and returns all converted rows.
    ///
    /// The bound reader's existing listener runs first, followed by the
    /// synchronous collecting listener, matching Java listener registration
    /// order.
    ///
    /// # Errors
    ///
    /// 当工作表解析失败时返回 [`ExcelError`]。
    pub fn do_read_sync(self) -> Result<Vec<T>>
    where
        T: Clone,
    {
        let sheet = self.sheet_builder.build();
        let mut listener = SheetSyncReadListener::default();
        self.excel_reader
            .read_with_additional_listener(&[sheet], &mut listener)?;
        self.excel_reader.finish();
        Ok(listener.rows)
    }
}

struct SheetSyncReadListener<T> {
    rows: Vec<T>,
}

impl<T> Default for SheetSyncReadListener<T> {
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}

impl<T> ReadListener<T> for SheetSyncReadListener<T> {
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> Result<()> {
        self.rows.push(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::core::DynamicRow;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::ReadOptions;

    struct CountingListener(Arc<AtomicUsize>);

    impl ReadListener<DynamicRow> for CountingListener {
        fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn csv_fixture() -> Result<NamedTempFile> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "alice,30")?;
        Ok(file)
    }

    #[test]
    fn bound_builder_do_read_executes_the_selected_sheet() -> Result<()> {
        let file = csv_fixture()?;
        let invoked = Arc::new(AtomicUsize::new(0));
        let listener = CountingListener(Arc::clone(&invoked));
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;

        ExcelReaderSheetBuilder::with_excel_reader(&mut reader)
            .sheet_no(0)
            .do_read()?;

        assert_eq!(invoked.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[test]
    fn bound_builder_do_read_sync_keeps_existing_listener_order_and_collects() -> Result<()> {
        let file = csv_fixture()?;
        let invoked = Arc::new(AtomicUsize::new(0));
        let listener = CountingListener(Arc::clone(&invoked));
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;

        let rows = ExcelReaderSheetBuilder::with_excel_reader(&mut reader)
            .sheet_name("Sheet1")
            .do_read_sync()?;

        assert_eq!(invoked.load(Ordering::SeqCst), 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get(0),
            Some(&crate::core::DynamicValue::String("alice".to_owned()))
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::core::DynamicRow;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::ReadOptions;

    struct SheetCountingListener(Arc<AtomicUsize>);

    impl ReadListener<DynamicRow> for SheetCountingListener {
        fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn counting_listener() -> SheetCountingListener {
        SheetCountingListener(Arc::new(AtomicUsize::new(0)))
    }

    #[test]
    fn unbound_builder_build_applies_all_sheet_parameters() {
        // 对应 Java：ExcelReaderSheetBuilder.build() 参数装配
        let sheet = ExcelReaderSheetBuilder::new()
            .sheet_no(-1)
            .head_row_number(3)
            .use_scientific_format(true)
            .build();
        assert_eq!(sheet.sheet_no(), 0);
        assert_eq!(sheet.head_row_number(), Some(3));
        assert_eq!(sheet.use_scientific_format(), Some(true));

        // sheet_name 单名 + 负数 sheet_no 收敛到 0（对应 Java：sheetNo.max(0)）
        let sheet = ExcelReaderSheetBuilder::new()
            .sheet_name("Report")
            .sheet_no(-5)
            .head_row_number(-2)
            .build();
        assert_eq!(sheet.sheet_name(), "Report");
        assert_eq!(sheet.sheet_no(), 0);
        assert_eq!(sheet.head_row_number(), Some(0));

        // parameter() 等价于 build()（对应 Java：protected ReadSheet parameter()）
        let parameter = ExcelReaderSheetBuilder::new().parameter();
        assert_eq!(parameter, ReadSheet::default_construction());
    }

    #[test]
    fn bound_builder_forwarding_methods_apply_to_built_sheet() -> Result<()> {
        // 对应 Java：ExcelReaderSheetBuilder(ExcelReader).headRowNumber/useScientificFormat
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "alice,30")?;
        let mut reader =
            ExcelReader::new(file.path(), ReadOptions::default(), counting_listener())?;

        let sheet = ExcelReaderSheetBuilder::with_excel_reader(&mut reader)
            .sheet_no(0)
            .head_row_number(0)
            .use_scientific_format(true)
            .build();
        assert_eq!(sheet.sheet_no(), 0);
        assert_eq!(sheet.head_row_number(), Some(0));
        assert_eq!(sheet.use_scientific_format(), Some(true));
        Ok(())
    }

    #[test]
    fn bound_builder_do_read_sync_respects_head_row_number_override() -> Result<()> {
        // 对应 Java：doReadSync() 使用本 sheet 的 headRowNumber 覆盖
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "alice,30")?;
        let mut reader =
            ExcelReader::new(file.path(), ReadOptions::default(), counting_listener())?;

        let rows = ExcelReaderSheetBuilder::with_excel_reader(&mut reader)
            .sheet_name("Sheet1")
            .head_row_number(0)
            .do_read_sync()?;
        // head_row_number=0 时表头行也作为数据读出（对应 Java：headRowNumber=0 语义）
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1].get(0),
            Some(&crate::core::DynamicValue::String("alice".to_owned()))
        );
        Ok(())
    }
}
