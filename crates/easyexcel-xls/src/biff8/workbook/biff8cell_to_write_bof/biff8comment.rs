/// BIFF8 单元格批注。
///
/// 对应 Java：`org.apache.poi.hssf.usermodel.HSSFComment`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Comment {
    pub(crate) row: u16,
    pub(crate) col: u8,
    pub(crate) text: String,
    pub(crate) author: String,
}

impl Biff8Comment {
    /// 创建单元格批注。对应 Java：`HSSFPatriarch#createCellComment`。
    #[must_use]
    pub fn new(row: u16, col: u8, text: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            row,
            col,
            text: text.into(),
            author: author.into(),
        }
    }
}
