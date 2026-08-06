//! 对应 Java：`com.alibaba.excel.util.IoUtils`。
//!
//! 字节复制实现位于 `easyexcel-io`；这里保留 `EasyExcel` 兼容错误类型。

#![allow(dead_code)]

use std::io::{Read, Write};

use crate::core::excel_error::ExcelError;

/// 对应 Java：com.alibaba.excel.util.IoUtils。 复制输入流的全部字节到输出流。
///
/// # Errors
///
/// 输入读取或输出写入失败时返回 I/O 错误。
pub fn copy(reader: &mut dyn Read, writer: &mut dyn Write) -> Result<u64, ExcelError> {
    easyexcel_io::io::io_utils::copy(reader, writer).map_err(ExcelError::from)
}
