//! Workbook read configuration shared by XLSX, XLS, and CSV engines.

use crate::core::converter::default_converter_loader::load_default_read_converter;
use crate::core::{
    ConverterRegistry, CsvCharset, CustomReadObject, ReadDefaultReturn,
};
use crate::reader::locale::ExcelLocale;
use crate::reader::read_cache::ReadCacheMode;
use crate::reader::scientific_format_mode::ScientificFormatMode;
use crate::reader::sheet_selector::SheetSelector;
use crate::reader::stored_read_cache_selector::StoredReadCacheSelector;
use std::collections::{HashMap, HashSet};

/// Workbook read configuration shared by XLSX, XLS, and CSV engines.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    /// Sheet selection.
    pub sheet: SheetSelector,
    /// Number of header rows. The final header row is used for name mapping.
    pub head_row_number: u32,
    /// Whether rows containing only empty cells are ignored.
    pub ignore_empty_row: bool,
    /// Whether leading and trailing whitespace is removed from string cells.
    pub auto_trim: bool,
    /// Whether numeric dates use Excel's 1904 windowing system.
    pub use_1904_windowing: bool,
    /// General-format rendering mode for extreme numbers.
    pub scientific_format: ScientificFormatMode,
    /// Locale used for formatted numeric and date display values.
    pub locale: ExcelLocale,
    /// Physical first row dispatched as data, zero-based and inclusive.
    ///
    /// Header rows are still analysed so name-based mapping remains available.
    pub start_row: Option<u32>,
    /// Physical last row dispatched as data, zero-based and inclusive.
    ///
    /// Header rows are still analysed so name-based mapping remains available.
    pub end_row: Option<u32>,
    /// Header aliases applied after optional Java-compatible trimming.
    ///
    /// Keys are workbook header names and values are names exposed to row mapping
    /// and `ReadListener::invoke_head`.
    pub header_aliases: HashMap<String, String>,
    /// User value exposed through every [`AnalysisContext`].
    pub custom_object: Option<CustomReadObject>,
    /// Value mode used by Java-compatible no-model [`crate::core::DynamicRow`] reads.
    pub read_default_return: ReadDefaultReturn,
    /// Extra worksheet metadata dispatched to `ReadListener::extra`.
    pub extra_read: HashSet<crate::core::CellExtraType>,
    /// Password used to decrypt an encrypted OOXML workbook.
    pub password: Option<String>,
    /// Character encoding used when reading CSV input.
    pub charset: CsvCharset,
    /// Java-style globally registered converters.
    pub converters: ConverterRegistry,
    /// Shared-string cache strategy used by the XLSX SAX reader.
    pub read_cache: ReadCacheMode,
    /// Optional Java-style cache selector overriding [`Self::read_cache`].
    pub read_cache_selector: Option<StoredReadCacheSelector>,
}

impl PartialEq for ReadOptions {
    fn eq(&self, other: &Self) -> bool {
        self.sheet == other.sheet
            && self.head_row_number == other.head_row_number
            && self.ignore_empty_row == other.ignore_empty_row
            && self.auto_trim == other.auto_trim
            && self.use_1904_windowing == other.use_1904_windowing
            && self.scientific_format == other.scientific_format
            && self.locale == other.locale
            && self.start_row == other.start_row
            && self.end_row == other.end_row
            && self.header_aliases == other.header_aliases
            && self.read_default_return == other.read_default_return
            && self.extra_read == other.extra_read
            && self.password == other.password
            && self.charset == other.charset
            && self.read_cache == other.read_cache
            && self.read_cache_selector == other.read_cache_selector
    }
}

impl Eq for ReadOptions {}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            sheet: SheetSelector::First,
            head_row_number: 1,
            ignore_empty_row: true,
            auto_trim: true,
            use_1904_windowing: false,
            scientific_format: ScientificFormatMode::Plain,
            locale: ExcelLocale::default(),
            start_row: None,
            end_row: None,
            header_aliases: HashMap::new(),
            custom_object: None,
            read_default_return: ReadDefaultReturn::default(),
            extra_read: HashSet::new(),
            password: None,
            charset: CsvCharset::default(),
            converters: load_default_read_converter(),
            read_cache: ReadCacheMode::default(),
            read_cache_selector: None,
        }
    }
}
