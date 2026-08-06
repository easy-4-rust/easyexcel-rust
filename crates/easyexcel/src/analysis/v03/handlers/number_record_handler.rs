//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.NumberRecordHandler`.
//!
//! XLS BIFF decoding is owned by `easyexcel-xls`; these handlers preserve
//! the Java `processRecord` numeric-cell semantics for a future SAX path.

use super::super::xls_record_handler::XlsRecordHandler;

include!("number_record_handler/number_cell.rs");

/// 对应 Java：`NumberRecordHandler`.
#[derive(Debug, Default)]
pub struct NumberRecordHandler {
    /// Most recently decoded number cell.
    pub last_cell: Option<NumberCell>,
}

impl NumberRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.NumberRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.NumberRecordHandler。 Java `NumberRecordHandler.processRecord` (without `BuiltinFormats` lookup).
    #[must_use]
    pub fn process_number(row: u32, column: usize, value: f64, format_index: u16) -> NumberCell {
        NumberCell {
            row,
            column,
            value,
            format_index,
        }
    }
}

/// BIFF `Number` record sid. (POI `NumberRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::NUMBER_SID;

impl XlsRecordHandler for NumberRecordHandler {
    /// Java `NumberRecordHandler.processRecord` — parses BIFF Number body
    /// (`row|col|xf|f64`). Formatting lookup stays in [`Self::process_number`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != NUMBER_SID {
            return;
        }
        if let Some(record) = easyexcel_xls::biff8::event_record::decode_number_record(data) {
            self.last_cell = Some(Self::process_number(
                record.header.row,
                record.header.column,
                record.value,
                record.header.xf_index,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // 对应 Java：测试断言为位级精确的 `f64` 往返值（`from_le_bytes` 后原值不变），
    // 精确相等即预期语义，不做容差比较。
    #[allow(clippy::float_cmp)]
    fn process_number_keeps_value() {
        let cell = NumberRecordHandler::process_number(1, 2, 3.5, 0);
        assert_eq!(cell.row, 1);
        assert_eq!(cell.column, 2);
        assert_eq!(cell.value, 3.5);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_gates_on_sid_and_length() {
        // 对应 Java：NumberRecordHandler.processRecord 的 sid/长度门控
        let mut handler = NumberRecordHandler::new();
        handler.process_record(0xFFFF, &[2, 0, 3, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(handler.last_cell.is_none());
        handler.process_record(NUMBER_SID, &[2, 0, 3, 0, 7, 0]);
        assert!(handler.last_cell.is_none());
        assert_eq!(handler.last_cell, None, "不足 14 字节时保持原状态");
    }
}
