//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteFont` (the
//! annotation-driven subset carried by `ExcelFontStyle`).

use std::hash::{Hash, Hasher};

use crate::core::excel_color::ExcelColor;
use crate::core::excel_font_script::ExcelFontScript;
use crate::core::excel_underline::ExcelUnderline;

/// 对应 Java：com.alibaba.excel.write.metadata.style.WriteFont。 Font properties generated from `HeadFontStyle` or `ContentFontStyle` equivalents.
///
/// All nine fields correspond one-for-one to Java's `WriteFont`. `font_name`
/// is constrained to `&'static str` so the struct can stay `Copy`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExcelFontStyle {
    /// Font family name.
    pub font_name: Option<&'static str>,
    /// Font size in points.
    pub font_height_in_points: Option<f64>,
    /// Italic rendering.
    pub italic: Option<bool>,
    /// Strike-through rendering.
    pub strikeout: Option<bool>,
    /// Font indexed or RGB color.
    pub color: Option<ExcelColor>,
    /// Superscript or subscript positioning.
    pub type_offset: Option<ExcelFontScript>,
    /// Underline rendering.
    pub underline: Option<ExcelUnderline>,
    /// Font character set.
    pub charset: Option<u8>,
    /// Bold rendering.
    pub bold: Option<bool>,
}

impl PartialEq for ExcelFontStyle {
    fn eq(&self, other: &Self) -> bool {
        self.font_name == other.font_name
            && self.font_height_in_points.map(java_double_bits)
                == other.font_height_in_points.map(java_double_bits)
            && self.italic == other.italic
            && self.strikeout == other.strikeout
            && self.color == other.color
            && self.type_offset == other.type_offset
            && self.underline == other.underline
            && self.charset == other.charset
            && self.bold == other.bold
    }
}

impl Eq for ExcelFontStyle {}

impl Hash for ExcelFontStyle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_name.hash(state);
        self.font_height_in_points.map(java_double_bits).hash(state);
        self.italic.hash(state);
        self.strikeout.hash(state);
        self.color.hash(state);
        self.type_offset.hash(state);
        self.underline.hash(state);
        self.charset.hash(state);
        self.bold.hash(state);
    }
}

/// 对齐 Java `Double.doubleToLongBits`：所有 NaN 规范化，正负零保持不同。
fn java_double_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

impl ExcelFontStyle {
    /// Creates an annotation font style with every property unspecified. (Java `WriteFont()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteFont。
    pub const fn new() -> Self {
        Self {
            font_name: None,
            font_height_in_points: None,
            italic: None,
            strikeout: None,
            color: None,
            type_offset: None,
            underline: None,
            charset: None,
            bold: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn new_creates_all_none() {
        let style = ExcelFontStyle::new();
        assert!(style.font_name.is_none());
        assert!(style.font_height_in_points.is_none());
        assert!(style.italic.is_none());
        assert!(style.strikeout.is_none());
        assert!(style.color.is_none());
        assert!(style.type_offset.is_none());
        assert!(style.underline.is_none());
        assert!(style.charset.is_none());
        assert!(style.bold.is_none());
    }

    #[test]
    fn partial_eq_compares_all_fields() {
        let a = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            ..ExcelFontStyle::new()
        };
        let b = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            ..ExcelFontStyle::new()
        };
        let c = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(false),
            ..ExcelFontStyle::new()
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn partial_eq_normalizes_nan() {
        let a = ExcelFontStyle {
            font_height_in_points: Some(f64::NAN),
            ..ExcelFontStyle::new()
        };
        let b = ExcelFontStyle {
            font_height_in_points: Some(f64::NAN),
            ..ExcelFontStyle::new()
        };
        assert_eq!(a, b); // NaN 应该等于 NaN（Java 行为）
    }

    #[test]
    fn hash_consistent_with_eq() {
        let a = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            ..ExcelFontStyle::new()
        };
        let b = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            ..ExcelFontStyle::new()
        };
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn hash_normalizes_nan() {
        let a = ExcelFontStyle {
            font_height_in_points: Some(f64::NAN),
            ..ExcelFontStyle::new()
        };
        let b = ExcelFontStyle {
            font_height_in_points: Some(f64::NAN),
            ..ExcelFontStyle::new()
        };
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn debug_format_works() {
        let style = ExcelFontStyle {
            font_name: Some("Arial"),
            ..ExcelFontStyle::new()
        };
        let debug = format!("{:?}", style);
        assert!(debug.contains("Arial"));
    }

    #[test]
    fn clone_preserves_fields() {
        let style = ExcelFontStyle {
            font_name: Some("Arial"),
            bold: Some(true),
            italic: Some(true),
            ..ExcelFontStyle::new()
        };
        let cloned = style;
        assert_eq!(style, cloned);
    }
}
