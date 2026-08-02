//! BIFF8 segmented Unicode and shared-string decoding.

use easyexcel_core::{ExcelError, Result};

/// Decodes an SST record body plus its physical CONTINUE record bodies.
pub(crate) fn decode_sst_segments(segments: &[Vec<u8>]) -> Result<Vec<String>> {
    let mut cursor = SegmentCursor::new(segments);
    let _total = cursor.read_u32("SST total string count")?;
    let unique = usize::try_from(cursor.read_u32("SST unique string count")?)
        .map_err(|_| ExcelError::Format("SST unique string count exceeds usize".to_owned()))?;
    let available = segments.iter().map(Vec::len).sum::<usize>();
    if unique > available.saturating_div(3).saturating_add(1) {
        return Err(ExcelError::Format(format!(
            "SST declares {unique} unique strings in only {available} bytes"
        )));
    }

    let mut strings = Vec::with_capacity(unique.min(16_384));
    for index in 0..unique {
        strings.push(cursor.read_rich_extended_string(index)?);
    }
    Ok(strings)
}

/// Decodes a BIFF8 `XLUnicodeString` split over CONTINUE records.
pub(crate) fn decode_unicode_string_segments(segments: &[Vec<u8>]) -> Result<String> {
    let mut cursor = SegmentCursor::new(segments);
    let character_count = cursor.read_u16("String character count")? as usize;
    let flags = cursor.read_u8_plain("String flags")?;
    cursor.read_characters(character_count, flags & 0x01 != 0, "String")
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

    fn read_rich_extended_string(&mut self, index: usize) -> Result<String> {
        let context = format!("SST string {index}");
        let character_count = self.read_u16(&format!("{context} character count"))? as usize;
        let flags = self.read_u8_plain(&format!("{context} flags"))?;
        let rich_run_count = if flags & 0x08 != 0 {
            self.read_u16(&format!("{context} rich-run count"))? as usize
        } else {
            0
        };
        let extension_size = if flags & 0x04 != 0 {
            usize::try_from(self.read_u32(&format!("{context} extension size"))?).map_err(|_| {
                ExcelError::Format(format!("{context} extension size exceeds usize"))
            })?
        } else {
            0
        };

        let value = self.read_characters(character_count, flags & 0x01 != 0, context.as_str())?;
        self.skip_plain(
            rich_run_count.checked_mul(4).ok_or_else(|| {
                ExcelError::Format(format!("{context} rich-run byte count overflow"))
            })?,
            &format!("{context} rich runs"),
        )?;
        self.skip_plain(extension_size, &format!("{context} extension"))?;
        Ok(value)
    }

    fn read_characters(
        &mut self,
        character_count: usize,
        mut wide: bool,
        context: &str,
    ) -> Result<String> {
        let mut units = Vec::with_capacity(character_count.min(16_384));
        for _ in 0..character_count {
            if self.current_exhausted() {
                self.advance_segment().ok_or_else(|| {
                    ExcelError::Format(format!(
                        "truncated {context} character data across BIFF CONTINUE records"
                    ))
                })?;
                let continuation_flags =
                    self.read_u8_current(&format!("{context} continuation flags"))?;
                wide = continuation_flags & 0x01 != 0;
            }

            if wide {
                let segment = self.current_segment().ok_or_else(|| {
                    ExcelError::Format(format!("truncated {context} UTF-16 character data"))
                })?;
                if self.offset + 2 > segment.len() {
                    return Err(ExcelError::Format(format!(
                        "{context} UTF-16 code unit is split at a BIFF record boundary"
                    )));
                }
                units.push(u16::from_le_bytes([
                    segment[self.offset],
                    segment[self.offset + 1],
                ]));
                self.offset += 2;
            } else {
                units.push(u16::from(
                    self.read_u8_current(&format!("{context} compressed character"))?,
                ));
            }
        }
        Ok(String::from_utf16_lossy(&units))
    }

    fn read_u8_plain(&mut self, context: &str) -> Result<u8> {
        if self.current_exhausted() {
            self.advance_segment().ok_or_else(|| {
                ExcelError::Format(format!("truncated {context} across BIFF records"))
            })?;
        }
        self.read_u8_current(context)
    }

    fn read_u8_current(&mut self, context: &str) -> Result<u8> {
        let value = self
            .current_segment()
            .and_then(|segment| segment.get(self.offset))
            .copied()
            .ok_or_else(|| ExcelError::Format(format!("truncated {context}")))?;
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

        assert_eq!(decode_sst_segments(&[body])?, vec!["one", "你好"]);
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

        assert_eq!(decode_sst_segments(&[first, second])?, vec!["ab你好"]);
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

        assert_eq!(decode_sst_segments(&[first, second])?, vec!["x"]);
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
}
