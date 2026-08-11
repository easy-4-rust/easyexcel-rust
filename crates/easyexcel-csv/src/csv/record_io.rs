//! CSV 增量记录读写后端。

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use easyexcel_io::{Error, Result};
use encoding_rs_io::DecodeReaderBytes;

use super::{CsvCharset, CsvEncodingWriter, csv_bom, csv_encoding, decode_reader};

include!("record_io/csv_record_writer.rs");

include!("record_io/csv_record_reader.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    /// 共享缓冲区，实现 `Write + Send + 'static`，用于测试 `CsvRecordWriter`。
    #[derive(Clone)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl SharedBuf {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Vec::new())))
        }
        fn into_bytes(self) -> Vec<u8> {
            Arc::try_unwrap(self.0)
                .expect("shared buf still borrowed")
                .into_inner()
                .unwrap()
        }
    }

    impl Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn record_writer_utf8_no_bom() {
        let buf = SharedBuf::new();
        let mut writer =
            CsvRecordWriter::new(Box::new(buf.clone()), &CsvCharset::utf8(), false).unwrap();
        writer.write_record(["a", "b", "c"]).unwrap();
        writer.finish().unwrap();
        let text = String::from_utf8(buf.into_bytes()).unwrap();
        assert_eq!(text, "a,b,c\n");
    }

    #[test]
    fn record_writer_utf8_with_bom() {
        let buf = SharedBuf::new();
        let mut writer =
            CsvRecordWriter::new(Box::new(buf.clone()), &CsvCharset::utf8(), true).unwrap();
        writer.write_record(["x"]).unwrap();
        writer.finish().unwrap();
        let bytes = buf.into_bytes();
        // 前 3 字节为 UTF-8 BOM
        assert_eq!(&bytes[..3], b"\xEF\xBB\xBF");
        let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
        assert_eq!(text, "x\n");
    }

    #[test]
    fn record_writer_multiple_records() {
        let buf = SharedBuf::new();
        let mut writer =
            CsvRecordWriter::new(Box::new(buf.clone()), &CsvCharset::utf8(), false).unwrap();
        writer.write_record(["a", "b"]).unwrap();
        writer.write_record(["c", "d"]).unwrap();
        writer.finish().unwrap();
        let text = String::from_utf8(buf.into_bytes()).unwrap();
        assert_eq!(text, "a,b\nc,d\n");
    }

    #[test]
    fn record_reader_utf8() {
        let data = b"a,b\nc,d\n";
        let mut reader = CsvRecordReader::new(&data[..], &CsvCharset::utf8()).unwrap();
        let records: Vec<Vec<String>> = reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["a", "b"]);
        assert_eq!(records[1], vec!["c", "d"]);
    }

    #[test]
    fn record_reader_empty() {
        let data = b"";
        let mut reader = CsvRecordReader::new(&data[..], &CsvCharset::utf8()).unwrap();
        let records: Vec<Vec<String>> = reader.records().map(|r| r.unwrap()).collect();
        assert!(records.is_empty());
    }

    #[test]
    fn record_reader_with_bom() {
        let mut data = b"\xEF\xBB\xBF".to_vec();
        data.extend_from_slice(b"a,b");
        let mut reader = CsvRecordReader::new(&data[..], &CsvCharset::utf8()).unwrap();
        let records: Vec<Vec<String>> = reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], vec!["a", "b"]);
    }

    #[test]
    fn record_writer_unsupported_charset() {
        let buf = SharedBuf::new();
        let result = CsvRecordWriter::new(Box::new(buf), &CsvCharset::new("INVALID"), false);
        assert!(result.is_err());
    }

    #[test]
    fn record_reader_unsupported_charset() {
        let data = b"a";
        let result = CsvRecordReader::new(&data[..], &CsvCharset::new("INVALID"));
        assert!(result.is_err());
    }
}