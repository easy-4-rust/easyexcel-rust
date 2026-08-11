//! BIFF8 segmented Unicode and shared-string decoding.

use easyexcel_io::{Error as ExcelError, Result};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decodes an SST record body plus its physical CONTINUE record bodies.
///
/// # Errors
///
/// SST 元数据或分段字符串损坏时返回错误。
pub fn decode_sst_segments(segments: &[Vec<u8>]) -> Result<Vec<crate::xls::Biff8SstString>> {
    let mut cursor = SegmentCursor::new(segments);
    let _total = cursor.read_u32("SST total string count")?;
    let unique = usize::try_from(cursor.read_u32("SST unique string count")?)
        .map_err(|_| ExcelError::Xls("SST unique string count exceeds usize".to_owned()))?;
    let available = segments.iter().map(Vec::len).sum::<usize>();
    if unique > available.saturating_div(3).saturating_add(1) {
        return Err(ExcelError::Xls(format!(
            "SST declares {unique} unique strings in only {available} bytes"
        )));
    }

    let mut strings = Vec::with_capacity(unique.min(16_384));
    for index in 0..unique {
        strings.push(cursor.read_rich_extended_string(index)?);
    }
    Ok(strings)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decodes a BIFF8 `XLUnicodeString` split over CONTINUE records.
///
/// # Errors
///
/// 字符计数、标志或跨 CONTINUE 片段的数据不完整时返回错误。
pub fn decode_unicode_string_segments(segments: &[Vec<u8>]) -> Result<String> {
    let mut cursor = SegmentCursor::new(segments);
    let character_count = cursor.read_u16("String character count")? as usize;
    let flags = cursor.read_u8_plain("String flags")?;
    // 使用 usize::MAX 作为哨兵索引，错误消息中标识为 Unicode STRING 记录
    cursor.read_characters(character_count, flags & 0x01 != 0, usize::MAX)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码未分段的 BIFF8 `XLUnicodeString` 记录体。
///
/// # Errors
///
/// 字符计数、编码标志或字符数据损坏/截断时返回错误。
pub fn decode_unicode_string_record(data: &[u8]) -> Result<String> {
    decode_unicode_string_segments(&[data.to_vec()])
}

struct SegmentCursor<'a> {
    segments: &'a [Vec<u8>],
    segment_index: usize,
    offset: usize,
}

impl<'a> SegmentCursor<'a> {
    const fn new(segments: &'a [Vec<u8>]) -> Self {
        Self {
            segments,
            segment_index: 0,
            offset: 0,
        }
    }

    fn read_rich_extended_string(&mut self, index: usize) -> Result<crate::xls::Biff8SstString> {
        let character_count = self.read_u16_ctx(index, "character count")? as usize;
        let flags = self.read_u8_ctx(index, "flags")?;
        let rich_run_count = if flags & 0x08 != 0 {
            self.read_u16_ctx(index, "rich-run count")? as usize
        } else {
            0
        };
        let extension_size = if flags & 0x04 != 0 {
            let raw = self.read_u32_ctx(index, "extension size")?;
            usize::try_from(raw).map_err(|_| {
                ExcelError::Xls(format!("SST string {index} extension size exceeds usize"))
            })?
        } else {
            0
        };

        let value = self.read_characters(character_count, flags & 0x01 != 0, index)?;
        let mut formatting_runs = Vec::with_capacity(rich_run_count);
        for _ in 0..rich_run_count {
            formatting_runs.push((
                self.read_u16_ctx(index, "rich-run character index")?,
                self.read_u16_ctx(index, "rich-run font index")?,
            ));
        }
        self.skip_plain_ctx(extension_size, index, "extension")?;
        Ok(crate::xls::Biff8SstString::new(value, formatting_runs))
    }

    fn read_characters(
        &mut self,
        character_count: usize,
        mut wide: bool,
        index: usize,
    ) -> Result<String> {
        let label = if index == usize::MAX {
            "Unicode STRING".to_owned()
        } else {
            format!("SST string {index}")
        };
        let mut units = Vec::with_capacity(character_count.min(16_384));
        for _ in 0..character_count {
            if self.current_exhausted() {
                self.advance_segment().ok_or_else(|| {
                    ExcelError::Xls(format!(
                        "truncated {label} character data across BIFF CONTINUE records"
                    ))
                })?;
                let continuation_flags =
                    self.read_u8_current(&format!("{label} continuation flags"))?;
                wide = continuation_flags & 0x01 != 0;
            }

            if wide {
                let segment = self.current_segment().ok_or_else(|| {
                    ExcelError::Xls(format!("truncated {label} UTF-16 character data"))
                })?;
                if self.offset + 2 > segment.len() {
                    return Err(ExcelError::Xls(format!(
                        "{label} UTF-16 code unit split at BIFF record boundary"
                    )));
                }
                units.push(u16::from_le_bytes([
                    segment[self.offset],
                    segment[self.offset + 1],
                ]));
                self.offset += 2;
            } else {
                units.push(u16::from(
                    self.read_u8_current(&format!("{label} compressed character"))?,
                ));
            }
        }
        Ok(String::from_utf16_lossy(&units))
    }

    fn read_u8_plain(&mut self, context: &str) -> Result<u8> {
        if self.current_exhausted() {
            self.advance_segment().ok_or_else(|| {
                ExcelError::Xls(format!("truncated {context} across BIFF records"))
            })?;
        }
        self.read_u8_current(context)
    }

    fn read_u8_current(&mut self, context: &str) -> Result<u8> {
        let value = self
            .current_segment()
            .and_then(|segment| segment.get(self.offset))
            .copied()
            .ok_or_else(|| ExcelError::Xls(format!("truncated {context}")))?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self, context: &str) -> Result<u16> {
        let bytes = self.read_plain::<2>(context)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, context: &str) -> Result<u32> {
        let bytes = self.read_plain::<4>(context)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_plain<const N: usize>(&mut self, context: &str) -> Result<[u8; N]> {
        let mut bytes = [0; N];
        for byte in &mut bytes {
            *byte = self.read_u8_plain(context)?;
        }
        Ok(bytes)
    }

    fn skip_plain(&mut self, count: usize, context: &str) -> Result<()> {
        for _ in 0..count {
            let _ = self.read_u8_plain(context)?;
        }
        Ok(())
    }

    // -- 延迟格式化版本：仅在错误时分配 String --

    fn read_u8_current_lazy(&mut self, index: usize, field: &str) -> Result<u8> {
        let value = self
            .current_segment()
            .and_then(|segment| segment.get(self.offset))
            .copied()
            .ok_or_else(|| {
                ExcelError::Xls(format!("SST string {index} truncated {field}"))
            })?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u8_ctx(&mut self, index: usize, field: &str) -> Result<u8> {
        if self.current_exhausted() {
            self.advance_segment().ok_or_else(|| {
                ExcelError::Xls(format!(
                    "SST string {index} {field} truncated across BIFF records"
                ))
            })?;
        }
        self.read_u8_current_lazy(index, field)
    }

    fn read_u16_ctx(&mut self, index: usize, field: &str) -> Result<u16> {
        let mut bytes = [0u8; 2];
        for byte in &mut bytes {
            *byte = self.read_u8_ctx(index, field)?;
        }
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32_ctx(&mut self, index: usize, field: &str) -> Result<u32> {
        let mut bytes = [0u8; 4];
        for byte in &mut bytes {
            *byte = self.read_u8_ctx(index, field)?;
        }
        Ok(u32::from_le_bytes(bytes))
    }

    fn skip_plain_ctx(&mut self, count: usize, index: usize, field: &str) -> Result<()> {
        for _ in 0..count {
            let _ = self.read_u8_ctx(index, field)?;
        }
        Ok(())
    }

    fn current_segment(&self) -> Option<&[u8]> {
        self.segments.get(self.segment_index).map(Vec::as_slice)
    }

    fn current_exhausted(&self) -> bool {
        self.current_segment()
            .is_none_or(|segment| self.offset >= segment.len())
    }

    fn advance_segment(&mut self) -> Option<()> {
        if self.segment_index + 1 >= self.segments.len() {
            return None;
        }
        self.segment_index += 1;
        self.offset = 0;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_compressed_and_utf16_sst_strings() -> Result<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&2u32.to_le_bytes());
        body.extend_from_slice(&2u32.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.push(0);
        body.extend_from_slice(b"one");
        body.extend_from_slice(&2u16.to_le_bytes());
        body.push(1);
        body.extend_from_slice(&[0x60, 0x4F, 0x7D, 0x59]);

        assert_eq!(
            decode_sst_segments(&[body])?,
            vec![
                crate::xls::Biff8SstString::new("one".to_owned(), Vec::new()),
                crate::xls::Biff8SstString::new("你好".to_owned(), Vec::new()),
            ]
        );
        Ok(())
    }

    #[test]
    fn continuation_can_switch_character_width_mid_string() -> Result<()> {
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&4u16.to_le_bytes());
        first.push(0);
        first.extend_from_slice(b"ab");
        let second = vec![1, 0x60, 0x4F, 0x7D, 0x59];

        assert_eq!(
            decode_sst_segments(&[first, second])?,
            vec![crate::xls::Biff8SstString::new(
                "ab你好".to_owned(),
                Vec::new()
            )]
        );
        Ok(())
    }

    #[test]
    fn rich_runs_and_extensions_may_cross_record_boundaries() -> Result<()> {
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u16.to_le_bytes());
        first.push(0x0C);
        first.extend_from_slice(&1u16.to_le_bytes());
        first.extend_from_slice(&2u32.to_le_bytes());
        first.push(b'x');
        first.extend_from_slice(&[0, 0]);
        let second = vec![1, 0, 0xAA, 0xBB];

        assert_eq!(
            decode_sst_segments(&[first, second])?,
            vec![crate::xls::Biff8SstString::new(
                "x".to_owned(),
                vec![(0, 1)]
            )]
        );
        Ok(())
    }

    #[test]
    fn rejects_truncated_continued_characters() {
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&2u16.to_le_bytes());
        first.push(0);
        first.push(b'a');
        assert!(decode_sst_segments(&[first]).is_err());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn rejects_implausible_unique_string_counts() {
        // 对应 Java：SST 声明的 unique 数量与可用字节不匹配时报错
        let mut body = Vec::new();
        body.extend_from_slice(&100u32.to_le_bytes());
        body.extend_from_slice(&100u32.to_le_bytes());
        body.extend_from_slice(&2u16.to_le_bytes());
        body.push(0);
        body.extend_from_slice(b"ab");
        assert!(decode_sst_segments(&[body]).is_err());
    }

    #[test]
    fn rejects_truncated_rich_runs() {
        // 对应 Java：rich-run 声明但缺少 run 数据时报错
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.push(0x08); // 带 rich-run
        body.extend_from_slice(&1u16.to_le_bytes()); // rich-run count = 1
        body.push(b'a'); // 字符本体
        // 缺少 4 字节 rich-run → skip 越界报错
        assert!(decode_sst_segments(&[body]).is_err());
    }

    #[test]
    fn rejects_utf16_unit_split_across_segments() {
        // 对应 Java：UTF-16 码元被 BIFF 记录边界切开时报错
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&2u16.to_le_bytes());
        first.push(1); // wide
        first.push(0x41); // 只有 1 字节，缺少另一半码元
        assert!(decode_sst_segments(&[first]).is_err());
    }

    #[test]
    fn rejects_headers_truncated_across_records() {
        // 对应 Java：字符串头（字符数/标志位）跨越记录边界时报错
        assert!(decode_unicode_string_segments(&[vec![3, 0]]).is_err());
        // 声明字符数超过数据量时报错
        assert!(decode_unicode_string_segments(&[vec![5, 0, 0, b'a']]).is_err());
    }

    #[test]
    fn decodes_wide_and_compressed_single_segment_strings() -> Result<()> {
        // 对应 Java：BIFF8 XLUnicodeString 宽/窄两种编码
        assert_eq!(
            decode_unicode_string_segments(&[vec![2, 0, 1, 0x60, 0x4F, 0x7D, 0x59]])?,
            "你好"
        );
        assert_eq!(
            decode_unicode_string_segments(&[vec![3, 0, 0, b'a', b'b', b'c']])?,
            "abc"
        );
        Ok(())
    }

    #[test]
    fn decode_unicode_string_record_single_segment() -> Result<()> {
        assert_eq!(
            decode_unicode_string_record(&[2, 0, 0, b'h', b'i'])?,
            "hi"
        );
        Ok(())
    }

    #[test]
    fn decode_sst_empty_segments() {
        // Empty segments vec should error (can't read total count)
        let result = decode_sst_segments(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn decode_sst_single_byte_segments_errors() {
        // Only 1 byte - can't read u32 header
        let result = decode_sst_segments(&[vec![0]]);
        assert!(result.is_err());
    }

    #[test]
    fn decode_sst_wide_string_with_continuation() -> Result<()> {
        // First segment: SST header + start of wide string
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes()); // total
        first.extend_from_slice(&1u32.to_le_bytes()); // unique
        first.extend_from_slice(&3u16.to_le_bytes()); // char count = 3
        first.push(0x01); // wide flag
        first.extend_from_slice(&[0x60, 0x4F]); // 你 (first char)
        // Second segment: continuation with compressed flag
        let second = vec![0x00, 0x7D, 0x59]; // compressed flag + rest as wide bytes
        let result = decode_sst_segments(&[first, second])?;
        assert_eq!(result.len(), 1);
        Ok(())
    }
}
