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
    Cell, CellAddress, CellError, CellRange, CellValue, ChartMutation, ChartRange, ChartSeries,
    ChartType, ColInfo, DATE_FORMAT_10, DATE_FORMAT_14, DATE_FORMAT_16,
    DATE_FORMAT_16_FORWARD_SLASH, DATE_FORMAT_17, DATE_FORMAT_19, DATE_FORMAT_19_FORWARD_SLASH,
    DAY_MILLISECONDS, DEFAULT_DATE_FORMAT, DEFAULT_LOCAL_DATE_FORMAT, DataFormatData, DateSystem,
    DefinedName, Error, ExcelDataFormat, FrozenPanes, HOURS_PER_DAY, MINUTES_PER_HOUR, MergeRange,
    Metadata, OpaquePart, Result, RichTextSegment, RowInfo, SECONDS_PER_DAY, SECONDS_PER_MINUTE,
    Sheet, Spill, StoredRow, Table, TabularCell, TabularDocument, TabularTable, Visibility,
    Workbook, chrono_date_format, date_to_excel_serial, datetime_to_excel_serial,
    excel_parts_to_datetime, infer_java_date_pattern, segment_utf16_text,
};
pub use model::{
    addr, chart_mutation, chart_range, chart_series, chart_type, data_format_data, dates, error,
    excel_data_format, merge_range, numfmt, rich_text_segment, styles, tabular, value,
};
