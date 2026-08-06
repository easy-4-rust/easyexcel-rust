//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.RkRecordHandler`.
//!
//! Note: Java oddly materialises an *empty* cell for RK records (historical
//! `EasyExcel` behaviour). We mirror that exactly.

use super::super::xls_record_handler::XlsRecordHandler;
use super::blank_record_handler::BlankCell;

/// 对应 Java：`RkRecordHandler`.
#[derive(Debug, Default)]
pub struct RkRecordHandler {
    /// Most recently decoded RK placement using Java's empty-cell quirk.
    pub last_cell: Option<BlankCell>,
}

impl RkRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `RkRecordHandler.processRecord` — always yields an empty cell.
    #[must_use]
    pub fn process_rk(row: u32, column: usize) -> BlankCell {
        BlankCell { row, column }
    }
}

/// BIFF `RK` record sid. (POI `RKRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::RK_SID;

impl XlsRecordHandler for RkRecordHandler {
    /// Java `RkRecordHandler.processRecord` — yields empty cell (`EasyExcel` quirk).
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != RK_SID {
            return;
        }
        if let Some((row, column)) = easyexcel_xls::biff8::event_record::decode_cell_position(data)
        {
            self.last_cell = Some(Self::process_rk(row, column));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_rk_is_empty_cell() {
        assert_eq!(
            RkRecordHandler::process_rk(3, 4),
            BlankCell { row: 3, column: 4 }
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_parses_placement() {
        // 对应 Java：RkRecordHandler.processRecord 产出空单元格（EasyExcel 历史行为）
        let mut handler = RkRecordHandler::new();
        handler.process_record(RK_SID, &[3, 0, 4, 0, 0, 0]);
        assert_eq!(handler.last_cell, Some(BlankCell { row: 3, column: 4 }));
        // 错误 sid / 数据不足时保持原状态
        handler.process_record(0xFFFF, &[3, 0, 4, 0, 0, 0]);
        handler.process_record(RK_SID, &[0, 0]);
        assert_eq!(handler.last_cell, Some(BlankCell { row: 3, column: 4 }));
    }
}
