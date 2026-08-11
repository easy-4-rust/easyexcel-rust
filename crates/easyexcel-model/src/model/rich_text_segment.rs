//! 后端无关的富文本 UTF-16 区间校验与分段。

use super::{Error, Result};

/// 一个经过 UTF-16 边界校验的富文本片段。
///
/// 对应 Java：`RichTextString#applyFont(int, int, Font)` 使用 Java 字符串
/// UTF-16 下标；XLS 的 FONT run 与 XLSX 的 `<r>` 都复用本对象。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTextSegment {
    /// 片段文本。
    pub text: String,
    /// 覆盖该片段的最后一个样式区间下标。
    pub interval_index: Option<usize>,
}

/// 按 UTF-16 区间切分富文本，并拒绝拆开代理对的边界。
///
/// 区间重叠时，后声明的区间覆盖先声明的区间，保持 Java 连续调用
/// `applyFont` 的可观察顺序。
///
/// # Errors
///
/// 区间为空、越界或边界落在 UTF-16 代理对中间时返回模型错误。
pub fn segment_utf16_text(
    text: &str,
    intervals: &[(usize, usize)],
) -> Result<Vec<RichTextSegment>> {
    let utf16_length = text.encode_utf16().count();
    let mut boundaries = vec![0, utf16_length];
    for &(start, end) in intervals {
        if start >= end || end > utf16_length {
            return Err(Error::Other(format!(
                "rich-text font range [{start}, {end}) is outside UTF-16 length {utf16_length}"
            )));
        }
        if utf16_byte_index(text, start).is_none() || utf16_byte_index(text, end).is_none() {
            return Err(Error::Other(format!(
                "rich-text font range [{start}, {end}) splits a UTF-16 surrogate pair"
            )));
        }
        boundaries.push(start);
        boundaries.push(end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    boundaries
        .windows(2)
        .map(|window| {
            let start = window[0];
            let end = window[1];
            let start_byte = utf16_byte_index(text, start).ok_or_else(|| {
                Error::Other("validated rich-text start boundary disappeared".to_owned())
            })?;
            let end_byte = utf16_byte_index(text, end).ok_or_else(|| {
                Error::Other("validated rich-text end boundary disappeared".to_owned())
            })?;
            let interval_index = intervals
                .iter()
                .enumerate()
                .rev()
                .find(|(_, (interval_start, interval_end))| {
                    *interval_start <= start && *interval_end >= end
                })
                .map(|(index, _)| index);
            Ok(RichTextSegment {
                text: text[start_byte..end_byte].to_owned(),
                interval_index,
            })
        })
        .collect()
}

fn utf16_byte_index(text: &str, target: usize) -> Option<usize> {
    let mut utf16_index = 0;
    for (byte_index, character) in text.char_indices() {
        if utf16_index == target {
            return Some(byte_index);
        }
        utf16_index += character.len_utf16();
        if utf16_index > target {
            return None;
        }
    }
    (utf16_index == target).then_some(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_utf16_text_no_intervals() {
        let result = segment_utf16_text("hello", &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "hello");
        assert_eq!(result[0].interval_index, None);
    }

    #[test]
    fn segment_utf16_text_single_interval() {
        let result = segment_utf16_text("hello world", &[(0, 5)]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "hello");
        assert_eq!(result[0].interval_index, Some(0));
        assert_eq!(result[1].text, " world");
        assert_eq!(result[1].interval_index, None);
    }

    #[test]
    fn segment_utf16_text_multiple_intervals() {
        let result = segment_utf16_text("abcdefghij", &[(0, 3), (5, 8)]).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].text, "abc");
        assert_eq!(result[0].interval_index, Some(0));
        assert_eq!(result[1].text, "de");
        assert_eq!(result[1].interval_index, None);
        assert_eq!(result[2].text, "fgh");
        assert_eq!(result[2].interval_index, Some(1));
        assert_eq!(result[3].text, "ij");
        assert_eq!(result[3].interval_index, None);
    }

    #[test]
    fn segment_utf16_text_overlapping_intervals() {
        let result = segment_utf16_text("abcdef", &[(0, 4), (2, 6)]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, "ab");
        assert_eq!(result[0].interval_index, Some(0));
        assert_eq!(result[1].text, "cd");
        // Later interval (1) covers this segment
        assert_eq!(result[1].interval_index, Some(1));
        assert_eq!(result[2].text, "ef");
        assert_eq!(result[2].interval_index, Some(1));
    }

    #[test]
    fn segment_utf16_text_with_multibyte() {
        let result = segment_utf16_text("hello中文world", &[(5, 7)]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, "hello");
        assert_eq!(result[0].interval_index, None);
        assert_eq!(result[1].text, "中文");
        assert_eq!(result[1].interval_index, Some(0));
        assert_eq!(result[2].text, "world");
        assert_eq!(result[2].interval_index, None);
    }

    #[test]
    fn segment_utf16_text_empty_interval_rejected() {
        let result = segment_utf16_text("hello", &[(2, 2)]);
        assert!(result.is_err());
    }

    #[test]
    fn segment_utf16_text_out_of_bounds_rejected() {
        let result = segment_utf16_text("hello", &[(0, 10)]);
        assert!(result.is_err());
    }

    #[test]
    fn segment_utf16_text_surrogate_pair_boundary() {
        // U+1F600 is a surrogate pair in UTF-16 (2 code units)
        let text = "a😀b";
        // Valid: split at surrogate pair boundary (after the emoji)
        let result = segment_utf16_text(text, &[(1, 3)]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, "a");
        assert_eq!(result[1].text, "😀");
        assert_eq!(result[2].text, "b");
    }

    #[test]
    fn segment_utf16_text_split_surrogate_pair_rejected() {
        // U+1F600 is a surrogate pair; splitting in the middle is invalid
        let text = "a😀b";
        let result = segment_utf16_text(text, &[(1, 2)]);
        assert!(result.is_err());
    }

    #[test]
    fn utf16_byte_index_basic() {
        assert_eq!(utf16_byte_index("hello", 0), Some(0));
        assert_eq!(utf16_byte_index("hello", 5), Some(5));
        assert_eq!(utf16_byte_index("hello", 3), Some(3));
    }

    #[test]
    fn utf16_byte_index_out_of_bounds() {
        assert_eq!(utf16_byte_index("hello", 6), None);
    }

    #[test]
    fn utf16_byte_index_with_multibyte() {
        // "中" is 1 UTF-16 code unit, 3 UTF-8 bytes
        assert_eq!(utf16_byte_index("中", 0), Some(0));
        assert_eq!(utf16_byte_index("中", 1), Some(3));
    }

    #[test]
    fn rich_text_segment_equality() {
        let a = RichTextSegment {
            text: "hello".to_owned(),
            interval_index: Some(0),
        };
        let b = RichTextSegment {
            text: "hello".to_owned(),
            interval_index: Some(0),
        };
        assert_eq!(a, b);
    }
}
