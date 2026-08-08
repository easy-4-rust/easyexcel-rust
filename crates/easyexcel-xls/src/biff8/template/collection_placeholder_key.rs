fn collection_placeholder_key<'a>(text: &'a str, collection_name: Option<&str>) -> Option<&'a str> {
    let prefix = collection_name.map_or_else(|| "{.".to_owned(), |name| format!("{{{name}."));
    if text.starts_with(&prefix) {
        return Some(text[prefix.len()..].trim_end_matches('}'));
    }
    text.strip_prefix('{').map(|key| key.trim_end_matches('}'))
}

fn shifted_row(row: u16, start_row: u16, delta: u16) -> Result<u16> {
    if row < start_row {
        return Ok(row);
    }
    row.checked_add(delta)
        .ok_or_else(|| ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned()))
}

fn shift_record_row(record: &mut RawRecord, start_row: u16, delta: u16) -> Result<()> {
    if record.data.len() < 2 {
        return Ok(());
    }
    let row = u16::from_le_bytes([record.data[0], record.data[1]]);
    record.data[0..2].copy_from_slice(&shifted_row(row, start_row, delta)?.to_le_bytes());
    Ok(())
}

fn shift_range_rows(data: &mut [u8], start_row: u16, delta: u16) -> Result<()> {
    if data.len() < 4 {
        return Ok(());
    }
    for offset in [0usize, 2] {
        let row = u16::from_le_bytes([data[offset], data[offset + 1]]);
        data[offset..offset + 2]
            .copy_from_slice(&shifted_row(row, start_row, delta)?.to_le_bytes());
    }
    Ok(())
}

fn shift_merge_rows(data: &mut [u8], start_row: u16, delta: u16) -> Result<()> {
    if data.len() < 2 {
        return Ok(());
    }
    let count = usize::from(u16::from_le_bytes([data[0], data[1]]));
    for index in 0..count {
        let offset = 2 + index * 8;
        if offset + 4 > data.len() {
            break;
        }
        shift_range_rows(&mut data[offset..offset + 4], start_row, delta)?;
    }
    Ok(())
}

/// 平移 `MSODRAWING` 中全部 Escher `ClientAnchor` 的首末行。
///
/// 对应 Java：`HSSFSheet#shiftRows` 对 comment、图片和嵌入图表 anchor 的迁移。
/// Escher `ClientAnchor` 的 payload 固定为 18 字节；行号位于 payload 的
/// `row1`/`row2` 字段。这里按嵌入记录头扫描，因此也能处理一个 BIFF
/// `MSODRAWING` 中嵌套的多个 shape。
fn shift_msodrawing_anchors(data: &mut [u8], start_row: u16, delta: u16) -> Result<()> {
    const CLIENT_ANCHOR_HEADER: [u8; 6] = [0x10, 0xF0, 18, 0, 0, 0];
    let mut offset = 0usize;
    while let Some(relative) = data[offset..]
        .windows(CLIENT_ANCHOR_HEADER.len())
        .position(|window| window == CLIENT_ANCHOR_HEADER)
    {
        // marker 从 Escher header 的 record type 开始，后续 4 字节为固定 payload 长度。
        let payload_start = offset + relative + CLIENT_ANCHOR_HEADER.len();
        let Some(anchor) = data.get_mut(payload_start..payload_start.saturating_add(18)) else {
            break;
        };
        for row_offset in [6usize, 14] {
            let row = u16::from_le_bytes([anchor[row_offset], anchor[row_offset + 1]]);
            anchor[row_offset..row_offset + 2]
                .copy_from_slice(&shifted_row(row, start_row, delta)?.to_le_bytes());
        }
        offset = payload_start.saturating_add(18);
    }
    Ok(())
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
        Biff8Value::RichText(rich) => {
            let units = rich.text.encode_utf16().collect::<Vec<_>>();
            let compressed = units.iter().all(|unit| *unit <= 0xFF);
            data.extend_from_slice(
                &u16::try_from(units.len())
                    .map_err(|_| ExcelError::Xls("BIFF8 rich text exceeds 65535 UTF-16 units".to_owned()))?
                    .to_le_bytes(),
            );
            // XLUnicodeString grbit: bit0=16-bit chars, bit3=rich-text runs。
            data.push(u8::from(!compressed) | 0x08);
            data.extend_from_slice(
                &u16::try_from(rich.runs.len())
                    .map_err(|_| ExcelError::Xls("too many BIFF8 rich-text runs".to_owned()))?
                    .to_le_bytes(),
            );
            if compressed {
                data.extend(units.into_iter().map(|unit| u8::try_from(unit).unwrap_or(b'?')));
            } else {
                for unit in units {
                    data.extend_from_slice(&unit.to_le_bytes());
                }
            }
            for &(start, font_index) in &rich.runs {
                data.extend_from_slice(&start.to_le_bytes());
                data.extend_from_slice(&font_index.to_le_bytes());
            }
            if data.len() > MAX_RECORD_DATA {
                return Err(ExcelError::Xls(
                    "xls template RSTRING cell exceeds BIFF record size".to_owned(),
                ));
            }
            Ok(RawRecord {
                typ: RICH_STRING_SID,
                data,
            })
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

fn apply_macro_policy(bytes: &[u8], policy: &Biff8MacroPolicy) -> Result<Vec<u8>> {
    const VBA_ROOT: &str = "/_VBA_PROJECT_CUR";
    if matches!(policy, Biff8MacroPolicy::Preserve) {
        return Ok(bytes.to_vec());
    }
    let mut output = Cursor::new(bytes.to_vec());
    {
        let mut destination = CompoundFile::open(&mut output)
            .map_err(|error| ExcelError::Cfb(format!("cannot open macro destination: {error}")))?;
        if destination.is_storage(VBA_ROOT) {
            destination.remove_storage_all(VBA_ROOT).map_err(|error| {
                ExcelError::Cfb(format!("cannot remove existing VBA project: {error}"))
            })?;
        } else if destination.is_stream(VBA_ROOT) {
            destination.remove_stream(VBA_ROOT).map_err(|error| {
                ExcelError::Cfb(format!("cannot remove existing VBA stream: {error}"))
            })?;
        }

        if let Biff8MacroPolicy::Replace(source_bytes) = policy {
            let mut source = CompoundFile::open(Cursor::new(source_bytes.clone())).map_err(|error| {
                ExcelError::Cfb(format!("replacement VBA bytes are not an OLE/CFB file: {error}"))
            })?;
            if !source.is_storage(VBA_ROOT) {
                return Err(ExcelError::Xls(
                    "replacement OLE/CFB file does not contain /_VBA_PROJECT_CUR".to_owned(),
                ));
            }
            let entries = source
                .walk_storage(VBA_ROOT)
                .map_err(|error| ExcelError::Cfb(error.to_string()))?
                .collect::<Vec<_>>();
            for entry in entries {
                let path = entry.path().to_path_buf();
                if entry.is_storage() {
                    destination
                        .create_storage_all(&path)
                        .map_err(|error| ExcelError::Cfb(error.to_string()))?;
                    destination
                        .set_storage_clsid(&path, *entry.clsid())
                        .map_err(|error| ExcelError::Cfb(error.to_string()))?;
                } else {
                    let mut payload = Vec::new();
                    source
                        .open_stream(&path)
                        .map_err(|error| ExcelError::Cfb(error.to_string()))?
                        .read_to_end(&mut payload)?;
                    destination
                        .create_stream(&path)
                        .map_err(|error| ExcelError::Cfb(error.to_string()))?
                        .write_all(&payload)?;
                }
                destination
                    .set_state_bits(&path, entry.state_bits())
                    .map_err(|error| ExcelError::Cfb(error.to_string()))?;
            }
        }
        destination
            .flush()
            .map_err(|error| ExcelError::Cfb(format!("cannot flush macro policy output: {error}")))?;
    }
    Ok(output.into_inner())
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
    let sheet_streams = top_level_substreams(records)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();
    if names.len() != sheet_streams.len() {
        return Err(ExcelError::Xls(format!(
            "BOUNDSHEET count ({}) does not match top-level sheet stream count ({})",
            names.len(),
            sheet_streams.len()
        )));
    }
    let mut sheets = Vec::new();
    for (bound_sheet_index, (name, (bof_index, eof_index))) in
        names.into_iter().zip(sheet_streams).enumerate()
    {
        // Chart/macro/VB-module sheets consume their own BOUNDSHEET name but are
        // not exposed as cell grids. Their complete substreams remain untouched.
        if !is_worksheet_bof(&records[bof_index].data) {
            continue;
        }
        let dimension_index =
            (bof_index + 1..eof_index).find(|index| records[*index].typ == DIMENSION);
        sheets.push(SheetSpan {
            name,
            bound_sheet_index: u16::try_from(bound_sheet_index).map_err(|_| {
                ExcelError::Xls("BIFF8 BOUNDSHEET index exceeds u16".to_owned())
            })?,
            bof_index,
            eof_index,
            dimension_index,
        });
    }
    Ok(sheets)
}

/// 返回全局流及每个 `BoundSheet` 顶层流的 `(BOF, EOF)` 记录索引。
/// 嵌入式 chart substream 的 BOF/EOF 由深度计数吸收，不会误当作 sheet。
fn top_level_substreams(records: &[RawRecord]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, record) in records.iter().enumerate() {
        if record.typ == BOF {
            if depth == 0 {
                start = Some(index);
            }
            depth = depth.saturating_add(1);
        } else if record.typ == EOF && depth > 0 {
            depth -= 1;
            if depth == 0
                && let Some(bof_index) = start.take()
            {
                spans.push((bof_index, index));
            }
        }
    }
    spans
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
        LABEL | LABELSST | NUMBER | RK | BOOLERR | BLANK | FORMULA => {
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

/// 返回 worksheet row block 中新增单元格的安全插入点。
///
/// 嵌入式 chart 的 `MSODRAWING/OBJ/BOF` 必须保持在全部 row/cell records 之后；
/// 因此优先接在最后一个现有单元格后，没有单元格时接在 `DIMENSION` 后。
fn sheet_cell_insert_index(records: &[RawRecord], sheet: &SheetSpan) -> usize {
    records
        .iter()
        .enumerate()
        .take(sheet.eof_index)
        .skip(sheet.bof_index + 1)
        .filter(|(_, record)| cell_coords(record).is_some())
        .map(|(index, _)| index + 1)
        .next_back()
        .or_else(|| sheet.dimension_index.map(|index| index + 1))
        .unwrap_or(sheet.bof_index + 1)
}

// 语义敏感：BOUNDSHEET 的 lbPlyPos 是 BIFF8 规范中的 u32 绝对偏移，
// 文件流不可能超过 4GiB，usize->u32 截断在此场景不可能发生。
#[allow(clippy::cast_possible_truncation)]
fn assemble_workbook(records: &[RawRecord]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut boundsheet_patches = Vec::new();
    let mut sheet_offsets = Vec::new();
    let sheet_bof_indices = top_level_substreams(records)
        .into_iter()
        .skip(1)
        .map(|(bof_index, _)| bof_index)
        .collect::<Vec<_>>();
    let mut next_sheet = 0usize;
    for (record_index, record) in records.iter().enumerate() {
        if record.typ == BOUNDSHEET {
            // Patch site: absolute offset of lbPlyPos inside the assembled stream.
            boundsheet_patches.push(out.len() + 4);
        }
        if sheet_bof_indices.get(next_sheet) == Some(&record_index) {
            sheet_offsets.push(out.len() as u32);
            next_sheet += 1;
        }
        write_raw_record(&mut out, record)?;
    }
    if boundsheet_patches.len() != sheet_offsets.len() {
        return Err(ExcelError::Xls(format!(
            "BOUNDSHEET count ({}) does not match top-level sheet BOF count ({})",
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns whether `bytes` look like an OLE `.xls` compound document.
#[must_use]
pub fn looks_like_xls(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0])
}

#[cfg(test)]
#[path = "../template_tests/tests.rs"]
mod tests;
