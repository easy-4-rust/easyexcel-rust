//! XLS (BIFF8) writer.
//!
//! Builds a valid BIFF8 `Workbook` stream inside an OLE2/CFB container that
//! Excel and LibreOffice can open. The whole stream is assembled in a `Vec<u8>`
//! so that BOUNDSHEET `lbPlyPos` offsets can be back-patched once each sheet's
//! substream position is known.
//!
//! PARITY: formula cells are written as FORMULA records carrying their cached
//! value plus a trivial constant RPN token (tNum / tBool / tStr / tErr) rather
//! than the original token stream — we never had the source RPN, only a cached
//! value (the reader does not decode RPN to text). This is a documented,
//! intentional limitation; the displayed/cached value round-trips correctly.

use std::collections::HashMap;
use std::io::{Seek, Write};

use crate::core::dates::DateSystem;
use crate::core::error::{Error, Result};
use crate::core::model::{Cell, Sheet, Workbook};
use crate::core::styles::HAlign;
use crate::core::value::CellValue;

use super::biff;
use super::sst;

/// Write a workbook as XLS (BIFF8) to any seekable writer.
pub fn write<W: Write + Seek>(wb: &Workbook, mut writer: W) -> Result<()> {
    let stream = build_workbook_stream(wb)?;

    // `cfb::CompoundFile::create` requires Read + Write + Seek (it reads back
    // sectors while assembling the FAT), but our frozen signature only promises
    // Write + Seek. Build the whole container in an in-memory Cursor, then copy
    // the finished bytes to the caller's writer.
    let mut mem = std::io::Cursor::new(Vec::<u8>::new());
    {
        let mut cf = cfb::CompoundFile::create(&mut mem)
            .map_err(|e| Error::Cfb(format!("cannot create OLE2 container: {e}")))?;
        {
            let mut s = cf
                .create_stream("Workbook")
                .map_err(|e| Error::Cfb(format!("cannot create Workbook stream: {e}")))?;
            s.write_all(&stream)?;
        }
        cf.flush()
            .map_err(|e| Error::Cfb(format!("cannot flush OLE2 container: {e}")))?;
    }
    writer.write_all(&mem.into_inner())?;
    Ok(())
}

/// Emit a single framed BIFF record (4-byte header + data) into `out`.
fn record(out: &mut Vec<u8>, typ: u16, data: &[u8]) {
    debug_assert!(data.len() <= biff::MAX_RECORD_DATA);
    out.extend_from_slice(&typ.to_le_bytes());
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(data);
}

/// Collect the unique number-format codes used by the workbook's styles that
/// require a custom FORMAT record (id >= 164), assigning fresh ids. Returns a
/// map style-number-format-string -> ifmt id, plus the list of (id, code).
struct FormatPlan {
    /// number_format string -> ifmt id to use in XF records.
    fmt_id: HashMap<String, u16>,
    /// (id, code) pairs to emit as FORMAT records.
    customs: Vec<(u16, String)>,
}

fn plan_formats(wb: &Workbook) -> FormatPlan {
    let mut fmt_id: HashMap<String, u16> = HashMap::new();
    let mut customs = Vec::new();
    let mut next_id: u16 = 164;
    for style in wb.styles.iter() {
        let code = style.number_format.trim();
        if code.is_empty() || code.eq_ignore_ascii_case("general") {
            continue;
        }
        if let Some(id) = builtin_format_id(code) {
            fmt_id.entry(code.to_string()).or_insert(id);
            continue;
        }
        if !fmt_id.contains_key(code) {
            fmt_id.insert(code.to_string(), next_id);
            customs.push((next_id, code.to_string()));
            next_id += 1;
        }
    }
    FormatPlan { fmt_id, customs }
}

/// Reverse of the reader's builtin map: known format code -> built-in id, so we
/// can reuse a built-in instead of emitting a redundant FORMAT record.
fn builtin_format_id(code: &str) -> Option<u16> {
    Some(match code {
        "0" => 1,
        "0.00" => 2,
        "#,##0" => 3,
        "#,##0.00" => 4,
        "0%" => 9,
        "0.00%" => 10,
        "0.00E+00" => 11,
        "m/d/yy" => 14,
        "d-mmm-yy" => 15,
        "d-mmm" => 16,
        "mmm-yy" => 17,
        "h:mm AM/PM" => 18,
        "h:mm:ss AM/PM" => 19,
        "h:mm" => 20,
        "h:mm:ss" => 21,
        "m/d/yy h:mm" => 22,
        "mm:ss" => 45,
        "[h]:mm:ss" => 46,
        "@" => 49,
        _ => return None,
    })
}

/// Build the entire Workbook stream bytes.
fn build_workbook_stream(wb: &Workbook) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();

    // ---- Globals substream -------------------------------------------------
    write_bof(&mut out, biff::DT_GLOBALS);

    // CODEPAGE = 1200 (UTF-16).
    record(&mut out, biff::CODEPAGE, &1200u16.to_le_bytes());

    // DATEMODE.
    let datemode: u16 = match wb.date_system {
        DateSystem::Date1900 => 0,
        DateSystem::Date1904 => 1,
    };
    record(&mut out, biff::DATEMODE, &datemode.to_le_bytes());

    // FONT records. Index 4 is skipped by convention, so write 5 fonts (0..=4)
    // where 4 is a dummy; indices >=5 would be real if we mapped fonts.
    for _ in 0..5 {
        write_default_font(&mut out);
    }

    // Custom FORMAT records.
    let plan = plan_formats(wb);
    for (id, code) in &plan.customs {
        write_format(&mut out, *id, code);
    }

    // XF records: 16 built-in style XFs, then one cell XF per interned style.
    // The first 16 (indices 0..15) are placeholders required by the format; cell
    // XFs begin at index 16. We map interned style index i -> XF index 16+i.
    for _ in 0..16 {
        write_style_xf(&mut out);
    }
    for style in wb.styles.iter() {
        let ifmt = style_ifmt(style, &plan);
        write_cell_xf(&mut out, ifmt, style.halign, style.wrap_text);
    }

    // One STYLE record (Normal) referencing built-in XF 0.
    {
        let mut d = Vec::new();
        d.extend_from_slice(&0x8000u16.to_le_bytes()); // ixfe | fBuiltIn
        d.push(0x00); // builtin id: Normal
        d.push(0xFF); // level
        record(&mut out, biff::STYLE, &d);
    }

    // Build the SST from all string cells.
    let (sst_strings, sst_index, total_refs) = build_sst(wb);

    // BOUNDSHEET records (placeholder offsets, patched later). Record the byte
    // offset of each BOUNDSHEET's lbPlyPos field so we can patch it.
    let mut boundsheet_patch_offsets: Vec<usize> = Vec::with_capacity(wb.sheets.len());
    for sheet in &wb.sheets {
        let off = write_boundsheet_placeholder(&mut out, sheet);
        boundsheet_patch_offsets.push(off);
    }

    // SST + EXTSST.
    if !sst_strings.is_empty() {
        let framed = sst::build_sst_records(&sst_strings, total_refs);
        out.extend_from_slice(&framed);
        // Minimal EXTSST (zero buckets) — optional but harmless.
        record(&mut out, biff::EXTSST, &[0, 0]);
    }

    // Globals EOF.
    record(&mut out, biff::EOF, &[]);

    // ---- Worksheet substreams ---------------------------------------------
    let mut sheet_offsets: Vec<u32> = Vec::with_capacity(wb.sheets.len());
    for sheet in &wb.sheets {
        let offset = out.len() as u32;
        sheet_offsets.push(offset);
        write_worksheet(&mut out, sheet, wb, &sst_index);
    }

    // ---- Back-patch BOUNDSHEET lbPlyPos ------------------------------------
    for (i, &patch_off) in boundsheet_patch_offsets.iter().enumerate() {
        let pos = sheet_offsets[i];
        out[patch_off..patch_off + 4].copy_from_slice(&pos.to_le_bytes());
    }

    Ok(out)
}

fn write_bof(out: &mut Vec<u8>, dt: u16) {
    let mut d = Vec::new();
    d.extend_from_slice(&biff::BIFF8_VERSION.to_le_bytes()); // vers
    d.extend_from_slice(&dt.to_le_bytes()); // dt (substream type)
    d.extend_from_slice(&0x0DBBu16.to_le_bytes()); // rupBuild
    d.extend_from_slice(&0x07CCu16.to_le_bytes()); // rupYear
    d.extend_from_slice(&0x0000_00C1u32.to_le_bytes()); // bfh
    d.extend_from_slice(&0x0000_0006u32.to_le_bytes()); // sfo
    record(out, biff::BOF, &d);
}

fn write_default_font(out: &mut Vec<u8>) {
    // FONT: dyHeight(u16), grbit(u16), icv(u16), bls(u16), sss(u16),
    // uls(u8), bFamily(u8), bCharSet(u8), reserved(u8), name(short string).
    let mut d = Vec::new();
    d.extend_from_slice(&200u16.to_le_bytes()); // 10pt * 20
    d.extend_from_slice(&0u16.to_le_bytes()); // grbit
    d.extend_from_slice(&0x7FFFu16.to_le_bytes()); // icv = auto
    d.extend_from_slice(&400u16.to_le_bytes()); // bls = normal
    d.extend_from_slice(&0u16.to_le_bytes()); // sss
    d.push(0); // uls
    d.push(0); // bFamily
    d.push(0); // bCharSet
    d.push(0); // reserved
    d.extend_from_slice(&biff::encode_short_unicode_string("Arial"));
    record(out, biff::FONT, &d);
}

fn write_format(out: &mut Vec<u8>, id: u16, code: &str) {
    let mut d = Vec::new();
    d.extend_from_slice(&id.to_le_bytes());
    d.extend_from_slice(&biff::encode_unicode_string(code));
    record(out, biff::FORMAT, &d);
}

/// A built-in style XF (fStyle set). Minimal but valid.
fn write_style_xf(out: &mut Vec<u8>) {
    let mut d = vec![0u8; 20];
    // ifnt=0, ifmt=0 already zero.
    // fStyle bit lives in the attributes; set a benign style flag.
    // Byte layout: [0..2] ifnt, [2..4] ifmt, [4..6] grbit, [6] align, ...
    d[4] = 0xF5; // fLocked|fStyle-ish flags commonly seen; harmless to readers
    d[5] = 0xFF;
    record(out, biff::XF, &d);
}

/// A cell XF with the given number-format id and alignment.
fn write_cell_xf(out: &mut Vec<u8>, ifmt: u16, halign: HAlign, wrap: bool) {
    let mut d = vec![0u8; 20];
    // ifnt = 0 (first font).
    d[0..2].copy_from_slice(&0u16.to_le_bytes());
    d[2..4].copy_from_slice(&ifmt.to_le_bytes());
    // grbit: fLocked(0x01) typical; bit2 (0x04) marks a cell XF (not style).
    d[4..6].copy_from_slice(&0x0001u16.to_le_bytes());
    // Alignment byte.
    let mut align = match halign {
        HAlign::General => 0,
        HAlign::Left => 1,
        HAlign::Center => 2,
        HAlign::Right => 3,
        HAlign::Fill => 4,
        HAlign::Justify => 5,
        HAlign::CenterContinuous => 6,
        HAlign::Distributed => 7,
    };
    if wrap {
        align |= 0x08;
    }
    d[6] = align;
    record(out, biff::XF, &d);
}

fn style_ifmt(style: &crate::core::styles::CellStyle, plan: &FormatPlan) -> u16 {
    let code = style.number_format.trim();
    if code.is_empty() || code.eq_ignore_ascii_case("general") {
        return 0;
    }
    *plan.fmt_id.get(code).unwrap_or(&0)
}

/// Write a BOUNDSHEET with a placeholder lbPlyPos. Returns the byte offset of
/// the lbPlyPos field (within `out`) for later patching.
fn write_boundsheet_placeholder(out: &mut Vec<u8>, sheet: &Sheet) -> usize {
    let mut d = Vec::new();
    d.extend_from_slice(&0u32.to_le_bytes()); // lbPlyPos placeholder
    // grbit: hidden state byte + type byte.
    let hidden = match sheet.visibility {
        crate::core::model::Visibility::Visible => 0u8,
        crate::core::model::Visibility::Hidden => 1,
        crate::core::model::Visibility::VeryHidden => 2,
    };
    d.push(hidden);
    d.push(0x00); // sheet type = worksheet
    d.extend_from_slice(&biff::encode_short_unicode_string(&sheet.name));

    // The lbPlyPos field sits at the start of this record's *data*, which is 4
    // bytes after the 4-byte record header.
    let record_start = out.len();
    record(out, biff::BOUNDSHEET, &d);
    record_start + 4
}

/// Build the SST: a deduplicated string list, a map text->index for cells, and
/// the total reference count.
fn build_sst(wb: &Workbook) -> (Vec<String>, HashMap<String, u32>, u32) {
    let mut strings: Vec<String> = Vec::new();
    let mut index: HashMap<String, u32> = HashMap::new();
    let mut total_refs: u32 = 0;
    for sheet in &wb.sheets {
        for cell in sheet.cells.values() {
            let text = match cell {
                Cell::Text(s) => Some(s.clone()),
                Cell::Formula {
                    cached: CellValue::Text(s),
                    ..
                } => Some(s.clone()),
                _ => None,
            };
            // Only non-formula text cells use LABELSST; formula string results
            // use a STRING record, not the SST. So count only Text cells here.
            if let Cell::Text(s) = cell {
                let _ = text;
                total_refs += 1;
                if !index.contains_key(s) {
                    index.insert(s.clone(), strings.len() as u32);
                    strings.push(s.clone());
                }
            }
        }
    }
    (strings, index, total_refs)
}

fn write_worksheet(
    out: &mut Vec<u8>,
    sheet: &Sheet,
    wb: &Workbook,
    sst_index: &HashMap<String, u32>,
) {
    write_bof(out, biff::DT_WORKSHEET);

    // DIMENSION: rwMic(u32), rwMac(u32 exclusive), colMic(u16), colMac(u16 excl), reserved(u16).
    let (max_row, max_col) = sheet.dimensions();
    {
        let mut d = Vec::new();
        d.extend_from_slice(&0u32.to_le_bytes());
        d.extend_from_slice(&max_row.to_le_bytes());
        d.extend_from_slice(&(0u16).to_le_bytes());
        d.extend_from_slice(&((max_col.min(0xFFFF)) as u16).to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        record(out, biff::DIMENSION, &d);
    }

    // Cell records. Iterate the sorted cell map; styled-but-empty cells emit
    // BLANK.
    for (&(row, col), cell) in &sheet.cells {
        if row > 0xFFFF || col > 0xFF {
            continue; // outside BIFF8 limits
        }
        let xf = xf_index_for(sheet, wb, row, col);
        write_cell(out, row as u16, col as u16, xf, cell, sst_index);
    }
    // Styled empty cells not present in `cells` -> BLANK.
    for &(row, col) in sheet.styles.keys() {
        if sheet.cells.contains_key(&(row, col)) {
            continue;
        }
        if row > 0xFFFF || col > 0xFF {
            continue;
        }
        let xf = xf_index_for(sheet, wb, row, col);
        write_blank(out, row as u16, col as u16, xf);
    }

    // MERGECELLS.
    if !sheet.merged.is_empty() {
        write_mergecells(out, sheet);
    }

    // WINDOW2 + PANE for frozen panes (best-effort).
    write_window2(out, sheet);
    if sheet.frozen.rows > 0 || sheet.frozen.cols > 0 {
        write_pane(out, sheet);
    }

    record(out, biff::EOF, &[]);
}

/// Map a cell's interned style index to its XF index (cell XFs start at 16).
fn xf_index_for(sheet: &Sheet, _wb: &Workbook, row: u32, col: u32) -> u16 {
    match sheet.style_at(row, col) {
        Some(idx) => (16 + idx) as u16,
        None => 15, // a generic cell XF (last built-in)
    }
}

fn write_cell(
    out: &mut Vec<u8>,
    row: u16,
    col: u16,
    xf: u16,
    cell: &Cell,
    sst_index: &HashMap<String, u32>,
) {
    match cell {
        Cell::Empty => write_blank(out, row, col, xf),
        Cell::Number(n) => write_number(out, row, col, xf, *n),
        Cell::Text(s) => write_labelsst(out, row, col, xf, s, sst_index),
        Cell::Bool(b) => write_boolerr(out, row, col, xf, if *b { 1 } else { 0 }, false),
        Cell::Error(e) => write_boolerr(out, row, col, xf, e.biff_code(), true),
        Cell::Formula { cached, .. } => write_formula(out, row, col, xf, cached),
    }
}

fn header(d: &mut Vec<u8>, row: u16, col: u16, xf: u16) {
    d.extend_from_slice(&row.to_le_bytes());
    d.extend_from_slice(&col.to_le_bytes());
    d.extend_from_slice(&xf.to_le_bytes());
}

fn write_blank(out: &mut Vec<u8>, row: u16, col: u16, xf: u16) {
    let mut d = Vec::new();
    header(&mut d, row, col, xf);
    record(out, biff::BLANK, &d);
}

fn write_number(out: &mut Vec<u8>, row: u16, col: u16, xf: u16, n: f64) {
    if let Some(rk) = biff::encode_rk(n) {
        let mut d = Vec::new();
        header(&mut d, row, col, xf);
        d.extend_from_slice(&rk.to_le_bytes());
        record(out, biff::RK, &d);
    } else {
        let mut d = Vec::new();
        header(&mut d, row, col, xf);
        d.extend_from_slice(&n.to_le_bytes());
        record(out, biff::NUMBER, &d);
    }
}

fn write_labelsst(
    out: &mut Vec<u8>,
    row: u16,
    col: u16,
    xf: u16,
    s: &str,
    sst_index: &HashMap<String, u32>,
) {
    let idx = sst_index.get(s).copied().unwrap_or(0);
    let mut d = Vec::new();
    header(&mut d, row, col, xf);
    d.extend_from_slice(&idx.to_le_bytes());
    record(out, biff::LABELSST, &d);
}

fn write_boolerr(out: &mut Vec<u8>, row: u16, col: u16, xf: u16, val: u8, is_err: bool) {
    let mut d = Vec::new();
    header(&mut d, row, col, xf);
    d.push(val);
    d.push(if is_err { 1 } else { 0 });
    record(out, biff::BOOLERR, &d);
}

fn write_formula(out: &mut Vec<u8>, row: u16, col: u16, xf: u16, cached: &CellValue) {
    let mut d = Vec::new();
    header(&mut d, row, col, xf);

    // 8-byte result field + grbit(u16) + chn(u32) + cce(u16) + rgce.
    // We then emit a trivial constant-RPN token matching the cached value.
    let mut rpn: Vec<u8> = Vec::new();
    let mut pending_string: Option<String> = None;

    match cached {
        CellValue::Number(n) => {
            d.extend_from_slice(&n.to_le_bytes());
            // tNum (0x1F) ptg + f64.
            rpn.push(0x1F);
            rpn.extend_from_slice(&n.to_le_bytes());
        }
        CellValue::Bool(b) => {
            let mut result = [0u8; 8];
            result[0] = 1; // bool type tag
            result[2] = if *b { 1 } else { 0 };
            result[6] = 0xFF;
            result[7] = 0xFF;
            d.extend_from_slice(&result);
            // tBool (0x1D) ptg + byte.
            rpn.push(0x1D);
            rpn.push(if *b { 1 } else { 0 });
        }
        CellValue::Error(e) => {
            let mut result = [0u8; 8];
            result[0] = 2; // error type tag
            result[2] = e.biff_code();
            result[6] = 0xFF;
            result[7] = 0xFF;
            d.extend_from_slice(&result);
            // tErr (0x1C) ptg + byte.
            rpn.push(0x1C);
            rpn.push(e.biff_code());
        }
        CellValue::Text(s) => {
            let mut result = [0u8; 8];
            result[0] = 0; // string type tag
            result[6] = 0xFF;
            result[7] = 0xFF;
            d.extend_from_slice(&result);
            // tStr (0x17) ptg + inline XLUnicodeString.
            rpn.push(0x17);
            rpn.extend_from_slice(&biff::encode_unicode_string(s));
            pending_string = Some(s.clone());
        }
        CellValue::Empty => {
            let mut result = [0u8; 8];
            result[0] = 3; // empty type tag
            result[6] = 0xFF;
            result[7] = 0xFF;
            d.extend_from_slice(&result);
            // tMissArg (0x16).
            rpn.push(0x16);
        }
    }

    d.extend_from_slice(&0u16.to_le_bytes()); // grbit (always calc)
    d.extend_from_slice(&0u32.to_le_bytes()); // chn
    d.extend_from_slice(&(rpn.len() as u16).to_le_bytes()); // cce
    d.extend_from_slice(&rpn);

    record(out, biff::FORMULA, &d);

    // A string-result formula must be followed by a STRING record.
    if let Some(s) = pending_string {
        let mut sd = Vec::new();
        sd.extend_from_slice(&biff::encode_unicode_string(&s));
        record(out, biff::STRING, &sd);
    }
}

fn write_mergecells(out: &mut Vec<u8>, sheet: &Sheet) {
    // MERGECELLS: count(u16) then 8 bytes per range (rwFirst, rwLast, colFirst, colLast).
    // Limit to what fits in one record (each range = 8 bytes; max ~1027 ranges).
    let max_per = (biff::MAX_RECORD_DATA - 2) / 8;
    for chunk in sheet.merged.chunks(max_per) {
        let mut d = Vec::new();
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        for r in chunk {
            d.extend_from_slice(&(r.start.row as u16).to_le_bytes());
            d.extend_from_slice(&(r.end.row as u16).to_le_bytes());
            d.extend_from_slice(&(r.start.col as u16).to_le_bytes());
            d.extend_from_slice(&(r.end.col as u16).to_le_bytes());
        }
        record(out, biff::MERGECELLS, &d);
    }
}

fn write_window2(out: &mut Vec<u8>, sheet: &Sheet) {
    let mut d = Vec::new();
    // grbit: fDspFmla=0, fDspGrid=1(0x02), fDspRwCol=1(0x04), fFrozen if panes,
    // fDspZeros=1(0x10), fDefaultHdr=1(0x40).
    let mut grbit: u16 = 0x02 | 0x04 | 0x10 | 0x40 | 0x80; // 0x80 fDspGuts
    let frozen = sheet.frozen.rows > 0 || sheet.frozen.cols > 0;
    if frozen {
        grbit |= 0x08; // fFrozen
        grbit |= 0x100; // fFrozenNoSplit
    }
    d.extend_from_slice(&grbit.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes()); // rwTop
    d.extend_from_slice(&0u16.to_le_bytes()); // colLeft
    d.extend_from_slice(&0x00000040u32.to_le_bytes()); // icvHdr default
    d.extend_from_slice(&0u16.to_le_bytes()); // wScaleSLV
    d.extend_from_slice(&0u16.to_le_bytes()); // wScaleNormal
    d.extend_from_slice(&0u32.to_le_bytes()); // reserved
    record(out, biff::WINDOW2, &d);
}

fn write_pane(out: &mut Vec<u8>, sheet: &Sheet) {
    let mut d = Vec::new();
    d.extend_from_slice(&(sheet.frozen.cols as u16).to_le_bytes()); // x = frozen cols
    d.extend_from_slice(&(sheet.frozen.rows as u16).to_le_bytes()); // y = frozen rows
    d.extend_from_slice(&(sheet.frozen.rows as u16).to_le_bytes()); // top row of bottom pane
    d.extend_from_slice(&(sheet.frozen.cols as u16).to_le_bytes()); // left col of right pane
    d.push(2); // active pane (bottom-left); arbitrary
    d.push(0); // padding (BIFF8 PANE is 9 bytes; extra reserved)
    record(out, biff::PANE, &d);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn writes_openable_container() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("Sheet1");
        s.set(0, 0, Cell::Number(1.0));
        s.set(0, 1, Cell::Text("hi".into()));
        wb.sheets.push(s);
        let mut buf = Vec::new();
        write(&wb, Cursor::new(&mut buf)).unwrap();
        // Should be a valid CFB.
        assert!(super::super::looks_like_cfb(&buf));
        cfb::CompoundFile::open(Cursor::new(&buf)).expect("valid cfb");
    }
}
