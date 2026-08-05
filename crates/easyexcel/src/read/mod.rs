//! Streaming OOXML XLSX, Calamine-backed XLS, and Rust CSV readers.

mod cell_conversion;
mod locale;
pub mod read_cache;
mod read_csv;
mod read_helpers;
mod read_options;
mod read_xls;
mod read_xlsx;
mod row_consumer;
mod row_processing;
mod scientific_format_mode;
mod sheet_selector;
pub mod stored_read_cache_selector;
mod xls_display;
mod xlsx_rows;
mod xlsx_source;

pub mod metadata;

/// Java `com.alibaba.excel.read` 包路径镜像（含 `ReadBasicParameter`）。
pub mod builder;
pub mod listener;
pub mod processor;

/// Holder 模块镜像 — 指向 `read/metadata/holder`。
pub use crate::read::metadata::holder;

#[path = "../excel_reader.rs"]
mod excel_reader;

mod global_configuration;
pub use crate::analysis::v03::XlsSaxAnalyser;
pub use crate::analysis::v07::XlsxSaxAnalyser;
pub use crate::cache::{
    Ehcache, EternalReadCacheSelector, MapCache, ReadCache, ReadCacheSelector,
    SimpleReadCacheSelector, XlsCache,
};
pub use builder::excel_reader_builder::ExcelReaderBuilder as CompatibleExcelReaderBuilder;
pub use builder::excel_reader_sheet_builder::ExcelReaderSheetBuilder as CompatibleExcelReaderSheetBuilder;
pub use excel_reader::ExcelReader;
pub use global_configuration::{
    apply_global_configuration_to_read_options, global_configuration_from_read_options,
};
pub use locale::ExcelLocale;
pub use read_cache::ReadCacheMode;
pub use read_csv::read_csv;
pub use read_options::ReadOptions;
pub use read_xls::{list_xls_sheets, read_xls};
pub use read_xlsx::{list_xlsx_sheets, read_xlsx};
pub use scientific_format_mode::ScientificFormatMode;
pub use sheet_selector::SheetSelector;
pub use stored_read_cache_selector::StoredReadCacheSelector;

// Internal helpers re-exported so test modules using `use super::*;` resolve.
#[cfg(test)]
pub(crate) use cell_conversion::{from_calamine, from_data};
#[cfg(test)]
pub(crate) use read_csv::{csv_row_index, csv_sheet_name, read_csv_records};
pub(crate) use read_helpers::sheet_name_matches;
#[cfg(test)]
pub(crate) use read_helpers::{format_error, java_trim, to_column_index, validate_read_options};
#[cfg(test)]
pub(crate) use read_xlsx::read_xlsx_source;
#[cfg(test)]
pub(crate) use row_consumer::{ReadFlow, TypedRowConsumer};
#[cfg(test)]
pub(crate) use row_processing::{
    process_row, read_range, select_sheet_names, select_xls_sheets, selected_sheet_names,
};
#[cfg(test)]
pub(crate) use xlsx_rows::XlsxRowMetadata;
#[cfg(test)]
mod missing_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn stored_selector_select_mode_dispatches_to_inner_selectors() {
        // 对应 Java：ReadCacheSelector.selectMode 委托给具体实现
        let simple = StoredReadCacheSelector::Simple(SimpleReadCacheSelector::new());
        assert_eq!(simple.select_mode(100), ReadCacheMode::Memory);
        assert_eq!(simple.select_mode(10_000_000), ReadCacheMode::Disk);

        let eternal_map = StoredReadCacheSelector::Eternal(EternalReadCacheSelector::map_cache());
        assert_eq!(eternal_map.select_mode(10_000_000), ReadCacheMode::Memory);

        let eternal_disk = StoredReadCacheSelector::Eternal(EternalReadCacheSelector::ehcache());
        assert_eq!(eternal_disk.select_mode(100), ReadCacheMode::Disk);
    }
}
