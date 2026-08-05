//! XLSX (OOXML SpreadsheetML) writer.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::{Seek, Write};

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use easyexcel_io::{Error, Result};
use easyexcel_model::addr::{CellAddress, col_index_to_letters};
use easyexcel_model::dates::DateSystem;
use easyexcel_model::model::{Cell, Sheet, Visibility, Workbook};
use easyexcel_model::value::CellValue;

use super::styles::write_styles;
use super::tables::build_table_xml;
use super::xmlutil::{attr, local_name, needs_preserve, xml_escape};

const TABLE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table";

/// Everything needed to emit the workbook's table objects: regenerated table
/// parts, sheet rels, the per-sheet `<tableParts>` relationship ids, and the
/// set of opaque parts those regenerated files supersede.
#[derive(Default)]
struct TablePlan {
    /// Per sheet index: the rIds to list in `<tableParts>`, in order.
    sheet_rids: Vec<Vec<String>>,
    /// `(part_path, xml_bytes)` for each `xl/tables/tableN.xml`.
    table_parts: Vec<(String, Vec<u8>)>,
    /// `(rels_path, xml)` for each regenerated `xl/worksheets/_rels/*.rels`.
    sheet_rels: Vec<(String, String)>,
    /// Part paths the plan regenerates — skipped when re-emitting opaque parts.
    superseded: HashSet<String>,
    /// `/xl/tables/tableN.xml` paths, for the content-type overrides.
    table_part_names: Vec<String>,
}

/// Parse a `.rels` part into ordered `(Id, Type, Target)` triples.
fn parse_rels_triples(xml: &[u8]) -> Vec<(String, String, String)> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut out = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) | Err(_) => break,
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) if local_name(&e) == "Relationship" => {
                if let (Some(id), Some(ty), Some(tgt)) =
                    (attr(&e, "Id"), attr(&e, "Type"), attr(&e, "Target"))
                {
                    out.push((id, ty, tgt));
                }
            }
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Build the table plan for `wb`, numbering table parts globally (table1.xml,
/// table2.xml, …) and assigning each a sheet-local relationship id that does not
/// collide with relationships preserved opaquely for that sheet.
fn plan_tables(wb: &Workbook) -> TablePlan {
    let mut plan = TablePlan {
        sheet_rids: vec![Vec::new(); wb.sheets.len()],
        ..Default::default()
    };
    let mut part_counter: u32 = 0;

    // Index opaque parts by name for quick preserved-rels lookup.
    let opaque_by_name: HashMap<&str, &[u8]> = wb
        .opaque
        .iter()
        .map(|p| (p.name.as_str(), p.data.as_slice()))
        .collect();

    for (i, sheet) in wb.sheets.iter().enumerate() {
        if sheet.tables.is_empty() {
            continue;
        }
        let rels_path = format!("xl/worksheets/_rels/sheet{}.xml.rels", i + 1);
        // Preserve any non-table relationships already on this sheet.
        let preserved: Vec<(String, String, String)> = opaque_by_name
            .get(rels_path.as_str())
            .map(|b| parse_rels_triples(b))
            .unwrap_or_default();
        // Number new table rels above the *kept* (non-table) rels only, so the
        // ids stay stable across repeated saves instead of climbing each time.
        let kept: Vec<&(String, String, String)> = preserved
            .iter()
            .filter(|(_, ty, _)| ty != TABLE_REL_TYPE)
            .collect();
        let mut next_rid = kept
            .iter()
            .filter_map(|(id, _, _)| id.strip_prefix("rId").and_then(|n| n.parse::<u32>().ok()))
            .max()
            .unwrap_or(0)
            + 1;

        let mut rels_body = String::new();
        for (id, ty, tgt) in &preserved {
            if ty == TABLE_REL_TYPE {
                continue; // drop stale table rels; we re-add fresh ones
            }
            let _ = write!(
                rels_body,
                r#"<Relationship Id="{}" Type="{}" Target="{}"/>"#,
                xml_escape(id),
                xml_escape(ty),
                xml_escape(tgt)
            );
        }

        for table in &sheet.tables {
            part_counter += 1;
            let part_name = format!("xl/tables/table{part_counter}.xml");
            let rid = format!("rId{next_rid}");
            next_rid += 1;

            let _ = write!(
                rels_body,
                r#"<Relationship Id="{}" Type="{}" Target="../tables/table{}.xml"/>"#,
                xml_escape(&rid),
                TABLE_REL_TYPE,
                part_counter
            );

            plan.sheet_rids[i].push(rid);
            plan.table_parts
                .push((part_name.clone(), build_table_xml(table, part_counter)));
            plan.table_part_names.push(format!("/{part_name}"));
            plan.superseded.insert(part_name);
        }

        let mut rels_xml = String::new();
        rels_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        rels_xml.push_str(
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        rels_xml.push_str(&rels_body);
        rels_xml.push_str("</Relationships>");
        plan.superseded.insert(rels_path.clone());
        plan.sheet_rels.push((rels_path, rels_xml));
    }

    plan
}

/// Write a workbook as XLSX to any seekable writer.
pub fn write<W: Write + Seek>(wb: &Workbook, writer: W) -> Result<()> {
    let mut zip = ZipWriter::new(writer);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Build the shared string table (dedup) from all text cells.
    let (sst, sst_index) = build_shared_strings(wb);

    // Plan table objects (parts, sheet rels, content-type overrides).
    let table_plan = plan_tables(wb);

    // -- [Content_Types].xml --
    let content_types = build_content_types(wb, &table_plan.table_part_names);
    start(&mut zip, "[Content_Types].xml", &opts)?;
    zip.write_all(content_types.as_bytes())?;

    // -- _rels/.rels --
    start(&mut zip, "_rels/.rels", &opts)?;
    zip.write_all(ROOT_RELS.as_bytes())?;

    // -- xl/workbook.xml --
    start(&mut zip, "xl/workbook.xml", &opts)?;
    zip.write_all(build_workbook_xml(wb).as_bytes())?;

    // -- xl/_rels/workbook.xml.rels --
    start(&mut zip, "xl/_rels/workbook.xml.rels", &opts)?;
    zip.write_all(build_workbook_rels(wb).as_bytes())?;

    // -- xl/sharedStrings.xml --
    start(&mut zip, "xl/sharedStrings.xml", &opts)?;
    zip.write_all(build_shared_strings_xml(&sst).as_bytes())?;

    // -- xl/styles.xml --
    start(&mut zip, "xl/styles.xml", &opts)?;
    zip.write_all(&write_styles(&wb.styles))?;

    // -- worksheets --
    for (i, sheet) in wb.sheets.iter().enumerate() {
        let path = format!("xl/worksheets/sheet{}.xml", i + 1);
        start(&mut zip, &path, &opts)?;
        let table_rids = table_plan
            .sheet_rids
            .get(i)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let xml = build_worksheet_xml(wb, sheet, &sst_index, table_rids);
        zip.write_all(xml.as_bytes())?;
    }

    // -- table parts + regenerated sheet rels --
    for (name, xml) in &table_plan.table_parts {
        start(&mut zip, name, &opts)?;
        zip.write_all(xml)?;
    }
    for (name, xml) in &table_plan.sheet_rels {
        start(&mut zip, name, &opts)?;
        zip.write_all(xml.as_bytes())?;
    }

    // -- docProps/core.xml + app.xml --
    start(&mut zip, "docProps/core.xml", &opts)?;
    zip.write_all(build_core_props(wb).as_bytes())?;
    start(&mut zip, "docProps/app.xml", &opts)?;
    zip.write_all(build_app_props(wb).as_bytes())?;

    // -- preserved opaque parts (verbatim) --
    // Skip parts we generate ourselves and rels/content-types to avoid conflicts.
    for part in &wb.opaque {
        let n = part.name.as_str();
        if is_generated(n) || table_plan.superseded.contains(n) {
            continue;
        }
        start(&mut zip, n, &opts)?;
        zip.write_all(&part.data)?;
    }

    zip.finish().map_err(|e| Error::Zip(e.to_string()))?;
    Ok(())
}

fn is_generated(name: &str) -> bool {
    name == "[Content_Types].xml"
        || name == "_rels/.rels"
        || name == "xl/workbook.xml"
        || name == "xl/_rels/workbook.xml.rels"
        || name == "xl/sharedStrings.xml"
        || name == "xl/styles.xml"
        || name == "xl/calcChain.xml"
        || name == "docProps/core.xml"
        || name == "docProps/app.xml"
        || (name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml"))
}

fn start<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    name: &str,
    opts: &SimpleFileOptions,
) -> Result<()> {
    zip.start_file(name, *opts)
        .map_err(|e| Error::Zip(e.to_string()))
}

fn build_shared_strings(wb: &Workbook) -> (Vec<String>, HashMap<String, usize>) {
    let mut list: Vec<String> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for sheet in &wb.sheets {
        for cell in sheet.cells.values() {
            if let Cell::Text(s) = cell
                && !index.contains_key(s)
            {
                index.insert(s.clone(), list.len());
                list.push(s.clone());
            }
        }
        // 动态数组 spill 区域的值单元格也写入 sheetData（缓存持久化），
        // 其中的文本值必须进入共享字符串表。
        for spill in sheet.spills.values() {
            for v in &spill.values {
                if let CellValue::Text(s) = v
                    && !index.contains_key(s)
                {
                    index.insert(s.clone(), list.len());
                    list.push(s.clone());
                }
            }
        }
    }
    (list, index)
}

fn build_shared_strings_xml(sst: &[String]) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    let _ = write!(
        s,
        r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{}" uniqueCount="{}">"#,
        sst.len(),
        sst.len()
    );
    for text in sst {
        if needs_preserve(text) {
            let _ = write!(
                s,
                r#"<si><t xml:space="preserve">{}</t></si>"#,
                xml_escape(text)
            );
        } else {
            let _ = write!(s, "<si><t>{}</t></si>", xml_escape(text));
        }
    }
    s.push_str("</sst>");
    s
}

fn build_content_types(wb: &Workbook, table_part_names: &[String]) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#);
    s.push_str(r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#);
    s.push_str(r#"<Default Extension="xml" ContentType="application/xml"/>"#);
    // Common media defaults so preserved drawings/images don't break.
    s.push_str(r#"<Default Extension="png" ContentType="image/png"/>"#);
    s.push_str(r#"<Default Extension="jpeg" ContentType="image/jpeg"/>"#);
    s.push_str(r#"<Default Extension="emf" ContentType="image/x-emf"/>"#);
    s.push_str(r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#);
    s.push_str(r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#);
    s.push_str(r#"<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#);
    for i in 0..wb.sheets.len() {
        let _ = write!(
            s,
            r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
            i + 1
        );
    }
    for name in table_part_names {
        let _ = write!(
            s,
            r#"<Override PartName="{name}" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.table+xml"/>"#
        );
    }
    s.push_str(r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#);
    s.push_str(r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#);
    s.push_str("</Types>");
    s
}

const ROOT_RELS: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>"#,
    r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>"#,
    r#"<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>"#,
    r#"</Relationships>"#
);

fn build_workbook_xml(wb: &Workbook) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#);
    if wb.date_system == DateSystem::Date1904 {
        s.push_str(r#"<workbookPr date1904="1"/>"#);
    }
    s.push_str("<sheets>");
    for (i, sheet) in wb.sheets.iter().enumerate() {
        let state = match sheet.visibility {
            Visibility::Visible => "",
            Visibility::Hidden => r#" state="hidden""#,
            Visibility::VeryHidden => r#" state="veryHidden""#,
        };
        let _ = write!(
            s,
            r#"<sheet name="{}" sheetId="{}"{} r:id="rId{}"/>"#,
            xml_escape(&sheet.name),
            i + 1,
            state,
            i + 1
        );
    }
    s.push_str("</sheets>");
    if !wb.defined_names.is_empty() {
        s.push_str("<definedNames>");
        for dn in &wb.defined_names {
            s.push_str("<definedName");
            let _ = write!(s, r#" name="{}""#, xml_escape(&dn.name));
            if let Some(scope) = dn.scope {
                let _ = write!(s, r#" localSheetId="{}""#, scope);
            }
            if dn.hidden {
                s.push_str(r#" hidden="1""#);
            }
            let _ = write!(s, ">{}</definedName>", xml_escape(&dn.refers_to));
        }
        s.push_str("</definedNames>");
    }
    s.push_str("</workbook>");
    s
}

fn build_workbook_rels(wb: &Workbook) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for i in 0..wb.sheets.len() {
        let _ = write!(
            s,
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
            i + 1,
            i + 1
        );
    }
    let n = wb.sheets.len();
    let _ = write!(
        s,
        r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        n + 1
    );
    let _ = write!(
        s,
        r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>"#,
        n + 2
    );
    s.push_str("</Relationships>");
    s
}

fn build_worksheet_xml(
    wb: &Workbook,
    sheet: &Sheet,
    sst: &HashMap<String, usize>,
    table_rids: &[String],
) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#);

    // dimension
    let (max_row, max_col) = sheet.dimensions();
    if max_row > 0 && max_col > 0 {
        let start = CellAddress::new(0, 0).to_a1_relative();
        let end = CellAddress::new(max_row - 1, max_col - 1).to_a1_relative();
        let _ = write!(s, r#"<dimension ref="{}:{}"/>"#, start, end);
    } else {
        s.push_str(r#"<dimension ref="A1"/>"#);
    }

    // sheetViews with frozen panes
    if sheet.frozen.rows > 0 || sheet.frozen.cols > 0 {
        let top_left = CellAddress::new(sheet.frozen.rows, sheet.frozen.cols).to_a1_relative();
        s.push_str("<sheetViews><sheetView workbookViewId=\"0\">");
        let _ = write!(
            s,
            r#"<pane xSplit="{}" ySplit="{}" topLeftCell="{}" activePane="bottomRight" state="frozen"/>"#,
            sheet.frozen.cols, sheet.frozen.rows, top_left
        );
        s.push_str("</sheetView></sheetViews>");
    }

    // cols
    if !sheet.columns.is_empty() {
        s.push_str("<cols>");
        for (col, info) in &sheet.columns {
            let _ = write!(s, r#"<col min="{}" max="{}""#, col + 1, col + 1);
            if let Some(w) = info.width {
                let _ = write!(s, r#" width="{}" customWidth="1""#, w);
            }
            if info.hidden {
                s.push_str(r#" hidden="1""#);
            }
            s.push_str("/>");
        }
        s.push_str("</cols>");
    }

    // sheetData: group cells by row.
    s.push_str("<sheetData>");
    // Build a map row -> sorted cells (already sorted via BTreeMap ordering by (row,col)).
    let mut rows: Vec<u32> = Vec::new();
    // Combine cell rows + style-only rows + row-info rows.
    let mut row_set: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for &(r, _) in sheet.cells.keys() {
        row_set.insert(r);
    }
    for &(r, _) in sheet.styles.keys() {
        row_set.insert(r);
    }
    for &r in sheet.rows.keys() {
        row_set.insert(r);
    }
    // 动态数组 spill 区域的值单元格（anchor 之外）也写出，持久化缓存。
    // anchor 本身是 cells 中的真实公式单元格，这里只补派生单元格的坐标。
    let mut spill_rows: std::collections::BTreeMap<u32, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for (&(ar, ac), sp) in &sheet.spills {
        for i in 0..sp.rows {
            let r = ar + i;
            for j in 0..sp.cols {
                if i == 0 && j == 0 {
                    continue; // anchor：由公式单元格写出
                }
                let v = &sp.values[(i * sp.cols + j) as usize];
                if !matches!(v, CellValue::Empty) {
                    spill_rows.entry(r).or_default().insert(ac + j);
                }
            }
        }
    }
    for r in spill_rows.keys() {
        row_set.insert(*r);
    }
    rows.extend(row_set);

    for r in rows {
        let _ = write!(s, r#"<row r="{}""#, r + 1);
        if let Some(info) = sheet.rows.get(&r) {
            if let Some(h) = info.height {
                let _ = write!(s, r#" ht="{}" customHeight="1""#, h);
            }
            if info.hidden {
                s.push_str(r#" hidden="1""#);
            }
        }
        s.push('>');

        // Collect columns present in this row from cells and styles.
        let mut cols: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        for (&(rr, cc), _) in sheet.cells.range((r, 0)..(r + 1, 0)) {
            debug_assert_eq!(rr, r);
            cols.insert(cc);
        }
        for (&(rr, cc), _) in sheet.styles.range((r, 0)..(r + 1, 0)) {
            debug_assert_eq!(rr, r);
            cols.insert(cc);
        }
        if let Some(spill_cols) = spill_rows.get(&r) {
            for c in spill_cols {
                cols.insert(*c);
            }
        }

        for c in cols {
            write_cell(&mut s, wb, sheet, r, c, sst);
        }
        s.push_str("</row>");
    }
    s.push_str("</sheetData>");

    // mergeCells
    if !sheet.merged.is_empty() {
        let _ = write!(s, r#"<mergeCells count="{}">"#, sheet.merged.len());
        for m in &sheet.merged {
            let a = m.start.to_a1_relative();
            let b = m.end.to_a1_relative();
            let _ = write!(s, r#"<mergeCell ref="{}:{}"/>"#, a, b);
        }
        s.push_str("</mergeCells>");
    }

    // tableParts must be the last child of <worksheet>.
    if !table_rids.is_empty() {
        let _ = write!(s, r#"<tableParts count="{}">"#, table_rids.len());
        for rid in table_rids {
            let _ = write!(s, r#"<tablePart r:id="{}"/>"#, xml_escape(rid));
        }
        s.push_str("</tableParts>");
    }

    s.push_str("</worksheet>");
    s
}

fn write_cell(
    s: &mut String,
    _wb: &Workbook,
    sheet: &Sheet,
    r: u32,
    c: u32,
    sst: &HashMap<String, usize>,
) {
    let ref_a1 = format!("{}{}", col_index_to_letters(c), r + 1);
    let style = sheet.style_at(r, c);
    let cell = sheet.get(r, c);

    // Open <c>.
    let _ = write!(s, r#"<c r="{}""#, ref_a1);
    if let Some(si) = style {
        let _ = write!(s, r#" s="{}""#, si);
    }

    match cell {
        None | Some(Cell::Empty) => {
            // 动态数组 spill 派生单元格：以缓存值写出（无公式），
            // 读回时无需重算即可见结果。anchor 走 Formula 分支。
            if let Some(spill) = sheet.spilled_at(r, c) {
                match spill {
                    CellValue::Number(n) => {
                        let _ = write!(s, "><v>{}</v></c>", fmt_num(*n));
                    }
                    CellValue::Bool(b) => {
                        let _ = write!(s, r#" t="b"><v>{}</v></c>"#, if *b { 1 } else { 0 });
                    }
                    CellValue::Error(e) => {
                        let _ = write!(s, r#" t="e"><v>{}</v></c>"#, xml_escape(e.as_str()));
                    }
                    CellValue::Text(text) => {
                        let idx = sst.get(text).copied().unwrap_or(0);
                        let _ = write!(s, r#" t="s"><v>{}</v></c>"#, idx);
                    }
                    _ => {
                        // 嵌套数组/引用等非常规值：无标量缓存可写
                        s.push_str("/>");
                    }
                }
            } else {
                // Style-only cell.
                s.push_str("/>");
            }
        }
        Some(Cell::Number(n)) => {
            let _ = write!(s, "><v>{}</v></c>", fmt_num(*n));
        }
        Some(Cell::Bool(b)) => {
            let _ = write!(s, r#" t="b"><v>{}</v></c>"#, if *b { 1 } else { 0 });
        }
        Some(Cell::Error(e)) => {
            let _ = write!(s, r#" t="e"><v>{}</v></c>"#, xml_escape(e.as_str()));
        }
        Some(Cell::Text(text)) => {
            let idx = sst.get(text).copied().unwrap_or(0);
            let _ = write!(s, r#" t="s"><v>{}</v></c>"#, idx);
        }
        Some(Cell::Formula { expr, cached }) => {
            write_formula_cell(s, expr, cached);
        }
    }
}

fn write_formula_cell(s: &mut String, expr: &str, cached: &CellValue) {
    // Type attribute depends on cached value.
    let expr_clean = expr.strip_prefix('=').unwrap_or(expr);
    match cached {
        CellValue::Text(t) => {
            let _ = write!(
                s,
                r#" t="str"><f>{}</f><v>{}</v></c>"#,
                xml_escape(expr_clean),
                xml_escape(t)
            );
        }
        CellValue::Bool(b) => {
            let _ = write!(
                s,
                r#" t="b"><f>{}</f><v>{}</v></c>"#,
                xml_escape(expr_clean),
                if *b { 1 } else { 0 }
            );
        }
        CellValue::Error(e) => {
            let _ = write!(
                s,
                r#" t="e"><f>{}</f><v>{}</v></c>"#,
                xml_escape(expr_clean),
                xml_escape(e.as_str())
            );
        }
        CellValue::Number(n) => {
            let _ = write!(
                s,
                "><f>{}</f><v>{}</v></c>",
                xml_escape(expr_clean),
                fmt_num(*n)
            );
        }
        CellValue::Empty => {
            let _ = write!(s, "><f>{}</f></c>", xml_escape(expr_clean));
        }
    }
}

fn fmt_num(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    format!("{}", n)
}

fn build_core_props(wb: &Workbook) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#);
    if let Some(t) = &wb.metadata.title {
        let _ = write!(s, "<dc:title>{}</dc:title>", xml_escape(t));
    }
    if let Some(a) = &wb.metadata.author {
        let _ = write!(s, "<dc:creator>{}</dc:creator>", xml_escape(a));
    }
    if let Some(c) = &wb.metadata.created {
        let _ = write!(
            s,
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>"#,
            xml_escape(c)
        );
    }
    if let Some(m) = &wb.metadata.modified {
        let _ = write!(
            s,
            r#"<dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>"#,
            xml_escape(m)
        );
    }
    s.push_str("</cp:coreProperties>");
    s
}

fn build_app_props(wb: &Workbook) -> String {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    s.push_str(r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">"#);
    let app = wb.metadata.application.as_deref().unwrap_or("xls-rs");
    let _ = write!(s, "<Application>{}</Application>", xml_escape(app));
    if let Some(co) = &wb.metadata.company {
        let _ = write!(s, "<Company>{}</Company>", xml_escape(co));
    }
    s.push_str("</Properties>");
    s
}
