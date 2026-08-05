//! 统一工作簿模型及其地址、日期、样式和值对象。

pub mod addr;
pub mod dates;
pub mod error;
pub mod numfmt;
pub mod styles;
pub mod value;
mod workbook;

pub use addr::{CellAddress, CellRange};
pub use dates::{
    DateSystem, chrono_date_format, date_to_excel_serial, datetime_to_excel_serial,
};
pub use error::{CellError, Error, Result};
pub use value::CellValue;
pub use workbook::{
    Cell, ColInfo, DefinedName, FrozenPanes, Metadata, OpaquePart, RowInfo, Sheet, Spill, Table,
    Visibility, Workbook,
};
