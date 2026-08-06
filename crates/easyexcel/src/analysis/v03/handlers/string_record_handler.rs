//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.StringRecordHandler`.
//!
//! Completes a pending string-formula cell created by [`super::formula_record_handler`].

use super::super::xls_record_handler::XlsRecordHandler;
use super::formula_record_handler::{FormulaCell, FormulaRecordHandler};

/// 对应 Java：`StringRecordHandler`.
#[derive(Debug, Default)]
pub struct StringRecordHandler {
    /// Most recently decoded formula string result.
    pub last_value: Option<String>,
}

impl StringRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.StringRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.StringRecordHandler。 Java `StringRecordHandler.processRecord` — applies string onto pending formula.
    pub fn process_string(
        formula_handler: &mut FormulaRecordHandler,
        value: String,
        auto_trim: bool,
    ) -> Option<(FormulaCell, String)> {
        let text = if auto_trim {
            easyexcel_utils::string_utils::java_trim(&value).to_owned()
        } else {
            value
        };
        formula_handler
            .complete_pending_string(text.clone())
            .map(|cell| (cell, text))
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.StringRecordHandler。 Stores an already decoded String record value.
    pub fn process_decoded(&mut self, value: String) {
        self.last_value = Some(value);
    }
}

/// BIFF `String` record sid (formula result). (POI `StringRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::STRING_SID;

impl XlsRecordHandler for StringRecordHandler {
    /// Java `StringRecordHandler.processRecord` — sid gate; pair with
    /// [`Self::process_string`] and a live [`FormulaRecordHandler`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != STRING_SID {
            return;
        }
        if let Ok(value) = easyexcel_xls::biff8::string::decode_unicode_string_record(data) {
            self.process_decoded(value);
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_string_without_trim_keeps_spaces() {
        // 对应 Java：autoTrim=false 时公式字符串保留空白
        let mut formula = FormulaRecordHandler::new();
        let cell = StringRecordHandler::process_string(&mut formula, " x ".to_owned(), false);
        assert!(cell.is_none(), "无 pending 公式时返回 None");
        let cell = StringRecordHandler::process_string(&mut formula, " x ".to_owned(), true);
        assert!(cell.is_none());
    }

    #[test]
    fn process_record_decodes_string_segments() {
        // 对应 Java：StringRecordHandler.processRecord 解码 BIFF8 字符串
        let mut handler = StringRecordHandler::new();
        let mut data = vec![3, 0, 0];
        data.extend_from_slice(b"abc");
        handler.process_record(STRING_SID, &data);
        assert_eq!(handler.last_value.as_deref(), Some("abc"));
        // 错误 sid 忽略
        handler.process_record(0xFFFF, &data);
        assert_eq!(handler.last_value.as_deref(), Some("abc"));
        // 解码失败时保持原状态
        handler.process_record(STRING_SID, &[5, 0, 0, b'a']);
        assert_eq!(handler.last_value.as_deref(), Some("abc"));
    }
}
