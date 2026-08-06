//! 与具体文件格式无关的电子表格工作簿模型。
//!
//! 模型源自 `easy-4-rust/xls` fork 的 core，并在 EasyExcel-Rust 中作为
//! XLS、XLSX、CSV、公式和命令应用层共同依赖的稳定基础层维护。

#![allow(
    missing_docs,
    reason = "迁入的 xls 公共模型仍保留上游语义注释；中文 API 文档按来源矩阵持续补齐"
)]

pub mod model;

pub use model::{
    Cell, CellAddress, CellError, CellRange, CellValue, ColInfo, DataFormatData, DateSystem,
    DefinedName, Error, ExcelDataFormat, FrozenPanes, Metadata, OpaquePart, Result, RowInfo, Sheet,
    Spill, StoredRow, Table, Visibility, Workbook, chrono_date_format, date_to_excel_serial,
    datetime_to_excel_serial,
};
pub use model::{addr, data_format_data, dates, error, excel_data_format, numfmt, styles, value};
