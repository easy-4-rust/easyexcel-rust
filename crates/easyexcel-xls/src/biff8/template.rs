//! Minimal BIFF8 `.xls` template package (Java `withTemplate` / HSSF subset).
//!
//! # Approach
//!
//! Loads the OLE/CFB container, parses the `Workbook` stream into BIFF records,
//! and **preserves every untouched record byte-for-byte** (FONT / XF / SST /
//! MERGECELLS / existing cells). New values are inserted as inline `LABEL`
//! (0x0204) or `NUMBER` / `BOOLERR` / `BLANK` records immediately before the
//! target sheet's `EOF`, then `DIMENSION` and `BOUNDSHEET` stream offsets are
//! repaired. Other OLE streams (`SummaryInformation`, …) are kept by rewriting
//! only the `Workbook` / `Book` stream in place.
//!
//! # Java mapping
//!
//! | Java `EasyExcel` / POI | Rust |
//! |---|---|
//! | `EasyExcel.write(...).withTemplate(xls).sheet().doWrite(data)` | [`Biff8TemplatePackage`] + writer wiring |
//! | `HSSFWorkbook(templateStream)` | OLE open + Workbook record parse |
//! | `sheet.createRow(...).createCell(...).setCellValue(...)` | [`Biff8TemplatePackage::set_cell`] |
//! | POI keeps unedited records | unchanged BIFF records copied verbatim |
//!
//! # Still unsupported
//!
//! Placeholder `fill` (Java `ExcelWriter.fill` on POI `HSSFWorkbook`) remains
//! [`ExcelError::Unsupported`] at the template crate — list / `forceNewRow` /
//! horizontal fill need row insertion and SST mutation beyond this MVP.
//! Password-encrypted legacy workbooks are rejected.
//!
//! For `.xls` cell append (Java `withTemplate` + `doWrite`), use this package
//! via the writer facade instead of OOXML fill.

use std::io::{Cursor, Read, Write};
use std::path::Path;

use cfb::CompoundFile;
use easyexcel_io::{Error as ExcelError, Result};

use super::encode::{
    BLANK, BOF, BOOLERR, BOUNDSHEET, DIMENSION, DT_WORKSHEET, EOF, FORMULA, LABEL, LABELSST,
    MAX_RECORD_DATA, MERGECELLS, NUMBER, RK, SST, XF_GENERAL, encode_rk, encode_unicode_string,
    pack_merge_range,
};
use super::{Biff8Cell, Biff8Merge, Biff8Value};

/// One framed BIFF record (`type` + payload).
#[derive(Debug, Clone)]
struct RawRecord {
    typ: u16,
    data: Vec<u8>,
}

/// Worksheet location inside the globals / sheet record list.
#[derive(Debug, Clone)]
struct SheetSpan {
    name: String,
    /// Index of the worksheet `BOF` record.
    bof_index: usize,
    /// Index of the worksheet `EOF` record (exclusive insert point is this index).
    eof_index: usize,
    /// Index of the `DIMENSION` record inside this sheet, when present.
    dimension_index: Option<usize>,
}

/// In-memory `.xls` template with record-preserving cell writes.
///
/// Corresponds to a loaded POI `HSSFWorkbook` used only for appending / overlay
/// cells while leaving the rest of the BIFF stream intact.
#[derive(Debug, Clone)]
pub struct Biff8TemplatePackage {
    /// Full OLE/CFB bytes (all streams); only `Workbook` is rewritten on save.
    ole_bytes: Vec<u8>,
    /// Workbook stream path (`Workbook` or `Book`).
    workbook_path: String,
    /// Parsed BIFF records from the Workbook stream.
    records: Vec<RawRecord>,
    /// Bound sheets in workbook order.
    sheets: Vec<SheetSpan>,
}

impl Biff8TemplatePackage {
    /// Loads an OLE `.xls` template from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when the bytes are not a readable BIFF8
    /// workbook, or [`ExcelError::Unsupported`] for empty / unusable templates.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            return Err(ExcelError::Xls(
                "xls template is not an OLE Compound File".to_owned(),
            ));
        }
        let (workbook_path, workbook) = read_workbook_stream(bytes)?;
        let records = split_records(&workbook)?;
        let sheets = discover_sheets(&records)?;
        if sheets.is_empty() {
            return Err(ExcelError::Xls(
                "xls template Workbook contains no worksheets".to_owned(),
            ));
        }
        Ok(Self {
            ole_bytes: bytes.to_vec(),
            workbook_path,
            records,
            sheets,
        })
    }

    /// Loads an OLE `.xls` template from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors from [`Self::from_bytes`].
    pub fn from_path(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(ExcelError::from)?;
        Self::from_bytes(&bytes)
    }

    /// Returns worksheet names in `BoundSheet` order.
    #[must_use]
    pub fn sheet_names(&self) -> Vec<String> {
        self.sheets.iter().map(|sheet| sheet.name.clone()).collect()
    }

    /// Returns the next zero-based append row for a sheet (Java `lastRowNum + 1`).
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when the sheet is absent.
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        let sheet = self.sheet(sheet_name)?;
        Ok(sheet_max_row(&self.records, sheet).map_or(0, |row| u32::from(row).saturating_add(1)))
    }

    /// Writes a cell value at `(row, col)`, replacing any existing cell record.
    ///
    /// Existing XF indexes are reused when overwriting a cell; new cells use
    /// [`XF_GENERAL`]. Unrelated records are left untouched.
    ///
    /// # Errors
    ///
    /// Returns format errors for out-of-range coordinates or unsupported values.
    pub fn set_cell(
        &mut self,
        sheet_name: &str,
        row: u32,
        col: usize,
        cell: &Biff8Cell,
    ) -> Result<()> {
        let row = u16::try_from(row)
            .map_err(|_| ExcelError::Xls("BIFF8 supports at most 65536 rows".to_owned()))?;
        let col = u8::try_from(col)
            .map_err(|_| ExcelError::Xls("BIFF8 supports at most 256 columns".to_owned()))?;
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = self.sheets[sheet_index].clone();
        let existing = find_cell_record(&self.records, &sheet, row, col);
        let xf = if let Some(index) = existing {
            // Preserve the template cell's XF (styles) when overwriting a value.
            if self.records[index].data.len() >= 6 {
                u16::from_le_bytes([self.records[index].data[4], self.records[index].data[5]])
            } else {
                cell.xf
            }
        } else {
            cell.xf
        };
        let payload = encode_cell_record(row, col, xf, &cell.value)?;
        if let Some(index) = existing {
            self.records[index] = payload;
        } else {
            let insert_at = self.sheets[sheet_index].eof_index;
            self.records.insert(insert_at, payload);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    /// Adds one inclusive merge range while preserving all existing BIFF records.
    ///
    /// Java `HSSFSheet.addMergedRegionUnsafe` permits multiple MERGECELLS
    /// records, so a one-range record can be inserted directly before the
    /// target worksheet EOF without rewriting pre-existing merge tables.
    ///
    /// # Errors
    ///
    /// Returns a format error when the sheet does not exist.
    pub fn add_merge_range(&mut self, sheet_name: &str, range: Biff8Merge) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let mut data = Vec::with_capacity(10);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&pack_merge_range(
            range.first_row,
            range.last_row,
            u16::from(range.first_col),
            u16::from(range.last_col),
        ));
        let insert_at = self.sheets[sheet_index].eof_index;
        self.records.insert(
            insert_at,
            RawRecord {
                typ: MERGECELLS,
                data,
            },
        );
        self.adjust_indices_after_insert(sheet_index, insert_at);
        Ok(())
    }

    /// Serializes the package back to OLE/CFB bytes.
    ///
    /// # Errors
    ///
    /// Returns format or I/O errors when the Workbook stream cannot be rewritten.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let workbook = assemble_workbook(&self.records)?;
        rewrite_workbook_stream(&self.ole_bytes, &self.workbook_path, &workbook)
    }

    /// Returns all cell placeholders (`{key}` patterns) found in
    /// LABEL/LABELSST records across the workbook, resolving SST
    /// references when an SST record is present.
    ///
    /// Each entry is `(sheet_name, row, col, placeholder_text)`.
    #[must_use]
    pub fn scan_placeholders(&self) -> Vec<(String, u16, u8, String)> {
        let sst_strings = parse_sst(&self.records);
        let mut placeholders = Vec::new();
        for sheet in &self.sheets {
            for (idx, record) in self.records.iter().enumerate() {
                if idx < sheet.bof_index || idx >= sheet.eof_index {
                    continue;
                }
                let (row, col, text) = match record.typ {
                    LABEL => decode_label_payload(&record.data),
                    LABELSST => {
                        let (row, col, sst_idx) = decode_labelsst_index(&record.data);
                        let text = sst_idx.and_then(|i| sst_strings.get(i as usize).cloned());
                        (row, col, text)
                    }
                    _ => continue,
                };
                if let Some(ref text) = text
                    && text.contains('{')
                    && text.contains('}')
                {
                    placeholders.push((sheet.name.clone(), row, col, text.clone()));
                }
            }
        }
        placeholders
    }

    /// Replaces a cell value at `(row, col)` on the given sheet with
    /// a new BIFF8 LABEL record containing the replacement text.
    /// If the original record was a LABELSST (SST reference), it is
    /// replaced with a LABEL record carrying the inline string value.
    ///
    /// # Errors
    ///
    /// Returns format errors for out-of-range coordinates.
    pub fn replace_label(
        &mut self,
        sheet_name: &str,
        row: u16,
        col: u8,
        replacement: &str,
    ) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = &self.sheets[sheet_index];
        let existing = find_cell_record(&self.records, sheet, row, col);
        let xf = if let Some(index) = existing {
            if self.records[index].data.len() >= 6 {
                u16::from_le_bytes([self.records[index].data[4], self.records[index].data[5]])
            } else {
                XF_GENERAL
            }
        } else {
            XF_GENERAL
        };
        // Always use LABEL (inline string) for replacements, even when
        // the original was LABELSST — this avoids SST mutation and
        // ensures the replacement text is self-contained.
        let _cell = Biff8Cell {
            value: Biff8Value::Text(replacement.to_owned()),
            xf,
        };
        // Force LABEL record type for replacement
        let payload = encode_label_record(row, col, xf, replacement)?;
        if let Some(index) = existing {
            self.records[index] = payload;
        } else {
            let insert_at = self.sheets[sheet_index].eof_index;
            self.records.insert(insert_at, payload);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    /// Writes the package to a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors.
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let bytes = self.to_bytes()?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes).map_err(ExcelError::from)
    }

    /// Writes the package to an arbitrary writer.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors.
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        let bytes = self.to_bytes()?;
        output.write_all(&bytes)?;
        output.flush()?;
        Ok(())
    }

    fn sheet(&self, name: &str) -> Result<&SheetSpan> {
        self.sheets
            .iter()
            .find(|sheet| sheet.name == name)
            .ok_or_else(|| ExcelError::Xls(format!("worksheet not found: {name}")))
    }

    fn sheet_index(&self, name: &str) -> Result<usize> {
        self.sheets
            .iter()
            .position(|sheet| sheet.name == name)
            .ok_or_else(|| ExcelError::Xls(format!("worksheet not found: {name}")))
    }

    /// After inserting a record at `insert_at`, shift later sheet indices.
    fn adjust_indices_after_insert(&mut self, sheet_index: usize, insert_at: usize) {
        for (index, sheet) in self.sheets.iter_mut().enumerate() {
            if sheet.bof_index >= insert_at {
                sheet.bof_index += 1;
            }
            if sheet.eof_index >= insert_at {
                sheet.eof_index += 1;
            }
            if let Some(dim) = sheet.dimension_index.as_mut()
                && *dim >= insert_at
            {
                *dim += 1;
            }
            if index == sheet_index {
                // Insert is always before EOF of this sheet.
                debug_assert!(sheet.eof_index > insert_at || sheet.eof_index == insert_at + 1);
            }
        }
    }

    fn refresh_dimension(&mut self, sheet_index: usize) {
        let sheet = self.sheets[sheet_index].clone();
        let (max_row, max_col) = sheet_dimensions(&self.records, &sheet);
        let mut data = Vec::with_capacity(14);
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&u32::from(max_row).to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&u16::from(max_col).to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        if let Some(dim_index) = sheet.dimension_index {
            self.records[dim_index] = RawRecord {
                typ: DIMENSION,
                data,
            };
        } else {
            let insert_at = sheet.bof_index + 1;
            self.records.insert(
                insert_at,
                RawRecord {
                    typ: DIMENSION,
                    data,
                },
            );
            self.sheets[sheet_index].dimension_index = Some(insert_at);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
    }
}

fn encode_cell_record(row: u16, col: u8, xf: u16, value: &Biff8Value) -> Result<RawRecord> {
    let mut data = Vec::new();
    data.extend_from_slice(&row.to_le_bytes());
    data.extend_from_slice(&u16::from(col).to_le_bytes());
    data.extend_from_slice(&xf.to_le_bytes());
    match value {
        Biff8Value::Blank => Ok(RawRecord { typ: BLANK, data }),
        Biff8Value::Bool(flag) => {
            data.push(u8::from(*flag));
            data.push(0);
            Ok(RawRecord { typ: BOOLERR, data })
        }
        Biff8Value::Number(number) => {
            if let Some(rk) = encode_rk(*number) {
                data.extend_from_slice(&rk.to_le_bytes());
                Ok(RawRecord { typ: RK, data })
            } else {
                data.extend_from_slice(&number.to_le_bytes());
                Ok(RawRecord { typ: NUMBER, data })
            }
        }
        Biff8Value::Formula(expr) => {
            let rgce = super::ptg::encode_formula_rpn(expr)?;
            data.extend_from_slice(&0.0f64.to_le_bytes());
            data.extend_from_slice(&0u16.to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
            // rgce 长度受 BIFF8 记录上限约束，usize->u16 不会截断
            #[allow(clippy::cast_possible_truncation)]
            data.extend_from_slice(&(rgce.len() as u16).to_le_bytes());
            data.extend_from_slice(&rgce);
            Ok(RawRecord { typ: FORMULA, data })
        }
        Biff8Value::Text(text) => {
            // Inline LABEL avoids mutating the template SST (preserves indices).
            let encoded = encode_unicode_string(text);
            if data.len() + encoded.len() > MAX_RECORD_DATA {
                return Err(ExcelError::Xls(
                    "xls template LABEL cell exceeds BIFF record size".to_owned(),
                ));
            }
            data.extend_from_slice(&encoded);
            Ok(RawRecord { typ: LABEL, data })
        }
    }
}

/// Encodes a BIFF8 LABEL record (0x0204) directly, without going
/// through the full `Biff8Value` dispatch. Used by `replace_label`
/// to force an inline-string cell even when the original was LABELSST.
fn encode_label_record(row: u16, col: u8, xf: u16, text: &str) -> Result<RawRecord> {
    let mut data = Vec::new();
    data.extend_from_slice(&row.to_le_bytes());
    data.extend_from_slice(&u16::from(col).to_le_bytes());
    data.extend_from_slice(&xf.to_le_bytes());
    let encoded = encode_unicode_string(text);
    if data.len() + encoded.len() > MAX_RECORD_DATA {
        return Err(ExcelError::Xls(
            "xls template LABEL cell exceeds BIFF record size".to_owned(),
        ));
    }
    data.extend_from_slice(&encoded);
    Ok(RawRecord { typ: LABEL, data })
}

fn read_workbook_stream(bytes: &[u8]) -> Result<(String, Vec<u8>)> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut cf = CompoundFile::open(cursor)
        .map_err(|error| ExcelError::Cfb(format!("cannot open xls OLE container: {error}")))?;
    for path in ["/Workbook", "/Book", "Workbook", "Book"] {
        if cf.is_stream(path) {
            #[rustfmt::skip]
            let mut stream = cf.open_stream(path).map_err(|error| ExcelError::Cfb(format!("cannot open {path} stream: {error}")))?;
            let mut workbook = Vec::new();
            stream.read_to_end(&mut workbook)?;
            let normalized = if path.ends_with("Book") && !path.ends_with("Workbook") {
                "Book"
            } else {
                "Workbook"
            };
            return Ok((normalized.to_owned(), workbook));
        }
    }
    Err(ExcelError::Xls(
        "xls template missing Workbook/Book stream".to_owned(),
    ))
}

fn rewrite_workbook_stream(
    ole_bytes: &[u8],
    workbook_path: &str,
    workbook: &[u8],
) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(ole_bytes.to_vec());
    {
        #[rustfmt::skip]
        let mut cf = CompoundFile::open(&mut cursor).map_err(|error| ExcelError::Cfb(format!("cannot reopen xls OLE container: {error}")))?;
        {
            #[rustfmt::skip]
            let mut stream = cf.open_stream(workbook_path).map_err(|error| ExcelError::Cfb(format!("cannot rewrite {workbook_path}: {error}")))?;
            #[rustfmt::skip]
            stream.set_len(0).map_err(|error| ExcelError::Cfb(format!("cannot truncate {workbook_path}: {error}")))?;
            stream.write_all(workbook)?;
            stream.flush()?;
        }
        cf.flush()
            .map_err(|error| ExcelError::Cfb(format!("cannot flush OLE container: {error}")))?;
    }
    Ok(cursor.into_inner())
}

fn split_records(workbook: &[u8]) -> Result<Vec<RawRecord>> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    while offset + 4 <= workbook.len() {
        let typ = u16::from_le_bytes([workbook[offset], workbook[offset + 1]]);
        let length = usize::from(u16::from_le_bytes([
            workbook[offset + 2],
            workbook[offset + 3],
        ]));
        offset += 4;
        if offset + length > workbook.len() {
            return Err(ExcelError::Xls(format!(
                "truncated BIFF record type=0x{typ:04X} len={length}"
            )));
        }
        records.push(RawRecord {
            typ,
            data: workbook[offset..offset + length].to_vec(),
        });
        offset += length;
    }
    if records.is_empty() {
        return Err(ExcelError::Xls(
            "xls template Workbook stream has no BIFF records".to_owned(),
        ));
    }
    Ok(records)
}

fn discover_sheets(records: &[RawRecord]) -> Result<Vec<SheetSpan>> {
    let mut names = Vec::new();
    for record in records {
        if record.typ == BOUNDSHEET {
            names.push(decode_boundsheet_name(&record.data)?);
        }
    }
    let mut sheets = Vec::new();
    let mut name_iter = names.into_iter();
    let mut index = 0usize;
    while index < records.len() {
        let record = &records[index];
        if record.typ == BOF && is_worksheet_bof(&record.data) {
            let name = name_iter
                .next()
                .unwrap_or_else(|| format!("Sheet{}", sheets.len() + 1));
            let bof_index = index;
            let mut dimension_index = None;
            let mut eof_index = None;
            index += 1;
            while index < records.len() {
                match records[index].typ {
                    DIMENSION if dimension_index.is_none() => dimension_index = Some(index),
                    EOF => {
                        eof_index = Some(index);
                        break;
                    }
                    BOF => {
                        return Err(ExcelError::Xls(
                            "xls template has nested worksheet BOF without EOF".to_owned(),
                        ));
                    }
                    _ => {}
                }
                index += 1;
            }
            let eof_index = eof_index.ok_or_else(|| {
                ExcelError::Xls(format!("xls template sheet `{name}` is missing EOF"))
            })?;
            sheets.push(SheetSpan {
                name,
                bof_index,
                eof_index,
                dimension_index,
            });
        }
        index += 1;
    }
    Ok(sheets)
}

fn is_worksheet_bof(data: &[u8]) -> bool {
    data.len() >= 4 && u16::from_le_bytes([data[2], data[3]]) == DT_WORKSHEET
}

fn decode_boundsheet_name(data: &[u8]) -> Result<String> {
    // lbPlyPos(4) + hsState(1) + dt(1) + short XLUnicodeString
    if data.len() < 8 {
        return Err(ExcelError::Xls("BOUNDSHEET record is too short".to_owned()));
    }
    let cch = usize::from(data[6]);
    let compressed = data[7] & 0x01 == 0;
    let raw = &data[8..];
    if compressed {
        let take = cch.min(raw.len());
        Ok(raw[..take].iter().map(|&byte| char::from(byte)).collect())
    } else {
        let take = cch.saturating_mul(2).min(raw.len());
        let units: Vec<u16> = raw[..take]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        Ok(String::from_utf16_lossy(&units))
    }
}

/// Parses the Shared String Table (SST) BIFF record if present,
/// returning a Vec of all unique strings indexed by position.
fn parse_sst(records: &[RawRecord]) -> Vec<String> {
    for record in records {
        if record.typ == SST && record.data.len() >= 8 {
            let _cst_total = u32::from_le_bytes([
                record.data[0],
                record.data[1],
                record.data[2],
                record.data[3],
            ]);
            let cst_unique = u32::from_le_bytes([
                record.data[4],
                record.data[5],
                record.data[6],
                record.data[7],
            ]);
            let body = &record.data[8..];
            let mut strings = Vec::with_capacity(cst_unique as usize);
            let mut pos = 0usize;
            for _ in 0..cst_unique {
                if pos + 2 > body.len() {
                    break;
                }
                let cch = u16::from_le_bytes([body[pos], body[pos + 1]]) as usize;
                pos += 2;
                if pos >= body.len() {
                    break;
                }
                let grbit = body[pos];
                pos += 1;
                let is_compressed = (grbit & 0x01) == 0;
                if is_compressed {
                    // 8-bit compressed
                    let end = (pos + cch).min(body.len());
                    let text = String::from_utf8_lossy(&body[pos..end]).into_owned();
                    strings.push(text);
                    pos = end;
                } else {
                    // 16-bit Unicode
                    let end = (pos + cch * 2).min(body.len());
                    let raw = &body[pos..end];
                    let mut units = Vec::with_capacity(cch);
                    for chunk in raw.chunks_exact(2) {
                        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                    }
                    strings.push(String::from_utf16_lossy(&units));
                    pos = end;
                }
            }
            return strings;
        }
    }
    Vec::new()
}

/// Decodes just the SST index from a LABELSST record.
// 语义敏感：BIFF8 列号合法范围 0..=255（工作簿最多 256 列），
// u16->u8 截断对合法文件无损；保留 as 以对齐 Java 的 byte 列号。
#[allow(clippy::cast_possible_truncation)]
fn decode_labelsst_index(data: &[u8]) -> (u16, u8, Option<u32>) {
    if data.len() < 10 {
        return (0, 0, None);
    }
    let row = u16::from_le_bytes([data[0], data[1]]);
    let col = u16::from_le_bytes([data[2], data[3]]);
    let sst_idx = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
    (row, col as u8, Some(sst_idx))
}

/// Decodes a BIFF8 LABEL record payload, returning `(row, col, text)`.
// 语义敏感：BIFF8 列号合法范围 0..=255，u16->u8 截断对合法文件无损。
#[allow(clippy::cast_possible_truncation)]
fn decode_label_payload(data: &[u8]) -> (u16, u8, Option<String>) {
    if data.len() < 8 {
        return (0, 0, None);
    }
    let row = u16::from_le_bytes([data[0], data[1]]);
    let col = u16::from_le_bytes([data[2], data[3]]);
    // Bytes 4-5 are XF index; bytes 6-7 are the XLUnicodeString length (cch),
    // followed by `grbit` + character data (BIFF8 LABEL inline string).
    let cch = u16::from_le_bytes([data[6], data[7]]) as usize;
    let string_data = &data[8..];
    let text = if string_data.is_empty() {
        String::new()
    } else if string_data[0] & 0x01 == 0 {
        // Compressed 8-bit characters.
        let take = cch.min(string_data.len().saturating_sub(1));
        String::from_utf8_lossy(&string_data[1..=take]).into_owned()
    } else {
        // 16-bit Unicode characters.
        let take = cch
            .saturating_mul(2)
            .min(string_data.len().saturating_sub(1));
        let units: Vec<u16> = string_data[1..=take]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    };
    (
        row,
        col as u8,
        if text.is_empty() { None } else { Some(text) },
    )
}

/// Decodes a BIFF8 LABELSST record payload, returning `(row, col, text)`.
/// LABELSST references the Shared String Table — since we don't have
/// the SST available here, we return None for the text and let the
/// caller handle SST lookups separately.
#[allow(dead_code)]
// 语义敏感：BIFF8 列号合法范围 0..=255，u16->u8 截断对合法文件无损。
#[allow(clippy::cast_possible_truncation)]
fn decode_labelsst_payload(data: &[u8]) -> (u16, u8, Option<String>) {
    if data.len() < 8 {
        return (0, 0, None);
    }
    let row = u16::from_le_bytes([data[0], data[1]]);
    let col = u16::from_le_bytes([data[2], data[3]]);
    // Bytes 4-5: XF, bytes 6-9: SST index (u32)
    if data.len() >= 10 {
        let _sst_index = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        // SST-based records can't be decoded without the shared string table;
        // caller should use LABEL records for placeholder detection.
        (row, col as u8, None)
    } else {
        (row, col as u8, None)
    }
}

fn sheet_max_row(records: &[RawRecord], sheet: &SheetSpan) -> Option<u16> {
    let mut maximum = None;
    for record in &records[sheet.bof_index..=sheet.eof_index] {
        if let Some((row, _)) = cell_coords(record) {
            maximum = Some(maximum.map_or(row, |current: u16| current.max(row)));
        }
    }
    maximum
}

fn sheet_dimensions(records: &[RawRecord], sheet: &SheetSpan) -> (u16, u8) {
    let mut max_row = 0u16;
    let mut max_col = 0u8;
    for record in &records[sheet.bof_index..=sheet.eof_index] {
        if let Some((row, col)) = cell_coords(record) {
            max_row = max_row.max(row.saturating_add(1));
            max_col = max_col.max(col.saturating_add(1));
        }
    }
    (max_row, max_col)
}

fn cell_coords(record: &RawRecord) -> Option<(u16, u8)> {
    match record.typ {
        LABEL | LABELSST | NUMBER | RK | BOOLERR | BLANK => {
            if record.data.len() < 4 {
                return None;
            }
            let row = u16::from_le_bytes([record.data[0], record.data[1]]);
            let col = u16::from_le_bytes([record.data[2], record.data[3]]);
            let col = u8::try_from(col).ok()?;
            Some((row, col))
        }
        _ => None,
    }
}

fn find_cell_record(records: &[RawRecord], sheet: &SheetSpan, row: u16, col: u8) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .take(sheet.eof_index + 1)
        .skip(sheet.bof_index)
        .find(|(_, record)| cell_coords(record) == Some((row, col)))
        .map(|(index, _)| index)
}

// 语义敏感：BOUNDSHEET 的 lbPlyPos 是 BIFF8 规范中的 u32 绝对偏移，
// 文件流不可能超过 4GiB，usize->u32 截断在此场景不可能发生。
#[allow(clippy::cast_possible_truncation)]
fn assemble_workbook(records: &[RawRecord]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut boundsheet_patches = Vec::new();
    let mut sheet_offsets = Vec::new();
    for record in records {
        if record.typ == BOUNDSHEET {
            // Patch site: absolute offset of lbPlyPos inside the assembled stream.
            boundsheet_patches.push(out.len() + 4);
        }
        if record.typ == BOF && is_worksheet_bof(&record.data) {
            sheet_offsets.push(out.len() as u32);
        }
        write_raw_record(&mut out, record)?;
    }
    if boundsheet_patches.len() != sheet_offsets.len() {
        return Err(ExcelError::Xls(format!(
            "BOUNDSHEET count ({}) does not match worksheet BOF count ({})",
            boundsheet_patches.len(),
            sheet_offsets.len()
        )));
    }
    for (patch_at, offset) in boundsheet_patches.into_iter().zip(sheet_offsets) {
        out[patch_at..patch_at + 4].copy_from_slice(&offset.to_le_bytes());
    }
    Ok(out)
}

// 语义敏感：上方已校验 data.len() <= MAX_RECORD_DATA（远小于 u16 上限），
// 记录长度字段按 BIFF8 规范为 u16，保留 as 转换。
#[allow(clippy::cast_possible_truncation)]
fn write_raw_record(out: &mut Vec<u8>, record: &RawRecord) -> Result<()> {
    if record.data.len() > MAX_RECORD_DATA {
        return Err(ExcelError::Xls(format!(
            "BIFF record 0x{:04X} payload exceeds {MAX_RECORD_DATA} bytes",
            record.typ
        )));
    }
    out.extend_from_slice(&record.typ.to_le_bytes());
    out.extend_from_slice(&(record.data.len() as u16).to_le_bytes());
    out.extend_from_slice(&record.data);
    Ok(())
}

/// Returns whether `bytes` look like an OLE `.xls` compound document.
#[must_use]
pub fn looks_like_xls(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ole_magic_and_rejects_non_ole_template() {
        assert!(looks_like_xls(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1,
        ]));
        assert!(!looks_like_xls(b"PK\x03\x04"));
        assert!(matches!(
            Biff8TemplatePackage::from_bytes(b"not an xls"),
            Err(ExcelError::Xls(_))
        ));
    }
}
