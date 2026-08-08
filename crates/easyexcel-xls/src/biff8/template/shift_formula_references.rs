/// 按插入行映射修正 FORMULA 记录内的本 Sheet 引用。
///
/// 对应 Java：`HSSFSheet#shiftRows` 通过 `FormulaShifter` 更新 `PtgRef`、
/// `PtgArea`、`PtgRefN` 与 `PtgAreaN`。相对引用先按旧公式行还原目标，
/// 迁移目标与公式单元格后再重新编码偏移，避免跨插入边界时引用漂移。
fn shift_formula_references(
    record: &mut RawRecord,
    start_row: u16,
    delta: u16,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    if record.data.len() < 22 {
        return Err(ExcelError::Xls(
            "truncated BIFF8 FORMULA record while shifting rows".to_owned(),
        ));
    }
    let formula_row = u16::from_le_bytes([record.data[0], record.data[1]]);
    let shifted_formula_row = shifted_row(formula_row, start_row, delta)?;
    let token_len = usize::from(u16::from_le_bytes([record.data[20], record.data[21]]));
    let token_end = 22usize
        .checked_add(token_len)
        .ok_or_else(|| ExcelError::Xls("BIFF8 FORMULA token length overflow".to_owned()))?;
    let tokens = record
        .data
        .get_mut(22..token_end)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 FORMULA tokens".to_owned()))?;
    shift_formula_token_rows(
        tokens,
        formula_row,
        shifted_formula_row,
        start_row,
        delta,
        true,
        current_sheet,
        extern_sheet_ranges,
    )
}

#[allow(clippy::too_many_arguments)]
fn shift_formula_token_rows(
    tokens: &mut [u8],
    formula_row: u16,
    shifted_formula_row: u16,
    start_row: u16,
    delta: u16,
    shift_local_references: bool,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    let mut cursor = 0usize;
    while cursor < tokens.len() {
        let raw = tokens[cursor];
        let base = if raw >= 0x20 {
            0x20 | (raw & 0x1F)
        } else {
            raw
        };
        match base {
            0x24 if shift_local_references => {
                shift_absolute_ptg_row(tokens, cursor + 1, start_row, delta)?;
            }
            0x2C if shift_local_references => {
                shift_ptg_row(
                    tokens,
                    cursor + 1,
                    cursor + 3,
                    formula_row,
                    shifted_formula_row,
                    start_row,
                    delta,
                )?;
            }
            0x25 if shift_local_references => {
                shift_absolute_ptg_row(tokens, cursor + 1, start_row, delta)?;
                shift_absolute_ptg_row(tokens, cursor + 3, start_row, delta)?;
            }
            0x2D if shift_local_references => {
                shift_ptg_row(
                    tokens,
                    cursor + 1,
                    cursor + 5,
                    formula_row,
                    shifted_formula_row,
                    start_row,
                    delta,
                )?;
                shift_ptg_row(
                    tokens,
                    cursor + 3,
                    cursor + 7,
                    formula_row,
                    shifted_formula_row,
                    start_row,
                    delta,
                )?;
            }
            0x3A => {
                if ptg_targets_sheet(tokens, cursor, current_sheet, extern_sheet_ranges)? {
                    shift_absolute_ptg_row(tokens, cursor + 3, start_row, delta)?;
                }
            }
            0x3B => {
                if ptg_targets_sheet(tokens, cursor, current_sheet, extern_sheet_ranges)? {
                    shift_absolute_ptg_row(tokens, cursor + 3, start_row, delta)?;
                    shift_absolute_ptg_row(tokens, cursor + 5, start_row, delta)?;
                }
            }
            // Deleted 3D refs remain deleted.
            0x3C | 0x3D => {}
            _ => {}
        }
        let length = ptg_encoded_len(tokens, cursor, base)?;
        cursor = cursor
            .checked_add(length)
            .ok_or_else(|| ExcelError::Xls("BIFF8 formula token cursor overflow".to_owned()))?;
    }
    Ok(())
}

fn shift_absolute_ptg_row(
    tokens: &mut [u8],
    row_offset: usize,
    start_row: u16,
    delta: u16,
) -> Result<()> {
    let bytes = tokens
        .get(row_offset..row_offset.saturating_add(2))
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 reference row".to_owned()))?;
    let row = u16::from_le_bytes([bytes[0], bytes[1]]);
    tokens[row_offset..row_offset + 2]
        .copy_from_slice(&shifted_row(row, start_row, delta)?.to_le_bytes());
    Ok(())
}

fn shift_name_references(
    record: &mut RawRecord,
    start_row: u16,
    delta: u16,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    if record.data.len() < 15 {
        return Err(ExcelError::Xls(
            "truncated BIFF8 NAME record while shifting rows".to_owned(),
        ));
    }
    let name_length = usize::from(record.data[3]);
    let token_length = usize::from(u16::from_le_bytes([record.data[4], record.data[5]]));
    let scoped_sheet = u16::from_le_bytes([record.data[8], record.data[9]]);
    let wide_name = record.data[14] & 0x01 != 0;
    let token_start = 15usize
        .checked_add(name_length.saturating_mul(if wide_name { 2 } else { 1 }))
        .ok_or_else(|| ExcelError::Xls("BIFF8 NAME token offset overflow".to_owned()))?;
    let token_end = token_start
        .checked_add(token_length)
        .ok_or_else(|| ExcelError::Xls("BIFF8 NAME token length overflow".to_owned()))?;
    let tokens = record
        .data
        .get_mut(token_start..token_end)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 NAME formula tokens".to_owned()))?;
    shift_formula_token_rows(
        tokens,
        0,
        0,
        start_row,
        delta,
        scoped_sheet == current_sheet.saturating_add(1),
        current_sheet,
        extern_sheet_ranges,
    )
}

fn shift_conditional_format_header(
    data: &mut [u8],
    start_row: u16,
    delta: u16,
) -> Result<(u16, u16)> {
    if data.len() < 14 {
        return Err(ExcelError::Xls(
            "truncated BIFF8 CONDFMT record while shifting rows".to_owned(),
        ));
    }
    let original_base = u16::from_le_bytes([data[4], data[5]]);
    shift_range_rows(&mut data[4..12], start_row, delta)?;
    shift_sqref_rows(data, 12, start_row, delta)?;
    Ok((original_base, shifted_row(original_base, start_row, delta)?))
}

#[allow(clippy::too_many_arguments)]
fn shift_conditional_format_rule(
    data: &mut [u8],
    formula_row: u16,
    shifted_formula_row: u16,
    start_row: u16,
    delta: u16,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    if data.len() < 12 {
        return Err(ExcelError::Xls(
            "truncated BIFF8 CF record while shifting rows".to_owned(),
        ));
    }
    let formula1_length = usize::from(u16::from_le_bytes([data[2], data[3]]));
    let formula2_length = usize::from(u16::from_le_bytes([data[4], data[5]]));
    let formatting_options = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
    let formatting_length = 6usize
        + if formatting_options & 0x0400_0000 != 0 { 118 } else { 0 }
        + if formatting_options & 0x1000_0000 != 0 { 8 } else { 0 }
        + if formatting_options & 0x2000_0000 != 0 { 4 } else { 0 };
    let formula1_start = 6usize.saturating_add(formatting_length);
    let formula1_end = formula1_start
        .checked_add(formula1_length)
        .ok_or_else(|| ExcelError::Xls("BIFF8 CF formula length overflow".to_owned()))?;
    let formula2_end = formula1_end
        .checked_add(formula2_length)
        .ok_or_else(|| ExcelError::Xls("BIFF8 CF formula length overflow".to_owned()))?;
    if formula2_end > data.len() {
        return Err(ExcelError::Xls(
            "truncated BIFF8 CF formula tokens".to_owned(),
        ));
    }
    let (formula1, remainder) = data[formula1_start..formula2_end].split_at_mut(formula1_length);
    shift_formula_token_rows(
        formula1,
        formula_row,
        shifted_formula_row,
        start_row,
        delta,
        true,
        current_sheet,
        extern_sheet_ranges,
    )?;
    shift_formula_token_rows(
        remainder,
        formula_row,
        shifted_formula_row,
        start_row,
        delta,
        true,
        current_sheet,
        extern_sheet_ranges,
    )
}

fn shift_data_validation(
    data: &mut [u8],
    start_row: u16,
    delta: u16,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    if data.len() < 4 {
        return Err(ExcelError::Xls(
            "truncated BIFF8 DV record while shifting rows".to_owned(),
        ));
    }
    let mut offset = 4usize;
    for _ in 0..4 {
        offset = unicode_string_end(data, offset)?;
    }
    let formula1_length = read_u16_at(data, offset, "DV formula1 length")? as usize;
    let formula1_start = offset.saturating_add(4);
    let formula1_end = formula1_start
        .checked_add(formula1_length)
        .ok_or_else(|| ExcelError::Xls("BIFF8 DV formula length overflow".to_owned()))?;
    let formula2_length = read_u16_at(data, formula1_end, "DV formula2 length")? as usize;
    let formula2_start = formula1_end.saturating_add(4);
    let sqref_offset = formula2_start
        .checked_add(formula2_length)
        .ok_or_else(|| ExcelError::Xls("BIFF8 DV formula length overflow".to_owned()))?;
    let base_row = first_sqref_row(data, sqref_offset)?;
    let shifted_base_row = shifted_row(base_row, start_row, delta)?;
    if sqref_offset > data.len() {
        return Err(ExcelError::Xls("truncated BIFF8 DV formulas".to_owned()));
    }
    let (formula1, tail) = data[formula1_start..sqref_offset].split_at_mut(formula1_length);
    let formula2 = tail
        .get_mut(4..4usize.saturating_add(formula2_length))
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 DV formula2 tokens".to_owned()))?;
    shift_formula_token_rows(
        formula1,
        base_row,
        shifted_base_row,
        start_row,
        delta,
        true,
        current_sheet,
        extern_sheet_ranges,
    )?;
    shift_formula_token_rows(
        formula2,
        base_row,
        shifted_base_row,
        start_row,
        delta,
        true,
        current_sheet,
        extern_sheet_ranges,
    )?;
    shift_sqref_rows(data, sqref_offset, start_row, delta)
}

fn unicode_string_end(data: &[u8], offset: usize) -> Result<usize> {
    let character_count = usize::from(read_u16_at(data, offset, "Unicode string length")?);
    let flags = *data
        .get(offset.saturating_add(2))
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 Unicode string flags".to_owned()))?;
    let mut cursor = offset.saturating_add(3);
    let rich_runs = if flags & 0x08 != 0 {
        let count = usize::from(read_u16_at(data, cursor, "Unicode rich-run count")?);
        cursor = cursor.saturating_add(2);
        count
    } else {
        0
    };
    let extension_size = if flags & 0x04 != 0 {
        let bytes = data
            .get(cursor..cursor.saturating_add(4))
            .ok_or_else(|| ExcelError::Xls("truncated BIFF8 Unicode extension size".to_owned()))?;
        cursor = cursor.saturating_add(4);
        usize::try_from(u32::from_le_bytes(bytes.try_into().map_err(|_| {
            ExcelError::Xls("invalid BIFF8 Unicode extension size".to_owned())
        })?))
        .map_err(|_| ExcelError::Xls("BIFF8 Unicode extension exceeds usize".to_owned()))?
    } else {
        0
    };
    let character_bytes = character_count.saturating_mul(if flags & 0x01 != 0 { 2 } else { 1 });
    let end = cursor
        .checked_add(character_bytes)
        .and_then(|value| value.checked_add(rich_runs.saturating_mul(4)))
        .and_then(|value| value.checked_add(extension_size))
        .ok_or_else(|| ExcelError::Xls("BIFF8 Unicode string length overflow".to_owned()))?;
    if end > data.len() {
        return Err(ExcelError::Xls(
            "truncated BIFF8 Unicode string payload".to_owned(),
        ));
    }
    Ok(end)
}

fn read_u16_at(data: &[u8], offset: usize, context: &str) -> Result<u16> {
    data.get(offset..offset.saturating_add(2))
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .ok_or_else(|| ExcelError::Xls(format!("truncated BIFF8 {context}")))
}

fn first_sqref_row(data: &[u8], offset: usize) -> Result<u16> {
    let count = usize::from(read_u16_at(data, offset, "SqRef count")?);
    if count == 0 {
        return Err(ExcelError::Xls("BIFF8 SqRef contains no ranges".to_owned()));
    }
    read_u16_at(data, offset.saturating_add(2), "SqRef first row")
}

fn shift_sqref_rows(data: &mut [u8], offset: usize, start_row: u16, delta: u16) -> Result<()> {
    let count = usize::from(read_u16_at(data, offset, "SqRef count")?);
    for index in 0..count {
        let range_offset = offset.saturating_add(2).saturating_add(index.saturating_mul(8));
        let range = data
            .get_mut(range_offset..range_offset.saturating_add(8))
            .ok_or_else(|| ExcelError::Xls("truncated BIFF8 SqRef range".to_owned()))?;
        shift_range_rows(range, start_row, delta)?;
    }
    Ok(())
}

/// 更新 chart `AI/LinkedDataRecord` 中的数据区域公式。
///
/// 对应 Java：`HSSFSheet#shiftRows` 后 chart series 的 `Area3DPtg` 修正。
fn shift_chart_ai_references(
    record: &mut RawRecord,
    start_row: u16,
    delta: u16,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<()> {
    if record.data.len() < 8 {
        return Ok(());
    }
    let token_len = usize::from(u16::from_le_bytes([record.data[6], record.data[7]]));
    if token_len == 0 {
        return Ok(());
    }
    let token_end = 8usize
        .checked_add(token_len)
        .ok_or_else(|| ExcelError::Xls("BIFF8 chart AI token length overflow".to_owned()))?;
    let tokens = record
        .data
        .get_mut(8..token_end)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 chart AI tokens".to_owned()))?;
    let mut cursor = 0usize;
    while cursor < tokens.len() {
        let raw = tokens[cursor];
        let base = if raw >= 0x20 {
            0x20 | (raw & 0x1F)
        } else {
            raw
        };
        match base {
            0x24 | 0x2C => {
                shift_chart_ptg_row(tokens, cursor + 1, cursor + 3, start_row, delta)?;
            }
            0x25 | 0x2D => {
                shift_chart_ptg_row(tokens, cursor + 1, cursor + 5, start_row, delta)?;
                shift_chart_ptg_row(tokens, cursor + 3, cursor + 7, start_row, delta)?;
            }
            0x3A if ptg_targets_sheet(tokens, cursor, current_sheet, extern_sheet_ranges)? => {
                shift_chart_ptg_row(tokens, cursor + 3, cursor + 5, start_row, delta)?;
            }
            0x3B if ptg_targets_sheet(tokens, cursor, current_sheet, extern_sheet_ranges)? => {
                shift_chart_ptg_row(tokens, cursor + 3, cursor + 7, start_row, delta)?;
                shift_chart_ptg_row(tokens, cursor + 5, cursor + 9, start_row, delta)?;
            }
            _ => {}
        }
        cursor = cursor
            .checked_add(ptg_encoded_len(tokens, cursor, base)?)
            .ok_or_else(|| ExcelError::Xls("BIFF8 chart token cursor overflow".to_owned()))?;
    }
    Ok(())
}

fn shift_chart_ptg_row(
    tokens: &mut [u8],
    row_offset: usize,
    column_offset: usize,
    start_row: u16,
    delta: u16,
) -> Result<()> {
    let column = tokens
        .get(column_offset..column_offset + 2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 chart reference column".to_owned()))?;
    if column & 0x8000 != 0 {
        return Err(ExcelError::Xls(
            "relative BIFF8 chart series references cannot be shifted safely".to_owned(),
        ));
    }
    let row = tokens
        .get(row_offset..row_offset + 2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 chart reference row".to_owned()))?;
    tokens[row_offset..row_offset + 2]
        .copy_from_slice(&shifted_row(row, start_row, delta)?.to_le_bytes());
    Ok(())
}

fn ptg_targets_sheet(
    tokens: &[u8],
    cursor: usize,
    current_sheet: u16,
    extern_sheet_ranges: &[Option<(u16, u16)>],
) -> Result<bool> {
    let bytes = tokens
        .get(cursor + 1..cursor + 3)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 3D reference ixti".to_owned()))?;
    let ixti = usize::from(u16::from_le_bytes([bytes[0], bytes[1]]));
    // 只有单 Sheet ixti 能无歧义地随该 Sheet 移动；Sheet range 共用一组
    // 行坐标，POI FormulaShifter 同样不会把它拆成多个 token。
    Ok(extern_sheet_ranges
        .get(ixti)
        .and_then(|entry| *entry)
        .is_some_and(|(first, last)| first == current_sheet && last == current_sheet))
}

fn internal_extern_sheet_ranges(records: &[RawRecord]) -> Vec<Option<(u16, u16)>> {
    let mut supbook_index = 0u16;
    let mut internal_supbook = None;
    for record in records {
        if record.typ == SUP_BOOK_SID {
            if record.data.len() == 4 && record.data[2..4] == [0x01, 0x04] {
                internal_supbook = Some(supbook_index);
            }
            supbook_index = supbook_index.saturating_add(1);
        }
    }
    let mut ranges = Vec::new();
    for record in records {
        if record.typ != EXTERNAL_SHEET_SID || record.data.len() < 2 {
            continue;
        }
        let count = usize::from(u16::from_le_bytes([record.data[0], record.data[1]]));
        for index in 0..count {
            let offset = 2 + index * 6;
            let Some(entry) = record.data.get(offset..offset + 6) else {
                ranges.push(None);
                continue;
            };
            let supbook = u16::from_le_bytes([entry[0], entry[1]]);
            let first = u16::from_le_bytes([entry[2], entry[3]]);
            let last = u16::from_le_bytes([entry[4], entry[5]]);
            ranges.push((Some(supbook) == internal_supbook).then_some((first, last)));
        }
    }
    ranges
}

#[allow(clippy::too_many_arguments)]
fn shift_ptg_row(
    tokens: &mut [u8],
    row_offset: usize,
    column_offset: usize,
    formula_row: u16,
    shifted_formula_row: u16,
    start_row: u16,
    delta: u16,
) -> Result<()> {
    let row_bytes = tokens
        .get(row_offset..row_offset + 2)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 reference row".to_owned()))?;
    let row = u16::from_le_bytes([row_bytes[0], row_bytes[1]]);
    let column_bytes = tokens
        .get(column_offset..column_offset + 2)
        .ok_or_else(|| ExcelError::Xls("truncated BIFF8 reference column".to_owned()))?;
    let column = u16::from_le_bytes([column_bytes[0], column_bytes[1]]);
    let shifted = if column & 0x8000 != 0 {
        let relative = i32::from(i16::from_le_bytes(row.to_le_bytes()));
        let target = i32::from(formula_row) + relative;
        if !(0..=i32::from(u16::MAX)).contains(&target) {
            return Err(ExcelError::Xls(
                "BIFF8 relative formula reference leaves worksheet".to_owned(),
            ));
        }
        let target = shifted_row(
            u16::try_from(target).map_err(|_| {
                ExcelError::Xls("BIFF8 relative formula target conversion failed".to_owned())
            })?,
            start_row,
            delta,
        )?;
        let relative = i32::from(target) - i32::from(shifted_formula_row);
        let relative = i16::try_from(relative).map_err(|_| {
            ExcelError::Xls("BIFF8 shifted relative formula offset exceeds i16".to_owned())
        })?;
        u16::from_le_bytes(relative.to_le_bytes())
    } else {
        shifted_row(row, start_row, delta)?
    };
    tokens[row_offset..row_offset + 2].copy_from_slice(&shifted.to_le_bytes());
    Ok(())
}

fn ptg_encoded_len(tokens: &[u8], cursor: usize, base: u8) -> Result<usize> {
    let fixed = match base {
        0x01 | 0x02 => 5,
        0x03..=0x16 => 1,
        0x17 => {
            let count = usize::from(*tokens.get(cursor + 1).ok_or_else(|| {
                ExcelError::Xls("truncated BIFF8 string formula token".to_owned())
            })?);
            let flags = *tokens.get(cursor + 2).ok_or_else(|| {
                ExcelError::Xls("truncated BIFF8 string formula flags".to_owned())
            })?;
            3usize.saturating_add(count.saturating_mul(if flags & 1 == 0 { 1 } else { 2 }))
        }
        0x18 => 1,
        0x19 => {
            let options = *tokens.get(cursor + 1).ok_or_else(|| {
                ExcelError::Xls("truncated BIFF8 attribute formula token".to_owned())
            })?;
            if options & 0x04 == 0 {
                4
            } else {
                let count = u16::from_le_bytes([
                    *tokens.get(cursor + 2).ok_or_else(|| {
                        ExcelError::Xls("truncated BIFF8 choose token".to_owned())
                    })?,
                    *tokens.get(cursor + 3).ok_or_else(|| {
                        ExcelError::Xls("truncated BIFF8 choose token".to_owned())
                    })?,
                ]);
                4usize.saturating_add(usize::from(count).saturating_add(1).saturating_mul(2))
            }
        }
        0x1A | 0x1B => 1,
        0x1C | 0x1D => 2,
        0x1E => 3,
        0x1F => 9,
        0x20 => 8,
        0x21 => 3,
        0x22 => 4,
        0x23 | 0x24 | 0x2A | 0x2C => 5,
        0x25 | 0x2B | 0x2D => 9,
        0x26..=0x28 => 7,
        0x29 => 3,
        0x39 => 7,
        0x3A | 0x3C => 7,
        0x3B | 0x3D => 11,
        other => {
            return Err(ExcelError::Xls(format!(
                "unsupported BIFF8 formula token 0x{other:02X} while shifting rows"
            )));
        }
    };
    if cursor.saturating_add(fixed) > tokens.len() {
        return Err(ExcelError::Xls(format!(
            "truncated BIFF8 formula token 0x{base:02X}"
        )));
    }
    Ok(fixed)
}
