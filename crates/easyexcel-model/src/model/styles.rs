//! Cell styling: fonts, fills, borders, number formats, alignment, and the
//! deduplicating style table that the readers/writers index into.

include!("styles/h_align.rs");

include!("styles/v_align.rs");

include!("styles/color.rs");

include!("styles/font.rs");

include!("styles/fill.rs");

include!("styles/fill_pattern.rs");

include!("styles/border_style.rs");

include!("styles/border_edge.rs");

include!("styles/borders.rs");

include!("styles/cell_style.rs");

include!("styles/style_table.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup() {
        let mut t = StyleTable::default();
        let mut s = CellStyle::default();
        s.font.bold = true;
        let a = t.intern(s.clone());
        let b = t.intern(s);
        assert_eq!(a, b);
        assert_eq!(t.len(), 2); // default + bold
    }

    #[test]
    fn date_detection() {
        let mut s = CellStyle {
            number_format: "yyyy-mm-dd".into(),
            ..Default::default()
        };
        assert!(s.is_date());
        s.number_format = "0.00".into();
        s.number_format_id = Some(2);
        assert!(!s.is_date());
    }
}
