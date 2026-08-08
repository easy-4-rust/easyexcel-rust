/// BIFF8 SST 富文本值。
///
/// 对应 Java：`org.apache.poi.hssf.usermodel.HSSFRichTextString`。
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Biff8RichText {
    pub(crate) text: String,
    pub(crate) runs: Vec<(u16, u16)>,
}

impl Biff8RichText {
    /// 创建富文本及其 `(UTF-16 起点, FONT 索引)` runs。
    /// 对应 Java：`HSSFRichTextString#applyFont`。
    #[must_use]
    pub fn new(text: impl Into<String>, runs: Vec<(u16, u16)>) -> Self {
        Self {
            text: text.into(),
            runs,
        }
    }

    pub(crate) fn plain(text: impl Into<String>) -> Self {
        Self::new(text, Vec::new())
    }
}
