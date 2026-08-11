//! 可复用的 BIFF8 底层 record、公式与加密原语。

mod cached;
mod biff8_font;
mod biff8_rich_text_cell;
mod continuation_chain;
mod continuation_decoder;
pub mod encode;
mod encrypt;
pub mod event_record;
mod format;
#[cfg(feature = "xls-lazy-sst")]
pub mod lazy_sst;
pub(crate) mod model;
mod numeric;
pub mod ptg;
mod protection;
mod rich_text;
pub mod record_sid;
pub mod record_stream;
pub mod string;
mod style;
mod template;
mod workbook;

pub use continuation_chain::Biff8ContinuationChain;
pub use biff8_font::Biff8Font;
pub use biff8_rich_text_cell::Biff8RichTextCell;
pub use continuation_decoder::{
    Biff8ContinuableRecordDecoder, Biff8ContinuableRecordKind, Biff8ContinuationStatus,
    Biff8DecodedContinuableRecord,
};
pub use encrypt::{
    Biff8CryptoApiEncryption, decrypt_crypto_api_workbook_stream,
    encrypt_crypto_api_workbook_stream, prepare_crypto_api_encryption,
};
pub(crate) use format::{builtin_format_code, builtin_format_id};
pub use numeric::{
    Biff8NumericCell, Biff8NumericSheets, Biff8SheetDisplays, decode_rk, format_numeric_displays,
    load_numeric_displays, load_numeric_displays_with_password, parse_format_record,
    scan_numeric_cells,
};
pub use protection::legacy_password_hash;
pub use rich_text::{load_rich_text_cells, load_rich_text_cells_with_password};
pub use style::{
    Biff8BorderStyle, Biff8Color, Biff8FillPattern, Biff8HorizontalAlignment, Biff8NumberFormat,
    Biff8StyleRequest, Biff8StyleTable, Biff8Underline, Biff8VerticalAlignment,
};
pub use template::{Biff8MacroPolicy, Biff8TemplatePackage, looks_like_xls};
pub use workbook::{
    Biff8Book, Biff8Cell, Biff8Chart, Biff8ChartKind, Biff8ChartRange, Biff8ChartSeries,
    Biff8Comment, Biff8Hyperlink, Biff8HyperlinkKind, Biff8Merge, Biff8RichText, Biff8Sheet,
    Biff8Value, GeneratedBiff8CellValue,
    date_to_excel_serial, date_to_excel_serial_with_windowing, datetime_to_excel_serial,
    datetime_to_excel_serial_with_windowing,
};
