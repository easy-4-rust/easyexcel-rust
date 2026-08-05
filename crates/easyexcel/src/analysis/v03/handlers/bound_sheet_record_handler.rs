//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.BoundSheetRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// Collected bound-sheet entry (name + BOF position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSheetEntry {
    /// Sheet display name. (Java `BoundSheetRecord.getSheetname`)
    pub name: String,
    /// Absolute BOF file position used for ordering.
    pub bof_position: u32,
}

/// 对应 Java：`BoundSheetRecordHandler`.
#[derive(Debug, Default)]
pub struct BoundSheetRecordHandler {
    /// Accumulated bound-sheet records. (Java workbook holder list)
    pub sheets: Vec<BoundSheetEntry>,
}

impl BoundSheetRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `BoundSheetRecordHandler.processRecord`.
    pub fn process_bound_sheet(&mut self, name: String, bof_position: u32) {
        self.sheets.push(BoundSheetEntry { name, bof_position });
    }

    /// Java `BoundSheetRecord.orderByBofPosition` — sort by BOF offset ascending.
    #[must_use]
    pub fn ordered_sheets(&self) -> Vec<BoundSheetEntry> {
        let mut sheets = self.sheets.clone();
        sheets.sort_by_key(|entry| entry.bof_position);
        sheets
    }
}

/// BIFF `BoundSheet` record sid. (POI `BoundSheetRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::BOUND_SHEET_SID;

impl XlsRecordHandler for BoundSheetRecordHandler {
    /// Java `BoundSheetRecordHandler.processRecord` — reads BOF position and
    /// the BIFF8 short-Unicode sheet name.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != BOUND_SHEET_SID {
            return;
        }
        if let Some(record) = easyexcel_xls::biff8::event_record::decode_bound_sheet_record(data) {
            self.process_bound_sheet(record.name, record.bof_position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_bof_position() {
        let mut handler = BoundSheetRecordHandler::new();
        handler.process_bound_sheet("B".into(), 200);
        handler.process_bound_sheet("A".into(), 100);
        let ordered = handler.ordered_sheets();
        assert_eq!(ordered[0].name, "A");
        assert_eq!(ordered[1].name, "B");
    }

    #[test]
    fn decodes_compressed_biff8_sheet_name() {
        let mut handler = BoundSheetRecordHandler::new();
        let mut payload = vec![0x20, 0, 0, 0, 0, 0, 4, 0];
        payload.extend_from_slice(b"Data");
        handler.process_record(BOUND_SHEET_SID, &payload);
        assert_eq!(handler.sheets[0].name, "Data");
        assert_eq!(handler.sheets[0].bof_position, 32);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn decodes_utf16_sheet_name_and_truncated_bodies() {
        // 对应 Java：BoundSheetRecordHandler 解码 UTF-16 工作表名
        let mut handler = BoundSheetRecordHandler::new();
        let mut payload = vec![0, 0, 0, 0, 0, 0, 2, 1];
        payload.extend_from_slice(&[0x41, 0, 0x42, 0]); // "AB"（UTF-16LE）
        handler.process_record(BOUND_SHEET_SID, &payload);
        assert_eq!(handler.sheets[0].name, "AB");

        // 截断的 UTF-16 与压缩名直接返回
        handler.process_record(BOUND_SHEET_SID, &[0, 0, 0, 0, 0, 0, 4, 1, 0x41, 0]);
        handler.process_record(BOUND_SHEET_SID, &[0, 0, 0, 0, 0, 0, 4, 0, b'A']);
        assert_eq!(handler.sheets.len(), 1);
        // 错误 sid 忽略
        handler.process_record(0xFFFF, &[0; 8]);
        assert_eq!(handler.sheets.len(), 1);
    }
}
