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

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.StringRecordHandler。 Java `StringRecordHandler.processRecord` — applies the untrimmed string onto a pending formula.
    pub fn process_string(
        formula_handler: &mut FormulaRecordHandler,
        value: String,
        _auto_trim: bool,
    ) -> Option<(FormulaCell, String)> {
        // Java StringRecordHandler 直接写入 StringRecord#getString()；与普通
        // LABEL/LABELSST 不同，公式字符串结果不会应用 global autoTrim。
        let text = value;
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
    fn process_string_keeps_spaces_for_both_trim_modes() {
        // 对应 Java：公式 StringRecord 不应用 global autoTrim。
        let mut formula = FormulaRecordHandler::new();
        let cell = StringRecordHandler::process_string(&mut formula, " x ".to_owned(), false);
        assert!(cell.is_none(), "无 pending 公式时返回 None");
        let _ = formula.process_formula(
            0,
            0,
            None,
            super::super::formula_record_handler::FormulaCachedType::String,
            None,
            None,
        );
        let cell = StringRecordHandler::process_string(&mut formula, " x ".to_owned(), true)
            .expect("pending string formula");
        assert_eq!(cell.1, " x ");
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
