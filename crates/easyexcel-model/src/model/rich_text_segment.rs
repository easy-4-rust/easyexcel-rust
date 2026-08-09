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
