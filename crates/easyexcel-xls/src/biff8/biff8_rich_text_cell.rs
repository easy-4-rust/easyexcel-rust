//! BIFF8 富文本单元格的中立表示。

use super::Biff8Font;

/// 从 SST、LABELSST 和 FONT 记录关联得到的富文本单元格。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8RichTextCell {
    text: String,
    runs: Vec<(usize, usize, Biff8Font)>,
}

impl Biff8RichTextCell {
    /// 创建一个包含 UTF-16 半开区间字体片段的富文本单元格。
    #[must_use]
    pub const fn new(text: String, runs: Vec<(usize, usize, Biff8Font)>) -> Self {
        Self { text, runs }
    }

    /// 返回原始文本。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 返回 `(起点, 终点, 字体)`；位置按 Excel/Java 的 UTF-16 索引解释。
    #[must_use]
    pub fn runs(&self) -> &[(usize, usize, Biff8Font)] {
        &self.runs
    }
}
