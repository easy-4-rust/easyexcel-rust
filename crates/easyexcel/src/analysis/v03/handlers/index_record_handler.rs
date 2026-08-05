//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.IndexRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`IndexRecordHandler`.
#[derive(Debug, Default)]
pub struct IndexRecordHandler {
    /// Approximate total rows from `IndexRecord.getLastRowAdd1`.
    pub approximate_total_row_number: Option<u32>,
}

impl IndexRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `IndexRecordHandler.processRecord`.
    pub fn process_index(&mut self, last_row_add_1: u32) {
        self.approximate_total_row_number = Some(last_row_add_1);
    }
}

/// BIFF `Index` record sid. (POI `IndexRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::INDEX_SID;

impl XlsRecordHandler for IndexRecordHandler {
    /// Java `IndexRecordHandler.processRecord` — reads `lastRowAdd1` when present.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != INDEX_SID {
            return;
        }
        if let Some(last_row_add_1) =
            easyexcel_xls::biff8::event_record::decode_index_last_row(data)
        {
            self.process_index(last_row_add_1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_index_stores_total() {
        let mut handler = IndexRecordHandler::new();
        handler.process_index(42);
        assert_eq!(handler.approximate_total_row_number, Some(42));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_reads_last_row_add_1() {
        // 对应 Java：IndexRecordHandler.processRecord 读取 lastRowAdd1
        let mut handler = IndexRecordHandler::new();
        let mut data = vec![0u8; 16];
        data[8..12].copy_from_slice(&9u32.to_le_bytes());
        handler.process_record(INDEX_SID, &data);
        assert_eq!(handler.approximate_total_row_number, Some(9));
        // 数据不足 / 错误 sid 忽略
        handler.process_record(INDEX_SID, &[0; 15]);
        handler.process_record(0xFFFF, &data);
        assert_eq!(handler.approximate_total_row_number, Some(9));
    }
}
