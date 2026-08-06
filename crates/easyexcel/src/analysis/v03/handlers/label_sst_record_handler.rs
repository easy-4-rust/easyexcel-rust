//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.LabelSstRecordHandler`.
//!
//! Resolves an SST index through a caller-supplied cache lookup, matching
//! Java's `ReadCache.get(sstIndex)` path.

use super::super::xls_record_handler::XlsRecordHandler;

include!("label_sst_record_handler/label_sst_cell.rs");

/// 对应 Java：`LabelSstRecordHandler`.
#[derive(Debug, Default)]
pub struct LabelSstRecordHandler {
    /// Most recently parsed raw `LabelSST` placement and cache index.
    pub last_reference: Option<LabelSstReference>,
}

include!("label_sst_record_handler/label_sst_reference.rs");

impl LabelSstRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.LabelSstRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.LabelSstRecordHandler。 Java `LabelSstRecordHandler.processRecord`.
    ///
    /// `resolve` maps SST index → string (`ReadCache.get`); `None` yields empty.
    pub fn process_label_sst(
        row: u32,
        column: usize,
        sst_index: usize,
        auto_trim: bool,
        resolve: &dyn Fn(usize) -> Option<String>,
    ) -> LabelSstCell {
        match resolve(sst_index) {
            None => LabelSstCell::Empty { row, column },
            Some(mut data) => {
                if auto_trim {
                    data = easyexcel_utils::string_utils::java_trim(&data).to_owned();
                }
                LabelSstCell::String {
                    row,
                    column,
                    value: data,
                }
            }
        }
    }
}

/// BIFF `LabelSST` record sid. (POI `LabelSSTRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::LABEL_SST_SID;

impl XlsRecordHandler for LabelSstRecordHandler {
    /// Java `LabelSstRecordHandler.processRecord` — accepts `LabelSST` sid and
    /// validates the 10-byte BIFF body (`row|col|xf|sstIndex`). Full cache
    /// resolution uses [`LabelSstRecordHandler::process_label_sst`].
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != LABEL_SST_SID {
            return;
        }
        if let Some(record) = easyexcel_xls::biff8::event_record::decode_label_sst_record(data) {
            self.last_reference = Some(LabelSstReference {
                row: record.header.row,
                column: record.header.column,
                sst_index: record.sst_index,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sst_yields_empty() {
        let cell = LabelSstRecordHandler::process_label_sst(0, 1, 9, false, &|_| None);
        assert_eq!(cell, LabelSstCell::Empty { row: 0, column: 1 });
    }

    #[test]
    fn auto_trim_strips_whitespace() {
        let cell =
            LabelSstRecordHandler::process_label_sst(0, 0, 0, true, &|_| Some("  hi  ".into()));
        assert_eq!(
            cell,
            LabelSstCell::String {
                row: 0,
                column: 0,
                value: "hi".into()
            }
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_validates_label_sst_body() {
        // 对应 Java：LabelSstRecordHandler.processRecord 校验 10 字节 BIFF body
        let mut handler = LabelSstRecordHandler::new();
        let mut data = vec![3, 0, 2, 0, 0, 0];
        data.extend_from_slice(&5u32.to_le_bytes()); // sstIndex=5
        handler.process_record(LABEL_SST_SID, &data);
        let reference = handler.last_reference.expect("label sst reference");
        assert_eq!(
            (reference.row, reference.column, reference.sst_index),
            (3, 2, 5)
        );
        // 数据不足 / 错误 sid 忽略
        handler.process_record(LABEL_SST_SID, &[0; 9]);
        handler.process_record(0xFFFF, &data);
        assert_eq!(handler.last_reference.unwrap().sst_index, 5);
    }
}
