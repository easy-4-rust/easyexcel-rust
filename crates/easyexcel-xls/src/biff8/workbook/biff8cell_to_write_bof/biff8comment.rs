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
    /// NOTE record 的可见标志；默认隐藏，与 Excel/POI 新建批注一致。
    pub(crate) visible: bool,
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
            visible: false,
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

    /// 设置 NOTE record 的初始可见性。
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

#[cfg(test)]
mod biff8comment_tests {
    use super::*;

    /// 验证 `new` 的默认值：无锚点、无格式区间、不可见。
    #[test]
    fn new_sets_defaults() {
        let c = Biff8Comment::new(1, 2, "text", "author");
        assert_eq!(c.row, 1);
        assert_eq!(c.col, 2);
        assert_eq!(c.text, "text");
        assert_eq!(c.author, "author");
        assert!(c.first_row.is_none());
        assert!(c.first_col.is_none());
        assert!(c.last_row.is_none());
        assert!(c.last_col.is_none());
        assert!(c.top.is_none());
        assert!(c.right.is_none());
        assert!(c.bottom.is_none());
        assert!(c.left.is_none());
        assert!(c.formatting_runs.is_empty());
        assert!(!c.visible);
    }

    /// 验证 `new` 接受 &str 和 String。
    #[test]
    fn new_accepts_string_and_str() {
        let c1 = Biff8Comment::new(0, 0, "a", "b");
        let c2 = Biff8Comment::new(0, 0, String::from("a"), String::from("b"));
        assert_eq!(c1, c2);
    }

    /// 验证 `with_anchor` 设置所有锚点字段。
    #[test]
    fn with_anchor_sets_all_fields() {
        let c = Biff8Comment::new(0, 0, "t", "a").with_anchor(1, 2, 3, 4, Some(10), Some(20), Some(30), Some(40));
        assert_eq!(c.first_row, Some(1));
        assert_eq!(c.first_col, Some(2));
        assert_eq!(c.last_row, Some(3));
        assert_eq!(c.last_col, Some(4));
        assert_eq!(c.top, Some(10));
        assert_eq!(c.right, Some(20));
        assert_eq!(c.bottom, Some(30));
        assert_eq!(c.left, Some(40));
    }

    /// 验证 `with_anchor` 偏移量可以为 None。
    #[test]
    fn with_anchor_none_offsets() {
        let c = Biff8Comment::new(0, 0, "t", "a").with_anchor(1, 2, 3, 4, None, None, None, None);
        assert!(c.top.is_none());
        assert!(c.right.is_none());
        assert!(c.bottom.is_none());
        assert!(c.left.is_none());
    }

    /// 验证 `with_formatting_runs` 设置格式区间。
    #[test]
    fn with_formatting_runs_sets_runs() {
        let runs = vec![(0u16, 1u16), (5, 2)];
        let c = Biff8Comment::new(0, 0, "t", "a").with_formatting_runs(runs.clone());
        assert_eq!(c.formatting_runs, runs);
    }

    /// 验证 `with_visible` 设为 true。
    #[test]
    fn with_visible_true() {
        let c = Biff8Comment::new(0, 0, "t", "a").with_visible(true);
        assert!(c.visible);
    }

    /// 验证 `with_visible` 设为 false。
    #[test]
    fn with_visible_false() {
        let c = Biff8Comment::new(0, 0, "t", "a").with_visible(true).with_visible(false);
        assert!(!c.visible);
    }

    /// 验证链式调用：anchor → formatting_runs → visible。
    #[test]
    fn builder_chain() {
        let c = Biff8Comment::new(5, 10, "comment", "me")
            .with_anchor(0, 0, 2, 2, Some(1), Some(2), Some(3), Some(4))
            .with_formatting_runs(vec![(0, 1)])
            .with_visible(true);
        assert_eq!(c.row, 5);
        assert_eq!(c.col, 10);
        assert_eq!(c.first_row, Some(0));
        assert_eq!(c.formatting_runs.len(), 1);
        assert!(c.visible);
    }

    /// 验证 Clone 和 PartialEq。
    #[test]
    fn clone_and_eq() {
        let c1 = Biff8Comment::new(1, 2, "t", "a").with_visible(true);
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }

    /// 验证 Debug 输出不 panic。
    #[test]
    fn debug_does_not_panic() {
        let c = Biff8Comment::new(0, 0, "t", "a");
        let _ = format!("{c:?}");
    }
}
