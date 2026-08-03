//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.NoteRecordHandler`.

use easyexcel_core::{CellExtra, CellExtraType};

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`NoteRecordHandler` (comment / note).
#[derive(Debug, Default)]
pub struct NoteRecordHandler {
    /// Whether comment extras are enabled. (Java `support`)
    pub enabled: bool,
    /// Last parsed comment extra.
    pub last_extra: Option<CellExtra>,
}

impl NoteRecordHandler {
    /// Creates a handler; `enabled` mirrors Java `support(XlsReadContext)`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_extra: None,
        }
    }

    /// Java `NoteRecordHandler.processRecord`.
    ///
    /// `text` comes from `objectCacheMap.get(shapeId)` in Java.
    pub fn process_note(&mut self, text: Option<String>, row: u32, column: usize) {
        if !self.enabled {
            return;
        }
        self.last_extra = Some(CellExtra::new(
            CellExtraType::Comment,
            text,
            row,
            row,
            column,
            column,
        ));
    }
}

impl XlsRecordHandler for NoteRecordHandler {
    fn support(&self) -> bool {
        self.enabled
    }

    /// Java `NoteRecordHandler.processRecord` — parses row/col; text via cache.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        /// BIFF `Note` sid (POI `NoteRecord.sid`)
        const NOTE_SID: u16 = 0x001C;
        if !self.enabled || record_sid != NOTE_SID || data.len() < 6 {
            return;
        }
        let row = u16::from_le_bytes([data[0], data[1]]) as u32;
        let column = u16::from_le_bytes([data[2], data[3]]) as usize;
        self.process_note(None, row, column);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_note_requires_enabled() {
        // 对应 Java：NoteRecordHandler.support() 控制注释是否物化
        let mut disabled = NoteRecordHandler::new(false);
        assert!(!disabled.support());
        disabled.process_note(Some("note".to_owned()), 1, 2);
        assert!(disabled.last_extra.is_none());

        let mut handler = NoteRecordHandler::new(true);
        handler.process_note(Some("note text".to_owned()), 1, 2);
        let extra = handler.last_extra.as_ref().expect("comment extra");
        assert_eq!(extra.extra_type(), CellExtraType::Comment);
        assert_eq!(extra.text(), Some("note text"));
        assert_eq!((extra.first_row_index(), extra.last_row_index()), (1, 1));
        assert_eq!(
            (extra.first_column_index(), extra.last_column_index()),
            (2, 2)
        );
    }

    #[test]
    fn process_record_parses_note_coordinates() {
        // 对应 Java：NoteRecordHandler.processRecord 解析 row|col
        let mut handler = NoteRecordHandler::new(true);
        handler.process_record(0x001C, &[3, 0, 4, 0, 0, 0]);
        let extra = handler.last_extra.as_ref().expect("comment extra");
        assert_eq!(
            (extra.first_row_index(), extra.first_column_index()),
            (3, 4)
        );
        // 数据不足 / 错误 sid 忽略
        handler.process_record(0x001C, &[0, 0]);
        handler.process_record(0xFFFF, &[3, 0, 4, 0, 0, 0]);
        assert!(handler.last_extra.is_some());
    }
}
