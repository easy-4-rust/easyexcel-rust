//! BIFF8 工作簿引擎兼容重导出。
//!
//! 实现已迁移至 `easyexcel-xls`，保留本模块路径以兼容 EasyExcel 内部调用。

pub use easyexcel_xls::biff8::{
    Biff8Book, Biff8Cell, Biff8Merge, Biff8Sheet, Biff8Value, date_to_excel_serial,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial,
    datetime_to_excel_serial_with_windowing,
};
