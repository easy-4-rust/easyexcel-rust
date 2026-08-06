//! 表格单元格 gzip spill 的中立二进制协议。
//!
//! 该模块定义压缩临时行的稳定 tag/length 编码，不依赖 `EasyExcel` 门面
//! `CellValue`、builder、listener 或具体 XLSX 写入器。

use std::path::{Path, PathBuf};

use crate::{Error, Result};

use super::gzip_record::{GzipRecordReader, GzipRecordSnapshot, GzipRecordWriter};

include!("gzip_cell_record/gzip_cell_spill_snapshot.rs");

include!("gzip_cell_record/gzip_cell_value.rs");

include!("gzip_cell_record/gzip_cell_record_writer.rs");

include!("gzip_cell_record/gzip_cell_record_reader.rs");

include!("gzip_cell_record/gzip_cell_spill_writer.rs");

include!("gzip_cell_record/gzip_cell_spill_reader.rs");

fn spill_snapshot(sheet_name: String, snapshot: GzipRecordSnapshot) -> GzipCellSpillSnapshot {
    GzipCellSpillSnapshot {
        sheet_name,
        path: snapshot.path,
        is_gzip: snapshot.is_gzip,
        compressed_len: snapshot.compressed_len,
        uncompressed_len: snapshot.uncompressed_len,
    }
}

fn encode_row(cells: &[GzipCellValue]) -> Result<Vec<u8>> {
    let mut body = Vec::with_capacity(cells.len().saturating_mul(16));
    write_u32(
        &mut body,
        u32::try_from(cells.len())
            .map_err(|_| Error::Other("row cell count exceeds u32".to_owned()))?,
    );
    for cell in cells {
        encode_cell(&mut body, cell)?;
    }
    Ok(body)
}

fn decode_row(payload: &[u8]) -> Result<Vec<GzipCellValue>> {
    let mut cursor = 0usize;
    let count = read_u32(payload, &mut cursor)? as usize;
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        cells.push(decode_cell(payload, &mut cursor)?);
    }
    Ok(cells)
}

fn encode_cell(out: &mut Vec<u8>, value: &GzipCellValue) -> Result<()> {
    match value {
        GzipCellValue::Empty => out.push(0),
        GzipCellValue::Text(text) => write_tagged_string(out, 1, text)?,
        GzipCellValue::Bool(flag) => {
            out.push(2);
            out.push(u8::from(*flag));
        }
        GzipCellValue::Int(number) => {
            out.push(3);
            out.extend_from_slice(&number.to_le_bytes());
        }
        GzipCellValue::Float(number) => {
            out.push(4);
            out.extend_from_slice(&number.to_le_bytes());
        }
        GzipCellValue::Decimal(number) => write_tagged_string(out, 5, number)?,
        GzipCellValue::Date(date) => write_tagged_string(out, 6, date)?,
        GzipCellValue::DateTime(date_time) => write_tagged_string(out, 7, date_time)?,
        GzipCellValue::Error(text) => write_tagged_string(out, 8, text)?,
        GzipCellValue::Formula(text) => write_tagged_string(out, 9, text)?,
        GzipCellValue::Hyperlink { url, text } => {
            out.push(10);
            write_str(out, url)?;
            write_str(out, text)?;
        }
        GzipCellValue::Comment { value, text } => {
            out.push(11);
            write_str(out, text)?;
            encode_cell(out, value)?;
        }
        GzipCellValue::Image(bytes) => {
            out.push(12);
            write_bytes(out, bytes)?;
        }
        GzipCellValue::RichText(text) => write_tagged_string(out, 13, text)?,
        GzipCellValue::Images { value, images } => {
            out.push(14);
            encode_cell(out, value)?;
            write_u32(
                out,
                u32::try_from(images.len())
                    .map_err(|_| Error::Other("image list exceeds u32".to_owned()))?,
            );
            for image in images {
                write_bytes(out, image)?;
            }
        }
    }
    Ok(())
}

fn decode_cell(buf: &[u8], cursor: &mut usize) -> Result<GzipCellValue> {
    Ok(match read_u8(buf, cursor)? {
        0 => GzipCellValue::Empty,
        1 => GzipCellValue::Text(read_str(buf, cursor)?),
        2 => GzipCellValue::Bool(read_u8(buf, cursor)? != 0),
        3 => GzipCellValue::Int(i64::from_le_bytes(read_exact(buf, cursor)?)),
        4 => GzipCellValue::Float(f64::from_le_bytes(read_exact(buf, cursor)?)),
        5 => GzipCellValue::Decimal(read_str(buf, cursor)?),
        6 => GzipCellValue::Date(read_str(buf, cursor)?),
        7 => GzipCellValue::DateTime(read_str(buf, cursor)?),
        8 => GzipCellValue::Error(read_str(buf, cursor)?),
        9 => GzipCellValue::Formula(read_str(buf, cursor)?),
        10 => GzipCellValue::Hyperlink {
            url: read_str(buf, cursor)?,
            text: read_str(buf, cursor)?,
        },
        11 => GzipCellValue::Comment {
            text: read_str(buf, cursor)?,
            value: Box::new(decode_cell(buf, cursor)?),
        },
        12 => GzipCellValue::Image(read_bytes(buf, cursor)?),
        13 => GzipCellValue::RichText(read_str(buf, cursor)?),
        14 => {
            let value = Box::new(decode_cell(buf, cursor)?);
            let count = read_u32(buf, cursor)? as usize;
            let mut images = Vec::with_capacity(count);
            for _ in 0..count {
                images.push(read_bytes(buf, cursor)?);
            }
            GzipCellValue::Images { value, images }
        }
        other => {
            return Err(Error::Other(format!(
                "unknown gzip spill cell tag: {other}"
            )));
        }
    })
}

fn write_tagged_string(out: &mut Vec<u8>, tag: u8, value: &str) -> Result<()> {
    out.push(tag);
    write_str(out, value)
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_str(out: &mut Vec<u8>, value: &str) -> Result<()> {
    write_bytes(out, value.as_bytes())
}

fn write_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    write_u32(
        out,
        u32::try_from(value.len())
            .map_err(|_| Error::Other("spill byte length exceeds u32".to_owned()))?,
    );
    out.extend_from_slice(value);
    Ok(())
}

fn read_u8(buf: &[u8], cursor: &mut usize) -> Result<u8> {
    let value = *buf
        .get(*cursor)
        .ok_or_else(|| Error::Other("gzip spill truncated (u8)".to_owned()))?;
    *cursor += 1;
    Ok(value)
}

fn read_u32(buf: &[u8], cursor: &mut usize) -> Result<u32> {
    Ok(u32::from_le_bytes(read_exact(buf, cursor)?))
}

fn read_exact<const N: usize>(buf: &[u8], cursor: &mut usize) -> Result<[u8; N]> {
    let end = cursor
        .checked_add(N)
        .ok_or_else(|| Error::Other("gzip spill cursor overflow".to_owned()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| Error::Other("gzip spill truncated".to_owned()))?;
    let mut out = [0u8; N];
    out.copy_from_slice(slice);
    *cursor = end;
    Ok(out)
}

fn read_str(buf: &[u8], cursor: &mut usize) -> Result<String> {
    String::from_utf8(read_bytes(buf, cursor)?)
        .map_err(|error| Error::Other(format!("gzip spill utf-8: {error}")))
}

fn read_bytes(buf: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    let len = read_u32(buf, cursor)? as usize;
    let end = cursor
        .checked_add(len)
        .ok_or_else(|| Error::Other("gzip spill cursor overflow".to_owned()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| Error::Other("gzip spill truncated (bytes)".to_owned()))?;
    *cursor = end;
    Ok(slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{GzipCellSpillWriter, GzipCellValue, decode_cell, decode_row, encode_row};

    #[test]
    fn row_protocol_round_trips_nested_values() {
        let values = vec![
            GzipCellValue::Empty,
            GzipCellValue::Text("文本".to_owned()),
            GzipCellValue::Comment {
                value: Box::new(GzipCellValue::Int(7)),
                text: "批注".to_owned(),
            },
            GzipCellValue::Images {
                value: Box::new(GzipCellValue::Bool(true)),
                images: vec![vec![1, 2, 3], vec![4, 5]],
            },
        ];
        let encoded = encode_row(&values).expect("encode row");
        assert_eq!(decode_row(&encoded).expect("decode row"), values);
    }

    #[test]
    fn cell_protocol_rejects_unknown_truncated_and_invalid_utf8_payloads() {
        let mut cursor = 0;
        assert!(decode_cell(&[99], &mut cursor).is_err());

        cursor = 0;
        assert!(decode_cell(&[1, 10, 0, 0, 0], &mut cursor).is_err());

        cursor = 0;
        assert!(decode_cell(&[8, 1, 0, 0, 0, 0xff], &mut cursor).is_err());
    }

    #[test]
    fn row_protocol_rejects_truncated_cell_count() {
        assert!(decode_row(&[1, 0, 0]).is_err());
    }

    #[test]
    fn sheet_spill_owns_name_snapshot_and_writer_to_reader_lifecycle() {
        let directory = tempfile::tempdir().expect("tempdir");
        let mut writer =
            GzipCellSpillWriter::create(directory.path(), "Data", "easyexcel-io-", ".rows.gz")
                .expect("create spill");
        let row = vec![
            GzipCellValue::Text("name".to_owned()),
            GzipCellValue::Int(7),
        ];
        writer.write_row(&row).expect("write row");
        let active = writer.snapshot().expect("active snapshot");
        assert_eq!(active.sheet_name, "Data");
        assert!(active.is_gzip);
        assert!(active.uncompressed_len > 0);

        let mut reader = writer.finish().expect("finish spill");
        let finished = reader.snapshot();
        assert_eq!(finished.sheet_name, "Data");
        assert_eq!(finished.path, active.path);
        assert_eq!(reader.next_row().expect("read row"), Some(row));
        assert_eq!(reader.next_row().expect("read eof"), None);
    }
}
