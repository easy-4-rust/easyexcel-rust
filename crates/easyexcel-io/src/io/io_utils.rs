//! 通用字节流复制操作。

#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::Error as ExcelError;

/// Java `IoUtils.EOF`：流读取到末尾时使用的哨兵值。
pub const EOF: i32 = -1;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Mirrors `org.apache.commons.io.IOUtils#copy`.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 读取输入流剩余的全部字节。
///
/// # Errors
///
/// 输入流读取失败时返回 [`ExcelError::Io`]。
pub fn read_all(reader: &mut dyn Read) -> Result<Vec<u8>, ExcelError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入全部字节并刷新输出流。
///
/// # Errors
///
/// 输出流写入或刷新失败时返回 [`ExcelError::Io`]。
pub fn write_all_and_flush<W>(writer: &mut W, bytes: &[u8]) -> Result<(), ExcelError>
where
    W: Write + ?Sized,
{
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
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
