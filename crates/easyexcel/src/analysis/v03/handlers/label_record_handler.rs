//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler`.

use super::super::xls_record_handler::XlsRecordHandler;

include!("label_record_handler/label_cell.rs");

/// 对应 Java：`LabelRecordHandler`.
#[derive(Debug, Default)]
pub struct LabelRecordHandler {
    /// 最近一次成功解析的内联字符串单元格。
    pub last_cell: Option<LabelCell>,
}

impl LabelRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// 解析物理 `LABEL` 记录，并按当前读取配置应用 Java `autoTrim` 语义。
    pub(crate) fn process_record_with_auto_trim(
        &mut self,
        record_sid: u16,
        data: &[u8],
        auto_trim: bool,
    ) {
        if record_sid != LABEL_SID {
            return;
        }
        if let Some((row, column, value)) =
            easyexcel_xls::biff8::event_record::decode_label_record(data)
        {
            self.last_cell = Some(Self::process_label(row, column, &value, auto_trim));
        }
    }
}

/// BIFF `Label` record sid. (POI `LabelRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::LABEL_SID;

impl XlsRecordHandler for LabelRecordHandler {
    /// Java `LabelRecordHandler.processRecord` — parses coordinates and the
    /// complete inline string. The dispatcher supplies the configured trim mode.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        self.process_record_with_auto_trim(record_sid, data, false);
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
        let data = [1, 0, 2, 0, 0, 0, 1, 0, 0, b'x'];
        handler.process_record(LABEL_SID, &data);
        assert_eq!(
            handler.last_cell.as_ref().map(|cell| cell.value.as_str()),
            Some("x")
        );
        handler.process_record(0xFFFF, &data);
        handler.process_record(LABEL_SID, &[0, 0]);
        assert_eq!(
            handler.last_cell.as_ref().map(|cell| cell.value.as_str()),
            Some("x")
        );
    }
}
