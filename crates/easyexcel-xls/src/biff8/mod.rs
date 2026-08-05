//! 可复用的 BIFF8 底层 record、公式与加密原语。

mod cached;
pub mod encode;
pub mod encrypt;
mod format;
mod numeric;
pub mod ptg;
pub mod record_stream;
pub mod string;
mod style;
mod template;
mod workbook;

pub use numeric::{
    Biff8NumericCell, Biff8NumericSheets, decode_rk, parse_format_record, scan_numeric_cells,
};
pub use style::{Biff8NumberFormat, Biff8StyleRequest, Biff8StyleTable};
pub use template::{Biff8TemplatePackage, looks_like_xls};
pub use workbook::{
    Biff8Book, Biff8Cell, Biff8Merge, Biff8Sheet, Biff8Value, date_to_excel_serial,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial,
    datetime_to_excel_serial_with_windowing,
};
