//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.HyperlinkRecordHandler`.

use easyexcel_core::{CellExtra, CellExtraType};

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`HyperlinkRecordHandler`.
#[derive(Debug, Default)]
pub struct HyperlinkRecordHandler {
    /// Whether hyperlink extras are enabled. (Java `support`)
    pub enabled: bool,
    /// Last parsed hyperlink extra.
    pub last_extra: Option<CellExtra>,
}

impl HyperlinkRecordHandler {
    /// Creates a handler; `enabled` mirrors Java `support(XlsReadContext)`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_extra: None,
        }
    }

    /// Java `HyperlinkRecordHandler.processRecord`.
    pub fn process_hyperlink(
        &mut self,
        address: Option<String>,
        first_row: u32,
        last_row: u32,
        first_column: usize,
        last_column: usize,
    ) {
        if !self.enabled {
            return;
        }
        self.last_extra = Some(CellExtra::new(
            CellExtraType::Hyperlink,
            address,
            first_row,
            last_row,
            first_column,
            last_column,
        ));
    }
}

impl XlsRecordHandler for HyperlinkRecordHandler {
    fn support(&self) -> bool {
        self.enabled
    }

    /// Java `HyperlinkRecordHandler.processRecord` — sid/range gate; address via helper.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        /// BIFF `Hyperlink` sid (POI `HyperlinkRecord.sid`)
        const HYPERLINK_SID: u16 = 0x01B8;
        if !self.enabled || record_sid != HYPERLINK_SID || data.len() < 8 {
            return;
        }
        let first_row = u16::from_le_bytes([data[0], data[1]]) as u32;
        let last_row = u16::from_le_bytes([data[2], data[3]]) as u32;
        let first_column = u16::from_le_bytes([data[4], data[5]]) as usize;
        let last_column = u16::from_le_bytes([data[6], data[7]]) as usize;
        self.process_hyperlink(None, first_row, last_row, first_column, last_column);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_hyperlink_requires_enabled() {
        // 对应 Java：HyperlinkRecordHandler.support() 控制是否物化
        let mut disabled = HyperlinkRecordHandler::new(false);
        assert!(!disabled.support());
        disabled.process_hyperlink(Some("x".to_owned()), 0, 1, 0, 1);
        assert!(disabled.last_extra.is_none());

        let mut handler = HyperlinkRecordHandler::new(true);
        assert!(handler.support());
        handler.process_hyperlink(Some("https://example.com".to_owned()), 0, 1, 0, 1);
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!(extra.extra_type(), CellExtraType::Hyperlink);
        assert_eq!(extra.text(), Some("https://example.com"));
        assert_eq!((extra.first_row_index(), extra.last_row_index()), (0, 1));
        assert_eq!(
            (extra.first_column_index(), extra.last_column_index()),
            (0, 1)
        );
    }

    #[test]
    fn process_record_parses_hyperlink_range() {
        // 对应 Java：HyperlinkRecordHandler.processRecord 解析 4 个范围字段
        let mut handler = HyperlinkRecordHandler::new(true);
        handler.process_record(0x01B8, &[1, 0, 2, 0, 3, 0, 4, 0]);
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!((extra.first_row_index(), extra.last_row_index()), (1, 2));
        assert_eq!(
            (extra.first_column_index(), extra.last_column_index()),
            (3, 4)
        );

        // 禁用 / 数据不足 / 错误 sid 忽略
        let mut disabled = HyperlinkRecordHandler::new(false);
        disabled.process_record(0x01B8, &[1, 0, 2, 0, 3, 0, 4, 0]);
        assert!(disabled.last_extra.is_none());
        handler.process_record(0x01B8, &[0, 0]);
        handler.process_record(0xFFFF, &[1, 0, 2, 0, 3, 0, 4, 0]);
        assert!(handler.last_extra.is_some());
    }
}
