/// BIFF8 单元格批注。
///
/// 对应 Java：`org.apache.poi.hssf.usermodel.HSSFComment`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Comment {
    pub(crate) row: u16,
    pub(crate) col: u8,
    pub(crate) text: String,
    pub(crate) author: String,
    pub(crate) first_row: Option<u16>,
    pub(crate) first_col: Option<u8>,
    pub(crate) last_row: Option<u16>,
    pub(crate) last_col: Option<u8>,
    pub(crate) top: Option<u16>,
    pub(crate) right: Option<u16>,
    pub(crate) bottom: Option<u16>,
    pub(crate) left: Option<u16>,
    /// TXO formatting runs，元素为 `(UTF-16 起始下标, FONT 索引)`。
    pub(crate) formatting_runs: Vec<(u16, u16)>,
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
            first_row: None,
            first_col: None,
            last_row: None,
            last_col: None,
            top: None,
            right: None,
            bottom: None,
            left: None,
            formatting_runs: Vec::new(),
        }
    }

    /// 设置 HSSFClientAnchor 的单元格范围及四个偏移量。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn with_anchor(
        mut self,
        first_row: u16,
        first_col: u8,
        last_row: u16,
        last_col: u8,
        top: Option<u16>,
        right: Option<u16>,
        bottom: Option<u16>,
        left: Option<u16>,
    ) -> Self {
        self.first_row = Some(first_row);
        self.first_col = Some(first_col);
        self.last_row = Some(last_row);
        self.last_col = Some(last_col);
        self.top = top;
        self.right = right;
        self.bottom = bottom;
        self.left = left;
        self
    }

    /// 设置批注正文的 BIFF8 TXO 字体区间。
    ///
    /// 对应 Java：`HSSFRichTextString#applyFont(int, int, Font)` 写入批注正文。
    #[must_use]
    pub fn with_formatting_runs(mut self, formatting_runs: Vec<(u16, u16)>) -> Self {
        self.formatting_runs = formatting_runs;
        self
    }
}
