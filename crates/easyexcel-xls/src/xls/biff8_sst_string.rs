//! BIFF8 SST 富文本解码结果。

/// 一个共享字符串及其 `(UTF-16 起点, FONT 索引)` 格式 run。
///
/// 对应 Java：`org.apache.poi.hssf.usermodel.HSSFRichTextString`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8SstString {
    /// 解码后的 Unicode 文本。
    pub text: String,
    /// BIFF8 格式 run；字符位置按 UTF-16 code unit 计数。
    pub formatting_runs: Vec<(u16, u16)>,
}

impl Biff8SstString {
    /// 创建一个 SST 字符串值。
    #[must_use]
    pub const fn new(text: String, formatting_runs: Vec<(u16, u16)>) -> Self {
        Self {
            text,
            formatting_runs,
        }
    }
}
