//! XLSX 富文本 UTF-16 区间校验与分段。

use easyexcel_io::{Error, Result};

pub use easyexcel_model::RichTextSegment;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按 UTF-16 区间切分富文本，并拒绝拆开代理对的边界。
///
/// 区间使用 Java 字符串一致的 UTF-16 单元下标；区间重叠时，后声明的
/// 区间覆盖先声明的区间。
///
/// # Errors
///
/// 区间为空、越界或边界落在 UTF-16 代理对中间时返回 XLSX 格式错误。
pub fn segment_utf16_text(
    text: &str,
    intervals: &[(usize, usize)],
) -> Result<Vec<RichTextSegment>> {
    easyexcel_model::segment_utf16_text(text, intervals)
        .map_err(|error| Error::Xlsx(error.to_string()))
}

#[cfg(test)]
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
    fn utf16_boundaries_reject_surrogate_pair_splits() {
        assert_eq!(utf16_byte_index("A😀BC", 0), Some(0));
        assert_eq!(utf16_byte_index("A😀BC", 1), Some(1));
        assert_eq!(utf16_byte_index("A😀BC", 2), None);
        assert_eq!(utf16_byte_index("A😀BC", 5), Some("A😀BC".len()));
        assert_eq!(utf16_byte_index("A😀BC", 6), None);

        let segments = segment_utf16_text("A😀BC", &[(1, 3)]).expect("valid UTF-16 range");
        assert_eq!(
            segments
                .iter()
                .map(|segment| segment.text.as_str())
                .collect::<Vec<_>>(),
            ["A", "😀", "BC"]
        );
        assert!(segment_utf16_text("😀", &[(0, 1)]).is_err());
    }
}
