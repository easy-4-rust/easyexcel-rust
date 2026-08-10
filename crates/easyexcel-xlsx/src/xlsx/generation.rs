//! `rust_xlsxwriter` 生成后端与加密落盘边界。
//!
//! `EasyExcel` 门面通过本模块获得工作簿句柄；底层依赖、序列化、文件创建和
//! MS-OFFCRYPTO 包装由 `easyexcel-xlsx` 统一拥有。

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{Error, Result};

use rust_xlsxwriter::{Chart, ChartType, Image, Note};
pub use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript, FormatUnderline,
    ObjectMovement, Workbook, Worksheet,
};

use super::encrypt::{ReadWriteSeek, encrypt_package_to};

mod generated_cell_value;
mod generated_chart;

pub use generated_chart::add_chart;
pub use generated_cell_value::GeneratedCellValue;

/// Worksheet XML/ZIP 输出聚合缓冲区。128 KiB 位于发布计划要求的
/// 64–256 KiB 区间，可显著减少大表写入的小块系统调用。
const XLSX_OUTPUT_BUFFER_CAPACITY: usize = 128 * 1024;

include!("generation/xlsx_max_rows_to_build_format.rs");
include!("generation/apply_format_spec_to_tests.rs");
