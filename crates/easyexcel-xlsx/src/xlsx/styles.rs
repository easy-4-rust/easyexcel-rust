//! Parse and emit `xl/styles.xml` (number formats, fonts, fills, borders, cellXfs).

use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use easyexcel_io::Result;
use easyexcel_model::numfmt::builtin_format_code;
use easyexcel_model::styles::{
    BorderEdge, BorderStyle, Borders, CellStyle, Color, Fill, FillPattern, Font, HAlign,
    StyleTable, VAlign,
};

use super::xmlutil::{attr, local_name, local_name_end};

/// First custom number-format id Excel permits.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const FIRST_CUSTOM_NUMFMT_ID: u16 = 164;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse styles.xml. Returns the list of `CellStyle`s, one per `<xf>` in
/// `<cellXfs>` (indexed by the `s=` attribute used on cells).
pub fn parse_styles(xml: &[u8]) -> Result<Vec<CellStyle>> {
    let mut reader = Reader::from_reader(xml);
    let config = reader.config_mut();
    config.trim_text(false);

    // Collected intermediate tables.
    let mut num_fmts: BTreeMap<u16, String> = BTreeMap::new();
    let mut fonts: Vec<Font> = Vec::new();
    let mut fills: Vec<Fill> = Vec::new();
    let mut borders: Vec<Borders> = Vec::new();
    let mut cell_xfs: Vec<RawXf> = Vec::new();

    // Section tracking.
    let mut in_cell_xfs = false; // inside <cellXfs> (vs <cellStyleXfs>)
    let mut in_fonts = false;
    let mut in_fills = false;
    let mut in_borders = false;

    // Current builders.
    let mut cur_font: Option<Font> = None;
    let mut cur_fill: Option<Fill> = None;
    let mut cur_border: Option<Borders> = None;
    let mut cur_border_edge: Option<&'static str> = None; // which edge tag we're inside
    let mut cur_xf: Option<RawXf> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "fonts" => in_fonts = true,
                    "fills" => in_fills = true,
                    "borders" => in_borders = true,
                    "cellXfs" => in_cell_xfs = true,
                    "font" if in_fonts => cur_font = Some(Font::default()),
                    "fill" if in_fills => cur_fill = Some(Fill::default()),
                    "border" if in_borders => cur_border = Some(Borders::default()),
                    "left" | "right" | "top" | "bottom" if cur_border.is_some() => {
                        cur_border_edge = Some(edge_tag(&name));
                        if let Some(b) = cur_border.as_mut() {
                            apply_border_edge(b, cur_border_edge.unwrap(), &e);
                        }
                    }
                    "xf" if in_cell_xfs => cur_xf = Some(parse_xf(&e)),
                    "patternFill" if cur_fill.is_some() => {
                        if let Some(fill) = cur_fill.as_mut() {
                            apply_pattern_fill_empty(fill, &e);
                        }
                    }
                    "fgColor" if cur_fill.is_some() => {
                        if let Some(fill) = cur_fill.as_mut() {
                            fill.fg = parse_color(&e);
                        }
                    }
                    "bgColor" if cur_fill.is_some() => {
                        if let Some(fill) = cur_fill.as_mut() {
                            fill.bg = parse_color(&e);
                        }
                    }
                    "alignment" => {
                        if let Some(xf) = cur_xf.as_mut() {
                            apply_alignment(xf, &e);
                        }
                    }
                    _ => {
                        // nested font props handled via Empty events below; but some
                        // may appear as Start with children. Handle the common ones.
                        if let Some(f) = cur_font.as_mut() {
                            apply_font_prop(f, &name, &e);
                        }
                        if cur_border_edge.is_some()
                            && let (Some(b), Some(edge)) = (cur_border.as_mut(), cur_border_edge)
                            && name == "color"
                        {
                            set_edge_color(b, edge, &e);
                        }
                    }
                }
            }
            Event::Empty(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "numFmt" => {
                        if let (Some(id), Some(code)) =
                            (attr(&e, "numFmtId"), attr(&e, "formatCode"))
                            && let Ok(id) = id.parse::<u16>()
                        {
                            num_fmts.insert(id, code);
                        }
                    }
                    "xf" if in_cell_xfs => cell_xfs.push(parse_xf(&e)),
                    "alignment" => {
                        if let Some(xf) = cur_xf.as_mut() {
                            apply_alignment(xf, &e);
                        }
                    }
                    "left" | "right" | "top" | "bottom" if cur_border.is_some() => {
                        if let Some(b) = cur_border.as_mut() {
                            apply_border_edge(b, edge_tag(&name), &e);
                        }
                    }
                    "color" => {
                        if let Some(f) = cur_font.as_mut() {
                            f.color = parse_color(&e);
                        } else if let (Some(b), Some(edge)) = (cur_border.as_mut(), cur_border_edge)
                        {
                            set_edge_color(b, edge, &e);
                        }
                    }
                    "patternFill" => {
                        if let Some(fl) = cur_fill.as_mut() {
                            apply_pattern_fill_empty(fl, &e);
                        }
                    }
                    "fgColor" => {
                        if let Some(fill) = cur_fill.as_mut() {
                            fill.fg = parse_color(&e);
                        }
                    }
                    "bgColor" => {
                        if let Some(fill) = cur_fill.as_mut() {
                            fill.bg = parse_color(&e);
                        }
                    }
                    _ => {
                        if let Some(f) = cur_font.as_mut() {
                            apply_font_prop(f, &name, &e);
                        }
                    }
                }
            }
            Event::End(e) => {
                let name = local_name_end(&e);
                match name.as_str() {
                    "fonts" => in_fonts = false,
                    "fills" => in_fills = false,
                    "borders" => in_borders = false,
                    "cellXfs" => in_cell_xfs = false,
                    "font" => {
                        if let Some(f) = cur_font.take() {
                            fonts.push(f);
                        }
                    }
                    "fill" => {
                        if let Some(f) = cur_fill.take() {
                            fills.push(f);
                        }
                    }
                    "border" => {
                        if let Some(b) = cur_border.take() {
                            borders.push(b);
                        }
                    }
                    "left" | "right" | "top" | "bottom" => cur_border_edge = None,
                    "xf" if in_cell_xfs => {
                        if let Some(xf) = cur_xf.take() {
                            cell_xfs.push(xf);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    // Resolve cellXfs into CellStyles.
    let mut out = Vec::with_capacity(cell_xfs.len());
    for xf in &cell_xfs {
        let mut style = CellStyle::default();
        if let Some(fid) = xf.font_id
            && let Some(f) = fonts.get(fid)
        {
            style.font = f.clone();
        }
        if let Some(fid) = xf.fill_id
            && let Some(f) = fills.get(fid)
        {
            style.fill = *f;
        }
        if let Some(bid) = xf.border_id
            && let Some(b) = borders.get(bid)
        {
            style.borders = *b;
        }
        let nfid = xf.num_fmt_id.unwrap_or(0);
        style.number_format_id = Some(nfid);
        style.number_format = num_fmts
            .get(&nfid)
            .cloned()
            .or_else(|| builtin_format_code(nfid).map(std::string::ToString::to_string))
            .unwrap_or_default();
        if style.number_format.eq_ignore_ascii_case("general") {
            style.number_format.clear();
        }
        style.halign = xf.halign.unwrap_or_default();
        style.valign = xf.valign.unwrap_or_default();
        style.wrap_text = xf.wrap_text;
        out.push(style);
    }
    Ok(out)
}

#[derive(Default, Clone)]
struct RawXf {
    num_fmt_id: Option<u16>,
    font_id: Option<usize>,
    fill_id: Option<usize>,
    border_id: Option<usize>,
    halign: Option<HAlign>,
    valign: Option<VAlign>,
    wrap_text: bool,
}

fn parse_xf(e: &BytesStart) -> RawXf {
    RawXf {
        num_fmt_id: attr(e, "numFmtId").and_then(|s| s.parse().ok()),
        font_id: attr(e, "fontId").and_then(|s| s.parse().ok()),
        fill_id: attr(e, "fillId").and_then(|s| s.parse().ok()),
        border_id: attr(e, "borderId").and_then(|s| s.parse().ok()),
        halign: None,
        valign: None,
        wrap_text: false,
    }
}

fn apply_alignment(xf: &mut RawXf, e: &BytesStart) {
    if let Some(h) = attr(e, "horizontal") {
        xf.halign = Some(parse_halign(&h));
    }
    if let Some(v) = attr(e, "vertical") {
        xf.valign = Some(parse_valign(&v));
    }
    if let Some(w) = attr(e, "wrapText") {
        xf.wrap_text = w == "1" || w.eq_ignore_ascii_case("true");
    }
}

fn parse_halign(s: &str) -> HAlign {
    match s {
        "left" => HAlign::Left,
        "center" => HAlign::Center,
        "right" => HAlign::Right,
        "fill" => HAlign::Fill,
        "justify" => HAlign::Justify,
        "centerContinuous" => HAlign::CenterContinuous,
        "distributed" => HAlign::Distributed,
        _ => HAlign::General,
    }
}

fn parse_valign(s: &str) -> VAlign {
    match s {
        "top" => VAlign::Top,
        "center" => VAlign::Center,
        "justify" => VAlign::Justify,
        "distributed" => VAlign::Distributed,
        _ => VAlign::Bottom,
    }
}

fn apply_font_prop(f: &mut Font, name: &str, e: &BytesStart) {
    match name {
        "b" => f.bold = bool_attr_default_true(e),
        "i" => f.italic = bool_attr_default_true(e),
        "u" => f.underline = true,
        "strike" => f.strike = bool_attr_default_true(e),
        "sz" => {
            if let Some(v) = attr(e, "val").and_then(|s| s.parse().ok()) {
                f.size = v;
            }
        }
        "name" | "rFont" => {
            if let Some(v) = attr(e, "val") {
                f.name = v;
            }
        }
        "color" => f.color = parse_color(e),
        _ => {}
    }
}

fn bool_attr_default_true(e: &BytesStart) -> bool {
    match attr(e, "val") {
        Some(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        None => true,
    }
}

fn parse_color(e: &BytesStart) -> Color {
    if let Some(rgb) = attr(e, "rgb")
        && let Ok(v) = u32::from_str_radix(rgb.trim(), 16)
    {
        return Color::rgb(v);
    }
    Color::AUTO
}

fn edge_tag(name: &str) -> &'static str {
    match name {
        "left" => "left",
        "right" => "right",
        "top" => "top",
        _ => "bottom",
    }
}

fn apply_border_edge(b: &mut Borders, edge: &str, e: &BytesStart) {
    let style = attr(e, "style").map_or(BorderStyle::None, |s| parse_border_style(&s));
    let target = match edge {
        "left" => &mut b.left,
        "right" => &mut b.right,
        "top" => &mut b.top,
        _ => &mut b.bottom,
    };
    target.style = style;
}

fn set_edge_color(b: &mut Borders, edge: &str, e: &BytesStart) {
    let color = parse_color(e);
    let target = match edge {
        "left" => &mut b.left,
        "right" => &mut b.right,
        "top" => &mut b.top,
        _ => &mut b.bottom,
    };
    target.color = color;
}

fn parse_border_style(s: &str) -> BorderStyle {
    match s {
        "medium" => BorderStyle::Medium,
        "thick" => BorderStyle::Thick,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        "hair" => BorderStyle::Hair,
        "none" => BorderStyle::None,
        _ => BorderStyle::Thin,
    }
}

fn apply_pattern_fill_empty(fl: &mut Fill, e: &BytesStart) {
    if let Some(p) = attr(e, "patternType") {
        fl.pattern = parse_pattern(&p);
    }
}

fn parse_pattern(s: &str) -> FillPattern {
    match s {
        "none" => FillPattern::None,
        "solid" => FillPattern::Solid,
        "gray125" => FillPattern::Gray125,
        _ => FillPattern::Other(0),
    }
}

// ----------------------------------------------------------------------------
// Writing
// ----------------------------------------------------------------------------

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Serialize a `StyleTable` to a styles.xml byte buffer. Returns the bytes plus
/// a map from style-table index → cellXfs index (they are 1:1 here, but the
/// default xf at index 0 stays index 0).
pub fn write_styles(table: &StyleTable) -> Vec<u8> {
    use std::collections::HashMap;
    use std::fmt::Write as _;

    // Collect distinct custom number formats (id >= 164).
    let mut custom_fmts: Vec<(u16, String)> = Vec::new();
    let mut fmt_id_for_style: Vec<u16> = Vec::with_capacity(table.len());
    let mut next_custom = FIRST_CUSTOM_NUMFMT_ID;
    let mut seen_custom: HashMap<String, u16> = HashMap::new();

    for st in table.iter() {
        let id = number_format_id_for(st, &mut next_custom, &mut seen_custom, &mut custom_fmts);
        fmt_id_for_style.push(id);
    }

    // Distinct fonts, fills, borders.
    let mut fonts: Vec<Font> = Vec::new();
    let mut fills: Vec<Fill> = vec![
        Fill {
            pattern: FillPattern::None,
            ..Default::default()
        },
        Fill {
            pattern: FillPattern::Gray125,
            ..Default::default()
        },
    ];
    let mut borders: Vec<Borders> = Vec::new();
    let mut font_idx: Vec<usize> = Vec::with_capacity(table.len());
    let mut fill_idx: Vec<usize> = Vec::with_capacity(table.len());
    let mut border_idx: Vec<usize> = Vec::with_capacity(table.len());

    for st in table.iter() {
        font_idx.push(index_of_or_push(&mut fonts, &st.font));
        fill_idx.push(index_of_or_push_copy(&mut fills, st.fill));
        border_idx.push(index_of_or_push_copy(&mut borders, st.borders));
    }

    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#);

    // numFmts
    if !custom_fmts.is_empty() {
        let _ = write!(s, r#"<numFmts count="{}">"#, custom_fmts.len());
        for (id, code) in &custom_fmts {
            let _ = write!(
                s,
                r#"<numFmt numFmtId="{}" formatCode="{}"/>"#,
                id,
                escape(code)
            );
        }
        s.push_str("</numFmts>");
    }

    // fonts
    let _ = write!(s, r#"<fonts count="{}">"#, fonts.len());
    for f in &fonts {
        write_font(&mut s, f);
    }
    s.push_str("</fonts>");

    // fills
    let _ = write!(s, r#"<fills count="{}">"#, fills.len());
    for f in &fills {
        write_fill(&mut s, f);
    }
    s.push_str("</fills>");

    // borders
    let _ = write!(s, r#"<borders count="{}">"#, borders.len());
    for b in &borders {
        write_border(&mut s, b);
    }
    s.push_str("</borders>");

    // cellStyleXfs (one default)
    s.push_str(
        r#"<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>"#,
    );

    // cellXfs
    let _ = write!(s, r#"<cellXfs count="{}">"#, table.len());
    for (i, st) in table.iter().enumerate() {
        let nfid = fmt_id_for_style[i];
        let _ = write!(
            s,
            r#"<xf numFmtId="{}" fontId="{}" fillId="{}" borderId="{}" xfId="0""#,
            nfid, font_idx[i], fill_idx[i], border_idx[i]
        );
        if nfid != 0 {
            s.push_str(r#" applyNumberFormat="1""#);
        }
        if font_idx[i] != 0 {
            s.push_str(r#" applyFont="1""#);
        }
        if fill_idx[i] != 0 {
            s.push_str(r#" applyFill="1""#);
        }
        if border_idx[i] != 0 {
            s.push_str(r#" applyBorder="1""#);
        }
        let has_align = st.halign != HAlign::General || st.valign != VAlign::Bottom || st.wrap_text;
        if has_align {
            s.push_str(r#" applyAlignment="1">"#);
            s.push_str("<alignment");
            if st.halign != HAlign::General {
                let _ = write!(s, r#" horizontal="{}""#, halign_str(st.halign));
            }
            if st.valign != VAlign::Bottom {
                let _ = write!(s, r#" vertical="{}""#, valign_str(st.valign));
            }
            if st.wrap_text {
                s.push_str(r#" wrapText="1""#);
            }
            s.push_str("/></xf>");
        } else {
            s.push_str("/>");
        }
    }
    s.push_str("</cellXfs>");

    s.push_str(
        r#"<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>"#,
    );
    s.push_str("</styleSheet>");
    s.into_bytes()
}

fn number_format_id_for(
    st: &CellStyle,
    next_custom: &mut u16,
    seen: &mut std::collections::HashMap<String, u16>,
    custom: &mut Vec<(u16, String)>,
) -> u16 {
    let code = st.number_format.trim();
    if code.is_empty() || code.eq_ignore_ascii_case("general") {
        return 0;
    }
    // Prefer a known builtin id if the original was a builtin.
    if let Some(id) = st.number_format_id
        && id < FIRST_CUSTOM_NUMFMT_ID
        && let Some(bc) = builtin_format_code(id)
        && bc.eq_ignore_ascii_case(code)
    {
        return id;
    }
    // Match against builtins by code.
    for id in 0u16..=49 {
        if let Some(bc) = builtin_format_code(id)
            && bc == code
        {
            return id;
        }
    }
    // Custom format.
    if let Some(&id) = seen.get(code) {
        return id;
    }
    let id = *next_custom;
    *next_custom += 1;
    seen.insert(code.to_string(), id);
    custom.push((id, code.to_string()));
    id
}

fn index_of_or_push(v: &mut Vec<Font>, item: &Font) -> usize {
    if let Some(i) = v.iter().position(|x| x == item) {
        i
    } else {
        v.push(item.clone());
        v.len() - 1
    }
}

fn index_of_or_push_copy<T: PartialEq + Copy>(v: &mut Vec<T>, item: T) -> usize {
    if let Some(i) = v.iter().position(|x| *x == item) {
        i
    } else {
        v.push(item);
        v.len() - 1
    }
}

fn write_font(s: &mut String, f: &Font) {
    use std::fmt::Write as _;
    s.push_str("<font>");
    if f.bold {
        s.push_str("<b/>");
    }
    if f.italic {
        s.push_str("<i/>");
    }
    if f.underline {
        s.push_str("<u/>");
    }
    if f.strike {
        s.push_str("<strike/>");
    }
    let _ = write!(s, r#"<sz val="{}"/>"#, fmt_f64(f.size));
    if let Some(argb) = f.color.0 {
        let _ = write!(s, r#"<color rgb="{argb:08X}"/>"#);
    }
    let _ = write!(s, r#"<name val="{}"/>"#, escape(&f.name));
    s.push_str("</font>");
}

fn write_fill(s: &mut String, f: &Fill) {
    use std::fmt::Write as _;
    let pt = match f.pattern {
        FillPattern::None | FillPattern::Other(_) => "none",
        FillPattern::Solid => "solid",
        FillPattern::Gray125 => "gray125",
    };
    if let (true, Some(fg)) = (matches!(f.pattern, FillPattern::Solid), f.fg.0) {
        let _ = write!(
            s,
            r#"<fill><patternFill patternType="solid"><fgColor rgb="{fg:08X}"/></patternFill></fill>"#
        );
    } else {
        let _ = write!(s, r#"<fill><patternFill patternType="{pt}"/></fill>"#);
    }
}

fn write_border(s: &mut String, b: &Borders) {
    s.push_str("<border>");
    write_edge(s, "left", &b.left);
    write_edge(s, "right", &b.right);
    write_edge(s, "top", &b.top);
    write_edge(s, "bottom", &b.bottom);
    s.push_str("<diagonal/>");
    s.push_str("</border>");
}

fn write_edge(s: &mut String, tag: &str, edge: &BorderEdge) {
    use std::fmt::Write as _;
    let style = match edge.style {
        BorderStyle::None => {
            let _ = write!(s, "<{tag}/>");
            return;
        }
        BorderStyle::Thin => "thin",
        BorderStyle::Medium => "medium",
        BorderStyle::Thick => "thick",
        BorderStyle::Dashed => "dashed",
        BorderStyle::Dotted => "dotted",
        BorderStyle::Double => "double",
        BorderStyle::Hair => "hair",
    };
    let _ = write!(s, r#"<{tag} style="{style}">"#);
    if let Some(argb) = edge.color.0 {
        let _ = write!(s, r#"<color rgb="{argb:08X}"/>"#);
    }
    let _ = write!(s, "</{tag}>");
}

fn halign_str(h: HAlign) -> &'static str {
    match h {
        HAlign::General => "general",
        HAlign::Left => "left",
        HAlign::Center => "center",
        HAlign::Right => "right",
        HAlign::Fill => "fill",
        HAlign::Justify => "justify",
        HAlign::CenterContinuous => "centerContinuous",
        HAlign::Distributed => "distributed",
    }
}

fn valign_str(v: VAlign) -> &'static str {
    match v {
        VAlign::Top => "top",
        VAlign::Bottom => "bottom",
        VAlign::Center => "center",
        VAlign::Justify => "justify",
        VAlign::Distributed => "distributed",
    }
}

// XML 数字格式要求无小数部分的有限值按整数文本输出；边界判断保证转换可表示。
#[allow(clippy::cast_possible_truncation)]
fn fmt_f64(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 && v.abs() < 9e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn escape(s: &str) -> String {
    super::xmlutil::xml_escape(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 空样式表解析。
    #[test]
    fn parse_styles_empty_xfs() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
                <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].number_format, "");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 带自定义数字格式的样式解析。
    #[test]
    fn parse_styles_with_custom_number_format() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <numFmts count="1"><numFmt numFmtId="164" formatCode="yyyy-mm-dd"/></numFmts>
                <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="164" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
                <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].number_format, "yyyy-mm-dd");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 带粗体字体的样式解析。
    #[test]
    fn parse_styles_with_bold_font() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="2">
                    <font><sz val="11"/><name val="Calibri"/></font>
                    <font><b/><sz val="11"/><name val="Calibri"/></font>
                </fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0"/></cellXfs>
                <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert!(styles[0].font.bold);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 write_styles roundtrip 测试。
    #[test]
    fn write_styles_roundtrip() {
        let mut table = StyleTable::default();
        let mut st = CellStyle::default();
        st.font.bold = true;
        st.font.size = 14.0;
        st.halign = HAlign::Center;
        st.number_format = "0.00".into();
        let _ = table.intern(st);

        let xml = write_styles(&table);
        let parsed = parse_styles(&xml).unwrap();
        // 默认样式 + 我们添加的样式
        assert!(parsed.len() >= 1);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 fmt_f64 整数路径。
    #[test]
    fn fmt_f64_integer_path() {
        assert_eq!(fmt_f64(42.0), "42");
        assert_eq!(fmt_f64(0.0), "0");
        assert_eq!(fmt_f64(-10.0), "-10");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 fmt_f64 小数路径。
    #[test]
    fn fmt_f64_decimal_path() {
        assert_eq!(fmt_f64(3.14), "3.14");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 fmt_f64 非有限值路径。
    #[test]
    fn fmt_f64_non_finite() {
        assert_eq!(fmt_f64(f64::INFINITY), "inf");
        assert_eq!(fmt_f64(f64::NAN), "NaN");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 halign_str 各变体。
    #[test]
    fn halign_str_all_variants() {
        assert_eq!(halign_str(HAlign::General), "general");
        assert_eq!(halign_str(HAlign::Left), "left");
        assert_eq!(halign_str(HAlign::Center), "center");
        assert_eq!(halign_str(HAlign::Right), "right");
        assert_eq!(halign_str(HAlign::Fill), "fill");
        assert_eq!(halign_str(HAlign::Justify), "justify");
        assert_eq!(halign_str(HAlign::CenterContinuous), "centerContinuous");
        assert_eq!(halign_str(HAlign::Distributed), "distributed");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 valign_str 各变体。
    #[test]
    fn valign_str_all_variants() {
        assert_eq!(valign_str(VAlign::Top), "top");
        assert_eq!(valign_str(VAlign::Bottom), "bottom");
        assert_eq!(valign_str(VAlign::Center), "center");
        assert_eq!(valign_str(VAlign::Justify), "justify");
        assert_eq!(valign_str(VAlign::Distributed), "distributed");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 parse_styles 带有 italic/underline 字体。
    #[test]
    fn parse_styles_with_italic_underline_font() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="2">
                    <font><sz val="11"/><name val="Calibri"/></font>
                    <font><b/><i/><u/><sz val="12"/><name val="Arial"/><color rgb="FFFF0000"/></font>
                </fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0"/></cellXfs>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert!(styles[0].font.bold);
        assert!(styles[0].font.italic);
        assert!(styles[0].font.underline);
        assert_eq!(styles[0].font.name, "Arial");
        assert_eq!(styles[0].font.size, 12.0);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 parse_styles 带有 solid 填充和颜色。
    #[test]
    fn parse_styles_with_solid_fill_color() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="solid"><fgColor rgb="FF00FF00"/></patternFill></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="1" borderId="0" xfId="0"/></cellXfs>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert!(styles[0].fill.fg.0.is_some());
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 parse_styles 带有边框。
    #[test]
    fn parse_styles_with_borders() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="2">
                    <border><left/><right/><top/><bottom/><diagonal/></border>
                    <border>
                        <left style="thin"><color rgb="FF000000"/></left>
                        <right style="medium"><color rgb="FFFF0000"/></right>
                        <top style="thick"/>
                        <bottom style="dashed"/>
                        <diagonal/>
                    </border>
                </borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0"/></cellXfs>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].borders.left.style, BorderStyle::Thin);
        assert_eq!(styles[0].borders.right.style, BorderStyle::Medium);
        assert_eq!(styles[0].borders.top.style, BorderStyle::Thick);
        assert_eq!(styles[0].borders.bottom.style, BorderStyle::Dashed);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 parse_styles 带有对齐。
    #[test]
    fn parse_styles_with_alignment() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
                <fills count="2">
                    <fill><patternFill patternType="none"/></fill>
                    <fill><patternFill patternType="gray125"/></fill>
                </fills>
                <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
                <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
                <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0" applyAlignment="1">
                    <alignment horizontal="center" vertical="bottom" wrapText="1"/>
                </xf></cellXfs>
            </styleSheet>"#;
        let styles = parse_styles(xml).unwrap();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].halign, HAlign::Center);
        assert_eq!(styles[0].valign, VAlign::Bottom);
        assert!(styles[0].wrap_text);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 write_styles 含边框。
    #[test]
    fn write_styles_with_border() {
        let mut table = StyleTable::default();
        let mut st = CellStyle::default();
        st.borders.left.style = BorderStyle::Thin;
        st.borders.right.style = BorderStyle::Medium;
        st.borders.top.style = BorderStyle::Thick;
        st.borders.bottom.style = BorderStyle::Dashed;
        let _ = table.intern(st);
        let xml = String::from_utf8(write_styles(&table)).unwrap();
        assert!(xml.contains("thin"));
        assert!(xml.contains("medium"));
        assert!(xml.contains("thick"));
        assert!(xml.contains("dashed"));
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 write_styles 含 solid 填充。
    #[test]
    fn write_styles_with_solid_fill() {
        let mut table = StyleTable::default();
        let mut st = CellStyle::default();
        st.fill.pattern = FillPattern::Solid;
        st.fill.fg = Color(Some(0xFF00FF00));
        let _ = table.intern(st);
        let xml = String::from_utf8(write_styles(&table)).unwrap();
        assert!(xml.contains("solid"));
        assert!(xml.contains("FF00FF00"));
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 测试 write_styles 含对齐。
    #[test]
    fn write_styles_with_alignment() {
        let mut table = StyleTable::default();
        let mut st = CellStyle::default();
        st.halign = HAlign::Right;
        st.valign = VAlign::Top;
        st.wrap_text = true;
        let _ = table.intern(st);
        let xml = String::from_utf8(write_styles(&table)).unwrap();
        assert!(xml.contains("right"));
        assert!(xml.contains("top"));
        assert!(xml.contains("wrapText"));
    }
}
