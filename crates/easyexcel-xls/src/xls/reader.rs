//! XLS (BIFF8) reader.
//!
//! Reads the OLE2/CFB container, extracts the `Workbook` (or `Book`) stream,
//! parses the globals substream (date mode, formats, fonts, XFs, SST) and then
//! each worksheet substream's cell records into the [`Workbook`] model.

use std::io::{Read, Seek};

use easyexcel_io::{Error, Result};
use easyexcel_model::CellError;
use easyexcel_model::addr::{CellAddress, CellRange};
use easyexcel_model::dates::DateSystem;
use easyexcel_model::model::{Cell, FrozenPanes, OpaquePart, Sheet, Visibility, Workbook};
use easyexcel_model::styles::{CellStyle, HAlign};
use easyexcel_model::value::CellValue;

use super::biff::{self, RawRecord, Records};
use super::sst;

/// Read an XLS workbook from any seekable reader.
pub fn read<R: Read + Seek>(reader: R) -> Result<Workbook> {
    let mut cf = cfb::CompoundFile::open(reader)
        .map_err(|e| Error::Cfb(format!("not a valid OLE2 file: {e}")))?;

    // Collect names of all streams so we can grab the workbook and preserve the
    // rest as opaque parts (best-effort round-trip).
    let mut stream_names: Vec<String> = Vec::new();
    for entry in cf.walk() {
        if entry.is_stream() && !entry.is_root() {
            stream_names.push(entry.path().to_string_lossy().to_string());
        }
    }

    // The workbook stream is named "Workbook" (BIFF8) or "Book" (older). Names
    // in CFB are case-insensitive-ish; match exactly first then case-insens.
    let wb_name = pick_workbook_stream(&stream_names)
        .ok_or_else(|| Error::Xls("no Workbook or Book stream found".into()))?;

    let mut wb_bytes = Vec::new();
    {
        let mut s = cf
            .open_stream(&wb_name)
            .map_err(|e| Error::Cfb(format!("cannot open workbook stream: {e}")))?;
        s.read_to_end(&mut wb_bytes)?;
    }

    // Preserve other streams verbatim as workbook-level opaque parts.
    let mut opaque = Vec::new();
    for name in &stream_names {
        if name == &wb_name {
            continue;
        }
        if let Ok(mut s) = cf.open_stream(name) {
            let mut buf = Vec::new();
            if s.read_to_end(&mut buf).is_ok() {
                opaque.push(OpaquePart {
                    name: name.clone(),
                    data: buf,
                });
            }
        }
    }

    let mut wb = parse_workbook_stream(&wb_bytes)?;
    wb.opaque = opaque;
    Ok(wb)
}

fn pick_workbook_stream(names: &[String]) -> Option<String> {
    // Prefer "Workbook", then "Book". Stream paths from `walk` are like
    // "/Workbook".
    let want = |target: &str| {
        names.iter().find(|n| {
            let base = n.trim_start_matches('/');
            base.eq_ignore_ascii_case(target)
        })
    };
    want("Workbook").or_else(|| want("Book")).cloned()
}

/// Per-XF style information gathered from the globals substream.
struct XfInfo {
    ifmt: u16,
    halign: HAlign,
    wrap: bool,
}

/// State accumulated while parsing the globals substream.
#[derive(Default)]
struct Globals {
    date_system: DateSystem,
    sst: Vec<String>,
    /// ifmt -> format code string (built-ins + custom).
    formats: std::collections::HashMap<u16, String>,
    xfs: Vec<XfInfo>,
    /// (name, stream byte offset, visibility, is_worksheet).
    boundsheets: Vec<BoundSheet>,
}

struct BoundSheet {
    name: String,
    pos: usize,
    visibility: Visibility,
    is_worksheet: bool,
}

fn parse_workbook_stream(buf: &[u8]) -> Result<Workbook> {
    let mut records = Records::new(buf);

    // The first record must be a BOF for the globals substream.
    let first = records
        .next()
        .ok_or_else(|| Error::Xls("empty workbook stream".into()))?;
    if first.typ != biff::BOF {
        return Err(Error::Xls("workbook does not start with BOF".into()));
    }

    let mut globals = Globals {
        date_system: DateSystem::Date1900,
        ..Default::default()
    };

    // Parse globals until its EOF.
    for rec in records.by_ref() {
        match rec.typ {
            biff::EOF => break,
            biff::FILEPASS => {
                return Err(Error::PasswordProtected(
                    "legacy XLS (BIFF8) RC4/XOR encryption; not supported".to_string(),
                ));
            }
            biff::DATEMODE if rec.data.len() >= 2 && biff::u16le(&rec.data, 0) == 1 => {
                globals.date_system = DateSystem::Date1904;
            }
            biff::FORMAT => parse_format(&rec, &mut globals),
            biff::XF => parse_xf(&rec, &mut globals),
            biff::SST => {
                globals.sst = sst::parse_sst(&rec.data, &rec.continue_breaks);
            }
            biff::BOUNDSHEET => parse_boundsheet(&rec, &mut globals),
            _ => {}
        }
    }

    // Build the workbook with styles interned. We keep an XF-index -> interned
    // style-index map.
    let mut wb = Workbook::empty();
    wb.date_system = globals.date_system;
    let xf_to_style = build_styles(&globals, &mut wb);

    // Parse each worksheet substream at its recorded offset.
    for bs in &globals.boundsheets {
        if !bs.is_worksheet {
            // Chart / macro sheets: add a placeholder sheet to keep indices, but
            // skip cell parsing.
            let mut sheet = Sheet::new(bs.name.clone());
            sheet.visibility = bs.visibility;
            wb.sheets.push(sheet);
            continue;
        }
        let mut sheet = Sheet::new(bs.name.clone());
        sheet.visibility = bs.visibility;
        parse_worksheet(buf, bs.pos, &globals, &xf_to_style, &mut sheet);
        wb.sheets.push(sheet);
    }

    if wb.sheets.is_empty() {
        // Degenerate file; give it an empty sheet so downstream code is happy.
        wb.sheets.push(Sheet::new("Sheet1"));
    }

    Ok(wb)
}

fn parse_format(rec: &RawRecord, g: &mut Globals) {
    if rec.data.len() < 2 {
        return;
    }
    let ifmt = biff::u16le(&rec.data, 0);
    let (s, _) = parse_biff8_string_u16len(&rec.data, 2);
    g.formats.insert(ifmt, s);
}

/// Parse an XLUnicodeString with a 2-byte char count (as used inside FORMAT and
/// LABEL records). Does not handle CONTINUE (these are short enough not to).
fn parse_biff8_string_u16len(d: &[u8], off: usize) -> (String, usize) {
    if off + 3 > d.len() {
        return (String::new(), off);
    }
    let cch = biff::u16le(d, off) as usize;
    let grbit = d[off + 2];
    let compressed = grbit & 0x01 == 0;
    let mut p = off + 3;
    let s = if compressed {
        let mut out = String::with_capacity(cch);
        for _ in 0..cch {
            if p >= d.len() {
                break;
            }
            out.push(d[p] as char);
            p += 1;
        }
        out
    } else {
        let mut units = Vec::with_capacity(cch);
        for _ in 0..cch {
            if p + 1 >= d.len() {
                break;
            }
            units.push(biff::u16le(d, p));
            p += 2;
        }
        String::from_utf16_lossy(&units)
    };
    (s, p)
}

fn parse_xf(rec: &RawRecord, g: &mut Globals) {
    // XF: ifnt(u16), ifmt(u16), attr(u16), align(u8: bits0-2 halign, bit3 wrap),
    // rotation(u8), ... We only need ifmt + alignment.
    let d = &rec.data;
    if d.len() < 6 {
        g.xfs.push(XfInfo {
            ifmt: 0,
            halign: HAlign::General,
            wrap: false,
        });
        return;
    }
    let ifmt = biff::u16le(d, 2);
    let align = d.get(6).copied().unwrap_or(0);
    let halign = match align & 0x07 {
        1 => HAlign::Left,
        2 => HAlign::Center,
        3 => HAlign::Right,
        4 => HAlign::Fill,
        5 => HAlign::Justify,
        6 => HAlign::CenterContinuous,
        7 => HAlign::Distributed,
        _ => HAlign::General,
    };
    let wrap = align & 0x08 != 0;
    g.xfs.push(XfInfo { ifmt, halign, wrap });
}

fn parse_boundsheet(rec: &RawRecord, g: &mut Globals) {
    // lbPlyPos(u32), grbit(u16: low byte visibility, high byte type), name(short str)
    let d = &rec.data;
    if d.len() < 6 {
        return;
    }
    let pos = biff::u32le(d, 0) as usize;
    let hidden_state = d[4]; // 0=visible, 1=hidden, 2=very hidden
    let sheet_type = d[5]; // 0=worksheet, 1=macro, 2=chart, 6=vb module
    let visibility = match hidden_state & 0x03 {
        1 => Visibility::Hidden,
        2 => Visibility::VeryHidden,
        _ => Visibility::Visible,
    };
    let (name, _) = biff::parse_short_unicode_string(d, 6);
    g.boundsheets.push(BoundSheet {
        name,
        pos,
        visibility,
        is_worksheet: sheet_type == 0,
    });
}

/// Standard BIFF built-in number format codes (subset that matters for date
/// detection and display). IDs not listed fall back to "General" or a custom
/// FORMAT record if present.
fn builtin_format(ifmt: u16) -> Option<&'static str> {
    Some(match ifmt {
        0 => "General",
        1 => "0",
        2 => "0.00",
        3 => "#,##0",
        4 => "#,##0.00",
        5 => "$#,##0_);($#,##0)",
        6 => "$#,##0_);[Red]($#,##0)",
        7 => "$#,##0.00_);($#,##0.00)",
        8 => "$#,##0.00_);[Red]($#,##0.00)",
        9 => "0%",
        10 => "0.00%",
        11 => "0.00E+00",
        12 => "# ?/?",
        13 => "# ??/??",
        14 => "m/d/yy",
        15 => "d-mmm-yy",
        16 => "d-mmm",
        17 => "mmm-yy",
        18 => "h:mm AM/PM",
        19 => "h:mm:ss AM/PM",
        20 => "h:mm",
        21 => "h:mm:ss",
        22 => "m/d/yy h:mm",
        37 => "#,##0_);(#,##0)",
        38 => "#,##0_);[Red](#,##0)",
        39 => "#,##0.00_);(#,##0.00)",
        40 => "#,##0.00_);[Red](#,##0.00)",
        45 => "mm:ss",
        46 => "[h]:mm:ss",
        47 => "mm:ss.0",
        48 => "##0.0E+0",
        49 => "@",
        _ => return None,
    })
}

/// Build interned styles from the XF list, returning xf-index -> style-index.
fn build_styles(g: &Globals, wb: &mut Workbook) -> Vec<u32> {
    let mut map = Vec::with_capacity(g.xfs.len());
    for xf in &g.xfs {
        let mut style = CellStyle::default();
        let fmt = g
            .formats
            .get(&xf.ifmt)
            .cloned()
            .or_else(|| builtin_format(xf.ifmt).map(|s| s.to_string()))
            .unwrap_or_default();
        if !fmt.eq_ignore_ascii_case("general") {
            style.number_format = fmt;
        }
        style.number_format_id = Some(xf.ifmt);
        style.halign = xf.halign;
        style.wrap_text = xf.wrap;
        map.push(wb.styles.intern(style));
    }
    map
}

/// Look up the interned style index for an XF index (falling back to default).
fn style_for_xf(xf_to_style: &[u32], xf: u16) -> Option<u32> {
    xf_to_style.get(xf as usize).copied()
}

/// Apply a cell's XF to the sheet's style map (only if non-default).
fn apply_style(sheet: &mut Sheet, row: u32, col: u32, xf: u16, xf_to_style: &[u32]) {
    if let Some(idx) = style_for_xf(xf_to_style, xf)
        && idx != 0
    {
        sheet.set_style(row, col, idx);
    }
}

fn parse_worksheet(buf: &[u8], start: usize, g: &Globals, xf_to_style: &[u32], sheet: &mut Sheet) {
    if start >= buf.len() {
        return;
    }
    let mut records = Records::new(&buf[start..]);

    // First record should be the sheet BOF.
    let first = match records.next() {
        Some(r) => r,
        None => return,
    };
    if first.typ != biff::BOF {
        return;
    }

    // Track a pending FORMULA whose result is a string (followed by STRING).
    let mut pending_string_formula: Option<(u32, u32)> = None;

    for rec in records.by_ref() {
        match rec.typ {
            biff::EOF => break,
            biff::NUMBER => {
                if let Some((r, c, xf)) = cell_header(&rec.data)
                    && rec.data.len() >= 14
                {
                    let v = biff::f64le(&rec.data, 6);
                    sheet.set(r, c, Cell::Number(v));
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::RK => {
                if let Some((r, c, xf)) = cell_header(&rec.data)
                    && rec.data.len() >= 10
                {
                    let v = biff::decode_rk(biff::u32le(&rec.data, 6));
                    sheet.set(r, c, Cell::Number(v));
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::MULRK => parse_mulrk(&rec, xf_to_style, sheet),
            biff::LABELSST => {
                if let Some((r, c, xf)) = cell_header(&rec.data)
                    && rec.data.len() >= 10
                {
                    let idx = biff::u32le(&rec.data, 6) as usize;
                    let text = g.sst.get(idx).cloned().unwrap_or_default();
                    sheet.set(r, c, Cell::Text(text));
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::LABEL => {
                if let Some((r, c, xf)) = cell_header(&rec.data) {
                    let (text, _) = parse_biff8_string_u16len(&rec.data, 6);
                    sheet.set(r, c, Cell::Text(text));
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::BOOLERR => {
                if let Some((r, c, xf)) = cell_header(&rec.data)
                    && rec.data.len() >= 8
                {
                    let val = rec.data[6];
                    let is_err = rec.data[7] != 0;
                    let cell = if is_err {
                        Cell::Error(CellError::from_biff_code(val))
                    } else {
                        Cell::Bool(val != 0)
                    };
                    sheet.set(r, c, cell);
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::BLANK => {
                if let Some((r, c, xf)) = cell_header(&rec.data) {
                    apply_style(sheet, r, c, xf, xf_to_style);
                }
            }
            biff::MULBLANK => parse_mulblank(&rec, xf_to_style, sheet),
            biff::FORMULA => {
                if let Some((r, c, xf, cell, is_str)) = parse_formula(&rec.data) {
                    sheet.set(r, c, cell);
                    apply_style(sheet, r, c, xf, xf_to_style);
                    if is_str {
                        pending_string_formula = Some((r, c));
                    }
                }
            }
            biff::STRING => {
                // Cached string result for the immediately-preceding FORMULA.
                if let Some((r, c)) = pending_string_formula.take() {
                    let (text, _) = parse_biff8_string_u16len(&rec.data, 0);
                    if let Some(Cell::Formula { expr, .. }) = sheet.get(r, c).cloned() {
                        sheet.set(
                            r,
                            c,
                            Cell::Formula {
                                expr,
                                cached: CellValue::Text(text),
                            },
                        );
                    }
                }
            }
            biff::MERGECELLS => parse_mergecells(&rec, sheet),
            biff::WINDOW2 => parse_window2(&rec, sheet),
            biff::PANE => parse_pane(&rec, sheet),
            _ => {}
        }
    }
}

/// Read the shared 6-byte cell header (row, col, xf).
fn cell_header(d: &[u8]) -> Option<(u32, u32, u16)> {
    if d.len() < 6 {
        return None;
    }
    let row = biff::u16le(d, 0) as u32;
    let col = biff::u16le(d, 2) as u32;
    let xf = biff::u16le(d, 4);
    Some((row, col, xf))
}

fn parse_mulrk(rec: &RawRecord, xf_to_style: &[u32], sheet: &mut Sheet) {
    let d = &rec.data;
    if d.len() < 6 {
        return;
    }
    let row = biff::u16le(d, 0) as u32;
    let first_col = biff::u16le(d, 2) as u32;
    // Last 2 bytes are the last column index.
    if d.len() < 8 {
        return;
    }
    let last_col = biff::u16le(d, d.len() - 2) as u32;
    let count = (last_col - first_col + 1) as usize;
    // Each entry: xf(u16) + rk(u32) = 6 bytes, starting at offset 4.
    let mut p = 4;
    for i in 0..count {
        if p + 6 > d.len() - 2 {
            break;
        }
        let xf = biff::u16le(d, p);
        let rk = biff::u32le(d, p + 2);
        p += 6;
        let col = first_col + i as u32;
        sheet.set(row, col, Cell::Number(biff::decode_rk(rk)));
        apply_style(sheet, row, col, xf, xf_to_style);
    }
}

fn parse_mulblank(rec: &RawRecord, xf_to_style: &[u32], sheet: &mut Sheet) {
    let d = &rec.data;
    if d.len() < 6 {
        return;
    }
    let row = biff::u16le(d, 0) as u32;
    let first_col = biff::u16le(d, 2) as u32;
    let last_col = biff::u16le(d, d.len() - 2) as u32;
    let count = (last_col - first_col + 1) as usize;
    let mut p = 4;
    for i in 0..count {
        if p + 2 > d.len() - 2 {
            break;
        }
        let xf = biff::u16le(d, p);
        p += 2;
        apply_style(sheet, row, first_col + i as u32, xf, xf_to_style);
    }
}

/// Parse a FORMULA record. Returns (row, col, xf, cell, result_is_string).
/// The cached value is decoded from the 8-byte result field. We store the
/// formula with a placeholder expr (RPN-to-text decoding is out of scope —
/// see the PARITY note in the writer).
fn parse_formula(d: &[u8]) -> Option<(u32, u32, u16, Cell, bool)> {
    if d.len() < 20 {
        return None;
    }
    let (row, col, xf) = cell_header(d)?;
    // Result is bytes 6..14. If bytes[12..14] == 0xFFFF it's a special type.
    let result = &d[6..14];
    let mut is_string = false;
    let cached = if result[6] == 0xFF && result[7] == 0xFF {
        match result[0] {
            0 => {
                // String — value supplied by a following STRING record.
                is_string = true;
                CellValue::Text(String::new())
            }
            1 => CellValue::Bool(result[2] != 0),
            2 => CellValue::Error(CellError::from_biff_code(result[2])),
            3 => CellValue::Empty,
            _ => CellValue::Empty,
        }
    } else {
        CellValue::Number(biff::f64le(d, 6))
    };
    let cell = Cell::Formula {
        // PARITY: BIFF stores RPN tokens, not text; we don't decode them.
        expr: String::new(),
        cached,
    };
    Some((row, col, xf, cell, is_string))
}

fn parse_mergecells(rec: &RawRecord, sheet: &mut Sheet) {
    let d = &rec.data;
    if d.len() < 2 {
        return;
    }
    let count = biff::u16le(d, 0) as usize;
    let mut p = 2;
    for _ in 0..count {
        if p + 8 > d.len() {
            break;
        }
        let r1 = biff::u16le(d, p) as u32;
        let r2 = biff::u16le(d, p + 2) as u32;
        let c1 = biff::u16le(d, p + 4) as u32;
        let c2 = biff::u16le(d, p + 6) as u32;
        p += 8;
        sheet.merged.push(CellRange::new(
            CellAddress::new(r1, c1),
            CellAddress::new(r2, c2),
        ));
    }
}

fn parse_window2(rec: &RawRecord, sheet: &mut Sheet) {
    // grbit bit3 (0x08) = fFrozen. We record frozen panes more precisely from
    // PANE; here just note that freezing is active.
    let d = &rec.data;
    if d.len() >= 2 {
        let grbit = biff::u16le(d, 0);
        if grbit & 0x08 != 0 && sheet.frozen == FrozenPanes::default() {
            // Provisional; PANE (if present) overrides with exact counts.
            sheet.frozen = FrozenPanes { rows: 0, cols: 0 };
        }
    }
}

fn parse_pane(rec: &RawRecord, sheet: &mut Sheet) {
    // PANE: x(u16 cols frozen), y(u16 rows frozen), ...
    let d = &rec.data;
    if d.len() >= 4 {
        let cols = biff::u16le(d, 0) as u32;
        let rows = biff::u16le(d, 2) as u32;
        sheet.frozen = FrozenPanes { rows, cols };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn rk_decode_matches_biff() {
        // 0x3FF00000 -> 1.0 (double form)
        assert_eq!(biff::decode_rk(0x3FF0_0000), 1.0);
    }

    #[test]
    fn roundtrip_via_writer() {
        // Build a workbook, write it, read it back.
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Data");
        sheet.set(0, 0, Cell::Number(42.0));
        sheet.set(0, 1, Cell::Text("hello".into()));
        sheet.set(0, 2, Cell::Bool(true));
        sheet.set(0, 3, Cell::Error(CellError::Div0));
        sheet.set(1, 0, Cell::Number(-9.88));
        sheet.set(1, 1, Cell::Number(1234.0)); // RK integer
        sheet.set(1, 2, Cell::Number(12.34)); // RK div100
        sheet.set(
            2,
            0,
            Cell::Formula {
                expr: "".into(),
                cached: CellValue::Number(99.0),
            },
        );
        sheet.merged.push(CellRange::new(
            CellAddress::new(3, 0),
            CellAddress::new(3, 2),
        ));
        wb.sheets.push(sheet);

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();

        let back = read(Cursor::new(&buf)).unwrap();
        assert_eq!(back.sheets.len(), 1);
        let s = &back.sheets[0];
        assert_eq!(s.name, "Data");
        assert_eq!(s.value(0, 0), CellValue::Number(42.0));
        assert_eq!(s.value(0, 1), CellValue::Text("hello".into()));
        assert_eq!(s.value(0, 2), CellValue::Bool(true));
        assert_eq!(s.value(0, 3), CellValue::Error(CellError::Div0));
        assert_eq!(s.value(1, 0), CellValue::Number(-9.88));
        assert_eq!(s.value(1, 1), CellValue::Number(1234.0));
        assert_eq!(s.value(1, 2), CellValue::Number(12.34));
        assert_eq!(s.value(2, 0), CellValue::Number(99.0));
        assert_eq!(s.merged.len(), 1);
    }

    #[test]
    fn roundtrip_multisheet_custom_format_and_string_formula() {
        let mut wb = Workbook::empty();
        // Custom number format (id >= 164).
        let custom = {
            let st = CellStyle {
                number_format: "0.000\" units\"".into(),
                ..Default::default()
            };
            wb.styles.intern(st)
        };

        let mut s1 = Sheet::new("First");
        s1.set(0, 0, Cell::Number(2.5));
        s1.set_style(0, 0, custom);
        s1.set(0, 1, Cell::Text("alpha".into()));
        s1.set(0, 2, Cell::Text("alpha".into())); // dedup in SST
        // String-result formula -> exercises STRING record path.
        s1.set(
            1,
            0,
            Cell::Formula {
                expr: "".into(),
                cached: CellValue::Text("computed".into()),
            },
        );
        wb.sheets.push(s1);

        let mut s2 = Sheet::new("Second");
        s2.set(0, 0, Cell::Number(7.0));
        s2.visibility = Visibility::Hidden;
        wb.sheets.push(s2);

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
        let back = read(Cursor::new(&buf)).unwrap();

        assert_eq!(back.sheets.len(), 2);
        assert_eq!(back.sheets[0].name, "First");
        assert_eq!(back.sheets[1].name, "Second");
        assert_eq!(back.sheets[1].visibility, Visibility::Hidden);
        assert_eq!(back.sheets[0].value(0, 0), CellValue::Number(2.5));
        assert_eq!(back.sheets[0].value(0, 1), CellValue::Text("alpha".into()));
        assert_eq!(back.sheets[0].value(0, 2), CellValue::Text("alpha".into()));
        assert_eq!(
            back.sheets[0].value(1, 0),
            CellValue::Text("computed".into())
        );
        assert_eq!(back.sheets[1].value(0, 0), CellValue::Number(7.0));

        // Custom format survived.
        let si = back.sheets[0].style_at(0, 0).unwrap();
        assert_eq!(
            back.styles.get(si).unwrap().number_format,
            "0.000\" units\""
        );
    }

    #[test]
    fn roundtrip_date_systems_and_format() {
        for ds in [DateSystem::Date1900, DateSystem::Date1904] {
            let mut wb = Workbook::empty();
            wb.date_system = ds;
            let mut sheet = Sheet::new("S");
            let style = {
                let st = CellStyle {
                    number_format: "yyyy-mm-dd".into(),
                    ..Default::default()
                };
                wb.styles.intern(st)
            };
            sheet.set(0, 0, Cell::Number(44000.0));
            sheet.set_style(0, 0, style);
            wb.sheets.push(sheet);

            let mut buf = Vec::new();
            super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
            let back = read(Cursor::new(&buf)).unwrap();
            assert_eq!(back.date_system, ds);
            let s = &back.sheets[0];
            assert_eq!(s.value(0, 0), CellValue::Number(44000.0));
            let style_idx = s.style_at(0, 0).expect("style preserved");
            let cs = back.styles.get(style_idx).unwrap();
            assert_eq!(cs.number_format, "yyyy-mm-dd");
        }
    }
}
