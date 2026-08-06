//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

include!("label_record_handler/label_cell.rs");

/// 对应 Java：`LabelRecordHandler`.
#[derive(Debug, Default)]
pub struct LabelRecordHandler;

impl LabelRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler。 Java `LabelRecordHandler.processRecord`.
    #[must_use]
    pub fn process_label(row: u32, column: usize, value: &str, auto_trim: bool) -> LabelCell {
        let value = if auto_trim {
            easyexcel_utils::string_utils::java_trim(value).to_owned()
        } else {
            value.to_owned()
        };
        LabelCell { row, column, value }
    }
}

/// BIFF `Label` record sid. (POI `LabelRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::LABEL_SID;

impl XlsRecordHandler for LabelRecordHandler {
    /// Java `LabelRecordHandler.processRecord` — parses coordinates; string body
    /// decoding is left to a higher-level BIFF reader / [`Self::process_label`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != LABEL_SID {
            return;
        }
        if let Some((row, column)) =
            easyexcel_xls::biff8::event_record::decode_label_record_position(data)
        {
            let _ = Self::process_label(row, column, "", false);
        }
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
