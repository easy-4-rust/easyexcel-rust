//! 对应 Java： com.alibaba.excel.util.IoUtils.

#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::core::excel_error::ExcelError;

/// Mirrors `org.apache.commons.io.IOUtils#copy`.
///
/// Copies all bytes from `reader` into `writer` using a 4 KiB stack
/// buffer (Java uses a 4 KiB byte array).
///
/// # Errors
///
/// 当读取或写入失败时返回 [`ExcelError::Io`]。
pub fn copy(reader: &mut dyn Read, writer: &mut dyn Write) -> Result<u64, ExcelError> {
    let n = io::copy(reader, writer)?;
    Ok(n)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    #[test]
    fn copy_transfers_all_bytes() {
        // 对应 Java：IOUtils.copy 全量拷贝
        let mut reader = io::Cursor::new(vec![1_u8, 2, 3, 4]);
        let mut writer = Vec::new();
        let copied = copy(&mut reader, &mut writer).expect("copies");
        assert_eq!(copied, 4);
        assert_eq!(writer, vec![1, 2, 3, 4]);
    }

    #[test]
    fn copy_reports_reader_errors() {
        // 对应 Java：读取失败时向上抛错
        let mut writer = Vec::new();
        let error = copy(&mut FailingReader, &mut writer).expect_err("fails");
        assert!(matches!(error, ExcelError::Io(_)));
    }
}
