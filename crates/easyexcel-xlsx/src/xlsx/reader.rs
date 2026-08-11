//! XLSX (OOXML `SpreadsheetML`) reader.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use quick_xml::Reader;
use quick_xml::events::Event;

use easyexcel_io::{Error, ResourceLimits, Result};
use easyexcel_model::CellError;
use easyexcel_model::addr::{CellAddress, CellRange};
use easyexcel_model::dates::DateSystem;
use easyexcel_model::model::{
    Cell, ColInfo, DefinedName, FrozenPanes, Metadata, OpaquePart, RowInfo, Sheet, Visibility,
    Workbook,
};
use easyexcel_model::value::CellValue;

use super::shared_strings::parse_shared_strings;
use super::styles::parse_styles;
use super::xmlutil::{attr, general_ref, local_name, local_name_end, text};

/// Parts we parse ourselves and therefore do NOT keep as opaque.
fn is_known_part(name: &str) -> bool {
    let n = name;
    n == "[Content_Types].xml"
        || n == "_rels/.rels"
        || n == "xl/workbook.xml"
        || n == "xl/_rels/workbook.xml.rels"
        || n == "xl/sharedStrings.xml"
        || n == "xl/styles.xml"
        || n == "xl/calcChain.xml"
        || n == "docProps/core.xml"
        || n == "docProps/app.xml"
        || (n.starts_with("xl/worksheets/") && n.ends_with(".xml") && !n.contains("_rels"))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLSX workbook from any seekable reader.
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn read<R: Read + Seek>(reader: R) -> Result<Workbook> {
    read_with_password(reader, None)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLSX workbook from any seekable reader，使用指定的资源限制。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，输入不符合格式约束，或解压后数据超过资源限制时返回错误。
pub fn read_with_limits<R: Read + Seek>(reader: R, limits: ResourceLimits) -> Result<Workbook> {
    read_with_password_and_limits(reader, None, limits)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLSX workbook, transparently decrypting a password-protected
/// (MS-OFFCRYPTO) file when `password` is supplied.
///
/// A plain `.xlsx` is a ZIP. A password-protected one is an OLE2/CFB container
/// holding the real ZIP encrypted in an `EncryptedPackage` stream; we detect
/// that by magic bytes and decrypt before parsing.
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn read_with_password<R: Read + Seek>(
    mut reader: R,
    password: Option<&str>,
) -> Result<Workbook> {
    read_with_password_and_limits_inner(&mut reader, password, ResourceLimits::default())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLSX workbook, transparently decrypting a password-protected
/// (MS-OFFCRYPTO) file when `password` is supplied，使用指定的资源限制。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，输入不符合格式约束，或解压后数据超过资源限制时返回错误。
pub fn read_with_password_and_limits<R: Read + Seek>(
    mut reader: R,
    password: Option<&str>,
    limits: ResourceLimits,
) -> Result<Workbook> {
    read_with_password_and_limits_inner(&mut reader, password, limits)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 内部实现：根据魔数判断是否需要解密，然后用指定资源限制解压 ZIP。
fn read_with_password_and_limits_inner<R: Read + Seek>(
    reader: &mut R,
    password: Option<&str>,
    limits: ResourceLimits,
) -> Result<Workbook> {
    let mut magic = [0u8; 8];
    let n = reader.read(&mut magic)?;
    reader.seek(SeekFrom::Start(0))?;

    if easyexcel_io::looks_like_cfb(&magic[..n]) {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let scheme = super::crypto::describe_scheme(&bytes)?;
        let Some(pw) = password else {
            return Err(Error::PasswordProtected(scheme));
        };
        let inner = super::crypto::decrypt(&bytes, pw)?;
        return read_zip_with_limits(Cursor::new(inner), limits);
    }

    read_zip_with_limits(reader, limits)
}

/// Parse a plain (unencrypted) XLSX ZIP from a seekable reader，使用默认资源限制。
fn read_zip<R: Read + Seek>(reader: R) -> Result<Workbook> {
    read_zip_with_limits(reader, ResourceLimits::default())
}

/// Parse a plain (unencrypted) XLSX ZIP from a seekable reader，使用指定资源限制。
///
/// 在解压循环中检查：
/// - 单个 entry 解压后大小不超过 `max_file_bytes`（防止单个 entry 为 ZIP bomb）
/// - 所有 entry 解压后累计大小不超过 `max_file_bytes`（防止多个小 entry 累积爆炸）
fn read_zip_with_limits<R: Read + Seek>(reader: R, limits: ResourceLimits) -> Result<Workbook> {
    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(e) => return Err(Error::Zip(e.to_string())),
    };

    let max_bytes = limits.max_file_bytes();

    // Read all parts into memory (so we can re-borrow the archive freely).
    let mut parts: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total_uncompressed: u64 = 0;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();

        // 单个 entry 解压后大小检查（防止 ZIP bomb：高压缩比单文件）
        let entry_size = f.size();
        if entry_size > max_bytes {
            return Err(Error::ResourceLimit {
                resource: "zip_entry_uncompressed_bytes",
                limit: max_bytes,
                actual: entry_size,
            });
        }

        // 累计解压后大小检查（防止 ZIP bomb：多个小 entry 累积爆炸）
        total_uncompressed += entry_size;
        if total_uncompressed > max_bytes {
            return Err(Error::ResourceLimit {
                resource: "zip_total_uncompressed_bytes",
                limit: max_bytes,
                actual: total_uncompressed,
            });
        }

        let capacity = usize::try_from(entry_size)
            .map_err(|_| Error::Zip("ZIP entry exceeds address space".into()))?;
        let mut data = Vec::with_capacity(capacity);
        f.read_to_end(&mut data)?;
        parts.insert(name, data);
    }

    // Fallback encryption detection (a stray EncryptedPackage entry inside a
    // ZIP). The common CFB-wrapped case is handled earlier in read_with_password.
    if parts.contains_key("EncryptedPackage")
        || parts
            .keys()
            .any(|k| k.eq_ignore_ascii_case("EncryptionInfo"))
    {
        return Err(Error::PasswordProtected(
            "ECMA-376 (EncryptedPackage)".to_string(),
        ));
    }

    let workbook_xml = parts
        .get("xl/workbook.xml")
        .ok_or_else(|| Error::Xlsx("missing xl/workbook.xml".into()))?
        .clone();

    // workbook relationships: r:id -> target path
    let rels = parts
        .get("xl/_rels/workbook.xml.rels")
        .map(|b| parse_rels(b))
        .transpose()?
        .unwrap_or_default();

    // shared strings
    let shared_strings = parts
        .get("xl/sharedStrings.xml")
        .map(|b| parse_shared_strings(b))
        .transpose()?
        .unwrap_or_default();

    // styles -> CellStyle per xf
    let xf_styles = parts
        .get("xl/styles.xml")
        .map(|b| parse_styles(b))
        .transpose()?
        .unwrap_or_default();

    // workbook.xml: sheets, date system, defined names
    let wb_info = parse_workbook(&workbook_xml)?;

    let mut wb = Workbook::empty();
    wb.date_system = wb_info.date_system;
    wb.defined_names = wb_info.defined_names;

    // Intern styles into the table; build xf-index -> interned-style-index map.
    let mut xf_to_interned: Vec<u32> = Vec::with_capacity(xf_styles.len());
    for st in &xf_styles {
        xf_to_interned.push(wb.styles.intern(st.clone()));
    }

    // Table parts we pull into the model (so they aren't also kept opaque).
    let mut consumed_tables: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Parse each sheet.
    for sref in &wb_info.sheets {
        // Resolve relationship to a worksheet path.
        let target = rels.get(&sref.rid).cloned();
        let path = match target {
            Some(t) => normalize_part_path(&t),
            None => continue,
        };
        let Some(data) = parts.get(&path) else {
            // Missing target; create empty sheet so indices align.
            let mut sheet = Sheet::new(sref.name.clone());
            sheet.visibility = sref.visibility;
            wb.sheets.push(sheet);
            continue;
        };
        let mut table_rids = Vec::new();
        let mut sheet = parse_worksheet(data, &shared_strings, &xf_to_interned, &mut table_rids)?;
        sheet.name.clone_from(&sref.name);
        sheet.visibility = sref.visibility;

        // Resolve table parts via the worksheet's own relationships.
        if !table_rids.is_empty() {
            let dir = path.rsplit_once('/').map_or("", |(directory, _)| directory);
            let file = path
                .rsplit_once('/')
                .map_or(path.as_str(), |(_, file)| file);
            let rels_path = format!("{dir}/_rels/{file}.rels");
            if let Some(rels_bytes) = parts.get(&rels_path) {
                let sheet_rels = parse_rels(rels_bytes)?;
                for rid in &table_rids {
                    let Some(target) = sheet_rels.get(rid) else {
                        continue;
                    };
                    let tpath = normalize_rel_path(dir, target);
                    if let Some(tbytes) = parts.get(&tpath)
                        && let Some(table) = super::tables::parse_table(tbytes)?
                    {
                        sheet.tables.push(table);
                        consumed_tables.insert(tpath);
                    }
                }
            }
        }
        wb.sheets.push(sheet);
    }

    // Metadata from docProps/core.xml + app.xml (best effort).
    if let Some(core) = parts.get("docProps/core.xml") {
        parse_core_props(core, &mut wb.metadata);
    }
    if let Some(app) = parts.get("docProps/app.xml") {
        parse_app_props(app, &mut wb.metadata);
    }

    // Preserve unknown parts opaquely.
    for (name, data) in &parts {
        if is_known_part(name) {
            continue;
        }
        // Table parts are now modeled (and regenerated on write).
        if consumed_tables.contains(name) {
            continue;
        }
        // Skip the workbook rels (we reconstruct on write) and content types.
        if name.ends_with(".rels") {
            // Keep non-workbook rels opaque so drawings etc. can resolve.
            if name == "xl/_rels/workbook.xml.rels" || name == "_rels/.rels" {
                continue;
            }
        }
        wb.opaque.push(OpaquePart {
            name: name.clone(),
            data: data.clone(),
        });
    }

    if wb.active_sheet >= wb.sheets.len() {
        wb.active_sheet = 0;
    }
    Ok(wb)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse `xl/_rels/workbook.xml.rels` into rId -> Target.
pub(super) fn parse_rels(xml: &[u8]) -> Result<HashMap<String, String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut map = HashMap::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Empty(e) | Event::Start(e) => {
                if local_name(&e) == "Relationship"
                    && let (Some(id), Some(target)) = (attr(&e, "Id"), attr(&e, "Target"))
                {
                    map.insert(id, target);
                }
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(map)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Normalize a relationship target into a zip part path. Targets are relative to
/// `xl/` (the dir of workbook.xml). Handle leading `/` (absolute) too.
pub(super) fn normalize_part_path(target: &str) -> String {
    normalize_rel_path("xl", target)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Resolve a relationship `target` against the directory `base_dir` of the part
/// that owns the relationship, yielding a zip part path. Handles `/`-absolute,
/// `./`, and `../` targets.
pub(super) fn normalize_rel_path(base_dir: &str, target: &str) -> String {
    if let Some(stripped) = target.strip_prefix('/') {
        return stripped.to_string();
    }
    let mut base: Vec<&str> = if base_dir.is_empty() {
        Vec::new()
    } else {
        base_dir.split('/').collect()
    };
    for seg in target.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                base.pop();
            }
            other => base.push(other),
        }
    }
    base.join("/")
}

include!("reader/sheet_ref.rs");
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) struct WorkbookInfo {
    pub(super) sheets: Vec<SheetRef>,
    pub(super) date_system: DateSystem,
    pub(super) defined_names: Vec<DefinedName>,
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) fn parse_workbook(xml: &[u8]) -> Result<WorkbookInfo> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut sheets = Vec::new();
    let mut date_system = DateSystem::Date1900;
    let mut defined_names = Vec::new();

    let mut buf = Vec::new();
    let mut cur_name: Option<(String, Option<usize>, bool)> = None; // (name, scope, hidden)
    let mut cur_name_text = String::new();
    let mut in_defined_name = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Empty(e) | Event::Start(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "workbookPr" => {
                        if let Some(v) = attr(&e, "date1904")
                            && (v == "1" || v.eq_ignore_ascii_case("true"))
                        {
                            date_system = DateSystem::Date1904;
                        }
                    }
                    "sheet" => {
                        let nm = attr(&e, "name").unwrap_or_default();
                        let rid = attr(&e, "id").unwrap_or_default();
                        let state = attr(&e, "state").unwrap_or_default();
                        let visibility = match state.as_str() {
                            "hidden" => Visibility::Hidden,
                            "veryHidden" => Visibility::VeryHidden,
                            _ => Visibility::Visible,
                        };
                        sheets.push(SheetRef {
                            name: nm,
                            rid,
                            visibility,
                        });
                    }
                    "definedName" => {
                        let nm = attr(&e, "name").unwrap_or_default();
                        let scope = attr(&e, "localSheetId").and_then(|s| s.parse::<usize>().ok());
                        let hidden = attr(&e, "hidden")
                            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
                        cur_name = Some((nm, scope, hidden));
                        cur_name_text.clear();
                        in_defined_name = true;
                    }
                    _ => {}
                }
            }
            Event::Text(t) if in_defined_name => {
                cur_name_text.push_str(&text(&t));
            }
            Event::GeneralRef(reference) if in_defined_name => {
                cur_name_text.push_str(&general_ref(&reference));
            }
            Event::End(e) if local_name_end(&e) == "definedName" => {
                if let Some((nm, scope, hidden)) = cur_name.take() {
                    defined_names.push(DefinedName {
                        name: nm,
                        refers_to: std::mem::take(&mut cur_name_text),
                        scope,
                        hidden,
                    });
                }
                in_defined_name = false;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(WorkbookInfo {
        sheets,
        date_system,
        defined_names,
    })
}

/// Parse a worksheet part. Any `<tablePart r:id="…"/>` relationship ids are
/// pushed into `table_rids` for the caller to resolve against the sheet rels.
fn parse_worksheet(
    xml: &[u8],
    shared: &[String],
    xf_to_interned: &[u32],
    table_rids: &mut Vec<String>,
) -> Result<Sheet> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut sheet = Sheet::new("");
    let mut buf = Vec::new();

    // Current cell state.
    let mut cur_row: u32 = 0;
    let mut cell_ref: Option<(u32, u32)> = None;
    let mut cell_type = String::new();
    let mut cell_style: Option<u32> = None; // interned style idx
    let mut in_v = false;
    let mut in_f = false;
    let mut in_is_t = false; // inline string text
    let mut v_text = String::new();
    let mut f_text = String::new();
    let mut is_text = String::new();
    let mut f_is_shared_member = false; // shared formula with no own text

    let mut in_cols = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "cols" => in_cols = true,
                    "row" => {
                        cur_row = attr(&e, "r")
                            .and_then(|s| s.parse::<u32>().ok())
                            .map_or(cur_row, |r| r.saturating_sub(1));
                        let mut info = RowInfo::default();
                        let mut has_info = false;
                        if let Some(h) = attr(&e, "ht").and_then(|s| s.parse::<f64>().ok()) {
                            info.height = Some(h);
                            has_info = true;
                        }
                        if attr(&e, "hidden").as_deref() == Some("1") {
                            info.hidden = true;
                            has_info = true;
                        }
                        if has_info {
                            sheet.rows.insert(cur_row, info);
                        }
                    }
                    "c" => {
                        cell_ref = parse_cell_ref(&e, cur_row);
                        cell_type = attr(&e, "t").unwrap_or_default();
                        cell_style = attr(&e, "s")
                            .and_then(|s| s.parse::<usize>().ok())
                            .and_then(|xf| xf_to_interned.get(xf).copied());
                        v_text.clear();
                        f_text.clear();
                        is_text.clear();
                        f_is_shared_member = false;
                    }
                    "v" => in_v = true,
                    "f" => {
                        in_f = true;
                        // shared formula member with no own text?
                        if attr(&e, "t").as_deref() == Some("shared") {
                            // text may still follow; treat normally
                        }
                    }
                    "t" => in_is_t = true,
                    _ => {}
                }
            }
            Event::Empty(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "col" if in_cols => {
                        parse_col(&e, &mut sheet);
                    }
                    "c" => {
                        // empty cell (likely style-only)
                        if let Some((r, col)) = parse_cell_ref(&e, cur_row)
                            && let Some(si) = attr(&e, "s")
                                .and_then(|s| s.parse::<usize>().ok())
                                .and_then(|xf| xf_to_interned.get(xf).copied())
                        {
                            sheet.set_style(r, col, si);
                        }
                    }
                    "f" => {
                        // self-closing formula = shared member with no text
                        f_is_shared_member = true;
                    }
                    "mergeCell" => {
                        if let Some(rstr) = attr(&e, "ref")
                            && let Some(range) = CellRange::parse_a1(&rstr)
                        {
                            sheet.merged.push(range);
                        }
                    }
                    "pane" => parse_pane(&e, &mut sheet),
                    "tablePart" => {
                        if let Some(rid) = attr(&e, "id") {
                            table_rids.push(rid);
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(t) => {
                if in_v {
                    v_text.push_str(&text(&t));
                } else if in_f {
                    f_text.push_str(&text(&t));
                } else if in_is_t {
                    is_text.push_str(&text(&t));
                }
            }
            Event::GeneralRef(reference) => {
                if in_v {
                    v_text.push_str(&general_ref(&reference));
                } else if in_f {
                    f_text.push_str(&general_ref(&reference));
                } else if in_is_t {
                    is_text.push_str(&general_ref(&reference));
                }
            }
            Event::End(e) => {
                let name = local_name_end(&e);
                match name.as_str() {
                    "cols" => in_cols = false,
                    "v" => in_v = false,
                    "f" => in_f = false,
                    "t" => in_is_t = false,
                    "c" => {
                        if let Some((r, col)) = cell_ref.take() {
                            let has_formula = !f_text.is_empty() || f_is_shared_member;
                            let cell = build_cell(
                                &cell_type,
                                &v_text,
                                &f_text,
                                &is_text,
                                has_formula,
                                shared,
                            );
                            if let Some(si) = cell_style.take() {
                                sheet.set_style(r, col, si);
                            }
                            sheet.set(r, col, cell);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(sheet)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) fn parse_cell_ref(
    e: &quick_xml::events::BytesStart,
    fallback_row: u32,
) -> Option<(u32, u32)> {
    match attr(e, "r") {
        Some(r) => {
            let a = CellAddress::parse_a1(&r)?;
            Some((a.row, a.col))
        }
        None => Some((fallback_row, 0)),
    }
}

fn parse_col(e: &quick_xml::events::BytesStart, sheet: &mut Sheet) {
    let min = attr(e, "min").and_then(|s| s.parse::<u32>().ok());
    let max = attr(e, "max").and_then(|s| s.parse::<u32>().ok());
    let (Some(min), Some(max)) = (min, max) else {
        return;
    };
    let width = attr(e, "width").and_then(|s| s.parse::<f64>().ok());
    let hidden = attr(e, "hidden").as_deref() == Some("1");
    for c in min..=max {
        if c == 0 {
            continue;
        }
        let info = ColInfo {
            width,
            hidden,
            style: None,
        };
        sheet.columns.insert(c - 1, info);
    }
}

// OOXML pane splits are serialized as floating-point values even though the workbook model stores
// whole row/column counts. Values are clamped before this deliberate conversion.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_pane(e: &quick_xml::events::BytesStart, sheet: &mut Sheet) {
    let state = attr(e, "state").unwrap_or_default();
    if state != "frozen" && state != "frozenSplit" {
        // Only frozen panes map to our model.
        return;
    }
    let x = attr(e, "xSplit")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y = attr(e, "ySplit")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let rows = y.clamp(0.0, f64::from(u32::MAX)).trunc() as u32;
    let cols = x.clamp(0.0, f64::from(u32::MAX)).trunc() as u32;
    sheet.frozen = FrozenPanes { rows, cols };
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) fn build_cell(
    t: &str,
    v: &str,
    f: &str,
    inline: &str,
    has_formula: bool,
    shared: &[String],
) -> Cell {
    if has_formula {
        let cached = cached_value(t, v, shared);
        return Cell::Formula {
            expr: f.to_string(),
            cached,
        };
    }
    match t {
        "s" => {
            // shared string index
            match v.trim().parse::<usize>() {
                Ok(idx) => Cell::Text(shared.get(idx).cloned().unwrap_or_default()),
                Err(_) => Cell::Text(String::new()),
            }
        }
        "str" => Cell::Text(v.to_string()),
        "inlineStr" => Cell::Text(inline.to_string()),
        "b" => Cell::Bool(v.trim() == "1"),
        "e" => Cell::Error(CellError::parse(v.trim()).unwrap_or(CellError::Value)),
        _ => {
            // number (t absent or "n")
            if v.is_empty() {
                Cell::Empty
            } else {
                match v.trim().parse::<f64>() {
                    Ok(n) => Cell::Number(n),
                    Err(_) => Cell::Text(v.to_string()),
                }
            }
        }
    }
}

fn cached_value(t: &str, v: &str, shared: &[String]) -> CellValue {
    match t {
        "s" => v
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|i| shared.get(i).cloned())
            .map_or(CellValue::Empty, CellValue::Text),
        "str" => CellValue::Text(v.to_string()),
        "b" => CellValue::Bool(v.trim() == "1"),
        "e" => CellValue::Error(CellError::parse(v.trim()).unwrap_or(CellError::Value)),
        _ => {
            if v.is_empty() {
                CellValue::Empty
            } else {
                match v.trim().parse::<f64>() {
                    Ok(n) => CellValue::Number(n),
                    Err(_) => CellValue::Text(v.to_string()),
                }
            }
        }
    }
}

fn parse_core_props(xml: &[u8], meta: &mut Metadata) {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut cur: Option<&'static str> = None;
    let mut text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) | Err(_) => break,
            Ok(Event::Start(e)) => {
                cur = match local_name(&e).as_str() {
                    "title" => Some("title"),
                    "creator" => Some("author"),
                    "created" => Some("created"),
                    "modified" => Some("modified"),
                    _ => None,
                };
                text.clear();
            }
            Ok(Event::Text(t)) if cur.is_some() => {
                text.push_str(&super::xmlutil::text(&t));
            }
            Ok(Event::GeneralRef(reference)) if cur.is_some() => {
                text.push_str(&super::xmlutil::general_ref(&reference));
            }
            Ok(Event::End(_)) => {
                if let Some(field) = cur.take() {
                    let val = std::mem::take(&mut text);
                    if !val.is_empty() {
                        match field {
                            "title" => meta.title = Some(val),
                            "author" => meta.author = Some(val),
                            "created" => meta.created = Some(val),
                            "modified" => meta.modified = Some(val),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        buf.clear();
    }
}

fn parse_app_props(xml: &[u8], meta: &mut Metadata) {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut cur: Option<&'static str> = None;
    let mut text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) | Err(_) => break,
            Ok(Event::Start(e)) => {
                cur = match local_name(&e).as_str() {
                    "Company" => Some("company"),
                    "Application" => Some("application"),
                    _ => None,
                };
                text.clear();
            }
            Ok(Event::Text(t)) if cur.is_some() => {
                text.push_str(&super::xmlutil::text(&t));
            }
            Ok(Event::GeneralRef(reference)) if cur.is_some() => {
                text.push_str(&super::xmlutil::general_ref(&reference));
            }
            Ok(Event::End(_)) => {
                if let Some(field) = cur.take() {
                    let val = std::mem::take(&mut text);
                    if !val.is_empty() {
                        match field {
                            "company" => meta.company = Some(val),
                            "application" => meta.application = Some(val),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        buf.clear();
    }
}

#[cfg(test)]
#[path = "reader_tests/tests.rs"]
mod tests;
