//! 可复用的 BIFF8 底层 record、公式与加密原语。

mod cached;
mod continuation_chain;
mod continuation_decoder;
pub mod encode;
pub mod encrypt;
pub mod event_record;
mod format;
mod numeric;
pub mod ptg;
pub mod record_sid;
pub mod record_stream;
pub mod string;
mod style;
mod template;
mod workbook;

pub use continuation_chain::Biff8ContinuationChain;
pub use continuation_decoder::{
    Biff8ContinuableRecordDecoder, Biff8ContinuableRecordKind, Biff8ContinuationStatus,
    Biff8DecodedContinuableRecord,
};
pub(crate) use format::{builtin_format_code, builtin_format_id};
pub use numeric::{
    Biff8NumericCell, Biff8NumericSheets, Biff8SheetDisplays, decode_rk, format_numeric_displays,
    load_numeric_displays, parse_format_record, scan_numeric_cells,
};
pub use style::{
    Biff8Color, Biff8FillPattern, Biff8HorizontalAlignment, Biff8NumberFormat, Biff8StyleRequest,
    Biff8StyleTable, Biff8VerticalAlignment,
};
pub use template::{Biff8TemplatePackage, looks_like_xls};
pub use workbook::{
    Biff8Book, Biff8Cell, Biff8Merge, Biff8Sheet, Biff8Value, date_to_excel_serial,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial,
    datetime_to_excel_serial_with_windowing,
};
