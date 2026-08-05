//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.MergeCellsRecordHandler`.

use crate::core::{CellExtra, CellExtraType};

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`MergeCellsRecordHandler`.
#[derive(Debug, Default)]
pub struct MergeCellsRecordHandler {
    /// Whether merge extras are enabled. (Java `support`)
    pub enabled: bool,
    /// Last emitted merge extras from one record (may contain multiple areas).
    pub last_extras: Vec<CellExtra>,
}

impl MergeCellsRecordHandler {
    /// Creates a handler; `enabled` mirrors Java `support(XlsReadContext)`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_extras: Vec::new(),
        }
    }

    /// Java `MergeCellsRecordHandler.support`.
    #[must_use]
    pub fn support(&self) -> bool {
        self.enabled
    }

    /// Java `MergeCellsRecordHandler.processRecord` for one merged area.
    pub fn process_area(
        &mut self,
        first_row: u32,
        last_row: u32,
        first_column: usize,
        last_column: usize,
    ) {
        if !self.enabled {
            return;
        }
        self.last_extras.push(CellExtra::new(
            CellExtraType::Merge,
            None,
            first_row,
            last_row,
            first_column,
            last_column,
        ));
    }

    /// Drains extras accumulated for the current record.
    pub fn take_extras(&mut self) -> Vec<CellExtra> {
        std::mem::take(&mut self.last_extras)
    }
}

impl XlsRecordHandler for MergeCellsRecordHandler {
    fn support(&self) -> bool {
        self.enabled
    }

    /// Java `MergeCellsRecordHandler.processRecord` — parses area count + ranges.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        /// BIFF `MergeCells` sid (POI `MergeCellsRecord.sid`)
        const MERGE_CELLS_SID: u16 = easyexcel_xls::biff8::record_sid::MERGE_CELLS_SID;
        if !self.enabled || record_sid != MERGE_CELLS_SID || data.len() < 2 {
            return;
        }
        self.last_extras.clear();
        for range in easyexcel_xls::biff8::event_record::decode_merge_ranges(data) {
            self.process_area(
                range.first_row,
                range.last_row,
                range.first_column,
                range.last_column,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_handler_ignores_areas() {
        let mut handler = MergeCellsRecordHandler::new(false);
        handler.process_area(0, 1, 0, 1);
        assert!(handler.take_extras().is_empty());
    }

    #[test]
    fn enabled_handler_collects_areas() {
        let mut handler = MergeCellsRecordHandler::new(true);
        handler.process_area(0, 1, 0, 2);
        let extras = handler.take_extras();
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0].extra_type(), CellExtraType::Merge);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_parses_areas_and_truncation() {
        // 对应 Java：MergeCellsRecordHandler.processRecord 解析多个合并区域
        let mut handler = MergeCellsRecordHandler::new(true);
        let mut data = vec![2, 0];
        data.extend_from_slice(&[0, 0, 1, 0, 0, 0, 1, 0]); // A1:B2
        data.extend_from_slice(&[2, 0, 3, 0, 2, 0, 3, 0]); // C3:D4
        handler.process_record(0x00E5, &data);
        let extras = handler.take_extras();
        assert_eq!(extras.len(), 2);
        assert_eq!(
            (extras[0].first_row_index(), extras[0].last_row_index()),
            (0, 1)
        );
        assert_eq!(
            (
                extras[1].first_column_index(),
                extras[1].last_column_index()
            ),
            (2, 3)
        );
        assert!(handler.support());

        // 截断的区域直接跳出（长度不足 8 字节）
        handler.process_record(0x00E5, &[2, 0, 0, 0, 1, 0, 0]);
        assert!(handler.take_extras().is_empty());
    }

    #[test]
    fn process_record_disabled_or_wrong_sid_is_noop() {
        // 对应 Java：support()=false 或 sid 不匹配时忽略
        let mut disabled = MergeCellsRecordHandler::new(false);
        disabled.process_record(0x00E5, &[1, 0, 0, 0, 1, 0, 0, 0, 1, 0]);
        assert!(disabled.take_extras().is_empty());

        let mut handler = MergeCellsRecordHandler::new(true);
        handler.process_record(0xFFFF, &[1, 0, 0, 0, 1, 0, 0, 0, 1, 0]);
        assert!(handler.take_extras().is_empty());
        handler.process_record(0x00E5, &[0]);
        assert!(handler.take_extras().is_empty());
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn trait_support_reflects_enabled_flag() {
        // 对应 Java：XlsRecordHandler.support() 与 enable 开关一致
        assert!(XlsRecordHandler::support(&MergeCellsRecordHandler::new(
            true
        )));
        assert!(!XlsRecordHandler::support(&MergeCellsRecordHandler::new(
            false
        )));
    }
}
