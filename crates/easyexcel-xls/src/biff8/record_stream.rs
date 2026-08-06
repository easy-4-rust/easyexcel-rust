//! Low-level BIFF record stream used by the XLS event compatibility layer.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use cfb::CompoundFile;
use easyexcel_io::{Error as ExcelError, Result};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Reads the BIFF workbook stream from an OLE2 `.xls` compound document.
///
/// # Errors
///
/// 文件、OLE2 容器或 Workbook 流无法读取时返回错误。
pub fn read_workbook_stream(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut compound = CompoundFile::open(file)
        .map_err(|error| ExcelError::Xls(format!("invalid XLS compound document: {error}")))?;
    let mut stream = compound
        .open_stream("/Workbook")
        .or_else(|_| compound.open_stream("/Book"))
        .map_err(|error| {
            ExcelError::Xls(format!("XLS Workbook/Book stream is missing: {error}"))
        })?;
    let mut workbook = Vec::new();
    stream.read_to_end(&mut workbook)?;
    Ok(workbook)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Walks every physical BIFF record in a workbook stream.
///
/// Unlike the former display-only parser, this reports truncated headers and
/// payloads instead of silently accepting a damaged stream.
///
/// # Errors
///
/// BIFF 记录损坏、长度溢出或回调处理失败时返回错误。
pub fn walk_biff_records(
    workbook: &[u8],
    mut process: impl FnMut(u16, &[u8]) -> Result<()>,
) -> Result<()> {
    let mut offset = 0usize;
    while offset < workbook.len() {
        let remaining = &workbook[offset..];
        if remaining.iter().all(|byte| *byte == 0) {
            break;
        }
        if remaining.len() < 4 {
            return Err(ExcelError::Xls(format!(
                "truncated BIFF record header at byte {offset}"
            )));
        }

        let sid = u16::from_le_bytes([remaining[0], remaining[1]]);
        let length = u16::from_le_bytes([remaining[2], remaining[3]]) as usize;
        let payload_start = offset + 4;
        let payload_end = payload_start.checked_add(length).ok_or_else(|| {
            ExcelError::Xls(format!("BIFF record length overflow at byte {offset}"))
        })?;
        if payload_end > workbook.len() {
            return Err(ExcelError::Xls(format!(
                "truncated BIFF record 0x{sid:04X} at byte {offset}: expected {length} payload bytes"
            )));
        }

        process(sid, &workbook[payload_start..payload_end])?;
        offset = payload_end;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_records_in_physical_order() -> Result<()> {
        let bytes = [0x03, 0x02, 0x02, 0x00, 0xAA, 0xBB, 0x0A, 0x00, 0x00, 0x00];
        let mut records = Vec::new();
        walk_biff_records(&bytes, |sid, payload| {
            records.push((sid, payload.to_vec()));
            Ok(())
        })?;
        assert_eq!(records, vec![(0x0203, vec![0xAA, 0xBB]), (0x000A, vec![])]);
        Ok(())
    }

    #[test]
    fn rejects_truncated_payload() {
        let error = walk_biff_records(&[0x03, 0x02, 0x04, 0x00, 0xAA], |_, _| Ok(()))
            .expect_err("payload is truncated");
        assert!(error.to_string().contains("truncated BIFF record 0x0203"));
    }

    #[test]
    fn ignores_zero_padding() -> Result<()> {
        let mut seen = 0;
        walk_biff_records(&[0; 16], |_, _| {
            seen += 1;
            Ok(())
        })?;
        assert_eq!(seen, 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_truncated_record_header() {
        // 对应 Java：损坏的 BIFF 流报错而非静默接受
        let error =
            walk_biff_records(&[0x03, 0x02], |_, _| Ok(())).expect_err("header is truncated");
        assert!(error.to_string().contains("truncated BIFF record header"));
    }

    #[test]
    fn rejects_record_length_exceeding_buffer() {
        // 对应 Java：record 声明长度超出缓冲时报错
        let error = walk_biff_records(&[0x03, 0x02, 0xFF, 0xFF], |_, _| Ok(()))
            .expect_err("length must exceed the buffer");
        assert!(error.to_string().contains("truncated BIFF record"));
    }

    #[test]
    fn propagates_handler_errors() {
        // 对应 Java：handler 异常向上传递
        let bytes = [0x03, 0x02, 0x02, 0x00, 0xAA, 0xBB];
        let error = walk_biff_records(&bytes, |_, _| {
            Err(ExcelError::Xls("handler failure".to_owned()))
        })
        .expect_err("handler error must propagate");
        assert!(error.to_string().contains("handler failure"));
    }

    #[test]
    fn read_workbook_stream_reports_missing_stream() -> Result<()> {
        // 对应 Java：OLE2 文档缺少 Workbook/Book 流报错
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("nostream.xls");
        let mut compound = cfb::create(&path)?;
        let mut stream = compound.create_stream("OtherStream")?;
        stream.write_all(b"data")?;
        drop(compound);

        let error = read_workbook_stream(&path).expect_err("missing Workbook stream");
        assert!(
            error
                .to_string()
                .contains("Workbook/Book stream is missing")
        );
        Ok(())
    }

    #[test]
    fn read_workbook_stream_reads_real_cfb_workbook() -> Result<()> {
        // 对应 Java：从 OLE2 复合文档读取 Workbook 流内容
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("workbook.xls");
        let mut compound = cfb::create(&path)?;
        let mut stream = compound.create_stream("/Workbook")?;
        stream.write_all(&[0x03, 0x02, 0x02, 0x00, 0xAA, 0xBB])?;
        drop(stream);
        compound.flush()?;
        drop(compound);

        let workbook = read_workbook_stream(&path)?;
        assert_eq!(workbook, vec![0x03, 0x02, 0x02, 0x00, 0xAA, 0xBB]);
        Ok(())
    }
}
