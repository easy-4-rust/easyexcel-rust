//! 对应 Java：`com.alibaba.excel.util.IoUtils`。
//!
//! 字节复制实现位于 `easyexcel-io`；这里保留 EasyExcel 兼容错误类型。

#![allow(dead_code)]

use std::io::{Read, Write};

use crate::core::excel_error::ExcelError;

/// 复制输入流的全部字节到输出流。
pub fn copy(reader: &mut dyn Read, writer: &mut dyn Write) -> Result<u64, ExcelError> {
    easyexcel_io::io::io_utils::copy(reader, writer).map_err(ExcelError::from)
}
