//! BIFF8 字符串解码的 EasyExcel 事件层适配。

use crate::core::{ExcelError, Result};

pub(crate) fn decode_sst_segments(segments: &[Vec<u8>]) -> Result<Vec<String>> {
    easyexcel_xls::biff8::string::decode_sst_segments(segments).map_err(ExcelError::from)
}

pub(crate) fn decode_unicode_string_segments(segments: &[Vec<u8>]) -> Result<String> {
    easyexcel_xls::biff8::string::decode_unicode_string_segments(segments).map_err(ExcelError::from)
}
