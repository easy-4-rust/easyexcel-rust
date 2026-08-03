//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

/// Decoded inline-label cell produced by [`LabelRecordHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelCell {
    /// Zero-based row. (Java `LabelRecord.getRow`)
    pub row: u32,
    /// Zero-based column. (Java `LabelRecord.getColumn`)
    pub column: usize,
    /// Label text (already trimmed when `auto_trim` was set).
    pub value: String,
}

/// 对应 Java：`LabelRecordHandler`.
#[derive(Debug, Default)]
pub struct LabelRecordHandler;

impl LabelRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Java `LabelRecordHandler.processRecord`.
    #[must_use]
    pub fn process_label(row: u32, column: usize, value: &str, auto_trim: bool) -> LabelCell {
        let value = if auto_trim {
            value.trim().to_owned()
        } else {
            value.to_owned()
        };
        LabelCell { row, column, value }
    }
}

/// BIFF `Label` record sid. (POI `LabelRecord.sid`)
pub const LABEL_SID: u16 = 0x0204;

impl XlsRecordHandler for LabelRecordHandler {
    /// Java `LabelRecordHandler.processRecord` — parses coordinates; string body
    /// decoding is left to a higher-level BIFF reader / [`Self::process_label`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != LABEL_SID || data.len() < 8 {
            return;
        }
        let row = u32::from(u16::from_le_bytes([data[0], data[1]]));
        let column = u16::from_le_bytes([data[2], data[3]]) as usize;
        let _ = Self::process_label(row, column, "", false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_label_trims_when_requested() {
        let cell = LabelRecordHandler::process_label(1, 2, " a ", true);
        assert_eq!(cell.value, "a");
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_label_keeps_value_without_trim() {
        // 对应 Java：autoTrim=false 时保留原始空白
        let cell = LabelRecordHandler::process_label(1, 2, " a ", false);
        assert_eq!(cell.value, " a ");
    }

    #[test]
    fn process_record_gates_on_sid_and_length() {
        // 对应 Java：LabelRecordHandler.processRecord 的 sid/长度门控
        let mut handler = LabelRecordHandler::new();
        handler.process_record(LABEL_SID, &[1, 0, 2, 0, 0, 0, 0, 0]);
        handler.process_record(0xFFFF, &[1, 0, 2, 0, 0, 0, 0, 0]);
        handler.process_record(LABEL_SID, &[0, 0]);
    }
}
