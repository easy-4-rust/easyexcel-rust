//! 对应 Java：`com.alibaba.excel.metadata.data.CellData<T>` plus
//! `CellDataTypeEnum`. In Rust the value and its type are fused into a single
//! `CellValue` enum; `CellDataType` is the dispatch key used by converters.

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};

use crate::core::enum_cell_data_type::CellDataType;
use crate::core::image_data::ImageData;
use crate::core::rich_text_string_data::RichTextStringData;
use crate::core::{CoordinateData, HyperlinkType};
use crate::metadata::data::comment_data::CommentData;

/// 对应 Java：com.alibaba.excel.metadata.data.`CellData<T>`。 A backend-neutral Excel cell value.
///
/// The Java counterpart stores a `CellDataTypeEnum` plus per-type fields in
/// `CellData<T>`. Rust collapses both into a single enum so the variant set
/// is exhaustive and `Option`/`Box` indirection is avoided for the common
/// scalar cases.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// An absent or blank cell.
    Empty,
    /// A string cell.
    String(String),
    /// A Boolean cell.
    Bool(bool),
    /// A signed integer cell.
    Int(i64),
    /// A floating-point cell.
    Float(f64),
    /// An arbitrary-precision decimal cell matching Java `BigDecimal` reads.
    Decimal(BigDecimal),
    /// A date-only cell.
    Date(NaiveDate),
    /// A date and time cell.
    DateTime(NaiveDateTime),
    /// An Excel error value.
    Error(String),
    /// An Excel formula expression without the leading `=` requirement.
    Formula(String),
    /// A clickable hyperlink and its displayed text.
    Hyperlink {
        /// Link target.
        url: String,
        /// Displayed cell text.
        text: String,
    },
    /// Java `HyperlinkData` 的完整超链接类型及覆盖坐标。
    HyperlinkWithMetadata {
        /// 链接目标。
        address: String,
        /// 显示文本。
        text: String,
        /// URL、文档、邮件、文件或 NONE 类型。
        hyperlink_type: HyperlinkType,
        /// 绝对/相对覆盖范围；写入时相对当前单元格求值。
        coordinates: CoordinateData,
    },
    /// A value decorated with an Excel cell note/comment.
    Comment {
        /// Underlying cell value.
        value: Box<CellValue>,
        /// Note text.
        text: String,
    },
    /// A value decorated with complete Java `CommentData` metadata.
    CommentWithMetadata {
        /// Underlying cell value.
        value: Box<CellValue>,
        /// Author, rich-text body and client-anchor metadata.
        comment: CommentData,
    },
    /// Encoded PNG, JPEG, GIF, or BMP image bytes.
    Image(Vec<u8>),
    /// Text with whole-string and interval font metadata.
    RichText(RichTextStringData),
    /// A scalar cell decorated with Java-compatible `WriteCellData.imageDataList` entries.
    Images {
        /// Scalar value written before the drawings are added.
        value: Box<CellValue>,
        /// Images and their worksheet anchor metadata.
        images: Vec<ImageData>,
    },
}

impl CellValue {
    /// Returns whether the cell is empty.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.`CellData<T>`。
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.`CellData<T>`。 Returns a deterministic textual representation used for header matching.
    #[must_use]
    pub fn as_text(&self) -> String {
        match self {
            Self::Empty | Self::Image(_) => String::new(),
            Self::String(value) | Self::Error(value) | Self::Formula(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Decimal(value) => value.to_string(),
            Self::Date(value) => value.format("%Y-%m-%d").to_string(),
            Self::DateTime(value) => value.format("%Y-%m-%d %H:%M:%S").to_string(),
            Self::Hyperlink { text, .. } | Self::HyperlinkWithMetadata { text, .. } => text.clone(),
            Self::RichText(value) => value.text_string().to_owned(),
            Self::Comment { value, .. }
            | Self::CommentWithMetadata { value, .. }
            | Self::Images { value, .. } => value.as_text(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.`CellData<T>`。 Returns the Java EasyExcel-compatible logical cell type used to select converters.
    #[must_use]
    pub fn data_type(&self) -> CellDataType {
        match self {
            Self::Empty => CellDataType::Empty,
            Self::String(_) | Self::Hyperlink { .. } | Self::HyperlinkWithMetadata { .. } => {
                CellDataType::String
            }
            Self::Bool(_) => CellDataType::Boolean,
            Self::Int(_) | Self::Float(_) | Self::Decimal(_) => CellDataType::Number,
            Self::Date(_) | Self::DateTime(_) => CellDataType::Date,
            Self::RichText(_) => CellDataType::RichTextString,
            Self::Error(_) => CellDataType::Error,
            Self::Formula(_) => CellDataType::Formula,
            Self::Comment { value, .. }
            | Self::CommentWithMetadata { value, .. }
            | Self::Images { value, .. } => value.data_type(),
            Self::Image(_) => CellDataType::Image,
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn hyperlink_as_text_returns_display_text() {
        // 对应 Java：Hyperlink 文本化取显示文本
        let link = CellValue::Hyperlink {
            url: "https://x".to_owned(),
            text: "链接".to_owned(),
        };
        assert_eq!(link.as_text(), "链接");
        assert_eq!(link.data_type(), CellDataType::String);
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn rich_text_as_text_returns_text_string() {
        // 对应 Java：RichTextStringData.asText 返回 textString
        let rich = CellValue::RichText(
            crate::core::rich_text_string_data::RichTextStringData::new("富文本"),
        );
        assert_eq!(rich.as_text(), "富文本");
        assert_eq!(rich.data_type(), CellDataType::RichTextString);
    }
}
