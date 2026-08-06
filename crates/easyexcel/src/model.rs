//! 与文件格式无关的工作簿模型门面。
//!
//! 这里重导出 [`easyexcel_model`] 的核心类型；这些名称与基础 crate 中的类型
//! 完全相同，不引入包装层或额外转换成本。

pub use easyexcel_model::{
    Cell, CellAddress, CellError, CellRange, CellValue, ColInfo, DataFormatData, DateSystem,
    DefinedName, Error, ExcelDataFormat, FrozenPanes, Metadata, OpaquePart, Result, RowInfo, Sheet,
    Spill, StoredRow, Table, Visibility, Workbook, addr, chrono_date_format, data_format_data,
    date_to_excel_serial, dates, datetime_to_excel_serial, error, excel_data_format, numfmt,
    styles, value,
};
