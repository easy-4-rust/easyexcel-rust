//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.EofRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// Actions requested by [`EofRecordHandler`] at sheet EOF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EofAction {
    /// Ignore — sheet was skipped or stop was requested without stop-sheet.
    Ignore,
    /// Call `endSheet` because the user stopped the current sheet.
    EndSheetOnly,
    /// Forge a final row flush (non-empty cellMap) then `endSheet`.
    FlushRowThenEndSheet,
    /// Just `endSheet`.
    EndSheet,
}

/// 对应 Java：`EofRecordHandler`.
#[derive(Debug, Default)]
pub struct EofRecordHandler;

impl EofRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Java `EofRecordHandler.processRecord` decision tree (pure).
    #[must_use]
    pub fn decide(
        has_sheet_holder: bool,
        ignore_record: bool,
        current_sheet_stopped: bool,
        cell_map_empty: bool,
    ) -> EofAction {
        if !has_sheet_holder {
            return EofAction::Ignore;
        }
        if ignore_record {
            return if current_sheet_stopped {
                EofAction::EndSheetOnly
            } else {
                EofAction::Ignore
            };
        }
        if !cell_map_empty {
            EofAction::FlushRowThenEndSheet
        } else {
            EofAction::EndSheet
        }
    }
}

/// BIFF `EOF` record sid. (POI `EOFRecord.sid`)
pub const EOF_SID: u16 = 0x000A;

impl XlsRecordHandler for EofRecordHandler {
    /// Java `EofRecordHandler.processRecord` — sid gate; use [`Self::decide`] for actions.
    fn process_record(&mut self, record_sid: u16, _data: &[u8]) {
        let _ = record_sid == EOF_SID;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_flush_when_cells_remain() {
        assert_eq!(
            EofRecordHandler::decide(true, false, false, false),
            EofAction::FlushRowThenEndSheet
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn decide_covers_all_java_branches() {
        // 对应 Java：EofRecordHandler.processRecord 决策树全分支
        assert_eq!(
            EofRecordHandler::decide(false, false, false, false),
            EofAction::Ignore
        );
        assert_eq!(
            EofRecordHandler::decide(true, true, true, true),
            EofAction::EndSheetOnly
        );
        assert_eq!(
            EofRecordHandler::decide(true, true, false, false),
            EofAction::Ignore
        );
        assert_eq!(
            EofRecordHandler::decide(true, false, false, true),
            EofAction::EndSheet
        );
    }

    #[test]
    fn process_record_gates_on_sid() {
        // 对应 Java：EOF sid 门控
        let mut handler = EofRecordHandler::new();
        handler.process_record(EOF_SID, &[]);
        handler.process_record(0xFFFF, &[]);
    }
}
