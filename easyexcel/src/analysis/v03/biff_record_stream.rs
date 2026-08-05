//! BIFF record 流的 EasyExcel 事件层适配。

use std::path::Path;

use crate::core::{ExcelError, Result};

pub(crate) fn read_workbook_stream(path: &Path) -> Result<Vec<u8>> {
    easyexcel_xls::biff8::record_stream::read_workbook_stream(path).map_err(ExcelError::from)
}

pub(crate) fn walk_biff_records(
    workbook: &[u8],
    mut process: impl FnMut(u16, &[u8]) -> Result<()>,
) -> Result<()> {
    easyexcel_xls::biff8::record_stream::walk_biff_records(workbook, |sid, payload| {
        process(sid, payload).map_err(|error| easyexcel_io::Error::Other(error.to_string()))
    })
    .map_err(ExcelError::from)
}
