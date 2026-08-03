//! 对应 Java：`com.alibaba.excel.metadata.property.ExcelHeadProperty` field
//! `Head.columnIndex` / `field` / `fieldName` / `headNameList` /
//! `columnWidthProperty` / `headStyleProperty` / etc.

use crate::core::cell_value::CellValue;
use crate::core::comment_data::CommentData;
use crate::core::excel_cell_style::ExcelCellStyle;
use crate::core::excel_font_style::ExcelFontStyle;
use crate::core::hyperlink_data::HyperlinkData;
use crate::metadata::property::LoopMergeProperty;
use crate::metadata::property::NumberRoundingMode;
use crate::metadata::property::data_validation_property::ExcelDataValidationMeta;
use crate::write::write_cell_data::WriteCellData;

/// Static metadata for one Rust struct field and Excel column.
///
/// Mirrors the union of fields that Java stores across
/// `Head` / `FieldCache` / `FieldWrapper`. The Rust port exposes a single
/// `Copy` struct so `#[derive(ExcelRow)]` can emit a `&'static [ExcelColumn]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExcelColumn {
    /// Rust field name. (Java `Head.fieldName`)
    pub field: &'static str,
    /// Declared Rust field type. (Java `Head.field.getType()`)
    ///
    /// Derive-generated schemas populate this with `stringify!(FieldType)`.
    /// Manually constructed schemas leave it unset unless explicitly supplied.
    pub field_type: Option<&'static str>,
    /// Excel header name. (Java `Head.headNameList[0]`)
    pub name: &'static str,
    /// Explicit zero-based column index. (Java `Head.forceIndex` + `index`)
    pub index: Option<usize>,
    /// Relative ordering when no explicit index is configured. (Java `@ExcelProperty.order`)
    pub order: i32,
    /// Optional date or number format. (Java `@ExcelProperty.format`)
    pub format: Option<&'static str>,
    /// Java `@NumberFormat.roundingMode`; defaults to `HALF_UP`.
    pub number_rounding_mode: Option<NumberRoundingMode>,
    /// Field-level override for Excel's 1904 date system.
    /// (Java `@DateTimeFormat.use1904windowing`)
    pub use_1904_windowing: Option<bool>,
    /// Optional annotation-driven column width in Excel character units. (Java `ColumnWidth`)
    pub column_width: Option<u16>,
    /// Field-level header cell style. (Java `@HeadStyle`)
    pub head_style: Option<ExcelCellStyle>,
    /// Field-level content cell style. (Java `@ContentStyle`)
    pub content_style: Option<ExcelCellStyle>,
    /// Field-level header font style. (Java `@HeadFontStyle`)
    pub head_font_style: Option<ExcelFontStyle>,
    /// Field-level content font style. (Java `@ContentFontStyle`)
    pub content_font_style: Option<ExcelFontStyle>,
    /// Field-level repeating content merge. (Java `@ContentLoopMerge` → `Head.loopMergeProperty`)
    pub loop_merge: Option<LoopMergeProperty>,

    // Phase 1: new annotation-derived fields (Phase 1 markers in
    // com.alibaba.excel.annotation.write.*ExcelImage / Comment / Hyperlink /
    // Formula / DataValidation / Conditional / Filter).
    /// Optional image path or URL for this column. (Java `@ExcelImage.image()`)
    pub image_path: Option<&'static str>,
    /// Optional cell comment / note. (Java `@ExcelComment.value()`)
    pub comment: Option<&'static str>,
    /// Optional hyperlink target. (Java `@ExcelHyperlink.value()`)
    pub hyperlink: Option<&'static str>,
    /// Optional formula override. (Java `@ExcelFormula.value()`)
    pub formula: Option<&'static str>,
    /// Optional data-validation metadata. (Java `@ExcelDataValidation`)
    pub data_validation: Option<ExcelDataValidationMeta>,
    /// Optional conditional-formatting tuple `(condition, font_color, bg_color)`.
    /// (Java `@ExcelConditional`)
    pub conditional_format: Option<(&'static str, &'static str, &'static str)>,
    /// Whether this column participates in auto-filtering. (Java `@ExcelFilter`)
    pub auto_filter: bool,
}

impl ExcelColumn {
    /// Creates static column metadata. (Java `Head(columnIndex, field, fieldName, headNameList, forceIndex, forceName)` subset)
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        field: &'static str,
        name: &'static str,
        index: Option<usize>,
        order: i32,
        format: Option<&'static str>,
    ) -> Self {
        Self {
            field,
            field_type: None,
            name,
            index,
            order,
            format,
            number_rounding_mode: None,
            use_1904_windowing: None,
            column_width: None,
            head_style: None,
            content_style: None,
            head_font_style: None,
            content_font_style: None,
            loop_merge: None,
            image_path: None,
            comment: None,
            hyperlink: None,
            formula: None,
            data_validation: None,
            conditional_format: None,
            auto_filter: false,
        }
    }

    /// Adds Java-compatible number rounding metadata.
    #[must_use]
    pub const fn with_number_rounding_mode(mut self, mode: NumberRoundingMode) -> Self {
        self.number_rounding_mode = Some(mode);
        self
    }

    /// Adds the declared Rust field type.
    ///
    /// 对应 Java：`Head.field.getType()` / `originalFieldClass`.
    #[must_use]
    pub const fn with_field_type(mut self, field_type: &'static str) -> Self {
        self.field_type = Some(field_type);
        self
    }

    /// Adds the field-level `@DateTimeFormat.use1904windowing` override.
    #[must_use]
    pub const fn with_use_1904_windowing(mut self, enabled: bool) -> Self {
        self.use_1904_windowing = Some(enabled);
        self
    }

    /// Adds annotation-driven column width. (Java `@ColumnWidth`)
    #[must_use]
    pub const fn with_column_width(mut self, width: u16) -> Self {
        self.column_width = Some(width);
        self
    }

    /// Adds a field-level header cell style. (Java `@HeadStyle`)
    #[must_use]
    pub const fn with_head_style(mut self, style: ExcelCellStyle) -> Self {
        self.head_style = Some(style);
        self
    }

    /// Adds a field-level content cell style. (Java `@ContentStyle`)
    #[must_use]
    pub const fn with_content_style(mut self, style: ExcelCellStyle) -> Self {
        self.content_style = Some(style);
        self
    }

    /// Adds a field-level header font style. (Java `@HeadFontStyle`)
    #[must_use]
    pub const fn with_head_font_style(mut self, style: ExcelFontStyle) -> Self {
        self.head_font_style = Some(style);
        self
    }

    /// Adds a field-level content font style. (Java `@ContentFontStyle`)
    #[must_use]
    pub const fn with_content_font_style(mut self, style: ExcelFontStyle) -> Self {
        self.content_font_style = Some(style);
        self
    }

    /// Adds a field-level repeating content merge. (Java `@ContentLoopMerge`)
    #[must_use]
    pub const fn with_loop_merge(mut self, property: LoopMergeProperty) -> Self {
        self.loop_merge = Some(property);
        self
    }

    // Phase 1: new annotation-derived column builders

    /// Adds a per-column image source. (Java `@ExcelImage`)
    #[must_use]
    pub const fn with_image_path(mut self, path: &'static str) -> Self {
        self.image_path = Some(path);
        self
    }

    /// Adds a per-column cell comment. (Java `@ExcelComment`)
    #[must_use]
    pub const fn with_comment(mut self, comment: &'static str) -> Self {
        self.comment = Some(comment);
        self
    }

    /// Adds a per-column hyperlink target. (Java `@ExcelHyperlink`)
    #[must_use]
    pub const fn with_hyperlink(mut self, link: &'static str) -> Self {
        self.hyperlink = Some(link);
        self
    }

    /// Adds a per-column formula override. (Java `@ExcelFormula`)
    #[must_use]
    pub const fn with_formula(mut self, formula: &'static str) -> Self {
        self.formula = Some(formula);
        self
    }

    /// Adds per-column data-validation metadata. (Java `@ExcelDataValidation`)
    #[must_use]
    pub const fn with_data_validation(mut self, meta: ExcelDataValidationMeta) -> Self {
        self.data_validation = Some(meta);
        self
    }

    /// Adds per-column conditional-formatting metadata. (Java `@ExcelConditional`)
    #[must_use]
    pub const fn with_conditional_format(
        mut self,
        cf: (&'static str, &'static str, &'static str),
    ) -> Self {
        self.conditional_format = Some(cf);
        self
    }

    /// Marks the column as participating in auto-filter. (Java `@ExcelFilter`)
    #[must_use]
    pub const fn with_auto_filter(mut self) -> Self {
        self.auto_filter = true;
        self
    }

    // -------- Phase 1.4: decoration helpers applied to WriteCellData --------

    /// Applies this column's annotation-driven decorations (hyperlink / formula /
    /// comment) onto a `WriteCellData`. (Java `Head.fillHeadAndWriteData` decorations)
    ///
    /// Order matches Java `ExcelBuilderImpl` write path:
    /// 1. formula override wraps the scalar (`CellValue::Formula`)
    /// 2. hyperlink wraps the display text (`CellValue::Hyperlink`)
    /// 3. comment wraps the underlying value (`CellValue::Comment`)
    #[must_use]
    pub fn apply_decorations(&self, mut data: WriteCellData) -> WriteCellData {
        if let Some(formula) = self.formula {
            data.set_value(CellValue::Formula(formula.to_owned()));
        }
        if let Some(url) = self.hyperlink {
            let text = match data.value() {
                CellValue::String(s) => s.clone(),
                other => other.as_text(),
            };
            data.set_value(CellValue::Hyperlink {
                url: url.to_owned(),
                text,
            });
            // Also reflect via HyperlinkData so writer layers can access
            // both the wrapped CellValue and the structured side-channel.
            data = data.hyperlink_data(HyperlinkData::new().address(url.to_owned()));
        }
        if let Some(comment_text) = self.comment {
            let comment = CommentData::new().text(comment_text.to_owned());
            data = data.comment_data(comment);
        }
        data
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn builder_methods_populate_annotation_fields() {
        // 对应 Java：Head 上各注解元数据的 builder
        let base = ExcelColumn::new("field", "名称", Some(0), 0, None);

        let rounded = base.with_number_rounding_mode(NumberRoundingMode::Ceiling);
        assert_eq!(
            rounded.number_rounding_mode,
            Some(NumberRoundingMode::Ceiling)
        );

        let typed = base.with_field_type("String");
        assert_eq!(typed.field_type, Some("String"));

        let wind = base.with_use_1904_windowing(true);
        assert_eq!(wind.use_1904_windowing, Some(true));

        let img = base.with_image_path("/tmp/a.png");
        assert_eq!(img.image_path, Some("/tmp/a.png"));

        let comment = base.with_comment("备注");
        assert_eq!(comment.comment, Some("备注"));

        let link = base.with_hyperlink("https://example.com");
        assert_eq!(link.hyperlink, Some("https://example.com"));

        let formula = base.with_formula("SUM(A1:A2)");
        assert_eq!(formula.formula, Some("SUM(A1:A2)"));

        let validation = ExcelDataValidationMeta::new("list", "between", "1", "2");
        let validated = base.with_data_validation(validation);
        assert_eq!(validated.data_validation, Some(validation));

        let cf = base.with_conditional_format((">10", "red", "yellow"));
        assert_eq!(cf.conditional_format, Some((">10", "red", "yellow")));

        let filtered = base.with_auto_filter();
        assert!(filtered.auto_filter);

        // 未设置字段保持默认
        assert_eq!(base.image_path, None);
        assert!(!base.auto_filter);
    }

    #[test]
    fn apply_decorations_wraps_hyperlink_over_non_string_value() {
        // 对应 Java：Head 装饰顺序 formula -> hyperlink -> comment，
        // 非字符串值走 as_text 文本化
        let column = ExcelColumn::new("field", "名称", None, 0, None)
            .with_hyperlink("https://example.com")
            .with_comment("说明");

        let mut data = WriteCellData::new(CellValue::Int(42));
        data = column.apply_decorations(data);

        // 数值先被 hyperlink 覆盖，文本为 as_text 结果。
        // 整值断言替代 match 的兜底 panic 臂（apply_decorations 恒构造 Hyperlink 包装，
        // other 臂数学不可达；if-let 空走会静默放行，改用整值断言保持失败可见）。
        assert_eq!(
            data.value(),
            &CellValue::Hyperlink {
                url: "https://example.com".to_owned(),
                text: "42".to_owned(),
            }
        );
        assert_eq!(
            data.get_comment_data()
                .map(crate::core::comment_data::CommentData::note_text)
                .as_deref(),
            Some("说明")
        );
        assert_eq!(
            data.get_hyperlink_data()
                .map(crate::core::hyperlink_data::HyperlinkData::get_address),
            Some(Some("https://example.com"))
        );
    }

    #[test]
    fn apply_decorations_formula_then_hyperlink_then_comment() {
        // 对应 Java：formula 先包装，hyperlink 显示文本取 formula 文本
        let column = ExcelColumn::new("field", "名称", None, 0, None)
            .with_formula("=A1*2")
            .with_hyperlink("https://example.com");
        let data = column.apply_decorations(WriteCellData::new(CellValue::Int(42)));
        // 整值断言替代 match 兜底 panic 臂（同 apply_decorations_wraps_hyperlink_over_non_string_value）。
        assert_eq!(
            data.value(),
            &CellValue::Hyperlink {
                url: "https://example.com".to_owned(),
                text: "=A1*2".to_owned(),
            }
        );
    }

    #[test]
    fn apply_decorations_keeps_string_text_for_hyperlink() {
        // 对应 Java：字符串值直接作为超链接显示文本
        let column = ExcelColumn::new("field", "名称", None, 0, None).with_hyperlink("https://x");
        let data = column.apply_decorations(WriteCellData::from_string("文本"));
        // 整值断言替代 match 兜底 panic 臂（同 apply_decorations_wraps_hyperlink_over_non_string_value）。
        assert_eq!(
            data.value(),
            &CellValue::Hyperlink {
                url: "https://x".to_owned(),
                text: "文本".to_owned(),
            }
        );
    }
}
