//! 对应 Java：`com.alibaba.excel.metadata.data.CommentData`.

use crate::core::client_anchor_data::ClientAnchorData;
use crate::core::rich_text_string_data::RichTextStringData;

/// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Cell comment metadata matching Java `CommentData extends ClientAnchorData`.
///
/// Rust uses composition for the anchor (same pattern as [`crate::ImageData`])
/// so `ClientAnchorData` stays `Copy`/`Default` without inheritance bookkeeping.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CommentData {
    author: Option<String>,
    rich_text_string_data: Option<RichTextStringData>,
    anchor: ClientAnchorData,
}

impl CommentData {
    /// Creates an empty comment. (Java default constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。
    pub const fn new() -> Self {
        Self {
            author: None,
            rich_text_string_data: None,
            anchor: ClientAnchorData::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Sets the original comment author. (Java `setAuthor(String)`)
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Sets the rich-text body. (Java `setRichTextStringData(RichTextStringData)`)
    #[must_use]
    pub fn rich_text_string_data(mut self, value: RichTextStringData) -> Self {
        self.rich_text_string_data = Some(value);
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Sets plain-text body convenience (wraps [`RichTextStringData::new`]).
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.rich_text_string_data = Some(RichTextStringData::new(text));
        self
    }

    /// Sets the client anchor. (Java inherited `ClientAnchorData` fields)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。
    pub const fn anchor(mut self, value: ClientAnchorData) -> Self {
        self.anchor = value;
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Returns the author. (Java `getAuthor()`)
    #[must_use]
    pub fn get_author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Java `setAuthor` 原位 setter。
    pub fn set_author(&mut self, value: Option<String>) { self.author = value; }

    /// Returns the rich-text body. (Java `getRichTextStringData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。
    pub const fn get_rich_text_string_data(&self) -> Option<&RichTextStringData> {
        self.rich_text_string_data.as_ref()
    }

    /// Java `setRichTextStringData` 原位 setter。
    pub fn set_rich_text_string_data(&mut self, value: Option<RichTextStringData>) {
        self.rich_text_string_data = value;
    }

    /// Returns the client anchor.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。
    pub const fn get_anchor(&self) -> ClientAnchorData {
        self.anchor
    }

    /// 设置继承的客户端锚点数据。
    pub const fn set_anchor(&mut self, value: ClientAnchorData) { self.anchor = value; }

    /// 对应 Java：com.alibaba.excel.metadata.data.CommentData。 Returns plain note text for writer backends that only accept a string.
    #[must_use]
    pub fn note_text(&self) -> String {
        self.rich_text_string_data
            .as_ref()
            .map(|r| r.text_string().to_owned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn builders_and_getters_round_trip() {
        // 对应 Java：CommentData 构建与 getter
        let anchor = ClientAnchorData::new();
        let comment = CommentData::new()
            .author("作者")
            .rich_text_string_data(RichTextStringData::new("富文本"))
            .anchor(anchor);

        assert_eq!(comment.get_author(), Some("作者"));
        assert_eq!(
            comment
                .get_rich_text_string_data()
                .map(RichTextStringData::text_string),
            Some("富文本")
        );
        assert_eq!(comment.get_anchor(), anchor);
        assert_eq!(comment.note_text(), "富文本");
    }

    #[test]
    fn text_convenience_and_missing_body() {
        // 对应 Java：text 便捷方法与空备注
        let comment = CommentData::new().text("纯文本");
        assert_eq!(comment.note_text(), "纯文本");

        let empty = CommentData::new();
        assert_eq!(empty.get_author(), None);
        assert!(empty.get_rich_text_string_data().is_none());
        assert_eq!(empty.note_text(), "");
    }
}
