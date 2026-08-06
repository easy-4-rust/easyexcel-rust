fn write_default_font(out: &mut Vec<u8>) {
    let mut data = Vec::new();
    data.extend_from_slice(&200u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0x7FFFu16.to_le_bytes());
    data.extend_from_slice(&400u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&[0, 0, 0, 0]);
    data.extend_from_slice(&encode_short_unicode_string("Arial"));
    record(out, FONT, &data);
}

fn write_style_xf(out: &mut Vec<u8>) {
    let mut data = vec![0u8; 20];
    data[4] = 0xF5;
    data[5] = 0xFF;
    record(out, XF, &data);
}

fn write_cell_xf(out: &mut Vec<u8>, ifmt: u16) {
    let mut data = vec![0u8; 20];
    data[2..4].copy_from_slice(&ifmt.to_le_bytes());
    data[4..6].copy_from_slice(&0x0001u16.to_le_bytes());
    record(out, XF, &data);
}

fn write_boundsheet_placeholder(out: &mut Vec<u8>, name: &str) -> usize {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&[0x00, 0x00]);
    data.extend_from_slice(&encode_short_unicode_string(name));
    let record_start = out.len();
    record(out, BOUNDSHEET, &data);
    record_start + 4
}

// 语义敏感：SST 条目数与 Excel 字符串表规模一致（远小于 u32 上限），
// usize->u32 不可能截断；保留 as 以对齐 BIFF8 规范。
#[allow(clippy::cast_possible_truncation)]
fn build_sst(sheets: &[Biff8Sheet]) -> (Vec<String>, HashMap<String, u32>, u32) {
    let mut strings = Vec::new();
    let mut index = HashMap::new();
    let mut total_refs = 0u32;
    for sheet in sheets {
        for cell in sheet.cells.values() {
            if let Biff8Value::Text(text) = &cell.value {
                total_refs += 1;
                if let std::collections::hash_map::Entry::Vacant(entry) = index.entry(text.clone())
                {
                    entry.insert(strings.len() as u32);
                    strings.push(text.clone());
                }
            }
        }
    }
    (strings, index, total_refs)
}

// 语义敏感：同上，SST 字符串计数转换为 BIFF8 u32 计数字段。
#[allow(clippy::cast_possible_truncation)]
fn build_sst_records(strings: &[String], total_refs: u32) -> Vec<u8> {
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let mut header = Vec::new();
    header.extend_from_slice(&total_refs.to_le_bytes());
    header.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    pieces.push(header);
    for s in strings {
        pieces.push(encode_unicode_string(s));
    }

    let mut out = Vec::new();
    let mut current = Vec::new();
    let mut first = true;
    for piece in pieces {
        if !current.is_empty() && current.len() + piece.len() > MAX_RECORD_DATA {
            flush_sst_chunk(&mut out, &mut current, &mut first);
        }
        if piece.len() > MAX_RECORD_DATA {
            let mut offset = 0;
            while offset < piece.len() {
                let room = MAX_RECORD_DATA.saturating_sub(current.len());
                if room == 0 {
                    flush_sst_chunk(&mut out, &mut current, &mut first);
                    continue;
                }
                let take = room.min(piece.len() - offset);
                current.extend_from_slice(&piece[offset..offset + take]);
                offset += take;
            }
        } else {
            current.extend_from_slice(&piece);
        }
    }
    if !current.is_empty() {
        flush_sst_chunk(&mut out, &mut current, &mut first);
    }
    out
}

fn flush_sst_chunk(out: &mut Vec<u8>, current: &mut Vec<u8>, first: &mut bool) {
    let typ = if *first { SST } else { CONTINUE };
    record(out, typ, current);
    *first = false;
    current.clear();
}

fn write_worksheet(
    out: &mut Vec<u8>,
    sheet: &Biff8Sheet,
    sst_index: &HashMap<String, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
) {
    write_bof(out, DT_WORKSHEET);
    let (max_row, max_col) = sheet.dimensions();
    {
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&max_row.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&max_col.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        record(out, DIMENSION, &data);
    }
    // COLINFO — Java HSSF ColumnInfoRecord / sheet.setColumnWidth
    for (&col, &width) in &sheet.column_widths {
        record(out, COLINFO, &pack_colinfo(col, col, width, XF_GENERAL));
    }
    // ROW — Java HSSF RowRecord / setHeightInPoints
    let last_col_exclusive = u8::try_from(max_col.min(256)).unwrap_or(0);
    for (&row, &height) in &sheet.row_heights {
        record(out, ROW, &pack_row(row, 0, last_col_exclusive, height));
    }
    write_cells(out, sheet, sst_index, caches);
    if !sheet.merges.is_empty() {
        let ranges: Vec<[u8; 8]> = sheet
            .merges
            .iter()
            .map(|m| {
                pack_merge_range(
                    m.first_row,
                    m.last_row,
                    u16::from(m.first_col),
                    u16::from(m.last_col),
                )
            })
            .collect();
        write_merge_cells(out, &ranges);
    }
    {
        let mut data = vec![0u8; 18];
        // options: fDspGrid | fDspRwCol | fDspZeros | fDefaultHdr | fDspGuts |
        // fUnsynced | fSelected | fDspSheet（与 LibreOffice 默认值 0x06B6 一致）
        data[0] = 0xB6;
        data[1] = 0x06;
        if let Some((_rows, _cols)) = sheet.freeze.filter(|&(r, c)| r > 0 || c > 0) {
            // fFrozen(bit3=0x0008) + fFrozenNoSplit(bit12=0x1000)
            data[0] |= 0x08;
            data[1] |= 0x10;
        }
        record(out, WINDOW2, &data);
        // WINDOW2 之后发射 PANE（xlwt/POI 流序一致）
        if let Some((rows, cols)) = sheet.freeze.filter(|&(r, c)| r > 0 || c > 0) {
            write_pane(out, rows, cols);
        }
    }
    record(out, EOF, &[]);
}

/// PANE 记录（0x0041）：冻结窗格布局。与 xlwt `PanesRecord` 字节一致——
/// px=冻结列数, py=冻结行数, rwTop=底窗格首个可见行, colLeft=右窗格首个
/// 可见列, pnnAct=活动窗格（行列都冻结→0, 仅列→1, 仅行→2）, 末字节保留。
/// 对应 Java：POI `PaneRecord`（字段顺序相同，POI 另写 pnnFrz）。
fn write_pane(out: &mut Vec<u8>, rows: u16, cols: u16) {
    let mut data = Vec::with_capacity(10);
    data.extend_from_slice(&cols.to_le_bytes()); // px: 冻结列数
    data.extend_from_slice(&rows.to_le_bytes()); // py: 冻结行数
    data.extend_from_slice(&rows.to_le_bytes()); // rwTop
    data.extend_from_slice(&cols.to_le_bytes()); // colLeft
    data.push(match (cols > 0, rows > 0) {
        (true, true) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (false, false) => 3,
    });
    data.push(0); // BIFF8 保留字节（xlwt 不写 pnnFrz）
    record(out, PANE, &data);
}

/// 逐行扫描单元格，把连续的 RK 可编码数字合并为 MULRK、连续空白合并为
/// MULBLANK（文件更小、Excel 打开更快）；其余类型逐格写出。
/// 对应 Java：POI `MulRKRecord` / `MulBlankRecord`。
fn write_cells(
    out: &mut Vec<u8>,
    sheet: &Biff8Sheet,
    sst_index: &HashMap<String, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
) {
    // BTreeMap 按键 (row, col) 有序：按行分组扫描
    let mut row_cells: Vec<(u16, u8, &Biff8Cell)> = Vec::new();
    let mut last_row = None;
    for (&(row, col), cell) in &sheet.cells {
        if last_row != Some(row) && !row_cells.is_empty() {
            flush_row(out, &mut row_cells, sst_index, caches);
            row_cells.clear();
        }
        last_row = Some(row);
        row_cells.push((row, col, cell));
    }
    if !row_cells.is_empty() {
        flush_row(out, &mut row_cells, sst_index, caches);
    }
}

fn flush_row(
    out: &mut Vec<u8>,
    cells: &mut [(u16, u8, &Biff8Cell)],
    sst_index: &HashMap<String, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
) {
    // 行内按列扫描，收集可合并段
    let mut i = 0;
    while i < cells.len() {
        let (row, col, cell) = cells[i];
        match &cell.value {
            Biff8Value::Blank => {
                // 收集连续 Blank
                let mut j = i;
                let mut cols = vec![(col, cell.xf)];
                while j + 1 < cells.len()
                    && cells[j + 1].1 == cols.last().map_or(0, |(c, _)| *c) + 1
                    && matches!(cells[j + 1].2.value, Biff8Value::Blank)
                {
                    j += 1;
                    cols.push((cells[j].1, cells[j].2.xf));
                }
                if cols.len() >= 2 {
                    write_mulblank(out, row, &cols);
                } else {
                    write_blank(out, row, col, cell.xf);
                }
                i = j + 1;
            }
            Biff8Value::Number(number) => {
                // 收集连续 RK 可编码数字
                let mut j = i;
                let mut entries: Vec<(u8, u16, u32)> = Vec::new();
                if let Some(rk) = encode_rk(*number) {
                    entries.push((col, cell.xf, rk));
                    while j + 1 < cells.len() {
                        let (nrow, ncol, ncell) = cells[j + 1];
                        if nrow != row || ncol != entries.last().map_or(0, |(c, _, _)| *c) + 1 {
                            break;
                        }
                        if let Biff8Value::Number(n) = &ncell.value
                            && let Some(nrk) = encode_rk(*n)
                        {
                            entries.push((ncol, ncell.xf, nrk));
                            j += 1;
                            continue;
                        }
                        break;
                    }
                }
                if entries.len() >= 2 {
                    write_mulrk(out, row, &entries);
                } else {
                    write_number(out, row, col, cell.xf, *number);
                }
                i = j + 1;
            }
            _ => {
                write_cell(out, row, col, cell, sst_index, caches.get(&(row, col)))
                    .expect("BIFF8 单元格序列化失败");
                i += 1;
            }
        }
    }
}

/// MULRK（0x00BD）：rw + colFirst + (xf, rk)*n + colLast
fn write_mulrk(out: &mut Vec<u8>, row: u16, entries: &[(u8, u16, u32)]) {
    let mut data = Vec::with_capacity(8 + entries.len() * 6);
    data.extend_from_slice(&row.to_le_bytes());
    data.extend_from_slice(&u16::from(entries[0].0).to_le_bytes());
    for &(_, xf, rk) in entries {
        data.extend_from_slice(&xf.to_le_bytes());
        data.extend_from_slice(&rk.to_le_bytes());
    }
    data.extend_from_slice(&u16::from(entries[entries.len() - 1].0).to_le_bytes());
    record(out, MULRK, &data);
}

/// MULBLANK（0x00BE）：rw + colFirst + xf*n + colLast
fn write_mulblank(out: &mut Vec<u8>, row: u16, entries: &[(u8, u16)]) {
    let mut data = Vec::with_capacity(6 + entries.len() * 2);
    data.extend_from_slice(&row.to_le_bytes());
    data.extend_from_slice(&u16::from(entries[0].0).to_le_bytes());
    for &(_, xf) in entries {
        data.extend_from_slice(&xf.to_le_bytes());
    }
    data.extend_from_slice(&u16::from(entries[entries.len() - 1].0).to_le_bytes());
    record(out, MULBLANK, &data);
}

fn write_cell(
    out: &mut Vec<u8>,
    row: u16,
    col: u8,
    cell: &Biff8Cell,
    sst_index: &HashMap<String, u32>,
    cached: Option<&Biff8Cached>,
) -> Result<()> {
    match &cell.value {
        Biff8Value::Blank => write_blank(out, row, col, cell.xf),
        Biff8Value::Text(text) => {
            let idx = *sst_index.get(text).unwrap_or(&0);
            write_labelsst(out, row, col, cell.xf, idx);
        }
        Biff8Value::Number(n) => write_number(out, row, col, cell.xf, *n),
        Biff8Value::Bool(b) => write_boolerr(out, row, col, cell.xf, u8::from(*b), false),
        Biff8Value::Formula(expr) => write_formula(out, row, col, cell.xf, expr, cached)?,
    }
    Ok(())
}

/// 发射 FORMULA 记录（0x0006）：8 字节缓存结果 + options(2) + chn(4) +
/// cce(2) + rgce（Ptg 令牌数组）。
///
/// 缓存结果优先取写入前求值得到的 [`Biff8Cached`]；缺失时写 0.0（全零，
/// 触发 `Excel` / `LibreOffice` 打开时重算）。
fn write_formula(
    out: &mut Vec<u8>,
    row: u16,
    col: u8,
    xf: u16,
    expr: &str,
    cached: Option<&Biff8Cached>,
) -> Result<()> {
    let rgce = super::ptg::encode_formula_rpn(expr)?;
    let mut data = Vec::with_capacity(22 + rgce.len());
    cell_header(&mut data, row, col, xf);
    match cached {
        Some(Biff8Cached::Number(number)) => {
            data.extend_from_slice(&number.to_le_bytes());
        }
        Some(Biff8Cached::Bool(flag)) => {
            let mut result = [0u8; 8];
            result[0] = 1; // 布尔类型标记
            result[2] = u8::from(*flag);
            result[6] = 0xFF;
            result[7] = 0xFF;
            data.extend_from_slice(&result);
        }
        Some(Biff8Cached::Error(code)) => {
            let mut result = [0u8; 8];
            result[0] = 2; // 错误类型标记
            result[2] = *code;
            result[6] = 0xFF;
            result[7] = 0xFF;
            data.extend_from_slice(&result);
        }
        Some(Biff8Cached::Text(text)) => {
            let mut result = [0u8; 8];
            result[6] = 0xFF; // 字符串标记：结果在后续 STRING 记录
            result[7] = 0xFF;
            data.extend_from_slice(&result);
            // 字符串缓存值：FORMULA 记录后跟随 STRING 记录（0x0207）
            let encoded = encode_unicode_string(text);
            if encoded.len() <= MAX_RECORD_DATA {
                record(out, STRING, &encoded);
            }
        }
        None => {
            // 数字 0.0（全零）→ Excel/LibreOffice 打开时自动重算
            data.extend_from_slice(&0.0f64.to_le_bytes());
        }
    }
    data.extend_from_slice(&0u16.to_le_bytes()); // options: fAlwaysCalc=0
    data.extend_from_slice(&0u32.to_le_bytes()); // chn
    // rgce 长度受 BIFF8 记录上限约束，usize->u16 不会截断
    #[allow(clippy::cast_possible_truncation)]
    data.extend_from_slice(&(rgce.len() as u16).to_le_bytes()); // cce
    data.extend_from_slice(&rgce);
    record(out, FORMULA, &data);
    Ok(())
}

fn cell_header(data: &mut Vec<u8>, row: u16, col: u8, xf: u16) {
    data.extend_from_slice(&row.to_le_bytes());
    data.extend_from_slice(&u16::from(col).to_le_bytes());
    data.extend_from_slice(&xf.to_le_bytes());
}

fn write_blank(out: &mut Vec<u8>, row: u16, col: u8, xf: u16) {
    let mut data = Vec::new();
    cell_header(&mut data, row, col, xf);
    record(out, BLANK, &data);
}

fn write_number(out: &mut Vec<u8>, row: u16, col: u8, xf: u16, n: f64) {
    if let Some(rk) = encode_rk(n) {
        let mut data = Vec::new();
        cell_header(&mut data, row, col, xf);
        data.extend_from_slice(&rk.to_le_bytes());
        record(out, RK, &data);
    } else {
        let mut data = Vec::new();
        cell_header(&mut data, row, col, xf);
        data.extend_from_slice(&n.to_le_bytes());
        record(out, NUMBER, &data);
    }
}

fn write_labelsst(out: &mut Vec<u8>, row: u16, col: u8, xf: u16, sst: u32) {
    let mut data = Vec::new();
    cell_header(&mut data, row, col, xf);
    data.extend_from_slice(&sst.to_le_bytes());
    record(out, LABELSST, &data);
}

fn write_boolerr(out: &mut Vec<u8>, row: u16, col: u8, xf: u16, value: u8, is_error: bool) {
    let mut data = Vec::new();
    cell_header(&mut data, row, col, xf);
    data.push(value);
    data.push(u8::from(is_error));
    record(out, BOOLERR, &data);
}

#[cfg(test)]
#[path = "../workbook_tests/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../workbook_tests/tests_extra.rs"]
mod tests_extra;
