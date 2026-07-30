//! XLSX 写入配置类型。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.WriteBasicParameter`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/WriteBasicParameter.java

/// XLSX write configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct WriteOptions {
    /// Explicit output type overriding the file extension.
    /// (Java `WriteWorkbook.excelType`)
    pub excel_type: Option<easyexcel_core::support::ExcelTypeEnum>,
    /// Worksheet name.
    pub sheet_name: String,
    /// Optional logical worksheet number, starting from zero.
    pub sheet_index: Option<usize>,
    /// Automatic trim for sheet names and string cells. (Java `autoTrim`)
    pub auto_trim: bool,
    /// Whether Excel 1904 date windowing is enabled. (Java `use1904windowing`)
    pub use_1904_windowing: bool,
    /// Locale name used for formatted output. (Java `locale`)
    pub locale: String,
    /// Whether scientific notation is used for extreme General-format numbers.
    /// (Java `useScientificFormat`)
    pub use_scientific_format: bool,
    /// Field-cache location for reflection metadata. (Java `filedCacheLocation`)
    pub filed_cache_location: CacheLocation,
    /// Whether to use a one-row constant-memory worksheet.
    pub constant_memory: bool,
    /// Whether streaming spill files use gzip (SXSSF `setCompressTempFiles`).
    ///
    /// Java mapping: `SXSSFWorkbook.setCompressTempFiles(true)` (often set in
    /// `WorkbookWriteHandler.afterWorkbookCreate`). When enabled:
    /// 1. Forces [`Self::constant_memory`] so `rust_xlsxwriter` keeps peak RAM
    ///    bounded (row window flush; avoids OOM on large batches).
    /// 2. Mirrors each data row into [`gzip_spill::GzipSheetDataWriter`] — a
    ///    true gzip tempfile (magic `1f 8b`), observable via
    ///    [`ExcelWriter::last_gzip_spill_snapshot`].
    ///
    /// **Remaining difference from POI:** POI replaces the sheet-XML spill with
    /// `GZIPSheetDataWriter` only. Here gzip is an explicit SXSSF-equivalent
    /// spill alongside the engine's constant-memory tempfile (engine tempfile
    /// stays uncompressed; final `.xlsx` is still ZIP Deflate).
    pub compress_temp_files: bool,
    /// Whether column headers are written.
    pub need_head: bool,
    /// Whether Java's built-in default style handler is enabled.
    ///
    /// This is distinct from [`Self::head_style`]: Java passes the flag to
    /// `DefaultWriteHandlerLoader`, which decides whether `DefaultStyle`
    /// participates in the actual handler chain.
    pub use_default_style: bool,
    /// Whether header rows are frozen.
    pub freeze_head: bool,
    /// Explicit freeze pane position as `(row, column)`.
    pub freeze_panes: Option<(u32, u16)>,
    /// Physical column indexes to include.
    pub include_column_indexes: Option<Vec<usize>>,
    /// Rust field names to include.
    pub include_column_field_names: Option<Vec<String>>,
    /// Physical column indexes to exclude.
    pub exclude_column_indexes: Vec<usize>,
    /// Rust field names to exclude.
    pub exclude_column_field_names: Vec<String>,
    /// Whether included columns follow the order of the include list.
    pub order_by_include_column: bool,
    /// Relative head row index. (Java `WriteBasicParameter.relativeHeadRowIndex`)
    pub relative_head_row_index: i32,
    /// Whether headers are auto-merged. (Java `WriteBasicParameter.automaticMergeHead`)
    pub automatic_merge_head: bool,
    /// Absolute ranges merged before row data is written.
    pub merge_ranges: Vec<MergeRange>,
    /// Whether used columns are auto-fitted after writing.
    pub auto_width: bool,
    /// Explicit column widths in Excel character units.
    pub column_widths: Vec<(u16, u16)>,
    /// Style applied to header cells.
    pub head_style: CellStyle,
    /// Content styles cycled by relative data-row index.
    pub content_styles: Vec<CellStyle>,
    /// Repeating merge strategies applied to data rows.
    pub loop_merges: Vec<MirroredLoopMergeStrategy>,
    /// Optional dynamic multi-level head paths, one path per selected column.
    pub dynamic_head: Option<Vec<Vec<String>>>,
    /// Password used for ECMA-376 Agile Encryption of XLSX output.
    pub password: Option<String>,
    /// Character encoding used for CSV output.
    pub charset: CsvCharset,
    /// Whether CSV output starts with the encoding's byte-order mark.
    pub with_bom: bool,
    /// Whether a stateful [`ExcelOutputStream`] is closed by `finish`.
    pub auto_close_stream: bool,
    /// Whether `finish_on_exception` emits rows accumulated before an error.
    pub write_excel_on_exception: bool,
    /// Java-style globally registered converters.
    pub converters: ConverterRegistry,
    /// Template file path. (Java `WriteWorkbook.templateFile`)
    ///
    /// When set, XLSX writes open this workbook as the write base and append
    /// typed rows after existing template content — matching Java
    /// `ExcelWriterBuilder.withTemplate(File)`. Default path preserves
    /// `styles.xml` / `mergeCells` via ZIP/OOXML; see
    /// [`Self::use_legacy_template_seed`] for the explicit value-only fallback.
    pub template_file: Option<PathBuf>,
    /// In-memory template bytes. (Java `WriteWorkbook.templateInputStream`)
    ///
    /// Builder helpers clear the other source so only one is active.
    pub template_bytes: Option<Vec<u8>>,
    /// When `true`, `with_template` uses the legacy calamine → `rust_xlsxwriter`
    /// value-replay path (styles/merges **not** preserved).
    ///
    /// Default is `false`: ZIP/OOXML preserve (`styles.xml` + `mergeCells` kept;
    /// new sheets are added as empty worksheet parts without rewriting existing
    /// sheets). Prefer leaving this off unless you explicitly need the legacy seed.
    pub use_legacy_template_seed: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            excel_type: None,
            sheet_name: "Sheet1".to_owned(),
            sheet_index: None,
            auto_trim: true,
            use_1904_windowing: false,
            locale: "default".to_owned(),
            use_scientific_format: false,
            filed_cache_location: CacheLocation::ThreadLocal,
            constant_memory: false,
            compress_temp_files: false,
            need_head: true,
            use_default_style: true,
            freeze_head: false,
            freeze_panes: None,
            include_column_indexes: None,
            include_column_field_names: None,
            exclude_column_indexes: Vec::new(),
            exclude_column_field_names: Vec::new(),
            order_by_include_column: false,
            merge_ranges: Vec::new(),
            auto_width: false,
            column_widths: Vec::new(),
            head_style: CellStyle::new().bold(true),
            content_styles: Vec::new(),
            loop_merges: Vec::new(),
            dynamic_head: None,
            password: None,
            charset: CsvCharset::default(),
            with_bom: true,
            auto_close_stream: true,
            write_excel_on_exception: false,
            converters: ConverterRegistry::default(),
            relative_head_row_index: 0,
            automatic_merge_head: true,
            template_file: None,
            template_bytes: None,
            use_legacy_template_seed: false,
        }
    }
}

/// Global write flags copied from [`WriteOptions`] for cell emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct WriteGlobalFlags {
    /// Automatic trim for sheet names and string cells.
    auto_trim: bool,
    /// Whether Excel 1904 date windowing is enabled.
    use_1904_windowing: bool,
    /// Whether scientific notation is used for extreme General-format numbers.
    use_scientific_format: bool,
}

impl From<&WriteOptions> for WriteGlobalFlags {
    fn from(options: &WriteOptions) -> Self {
        Self {
            auto_trim: options.auto_trim,
            use_1904_windowing: options.use_1904_windowing,
            use_scientific_format: options.use_scientific_format,
        }
    }
}

/// Returns the worksheet name after applying [`WriteOptions::auto_trim`].
fn effective_sheet_name(options: &WriteOptions) -> String {
    if options.auto_trim {
        options.sheet_name.trim().to_owned()
    } else {
        options.sheet_name.clone()
    }
}

/// Trims string cell text when auto-trim is enabled.
fn maybe_trim_cell_string(value: &str, auto_trim: bool) -> String {
    if auto_trim {
        value.trim().to_owned()
    } else {
        value.to_owned()
    }
}

