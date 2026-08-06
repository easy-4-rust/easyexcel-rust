/// 对应 Java：无直接对应对象；Rust 架构扩展。 Java-compatible writer builder whose destination is an output stream.
pub struct ExcelWriterOutputStreamBuilder<W> {
    builder: ExcelWriterBuilder,
    output: ExcelOutputStream<W>,
}

impl<W> ExcelWriterOutputStreamBuilder<W>
where
    W: std::io::Write + Send + 'static,
{
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds a stateful writer backed by the configured stream.
    #[must_use]
    pub fn build(self) -> ExcelWriter {
        let logical_path = self
            .builder
            .write_workbook
            .output_file
            .unwrap_or_else(|| PathBuf::from("easyexcel.xlsx"));
        ExcelWriter::with_output_stream(
            logical_path,
            self.output,
            self.builder.handlers,
            self.builder.write_workbook.options,
        )
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds a writer-bound default sheet.
    #[must_use]
    pub fn sheet(self) -> ExcelWriterSheetBuilder {
        let inherited_options = self.builder.write_workbook.options.clone();
        ExcelWriterSheetBuilder::with_excel_writer_and_options(self.build(), inherited_options)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds a writer-bound sheet selected by number.
    #[must_use]
    pub fn sheet_no(self, sheet_no: i32) -> ExcelWriterSheetBuilder {
        self.sheet().sheet_no(sheet_no)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds a writer-bound sheet selected by name.
    #[must_use]
    pub fn sheet_name(self, sheet_name: impl Into<String>) -> ExcelWriterSheetBuilder {
        self.sheet().sheet_name(sheet_name)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds a writer-bound sheet selected by number and name.
    #[must_use]
    pub fn sheet_with(
        self,
        sheet_no: i32,
        sheet_name: impl Into<String>,
    ) -> ExcelWriterSheetBuilder {
        self.sheet().sheet_no(sheet_no).sheet_name(sheet_name)
    }
}

