//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.SstRecordHandler`.
//!
//! The dispatcher assembles physical CONTINUE records and supplies the decoded
//! strings, matching Java's `XlsCache(SSTRecord)` responsibility.

use super::super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`SstRecordHandler`.
#[derive(Debug, Default)]
pub struct SstRecordHandler {
    /// Number of unique strings announced by the SST. (Java `getNumUniqueStrings`)
    pub unique_string_count: Option<u32>,
    /// Decoded shared strings in SST index order. (Java `XlsCache`)
    pub strings: Vec<String>,
}

impl SstRecordHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `SstRecordHandler.processRecord` — bookkeeping only (cache filled elsewhere).
    pub fn process_sst(&mut self, unique_string_count: u32) {
        self.unique_string_count = Some(unique_string_count);
    }

    /// Installs a fully decoded SST after CONTINUE records have been assembled.
    pub fn process_decoded_sst(&mut self, unique_string_count: u32, strings: Vec<String>) {
        self.unique_string_count = Some(unique_string_count);
        self.strings = strings;
    }

    /// Resolves one SST index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }
}

/// BIFF `SST` record sid. (POI `SSTRecord.sid`)
pub use easyexcel_xls::biff8::record_sid::SST_SID;

impl XlsRecordHandler for SstRecordHandler {
    /// Java `SstRecordHandler.processRecord` — reads `cstTotal`/`cstUnique` header.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid != SST_SID || data.len() < 8 {
            return;
        }
        if let Some(unique) = easyexcel_xls::biff8::event_record::decode_sst_unique_count(data) {
            self.process_sst(unique);
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn process_record_gates_on_sid_and_length() {
        // 对应 Java：SstRecordHandler.processRecord 的 sid/长度门控
        let mut handler = SstRecordHandler::new();
        let mut sst = Vec::new();
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&3u32.to_le_bytes());
        handler.process_record(SST_SID, &sst);
        assert_eq!(handler.unique_string_count, Some(3));
        handler.process_record(SST_SID, &[0, 0, 0]);
        handler.process_record(0xFFFF, &sst);
        assert_eq!(handler.unique_string_count, Some(3));
    }
}
