//! `rust_xlsxwriter` 生成后端与加密落盘边界。
//!
//! `EasyExcel` 门面通过本模块获得工作簿句柄；底层依赖、序列化、文件创建和
//! MS-OFFCRYPTO 包装由 `easyexcel-xlsx` 统一拥有。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{Error, Result};

pub use rust_xlsxwriter::{
    Chart, ChartType, Color, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript,
    FormatUnderline, Image, Note, ObjectMovement, Workbook, Worksheet,
};

use super::encrypt::{ReadWriteSeek, encrypt_package_to};

include!("generation/xlsx_max_rows_to_build_format.rs");
include!("generation/apply_format_spec_to_tests.rs");
