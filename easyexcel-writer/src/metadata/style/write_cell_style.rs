//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle`.

use easyexcel_core::ExcelCellStyle;

use crate::metadata::style::write_font::merge_excel_font_style;

/// 对应 Java：`WriteCellStyle`.
///
/// The Java side carries POI-typed fields plus nested `writeFont` and a
/// static `merge` helper. Rust reuses [`ExcelCellStyle`] for the data
/// (including [`ExcelCellStyle::font`]) and mirrors the merge method.
pub type WriteCellStyle = ExcelCellStyle;

/// 对应 Java：`WriteCellStyle.merge(WriteCellStyle source, WriteCellStyle target)`.
///
/// Java merges the source's non-null fields into the target, including
/// nested `WriteFont.merge`. The Rust port performs the same union over
/// [`ExcelCellStyle`]'s `Option` fields and [`ExcelCellStyle::font`].
pub fn merge_write_cell_style(
    source: &ExcelCellStyle,
    mut target: ExcelCellStyle,
) -> ExcelCellStyle {
    macro_rules! or {
        ($field:ident) => {
            if source.$field.is_some() {
                target.$field = source.$field;
            }
        };
    }
    or!(hidden);
    or!(locked);
    or!(quote_prefix);
    or!(horizontal_alignment);
    or!(wrapped);
    or!(vertical_alignment);
    or!(rotation);
    or!(indent);
    or!(border_left);
    or!(border_right);
    or!(border_top);
    or!(border_bottom);
    or!(left_border_color);
    or!(right_border_color);
    or!(top_border_color);
    or!(bottom_border_color);
    or!(fill_pattern);
    or!(fill_background_color);
    or!(fill_foreground_color);
    or!(shrink_to_fit);
    or!(data_format);
    // Java `WriteFont.merge(source.getWriteFont(), target.getWriteFont())`
    if let Some(source_font) = source.font {
        target.font = Some(match target.font {
            Some(existing) => merge_excel_font_style(&source_font, existing),
            None => source_font,
        });
    }
    target
}

#[cfg(test)]
mod tests {
    use easyexcel_core::{
        ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern,
        ExcelFontStyle, ExcelHorizontalAlignment, ExcelVerticalAlignment,
    };

    use super::*;

    #[test]
    fn merge_empty_source_preserves_target() {
        let source = ExcelCellStyle::new();
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged, ExcelCellStyle::new());
    }

    #[test]
    fn merge_copies_hidden_field() {
        let source = ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn merge_copies_locked_field() {
        let source = ExcelCellStyle {
            locked: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.locked, Some(true));
    }

    #[test]
    fn merge_copies_quote_prefix_field() {
        let source = ExcelCellStyle {
            quote_prefix: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.quote_prefix, Some(true));
    }

    #[test]
    fn merge_copies_horizontal_alignment() {
        let source = ExcelCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(
            merged.horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn merge_copies_wrapped_field() {
        let source = ExcelCellStyle {
            wrapped: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.wrapped, Some(true));
    }

    #[test]
    fn merge_copies_vertical_alignment() {
        let source = ExcelCellStyle {
            vertical_alignment: Some(ExcelVerticalAlignment::Center),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(
            merged.vertical_alignment,
            Some(ExcelVerticalAlignment::Center)
        );
    }

    #[test]
    fn merge_copies_rotation_field() {
        let source = ExcelCellStyle {
            rotation: Some(45),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.rotation, Some(45));
    }

    #[test]
    fn merge_copies_indent_field() {
        let source = ExcelCellStyle {
            indent: Some(2),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.indent, Some(2));
    }

    #[test]
    fn merge_copies_border_fields() {
        let source = ExcelCellStyle {
            border_left: Some(ExcelBorderStyle::Thin),
            border_right: Some(ExcelBorderStyle::Medium),
            border_top: Some(ExcelBorderStyle::Dashed),
            border_bottom: Some(ExcelBorderStyle::Dotted),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.border_left, Some(ExcelBorderStyle::Thin));
        assert_eq!(merged.border_right, Some(ExcelBorderStyle::Medium));
        assert_eq!(merged.border_top, Some(ExcelBorderStyle::Dashed));
        assert_eq!(merged.border_bottom, Some(ExcelBorderStyle::Dotted));
    }

    #[test]
    fn merge_copies_border_colors() {
        let source = ExcelCellStyle {
            left_border_color: Some(ExcelColor::Indexed(1)),
            right_border_color: Some(ExcelColor::Indexed(2)),
            top_border_color: Some(ExcelColor::Indexed(3)),
            bottom_border_color: Some(ExcelColor::Indexed(4)),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.left_border_color, Some(ExcelColor::Indexed(1)));
        assert_eq!(merged.right_border_color, Some(ExcelColor::Indexed(2)));
        assert_eq!(merged.top_border_color, Some(ExcelColor::Indexed(3)));
        assert_eq!(merged.bottom_border_color, Some(ExcelColor::Indexed(4)));
    }

    #[test]
    fn merge_copies_fill_pattern_and_colors() {
        let source = ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_background_color: Some(ExcelColor::Indexed(10)),
            fill_foreground_color: Some(ExcelColor::Indexed(20)),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.fill_pattern, Some(ExcelFillPattern::Solid));
        assert_eq!(merged.fill_background_color, Some(ExcelColor::Indexed(10)));
        assert_eq!(merged.fill_foreground_color, Some(ExcelColor::Indexed(20)));
    }

    #[test]
    fn merge_copies_shrink_to_fit() {
        let source = ExcelCellStyle {
            shrink_to_fit: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.shrink_to_fit, Some(true));
    }

    #[test]
    fn merge_copies_data_format() {
        let source = ExcelCellStyle {
            data_format: Some(ExcelDataFormat::Builtin(0)),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.data_format, Some(ExcelDataFormat::Builtin(0)));
    }

    #[test]
    fn merge_copies_font_when_target_has_none() {
        let source_font = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let source = ExcelCellStyle {
            font: Some(source_font),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert!(merged.font.is_some());
        assert_eq!(merged.font.unwrap().bold, Some(true));
    }

    #[test]
    fn merge_merges_font_when_target_has_font() {
        let source_font = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let target_font = ExcelFontStyle {
            italic: Some(true),
            ..ExcelFontStyle::default()
        };
        let source = ExcelCellStyle {
            font: Some(source_font),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle {
            font: Some(target_font),
            ..ExcelCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        let font = merged.font.unwrap();
        assert_eq!(font.bold, Some(true));
        assert_eq!(font.italic, Some(true));
    }

    #[test]
    fn merge_overwrites_target_when_source_has_value() {
        let source = ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        };
        let target = ExcelCellStyle {
            hidden: Some(false),
            ..ExcelCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn merge_preserves_target_when_source_field_is_none() {
        let source = ExcelCellStyle::new();
        let target = ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn write_cell_style_is_excel_cell_style_alias() {
        let style = ExcelCellStyle::new();
        let alias: WriteCellStyle = style;
        assert_eq!(alias, ExcelCellStyle::new());
    }
}
