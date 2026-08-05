//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.BofRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// POI `BOFRecord` type codes used by `EasyExcel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BofType {
    /// Workbook-level BOF.
    Workbook,
    /// Worksheet-level BOF.
    Worksheet,
    /// Other (chart, macro, …) — ignored by Java.
    Other,
}

/// Side-effects requested by [`BofRecordHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BofAction {
    /// Reset workbook sheet cursor. (`TYPE_WORKBOOK`)
    ResetWorkbook,
    /// Ignore non-worksheet BOF.
    Ignore,
    /// Begin / skip a worksheet sheet.
    BeginWorksheet {
        /// Whether the matched sheet should be read (`ignoreRecord = false`).
        read_sheet: bool,
        /// Next `readSheetIndex` after this BOF.
        next_read_sheet_index: usize,
    },
}

/// 对应 Java：`BofRecordHandler`.
#[derive(Debug, Default)]
pub struct BofRecordHandler;

impl BofRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Java `BofRecordHandler.processRecord` decision (sheet list already built).
    #[must_use]
    pub fn decide(
        bof_type: BofType,
        read_sheet_index: Option<usize>,
        sheet_matched: bool,
    ) -> BofAction {
        match bof_type {
            BofType::Workbook => BofAction::ResetWorkbook,
            BofType::Other => BofAction::Ignore,
            BofType::Worksheet => {
                let index = read_sheet_index.unwrap_or(0);
                BofAction::BeginWorksheet {
                    read_sheet: sheet_matched,
                    next_read_sheet_index: index.saturating_add(1),
                }
            }
        }
    }
}

/// BIFF `BOF` record sid. (POI `BOFRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::BOF_SID;

impl XlsRecordHandler for BofRecordHandler {
    /// Java `BofRecordHandler.processRecord` — parses type code; use [`Self::decide`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != BOF_SID {
            return;
        }
        let Some(bof_type) = easyexcel_xls::biff8::event_record::decode_bof_type(data) else {
            return;
        };
        let bof_type = match bof_type {
            easyexcel_xls::biff8::event_record::Biff8BofType::Workbook => BofType::Workbook,
            easyexcel_xls::biff8::event_record::Biff8BofType::Worksheet => BofType::Worksheet,
            easyexcel_xls::biff8::event_record::Biff8BofType::Other(_) => BofType::Other,
        };
        let _ = Self::decide(bof_type, None, false);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn decide_other_and_worksheet_branches() {
        // 对应 Java：BofRecordHandler.decide 全分支
        assert_eq!(
            BofRecordHandler::decide(BofType::Other, None, false),
            BofAction::Ignore
        );
        assert_eq!(
            BofRecordHandler::decide(BofType::Workbook, None, false),
            BofAction::ResetWorkbook
        );
        let action = BofRecordHandler::decide(BofType::Worksheet, Some(2), true);
        assert_eq!(
            action,
            BofAction::BeginWorksheet {
                read_sheet: true,
                next_read_sheet_index: 3
            }
        );
        let action = BofRecordHandler::decide(BofType::Worksheet, None, false);
        assert_eq!(
            action,
            BofAction::BeginWorksheet {
                read_sheet: false,
                next_read_sheet_index: 1
            }
        );
    }

    #[test]
    fn process_record_parses_type_code() {
        // 对应 Java：BOF 类型码 0x0005/0x0010/其他
        let mut handler = BofRecordHandler::new();
        handler.process_record(BOF_SID, &[0, 0, 0x05, 0x00]); // workbook
        handler.process_record(BOF_SID, &[0, 0, 0x10, 0x00]); // worksheet
        handler.process_record(BOF_SID, &[0, 0, 0x99, 0x00]); // other
        handler.process_record(BOF_SID, &[0, 0]); // 数据不足
        handler.process_record(0xFFFF, &[0, 0, 0x05, 0x00]); // 错误 sid
    }
}
