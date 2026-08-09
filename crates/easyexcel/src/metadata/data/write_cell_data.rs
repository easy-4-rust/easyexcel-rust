//! 对应 Java：`com.alibaba.excel.metadata.data.WriteCellData`.

use crate::WriteCellStyle;
use crate::core::cell_value::CellValue;
use crate::core::comment_data::CommentData;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_error::ExcelError;
use crate::core::formula_data::FormulaData;
use crate::core::from_excel_cell::FromExcelCell;
use crate::core::hyperlink_data::HyperlinkData;
use crate::core::image_data::ImageData;
use crate::core::into_excel_cell::IntoExcelCell;
use crate::core::rich_text_string_data::RichTextStringData;
use crate::metadata::data::DataFormatData;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};

/// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Java `WriteCellData` subset that preserves a scalar plus decorations.
///
/// Java `WriteCellData` extends `CellData` and adds image / comment / hyperlink
/// / formula fields. Rust keeps the same public surface on the hot path.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteCellData {
    value: CellValue,
    /// Java `CellData.type` 的显式覆盖；允许“有类型但尚无值”。
    declared_type: Option<crate::CellDataType>,
    image_data_list: Vec<ImageData>,
    comment_data: Option<CommentData>,
    hyperlink_data: Option<HyperlinkData>,
    formula_data: Option<FormulaData>,
    write_cell_style: Option<WriteCellStyle>,
    origin_cell_style: Option<WriteCellStyle>,
    data_format_data: Option<DataFormatData>,
}

impl WriteCellData {
    /// Java 无参构造器，对应未指定类型的空数据对象。
    #[must_use] pub const fn empty() -> Self { Self::new(CellValue::Empty) }
    /// Java `WriteCellData(CellDataTypeEnum)` 的后端中立构造。
    #[must_use]
    pub fn from_type(cell_type: crate::CellDataType) -> Self {
        let mut data = Self::empty();
        data.declared_type = Some(cell_type);
        data
    }
    /// Java `WriteCellData(CellDataTypeEnum, String)`，仅允许 STRING/ERROR。
    pub fn from_typed_string(cell_type: crate::CellDataType, value: impl Into<String>) -> Result<Self, ExcelError> {
        let value = value.into();
        match cell_type {
            crate::CellDataType::String | crate::CellDataType::DirectString => {
                let mut data = Self::new(CellValue::String(value));
                data.declared_type = Some(cell_type);
                Ok(data)
            }
            crate::CellDataType::Error => {
                let mut data = Self::new(CellValue::Error(value));
                data.declared_type = Some(cell_type);
                Ok(data)
            }
            _ => Err(ExcelError::Format("Only STRING, DIRECT_STRING and ERROR accept a string value".to_owned())),
        }
    }
    /// Java `WriteCellData(BigDecimal)`。
    #[must_use] pub const fn from_number(value: BigDecimal) -> Self { Self::new(CellValue::Decimal(value)) }
    /// Java `WriteCellData(Boolean)`。
    #[must_use] pub const fn from_boolean(value: bool) -> Self { Self::new(CellValue::Bool(value)) }
    /// Java `WriteCellData(LocalDateTime)`。
    #[must_use] pub const fn from_date_time(value: NaiveDateTime) -> Self { Self::new(CellValue::DateTime(value)) }
    /// Rust 日期无时分秒构造。
    #[must_use] pub const fn from_date(value: NaiveDate) -> Self { Self::new(CellValue::Date(value)) }
    /// 返回该单元格是否仅包含标量值、没有样式或附加对象。
    #[must_use]
    pub fn is_plain(&self) -> bool {
        self.image_data_list.is_empty()
            && self.comment_data.is_none()
            && self.hyperlink_data.is_none()
            && self.formula_data.is_none()
            && self.write_cell_style.is_none()
            && self.origin_cell_style.is_none()
            && self.data_format_data.is_none()
    }

    /// Creates decorated cell data from a scalar value. (Java `WriteCellData(WriteCellData)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn new(value: CellValue) -> Self {
        Self {
            value,
            declared_type: None,
            image_data_list: Vec::new(),
            comment_data: None,
            hyperlink_data: None,
            formula_data: None,
            write_cell_style: None,
            origin_cell_style: None,
            data_format_data: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Creates a string cell. (Java `WriteCellData(String)`)
    #[must_use]
    pub fn from_string(value: impl Into<String>) -> Self {
        Self::new(CellValue::String(value.into()))
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Creates an empty scalar cell with one image, matching Java's byte-array constructor.
    #[must_use]
    pub fn from_image(image: impl Into<Vec<u8>>) -> Self {
        Self::new(CellValue::Empty).image(ImageData::new(image))
    }

    /// Creates a rich-text cell, matching Java's `RICH_TEXT_STRING` cell data type.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn from_rich_text(value: RichTextStringData) -> Self {
        Self::new(CellValue::RichText(value))
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Creates a hyperlink cell with optional display text. (Java `WriteCellData.setHyperlinkData(...)`)
    #[must_use]
    pub fn from_hyperlink(url: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(CellValue::Hyperlink {
            url: url.into(),
            text: text.into(),
        })
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Creates a formula cell. (Java `WriteCellData.setFormulaData(...)`)
    #[must_use]
    pub fn from_formula(formula: impl Into<String>) -> Self {
        Self::new(CellValue::Formula(formula.into()))
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Creates a comment-decorated cell. (Java `WriteCellData.setCommentData(...)`)
    #[must_use]
    pub fn from_comment(value: impl Into<CellValue>, text: impl Into<String>) -> Self {
        Self::new(CellValue::Comment {
            value: Box::new(value.into()),
            text: text.into(),
        })
    }

    /// Replaces the underlying scalar value while keeping decorations intact.
    ///
    /// 对应 Java：'s `WriteCellData.setValue(...)` setter used by the writer
    /// when an annotation override (formula / hyperlink) needs to wrap the
    /// typed scalar without reallocating the cell structure.
    pub fn set_value(&mut self, value: impl Into<CellValue>) -> &mut Self {
        self.value = value.into();
        self.declared_type = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Appends one image entry. (Java `setImageDataList(List<ImageData>)` step)
    #[must_use]
    pub fn image(mut self, value: ImageData) -> Self {
        self.image_data_list.push(value);
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Replaces the full image list.
    #[must_use]
    pub fn image_data_list(mut self, value: impl IntoIterator<Item = ImageData>) -> Self {
        self.image_data_list = value.into_iter().collect();
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Sets comment metadata. (Java `setCommentData(CommentData)`)
    #[must_use]
    pub fn comment_data(mut self, value: CommentData) -> Self {
        self.comment_data = Some(value);
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Sets hyperlink metadata. (Java `setHyperlinkData(HyperlinkData)`)
    #[must_use]
    pub fn hyperlink_data(mut self, value: HyperlinkData) -> Self {
        self.hyperlink_data = Some(value);
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Sets formula metadata. (Java `setFormulaData(FormulaData)`)
    #[must_use]
    pub fn formula_data(mut self, value: FormulaData) -> Self {
        self.formula_data = Some(value);
        self
    }

    /// Returns the scalar cell value. (Java `getValue()` via `CellData.getData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn value(&self) -> &CellValue {
        &self.value
    }
    /// Java 父类数据值的后端中立 getter。
    #[must_use] pub const fn get_value(&self) -> &CellValue { self.value() }
    /// Java `getType`。
    #[must_use] pub fn get_type(&self) -> crate::CellDataType { self.declared_type.unwrap_or_else(|| self.value.data_type()) }
    /// Java `setType`。
    pub const fn set_type(&mut self, value: crate::CellDataType) { self.declared_type = Some(value); }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Returns all image entries in insertion order. (Java `getImageDataList()`)
    #[must_use]
    pub fn images(&self) -> &[ImageData] {
        &self.image_data_list
    }

    /// Returns comment metadata. (Java `getCommentData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn get_comment_data(&self) -> Option<&CommentData> {
        self.comment_data.as_ref()
    }
    /// Java `setCommentData` 原位 setter。
    pub fn set_comment_data(&mut self, value: Option<CommentData>) { self.comment_data = value; }

    /// Returns hyperlink metadata. (Java `getHyperlinkData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn get_hyperlink_data(&self) -> Option<&HyperlinkData> {
        self.hyperlink_data.as_ref()
    }
    /// Java `setHyperlinkData` 原位 setter。
    pub fn set_hyperlink_data(&mut self, value: Option<HyperlinkData>) { self.hyperlink_data = value; }

    /// Returns formula metadata. (Java `getFormulaData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn get_formula_data(&self) -> Option<&FormulaData> {
        self.formula_data.as_ref()
    }
    /// Java `setFormulaData` 原位 setter。
    pub fn set_formula_data(&mut self, value: Option<FormulaData>) { self.formula_data = value; }

    /// Java `getImageDataList` 别名。
    #[must_use]
    pub fn get_image_data_list(&self) -> &[ImageData] { &self.image_data_list }
    /// Java `setImageDataList` 原位 setter。
    pub fn set_image_data_list(&mut self, value: Vec<ImageData>) { self.image_data_list = value; }

    /// Returns the logical cell style. (Java `getWriteCellStyle()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn write_cell_style(&self) -> Option<&WriteCellStyle> {
        self.write_cell_style.as_ref()
    }
    /// Java `getWriteCellStyle` 别名。
    #[must_use] pub const fn get_write_cell_style(&self) -> Option<&WriteCellStyle> { self.write_cell_style() }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Replaces the logical cell style. (Java `setWriteCellStyle(...)`)
    pub fn set_write_cell_style(&mut self, style: Option<WriteCellStyle>) {
        self.write_cell_style = style;
    }

    /// 返回后端原始样式。对应 Java：`getOriginCellStyle()`。
    #[must_use]
    pub const fn get_origin_cell_style(&self) -> Option<&WriteCellStyle> {
        self.origin_cell_style.as_ref()
    }

    /// 设置后端原始样式。对应 Java：`setOriginCellStyle(CellStyle)`。
    pub const fn set_origin_cell_style(&mut self, style: Option<WriteCellStyle>) {
        self.origin_cell_style = style;
    }

    /// 返回富文本值。对应 Java：`getRichTextStringDataValue()`。
    #[must_use]
    pub fn get_rich_text_string_data_value(&self) -> Option<&RichTextStringData> {
        match &self.value {
            CellValue::RichText(value) => Some(value),
            _ => None,
        }
    }

    /// 设置富文本值。对应 Java：`setRichTextStringDataValue(...)`。
    pub fn set_rich_text_string_data_value(&mut self, value: Option<RichTextStringData>) {
        if let Some(value) = value {
            self.value = CellValue::RichText(value);
            self.declared_type = Some(crate::CellDataType::RichTextString);
        } else if matches!(self.value, CellValue::RichText(_)) {
            self.value = CellValue::Empty;
            self.declared_type = None;
        }
    }

    /// 返回日期时间值。对应 Java：`getDateValue()`。
    #[must_use]
    pub const fn get_date_value(&self) -> Option<&NaiveDateTime> {
        match &self.value {
            CellValue::DateTime(value) => Some(value),
            _ => None,
        }
    }

    /// 设置日期时间值。对应 Java：`setDateValue(LocalDateTime)`。
    pub fn set_date_value(&mut self, value: Option<NaiveDateTime>) {
        if let Some(value) = value {
            self.value = CellValue::DateTime(value);
            self.declared_type = Some(crate::CellDataType::Date);
        } else if matches!(self.value, CellValue::DateTime(_)) {
            self.value = CellValue::Empty;
            self.declared_type = None;
        }
    }

    /// Returns a mutable style, creating it when absent.
    ///
    /// 对应 Java：`WriteCellData#getOrCreateStyle`.
    pub fn get_or_create_style(&mut self) -> &mut WriteCellStyle {
        self.write_cell_style
            .get_or_insert_with(WriteCellStyle::default)
    }

    /// Returns the owned data-format metadata associated with the style.
    ///
    /// Java stores this object inside `WriteCellStyle`; Rust keeps the owned
    /// runtime string beside the copyable annotation style.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。
    pub const fn data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data.as_ref()
    }
    /// 返回数据格式元数据。
    #[must_use] pub const fn get_data_format_data(&self) -> Option<&DataFormatData> { self.data_format_data() }
    /// 替换数据格式元数据。
    pub fn set_data_format_data(&mut self, value: Option<DataFormatData>) { self.data_format_data = value; }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Returns mutable data-format metadata, creating it when absent.
    pub fn get_or_create_data_format(&mut self) -> &mut DataFormatData {
        self.data_format_data
            .get_or_insert_with(DataFormatData::default)
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.WriteCellData。 Resolves the scalar plus formula/link/comment/image decorations into
    /// the backend-neutral value written by an engine.
    ///
    /// The style and data-format fields intentionally remain on
    /// `WriteCellData`; Java applies them after conversion in
    /// `FillStyleCellWriteHandler`.
    #[must_use]
    pub fn effective_value(&self) -> CellValue {
        let mut value = self.value.clone();
        if let Some(formula) = &self.formula_data {
            value = CellValue::Formula(formula.formula_value().to_owned());
        }
        if let Some(link) = &self.hyperlink_data {
            let address = link.get_address().unwrap_or("").to_owned();
            let text = match &value {
                CellValue::String(s) => s.clone(),
                other => other.as_text(),
            };
            value = CellValue::HyperlinkWithMetadata {
                address,
                text,
                hyperlink_type: link.get_hyperlink_type(),
                coordinates: link.get_coordinates(),
            };
        }
        if let Some(comment) = &self.comment_data {
            value = CellValue::CommentWithMetadata {
                value: Box::new(value),
                comment: comment.clone(),
            };
        }
        if self.image_data_list.is_empty() {
            value
        } else {
            CellValue::Images {
                value: Box::new(value),
                images: self.image_data_list.clone(),
            }
        }
    }
}

impl Default for WriteCellData {
    fn default() -> Self { Self::empty() }
}

impl IntoExcelCell for WriteCellData {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(self.effective_value())
    }
}

impl FromExcelCell for WriteCellData {
    fn from_excel_cell(
        cell: Option<&CellValue>,
        _context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Ok(Self::new(cell.cloned().unwrap_or(CellValue::Empty)))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn from_hyperlink_and_from_comment_construct_values() {
        // 对应 Java：WriteCellData 超链接与备注构造
        let link = WriteCellData::from_hyperlink("https://x", "显示");
        assert_eq!(
            link.value(),
            &CellValue::Hyperlink {
                url: "https://x".to_owned(),
                text: "显示".to_owned()
            }
        );

        let comment = WriteCellData::from_comment(CellValue::Int(5), "说明");
        assert_eq!(
            comment.value(),
            &CellValue::Comment {
                value: Box::new(CellValue::Int(5)),
                text: "说明".to_owned()
            }
        );
    }

    #[test]
    fn decoration_getters_return_configured_metadata() {
        // 对应 Java：getCommentData / getHyperlinkData / getFormulaData
        let data = WriteCellData::from_string("x")
            .comment_data(CommentData::new().text("备注".to_owned()))
            .hyperlink_data(HyperlinkData::new().address("https://x".to_owned()))
            .formula_data(FormulaData::new("=A1"));

        assert_eq!(
            data.get_comment_data().map(CommentData::note_text),
            Some("备注".to_owned())
        );
        assert_eq!(
            data.get_hyperlink_data()
                .and_then(HyperlinkData::get_address),
            Some("https://x")
        );
        assert_eq!(
            data.get_formula_data().map(FormulaData::formula_value),
            Some("=A1")
        );
    }

    #[test]
    fn decoration_getters_return_none_when_absent() {
        // 对应 Java：未设置装饰时返回空
        let data = WriteCellData::from_string("plain");
        assert!(data.get_comment_data().is_none());
        assert!(data.get_hyperlink_data().is_none());
        assert!(data.get_formula_data().is_none());
    }
}
