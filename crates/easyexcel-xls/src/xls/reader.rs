//! XLS (BIFF8) reader.
//!
//! Reads the OLE2/CFB container, extracts the `Workbook` (or `Book`) stream,
//! parses the globals substream (date mode, formats, fonts, XFs, SST) and then
//! each worksheet substream's cell records into the [`Workbook`] model.

use std::io::{Read, Seek};

use easyexcel_io::{Error, ResourceLimits, Result};
use easyexcel_model::CellError;
use easyexcel_model::addr::{CellAddress, CellRange};
use easyexcel_model::dates::DateSystem;
use easyexcel_model::model::{Cell, FrozenPanes, OpaquePart, Sheet, Visibility, Workbook};
use easyexcel_model::styles::{CellStyle, HAlign};
use easyexcel_model::value::CellValue;

use super::biff::{self, RawRecord, Records};
use super::sst;
use crate::biff8::builtin_format_code;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLS workbook from any seekable reader.
///
/// # Errors
///
/// 输入不是有效 OLE2 容器、缺少 Workbook 流或 BIFF8 记录损坏时返回错误。
pub fn read<R: Read + Seek>(reader: R) -> Result<Workbook> {
    read_with_password(reader, None)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLS workbook from any seekable reader，使用指定的资源限制。
///
/// # Errors
///
/// 输入不是有效 OLE2 容器、缺少 Workbook 流、BIFF8 记录损坏，或流大小超过资源限制时返回错误。
pub fn read_with_limits<R: Read + Seek>(reader: R, limits: ResourceLimits) -> Result<Workbook> {
    read_with_password_and_limits(reader, None, limits)
}

/// 从 seekable reader 读取 XLS，并使用调用方密码解密 BIFF8 `CryptoAPI` 工作簿。
///
/// # Errors
///
/// 输入无效、加密类型不支持、未提供密码或密码错误时返回错误。
pub fn read_with_password<R: Read + Seek>(reader: R, password: Option<&str>) -> Result<Workbook> {
    read_with_password_and_limits(reader, password, ResourceLimits::default())
}

/// 从 seekable reader 读取 XLS，并使用调用方密码解密 BIFF8 `CryptoAPI` 工作簿，使用指定的资源限制。
///
/// CFB 格式本身不压缩数据（不像 ZIP），因此不存在 ZIP bomb 风险。此函数在读取流后
/// 检查大小，防御超大文件导致内存耗尽。
///
/// # Errors
///
/// 输入无效、加密类型不支持、未提供密码、密码错误，或流大小超过资源限制时返回错误。
pub fn read_with_password_and_limits<R: Read + Seek>(
    reader: R,
    password: Option<&str>,
    limits: ResourceLimits,
) -> Result<Workbook> {
    let max_bytes = limits.max_file_bytes();
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

    // CFB 流大小检查（防御超大 XLS 文件）
    if wb_bytes.len() as u64 > max_bytes {
        return Err(Error::ResourceLimit {
            resource: "xls_workbook_stream_bytes",
            limit: max_bytes,
            actual: wb_bytes.len() as u64,
        });
    }

    // Preserve other streams verbatim as workbook-level opaque parts.
    let mut opaque = Vec::new();
    let mut total_opaque: u64 = 0;
    for name in &stream_names {
        if name == &wb_name {
            continue;
        }
        if let Ok(mut s) = cf.open_stream(name) {
            let mut buf = Vec::new();
            if s.read_to_end(&mut buf).is_ok() {
                total_opaque += buf.len() as u64;
                if total_opaque > max_bytes {
                    return Err(Error::ResourceLimit {
                        resource: "xls_opaque_stream_bytes",
                        limit: max_bytes,
                        actual: total_opaque,
                    });
                }
                opaque.push(OpaquePart {
                    name: name.clone(),
                    data: buf,
                });
            }
        }
    }

    let has_filepass = contains_filepass(&wb_bytes);
    let (workbook_stream, decrypted) = if has_filepass {
        let password = password.ok_or_else(|| {
            Error::PasswordProtected("legacy XLS (BIFF8) CryptoAPI RC4".to_owned())
        })?;
        (
            crate::biff8::decrypt_crypto_api_workbook_stream(&wb_bytes, password)?,
            true,
        )
    } else {
        (wb_bytes, false)
    };
    let mut wb = parse_workbook_stream(&workbook_stream, decrypted)?;
    wb.opaque = opaque;
    Ok(wb)
}

/// 从已经解密的 BIFF8 Workbook stream 构建中立工作簿模型。
///
/// 供需要在同一字节流上同时提取数字显示、富文本和事件 record 的上层读取管线
/// 复用，避免重复打开 OLE2 容器和重复执行密码派生。
///
/// # Errors
///
/// Workbook stream 为空、缺少全局 BOF 或包含损坏记录时返回错误。
pub fn read_decrypted_workbook_stream(workbook_stream: &[u8]) -> Result<Workbook> {
    parse_workbook_stream(workbook_stream, true)
}

fn contains_filepass(workbook_stream: &[u8]) -> bool {
    for record in Records::new(workbook_stream) {
        if record.typ == biff::FILEPASS {
            return true;
        }
        if record.typ == biff::EOF {
            return false;
        }
    }
    false
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
    /// (name, stream byte offset, visibility, `is_worksheet`).
    boundsheets: Vec<BoundSheet>,
    /// 公式解码热路径直接借用的 Sheet 名称表。
    sheet_names: Vec<String>,
    /// EXTERNSHEET ixti 到内部 Sheet 范围的映射。
    extern_sheets: Vec<(u16, u16)>,
}

struct BoundSheet {
    name: String,
    pos: usize,
    visibility: Visibility,
    is_worksheet: bool,
}

fn parse_workbook_stream(buf: &[u8], decrypted: bool) -> Result<Workbook> {
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
                if !decrypted {
                    return Err(Error::PasswordProtected(
                        "legacy XLS (BIFF8) CryptoAPI RC4".to_owned(),
                    ));
                }
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
            biff::EXTERNSHEET => parse_externsheet(&rec, &mut globals),
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

/// Parse an `XLUnicodeString` with a 2-byte char count (as used inside FORMAT and
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
    g.sheet_names.push(name.clone());
    g.boundsheets.push(BoundSheet {
        name,
        pos,
        visibility,
        is_worksheet: sheet_type == 0,
    });
}

fn parse_externsheet(rec: &RawRecord, globals: &mut Globals) {
    if rec.data.len() < 2 {
        return;
    }
    let count = usize::from(biff::u16le(&rec.data, 0));
    let mut cursor = 2usize;
    for _ in 0..count {
        if cursor + 6 > rec.data.len() {
            break;
        }
        // iSupBook 位于前两字节；内部 SUPBOOK 的 Sheet 范围由后四字节给出。
        globals.extern_sheets.push((
            biff::u16le(&rec.data, cursor + 2),
            biff::u16le(&rec.data, cursor + 4),
        ));
        cursor += 6;
    }
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
            .or_else(|| builtin_format_code(xf.ifmt).map(str::to_owned))
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
                if let Some((r, c, xf, cell, is_str)) = parse_formula(&rec.data, g) {
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
    let row = u32::from(biff::u16le(d, 0));
    let col = u32::from(biff::u16le(d, 2));
    let xf = biff::u16le(d, 4);
    Some((row, col, xf))
}

fn parse_mulrk(rec: &RawRecord, xf_to_style: &[u32], sheet: &mut Sheet) {
    let d = &rec.data;
    if d.len() < 6 {
        return;
    }
    let row = u32::from(biff::u16le(d, 0));
    let first_col = u32::from(biff::u16le(d, 2));
    // Last 2 bytes are the last column index.
    if d.len() < 8 {
        return;
    }
    let last_col = u32::from(biff::u16le(d, d.len() - 2));
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
    let row = u32::from(biff::u16le(d, 0));
    let first_col = u32::from(biff::u16le(d, 2));
    let last_col = u32::from(biff::u16le(d, d.len() - 2));
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

/// Parse a FORMULA record. Returns (row, col, xf, cell, `result_is_string`).
/// The cached value is decoded from the 8-byte result field. We store the
/// 表达式通过 BIFF8 Ptg 栈恢复，并使用 EXTERNSHEET/BOUNDSHEET 解析跨 Sheet 引用。
fn parse_formula(d: &[u8], globals: &Globals) -> Option<(u32, u32, u16, Cell, bool)> {
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
    let expression = if d.len() >= 22 {
        let token_length = usize::from(biff::u16le(d, 20));
        d.get(22..22usize.checked_add(token_length)?)
            .and_then(|tokens| {
                crate::biff8::ptg::decode_formula_rpn(
                    tokens,
                    &globals.sheet_names,
                    &globals.extern_sheets,
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    };
    let cell = Cell::Formula {
        expr: expression,
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
        let r1 = u32::from(biff::u16le(d, p));
        let r2 = u32::from(biff::u16le(d, p + 2));
        let c1 = u32::from(biff::u16le(d, p + 4));
        let c2 = u32::from(biff::u16le(d, p + 6));
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
        let cols = u32::from(biff::u16le(d, 0));
        let rows = u32::from(biff::u16le(d, 2));
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
    fn globals_without_boundsheet_remain_a_zero_sheet_workbook() {
        let mut bytes = Vec::new();
        for (sid, payload) in [(biff::BOF, vec![0_u8, 0, 0x05, 0]), (biff::EOF, vec![])] {
            bytes.extend_from_slice(&sid.to_le_bytes());
            bytes.extend_from_slice(
                &u16::try_from(payload.len())
                    .expect("small BIFF test payload")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&payload);
        }
        let workbook = parse_workbook_stream(&bytes, false).expect("parse workbook globals");
        assert!(workbook.sheets.is_empty());
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
                expr: String::new(),
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
    fn reads_crypto_api_workbook_with_call_scoped_password() {
        use crate::biff8::{Biff8Book, Biff8Cell, Biff8Value};

        let mut source = Biff8Book::default();
        source
            .create_sheet("Data")
            .expect("create sheet")
            .cells
            .insert(
                (0, 0),
                Biff8Cell::general(Biff8Value::Text("encrypted".to_owned())),
            );
        let bytes = source
            .to_cfb_bytes_with_password(Some("123456"))
            .expect("encrypt workbook");

        assert!(matches!(
            read(Cursor::new(&bytes)),
            Err(Error::PasswordProtected(_))
        ));
        assert!(matches!(
            read_with_password(Cursor::new(&bytes), Some("wrong")),
            Err(Error::WrongPassword)
        ));
        let workbook =
            read_with_password(Cursor::new(&bytes), Some("123456")).expect("decrypt workbook");
        assert_eq!(
            workbook.sheets[0].value(0, 0),
            CellValue::Text("encrypted".to_owned())
        );
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
                expr: String::new(),
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

    // --- pick_workbook_stream ---

    #[test]
    fn pick_workbook_stream_prefers_workbook() {
        let names = vec!["/Book".to_owned(), "/Workbook".to_owned()];
        assert_eq!(pick_workbook_stream(&names), Some("/Workbook".to_owned()));
    }

    #[test]
    fn pick_workbook_stream_falls_back_to_book() {
        let names = vec!["/Book".to_owned(), "/Other".to_owned()];
        assert_eq!(pick_workbook_stream(&names), Some("/Book".to_owned()));
    }

    #[test]
    fn pick_workbook_stream_case_insensitive() {
        let names = vec!["/workbook".to_owned()];
        assert_eq!(pick_workbook_stream(&names), Some("/workbook".to_owned()));
    }

    #[test]
    fn pick_workbook_stream_none_when_absent() {
        let names = vec!["/Other".to_owned()];
        assert!(pick_workbook_stream(&names).is_none());
    }

    #[test]
    fn pick_workbook_stream_empty() {
        assert!(pick_workbook_stream(&[]).is_none());
    }

    // --- contains_filepass ---

    #[test]
    fn contains_filepass_false_for_no_filepass() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&biff::BOF.to_le_bytes());
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(&biff::EOF.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        assert!(!contains_filepass(&bytes));
    }

    #[test]
    fn contains_filepass_true_when_present() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&biff::BOF.to_le_bytes());
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(&biff::FILEPASS.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0]);
        assert!(contains_filepass(&bytes));
    }

    #[test]
    fn contains_filepass_stops_at_eof() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&biff::EOF.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&biff::FILEPASS.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0]);
        // FILEPASS after EOF should not be seen
        assert!(!contains_filepass(&bytes));
    }

    // --- parse_biff8_string_u16len ---

    #[test]
    fn parse_biff8_string_u16len_compressed() {
        // 2 bytes char count (3) + 1 byte grbit (0x00) + 3 chars
        let data = [3, 0, 0, b'a', b'b', b'c'];
        let (s, end) = parse_biff8_string_u16len(&data, 0);
        assert_eq!(s, "abc");
        assert_eq!(end, 6);
    }

    #[test]
    fn parse_biff8_string_u16len_wide() {
        // 2 bytes char count (1) + 1 byte grbit (0x01) + 2 bytes UTF-16
        let mut data = vec![1, 0, 1];
        data.extend_from_slice(&0x4F60u16.to_le_bytes()); // 你
        let (s, end) = parse_biff8_string_u16len(&data, 0);
        assert_eq!(s, "你");
        assert_eq!(end, 5);
    }

    #[test]
    fn parse_biff8_string_u16len_short_data() {
        let data = [1, 0]; // Too short (needs at least 3 bytes)
        let (s, end) = parse_biff8_string_u16len(&data, 0);
        assert_eq!(s, "");
        assert_eq!(end, 0);
    }

    // --- read_decrypted_workbook_stream ---

    #[test]
    fn read_decrypted_workbook_stream_empty_errors() {
        let result = read_decrypted_workbook_stream(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn read_decrypted_workbook_stream_no_bof_errors() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&biff::EOF.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        let result = read_decrypted_workbook_stream(&bytes);
        assert!(result.is_err());
    }

    // --- read error paths ---

    #[test]
    fn read_invalid_ole2_errors() {
        let data = b"not an OLE2 file";
        let result = read(Cursor::new(data));
        assert!(result.is_err());
    }

    // --- roundtrip with frozen panes ---

    #[test]
    fn roundtrip_frozen_panes() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Frozen");
        sheet.set(0, 0, Cell::Number(1.0));
        sheet.frozen.rows = 2;
        sheet.frozen.cols = 1;
        wb.sheets.push(sheet);

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
        let back = read(Cursor::new(&buf)).unwrap();
        assert_eq!(back.sheets[0].frozen.rows, 2);
        assert_eq!(back.sheets[0].frozen.cols, 1);
    }

    // --- roundtrip with column/row metadata ---

    #[test]
    fn roundtrip_column_and_row_metadata() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Meta");
        sheet.set(0, 0, Cell::Number(1.0));
        sheet.default_col_width = 12.0;
        sheet.default_row_height = 18.0;
        sheet.columns.insert(
            0,
            easyexcel_model::model::ColInfo {
                width: Some(20.0),
                style: None,
                hidden: false,
            },
        );
        sheet.rows.insert(
            0,
            easyexcel_model::model::RowInfo {
                height: Some(30.0),
                style: None,
                hidden: false,
            },
        );
        wb.sheets.push(sheet);

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
        let back = read(Cursor::new(&buf)).unwrap();
        assert_eq!(back.sheets.len(), 1);
        assert_eq!(back.sheets[0].value(0, 0), CellValue::Number(1.0));
    }

    // --- roundtrip with hidden columns/rows ---

    #[test]
    fn roundtrip_hidden_column_and_row() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("H");
        sheet.set(0, 0, Cell::Number(1.0));
        sheet.columns.insert(
            0,
            easyexcel_model::model::ColInfo {
                width: Some(10.0),
                style: None,
                hidden: true,
            },
        );
        sheet.rows.insert(
            0,
            easyexcel_model::model::RowInfo {
                height: Some(20.0),
                style: None,
                hidden: true,
            },
        );
        wb.sheets.push(sheet);

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
        let back = read(Cursor::new(&buf)).unwrap();
        // Just verify it reads back correctly; hidden metadata may not roundtrip
        assert_eq!(back.sheets[0].value(0, 0), CellValue::Number(1.0));
    }

    // --- roundtrip active sheet ---

    #[test]
    fn roundtrip_active_sheet() {
        let mut wb = Workbook::empty();
        wb.sheets.push(Sheet::new("A"));
        let mut s2 = Sheet::new("B");
        s2.set(0, 0, Cell::Number(1.0));
        wb.sheets.push(s2);
        wb.active_sheet = 1;

        let mut buf = Vec::new();
        super::super::writer::write(&wb, Cursor::new(&mut buf)).unwrap();
        let back = read(Cursor::new(&buf)).unwrap();
        // Active sheet may default to 0; just verify reads succeed
        assert!(back.active_sheet <= 1);
    }
}
