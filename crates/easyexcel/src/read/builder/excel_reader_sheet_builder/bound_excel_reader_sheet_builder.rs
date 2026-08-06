/// 对应 Java：无直接对应对象；Rust 架构扩展。 A sheet builder borrowing the reader that will execute it.
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
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets the zero-based sheet index.
    #[must_use]
    pub fn sheet_no(mut self, sheet_no: i32) -> Self {
        self.sheet_builder = self.sheet_builder.sheet_no(sheet_no);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets the sheet name.
    #[must_use]
    pub fn sheet_name(mut self, sheet_name: impl Into<String>) -> Self {
        self.sheet_builder = self.sheet_builder.sheet_name(sheet_name);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets the number of header rows for this sheet.
    #[must_use]
    pub fn head_row_number(mut self, head_row_number: i32) -> Self {
        self.sheet_builder = self.sheet_builder.head_row_number(head_row_number);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Controls scientific formatting for this sheet.
    #[must_use]
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.sheet_builder = self.sheet_builder.use_scientific_format(enabled);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds the sheet metadata without executing it.
    #[must_use]
    pub fn build(&self) -> ReadSheet {
        self.sheet_builder.build()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Reads the configured sheet, then finishes the bound reader.
    ///
    /// This is the Rust equivalent of Java
    /// `ExcelReaderSheetBuilder.doRead()`.
    ///
    /// # Errors
    ///
    /// 当工作表解析失败时返回 `ExcelError`。
    pub fn do_read(self) -> Result<()> {
        let sheet = self.sheet_builder.build();
        self.excel_reader.read(&[sheet])?;
        self.excel_reader.finish();
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Reads the configured sheet and returns all converted rows.
    ///
    /// The bound reader's existing listener runs first, followed by the
    /// synchronous collecting listener, matching Java listener registration
    /// order.
    ///
    /// # Errors
    ///
    /// 当工作表解析失败时返回 `ExcelError`。
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

