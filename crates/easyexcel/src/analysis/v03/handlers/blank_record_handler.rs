//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.BlankRecordHandler`.
//!
//! XLS BIFF decoding is owned by `calamine::Xls` today; these helpers encode
//! the Java `processRecord` semantics so a future `XlsSaxAnalyser` can call
//! them without re-deriving the rules.

use super::super::xls_record_handler::XlsRecordHandler;

/// Decoded blank-cell placement produced by [`BlankRecordHandler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlankCell {
    /// Zero-based row. (Java `BlankRecord.getRow`)
    pub row: u32,
    /// Zero-based column. (Java `BlankRecord.getColumn`)
    pub column: usize,
}

/// 对应 Java：`BlankRecordHandler`.
#[derive(Debug, Default)]
pub struct BlankRecordHandler {
    /// Most recently decoded blank cell.
    pub last_cell: Option<BlankCell>,
}

impl BlankRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `BlankRecordHandler.processRecord` — emit an empty cell at `(row, column)`.
    #[must_use]
    pub fn process_blank(row: u32, column: usize) -> BlankCell {
        BlankCell { row, column }
    }
}

/// BIFF `Blank` record sid. (POI `BlankRecord.sid`)
pub const BLANK_SID: u16 = 0x0201;

impl XlsRecordHandler for BlankRecordHandler {
    /// Java `BlankRecordHandler.processRecord` — parses `row|col|xf`.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != BLANK_SID || data.len() < 6 {
            return;
        }
        if let Some(header) = easyexcel_xls::biff8::event_record::decode_cell_header(data) {
            self.last_cell = Some(Self::process_blank(header.row, header.column));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_blank_keeps_coordinates() {
        assert_eq!(
            BlankRecordHandler::process_blank(2, 5),
            BlankCell { row: 2, column: 5 }
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_parses_coordinates() {
        // 对应 Java：BlankRecordHandler.processRecord 解析 row|col
        let mut handler = BlankRecordHandler::new();
        handler.process_record(BLANK_SID, &[2, 0, 5, 0, 9, 0]);
        assert_eq!(handler.last_cell, Some(BlankCell { row: 2, column: 5 }));
        // 错误 sid 或数据不足时保持原状态
        handler.process_record(0xFFFF, &[2, 0, 5, 0, 9, 0]);
        assert_eq!(handler.last_cell, Some(BlankCell { row: 2, column: 5 }));
        handler.process_record(BLANK_SID, &[0, 0]);
        assert_eq!(handler.last_cell, Some(BlankCell { row: 2, column: 5 }));
    }
}
