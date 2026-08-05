//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.BoolErrRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// Decoded boolean cell produced by [`BoolErrRecordHandler`].
///
/// Java's handler only materialises the boolean branch via
/// `BoolErrRecord.getBooleanValue()` (error branch is not exposed separately).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolCell {
    /// Zero-based row.
    pub row: u32,
    /// Zero-based column.
    pub column: usize,
    /// Boolean value. (Java `getBooleanValue`)
    pub value: bool,
}

/// 对应 Java：`BoolErrRecordHandler`.
#[derive(Debug, Default)]
pub struct BoolErrRecordHandler {
    /// Most recently decoded boolean cell.
    pub last_cell: Option<BoolCell>,
}

impl BoolErrRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `BoolErrRecordHandler.processRecord`.
    #[must_use]
    pub fn process_bool(row: u32, column: usize, value: bool) -> BoolCell {
        BoolCell { row, column, value }
    }
}

/// BIFF `BoolErr` record sid. (POI `BoolErrRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::BOOL_ERR_SID;

impl XlsRecordHandler for BoolErrRecordHandler {
    /// Java `BoolErrRecordHandler.processRecord` — boolean branch only.
    /// Layout: `row|col|xf|value:u8|isError:u8`.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != BOOL_ERR_SID || data.len() < 8 {
            return;
        }
        if let Some(Some((header, value))) =
            easyexcel_xls::biff8::event_record::decode_bool_err_record(data)
        {
            self.last_cell = Some(Self::process_bool(header.row, header.column, value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_bool_keeps_flag() {
        assert!(BoolErrRecordHandler::process_bool(0, 0, true).value);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_parses_boolean_and_skips_errors() {
        // 对应 Java：BoolErrRecordHandler 仅物化布尔分支，错误分支跳过
        let mut handler = BoolErrRecordHandler::new();
        // row|col|xf|value|isError —— value=1, isError=0
        handler.process_record(BOOL_ERR_SID, &[1, 0, 2, 0, 0, 0, 1, 0]);
        assert_eq!(
            handler.last_cell,
            Some(BoolCell {
                row: 1,
                column: 2,
                value: true
            })
        );
        // isError=1 → 不产出布尔值（保持上一次结果）
        handler.process_record(BOOL_ERR_SID, &[3, 0, 4, 0, 0, 0, 1, 1]);
        assert_eq!(
            handler.last_cell,
            Some(BoolCell {
                row: 1,
                column: 2,
                value: true
            })
        );
        // 数据不足 / 错误 sid 忽略
        handler.process_record(BOOL_ERR_SID, &[0, 0]);
        handler.process_record(0xFFFF, &[1, 0, 2, 0, 0, 0, 0, 0]);
        assert_eq!(
            handler.last_cell,
            Some(BoolCell {
                row: 1,
                column: 2,
                value: true
            })
        );
    }
}
