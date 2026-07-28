//! Mirrors Java `com.alibaba.excel.write.metadata.style.WriteFont`.

use easyexcel_core::{ExcelFontStyle, WriteFont};

/// Mirrors Java `WriteCellStyle`'s font side.
pub type WriteCellFont = WriteFont;

/// Mirrors Java `WriteFont.merge(WriteFont source, WriteFont target)`.
///
/// Java's `merge` copies every non-`None` field from source to target.
/// The Rust port performs the same union over the `Option` fields on
/// [`WriteFont`].
#[must_use]
pub fn merge_write_font(source: &WriteFont, mut target: WriteFont) -> WriteFont {
    if source.get_font_name().is_some() {
        target = target.font_name(source.get_font_name().unwrap().to_owned());
    }
    if let Some(height) = source.get_font_height_in_points() {
        target = target.font_height_in_points(height);
    }
    if let Some(italic) = source.get_italic() {
        target = target.italic(italic);
    }
    if let Some(strikeout) = source.get_strikeout() {
        target = target.strikeout(strikeout);
    }
    if let Some(color) = source.get_color() {
        target = target.color(color);
    }
    if let Some(script) = source.get_type_offset() {
        target = target.type_offset(script);
    }
    if let Some(underline) = source.get_underline() {
        target = target.underline(underline);
    }
    if let Some(charset) = source.get_charset() {
        target = target.charset(charset);
    }
    if let Some(bold) = source.get_bold() {
        target = target.bold(bold);
    }
    target
}

/// Merges annotation/strategy fonts. (Java `WriteFont.merge` over `ExcelFontStyle`)
///
/// Copies every non-`None` field from `source` onto `target`, matching the
/// Java `WriteFont.merge` null-skip semantics used when nesting fonts inside
/// `WriteCellStyle`.
#[must_use]
pub fn merge_excel_font_style(
    source: &ExcelFontStyle,
    mut target: ExcelFontStyle,
) -> ExcelFontStyle {
    if source.font_name.is_some() {
        target.font_name = source.font_name;
    }
    if source.font_height_in_points.is_some() {
        target.font_height_in_points = source.font_height_in_points;
    }
    if source.italic.is_some() {
        target.italic = source.italic;
    }
    if source.strikeout.is_some() {
        target.strikeout = source.strikeout;
    }
    if source.color.is_some() {
        target.color = source.color;
    }
    if source.type_offset.is_some() {
        target.type_offset = source.type_offset;
    }
    if source.underline.is_some() {
        target.underline = source.underline;
    }
    if source.charset.is_some() {
        target.charset = source.charset;
    }
    if source.bold.is_some() {
        target.bold = source.bold;
    }
    target
}

/// Converts runtime [`WriteFont`] into Copy [`ExcelFontStyle`] for strategy styles.
///
/// Mirrors nesting Java `WriteFont` into `WriteCellStyle.writeFont`. Owned
/// `font_name` strings cannot become `&'static str`; pass name via
/// [`ExcelFontStyle::font_name`] when a static label is available. All other
/// common fields (size, color, bold, italic, underline, strikeout, charset,
/// type offset) are preserved.
#[must_use]
pub fn excel_font_style_from_write_font(font: &WriteFont) -> ExcelFontStyle {
    ExcelFontStyle {
        font_name: None,
        font_height_in_points: font.get_font_height_in_points(),
        italic: font.get_italic(),
        strikeout: font.get_strikeout(),
        color: font.get_color(),
        type_offset: font.get_type_offset(),
        underline: font.get_underline(),
        charset: font.get_charset(),
        bold: font.get_bold(),
    }
}

#[cfg(test)]
mod tests {
    use easyexcel_core::{ExcelColor, ExcelFontScript, ExcelFontStyle, ExcelUnderline, WriteFont};

    use super::*;

    fn sample_color() -> ExcelColor {
        ExcelColor::Indexed(10)
    }

    #[test]
    fn merge_write_font_copies_all_non_none_fields() {
        let source = WriteFont::new()
            .font_name("Arial")
            .font_height_in_points(12.0)
            .italic(true)
            .strikeout(true)
            .color(sample_color())
            .type_offset(ExcelFontScript::Superscript)
            .underline(ExcelUnderline::Single)
            .charset(1)
            .bold(true);

        let target = WriteFont::new();
        let merged = merge_write_font(&source, target);

        assert_eq!(merged.get_font_name(), Some("Arial"));
        assert_eq!(merged.get_font_height_in_points(), Some(12.0));
        assert_eq!(merged.get_italic(), Some(true));
        assert_eq!(merged.get_strikeout(), Some(true));
        assert_eq!(merged.get_color(), Some(sample_color()));
        assert_eq!(merged.get_type_offset(), Some(ExcelFontScript::Superscript));
        assert_eq!(merged.get_underline(), Some(ExcelUnderline::Single));
        assert_eq!(merged.get_charset(), Some(1));
        assert_eq!(merged.get_bold(), Some(true));
    }

    #[test]
    fn merge_write_font_skips_none_fields() {
        let source = WriteFont::new().bold(true);
        let target = WriteFont::new().font_name("Times").italic(false);
        let merged = merge_write_font(&source, target);

        assert_eq!(merged.get_font_name(), Some("Times"));
        assert_eq!(merged.get_italic(), Some(false));
        assert_eq!(merged.get_bold(), Some(true));
        assert_eq!(merged.get_font_height_in_points(), None);
    }

    #[test]
    fn merge_write_font_overwrites_target_when_source_has_value() {
        let source = WriteFont::new().font_name("Arial").bold(true);
        let target = WriteFont::new().font_name("Courier").bold(false);
        let merged = merge_write_font(&source, target);

        assert_eq!(merged.get_font_name(), Some("Arial"));
        assert_eq!(merged.get_bold(), Some(true));
    }

    #[test]
    fn merge_write_font_preserves_target_when_source_is_empty() {
        let source = WriteFont::new();
        let target = WriteFont::new().font_name("Helvetica").font_height_in_points(14.0);
        let merged = merge_write_font(&source, target);

        assert_eq!(merged.get_font_name(), Some("Helvetica"));
        assert_eq!(merged.get_font_height_in_points(), Some(14.0));
    }

    #[test]
    fn merge_excel_font_style_copies_all_non_none_fields() {
        let source = ExcelFontStyle {
            font_name: Some("Arial"),
            font_height_in_points: Some(12.0),
            italic: Some(true),
            strikeout: Some(true),
            color: Some(sample_color()),
            type_offset: Some(ExcelFontScript::Superscript),
            underline: Some(ExcelUnderline::Single),
            charset: Some(1),
            bold: Some(true),
        };
        let target = ExcelFontStyle::default();
        let merged = merge_excel_font_style(&source, target);

        assert_eq!(merged.font_name, Some("Arial"));
        assert_eq!(merged.font_height_in_points, Some(12.0));
        assert_eq!(merged.italic, Some(true));
        assert_eq!(merged.strikeout, Some(true));
        assert_eq!(merged.color, Some(sample_color()));
        assert_eq!(merged.type_offset, Some(ExcelFontScript::Superscript));
        assert_eq!(merged.underline, Some(ExcelUnderline::Single));
        assert_eq!(merged.charset, Some(1));
        assert_eq!(merged.bold, Some(true));
    }

    #[test]
    fn merge_excel_font_style_skips_none_fields() {
        let source = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let target = ExcelFontStyle {
            font_name: Some("Times"),
            italic: Some(false),
            ..ExcelFontStyle::default()
        };
        let merged = merge_excel_font_style(&source, target);

        assert_eq!(merged.font_name, Some("Times"));
        assert_eq!(merged.italic, Some(false));
        assert_eq!(merged.bold, Some(true));
        assert_eq!(merged.font_height_in_points, None);
    }

    #[test]
    fn merge_excel_font_style_overwrites_target_when_source_has_value() {
        let source = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let target = ExcelFontStyle {
            font_name: Some("Courier"),
            bold: Some(false),
            ..ExcelFontStyle::default()
        };
        let merged = merge_excel_font_style(&source, target);

        assert_eq!(merged.font_name, Some("Arial"));
        assert_eq!(merged.bold, Some(true));
    }

    #[test]
    fn merge_excel_font_style_preserves_target_when_source_is_empty() {
        let source = ExcelFontStyle::default();
        let target = ExcelFontStyle {
            font_name: Some("Helvetica"),
            font_height_in_points: Some(14.0),
            ..ExcelFontStyle::default()
        };
        let merged = merge_excel_font_style(&source, target);

        assert_eq!(merged.font_name, Some("Helvetica"));
        assert_eq!(merged.font_height_in_points, Some(14.0));
    }

    #[test]
    fn excel_font_style_from_write_font_converts_all_fields() {
        let font = WriteFont::new()
            .font_height_in_points(12.0)
            .italic(true)
            .strikeout(true)
            .color(sample_color())
            .type_offset(ExcelFontScript::Superscript)
            .underline(ExcelUnderline::Single)
            .charset(1)
            .bold(true);

        let style = excel_font_style_from_write_font(&font);

        assert_eq!(style.font_name, None);
        assert_eq!(style.font_height_in_points, Some(12.0));
        assert_eq!(style.italic, Some(true));
        assert_eq!(style.strikeout, Some(true));
        assert_eq!(style.color, Some(sample_color()));
        assert_eq!(style.type_offset, Some(ExcelFontScript::Superscript));
        assert_eq!(style.underline, Some(ExcelUnderline::Single));
        assert_eq!(style.charset, Some(1));
        assert_eq!(style.bold, Some(true));
    }

    #[test]
    fn excel_font_style_from_write_font_handles_empty_font() {
        let font = WriteFont::new();
        let style = excel_font_style_from_write_font(&font);

        assert_eq!(style.font_name, None);
        assert_eq!(style.font_height_in_points, None);
        assert_eq!(style.italic, None);
        assert_eq!(style.strikeout, None);
        assert_eq!(style.color, None);
        assert_eq!(style.type_offset, None);
        assert_eq!(style.underline, None);
        assert_eq!(style.charset, None);
        assert_eq!(style.bold, None);
    }

    #[test]
    fn write_cell_font_is_write_font_alias() {
        let font = WriteFont::new().bold(true);
        let cell_font: WriteCellFont = font.clone();
        assert_eq!(cell_font.get_bold(), Some(true));
    }
}
