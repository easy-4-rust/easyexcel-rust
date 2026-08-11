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

fn write_boundsheet_placeholder(out: &mut Vec<u8>, sheet: &Biff8Sheet) -> usize {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes());
    data.push(match sheet.visibility {
        easyexcel_model::Visibility::Visible => 0,
        easyexcel_model::Visibility::Hidden => 1,
        easyexcel_model::Visibility::VeryHidden => 2,
    });
    data.push(0x00);
    data.extend_from_slice(&encode_short_unicode_string(&sheet.name));
    let record_start = out.len();
    record(out, BOUNDSHEET, &data);
    record_start + 4
}

// 语义敏感：SST 条目数与 Excel 字符串表规模一致（远小于 u32 上限），
// usize->u32 不可能截断；保留 as 以对齐 BIFF8 规范。
#[allow(clippy::cast_possible_truncation)]
fn build_sst(
    sheets: &[Biff8Sheet],
) -> (
    Vec<Biff8RichText>,
    HashMap<Biff8RichText, u32>,
    u32,
) {
    let mut strings = Vec::new();
    let mut index = HashMap::new();
    let mut total_refs = 0u32;
    for sheet in sheets {
        for cell in sheet.cells.values() {
            let rich = match &cell.value {
                Biff8Value::Text(text) => Some(Biff8RichText::plain(text.clone())),
                Biff8Value::RichText(rich) => Some(rich.clone()),
                _ => None,
            };
            let Some(rich) = rich else { continue };
            total_refs += 1;
            if let std::collections::hash_map::Entry::Vacant(entry) = index.entry(rich.clone()) {
                entry.insert(strings.len() as u32);
                strings.push(rich);
            }
        }
    }
    (strings, index, total_refs)
}

// 语义敏感：同上，SST 字符串计数转换为 BIFF8 u32 计数字段。
#[allow(clippy::cast_possible_truncation)]
fn build_sst_records(strings: &[Biff8RichText], total_refs: u32) -> Vec<u8> {
    let mut framer = Biff8SstFramer::new(total_refs, strings.len() as u32);
    for rich in strings {
        framer.push_rich_text(rich);
    }
    framer.finish()
}

fn write_worksheet(
    out: &mut Vec<u8>,
    sheet: &Biff8Sheet,
    sst_index: &HashMap<Biff8RichText, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
    link_table: &super::ptg::Biff8LinkTable,
    comment_drawing_id: Option<u16>,
    first_chart_drawing_id: u16,
    selected: bool,
) -> Result<()> {
    write_bof(out, DT_WORKSHEET);
    let physical_rows = physical_rows(sheet);
    let index_patch = write_index_placeholder(out, &physical_rows);
    if let Some(password_hash) = sheet.protection_password_hash {
        record(out, PROTECT, &1_u16.to_le_bytes());
        record(out, OBJECTPROTECT, &1_u16.to_le_bytes());
        record(out, SCENPROTECT, &1_u16.to_le_bytes());
        record(out, PASSWORD, &password_hash.to_le_bytes());
    }
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
    if let Some(width_units) = sheet.default_column_width_units {
        let width_chars = width_units.div_ceil(256).max(1);
        record(out, DEFCOLWIDTH, &width_chars.to_le_bytes());
        record(out, STANDARDWIDTH, &width_units.to_le_bytes());
    }
    if let Some(height_twips) = sheet.default_row_height_twips {
        let mut data = [0_u8; 4];
        data[0..2].copy_from_slice(&1_u16.to_le_bytes()); // fUnsynced: 显式默认行高
        data[2..4].copy_from_slice(&height_twips.to_le_bytes());
        record(out, DEFAULTROWHEIGHT, &data);
    }
    // COLINFO — Java HSSF ColumnInfoRecord / sheet.setColumnWidth
    let mut columns = sheet.column_widths.keys().copied().collect::<BTreeSet<_>>();
    columns.extend(sheet.column_width_units.keys().copied());
    columns.extend(sheet.column_xfs.keys().copied());
    columns.extend(sheet.hidden_columns.iter().copied());
    for col in columns {
        let width = sheet.column_width_units.get(&col).copied().unwrap_or_else(|| {
            sheet.column_widths.get(&col).map_or_else(
                || sheet.default_column_width_units.unwrap_or(8_u16.saturating_mul(256)),
                |width| width.saturating_mul(256),
            )
        });
        let xf = sheet.column_xfs.get(&col).copied().unwrap_or(XF_GENERAL);
        let user_set_width = sheet.column_user_set_widths.contains(&col);
        record(
            out,
            COLINFO,
            &pack_colinfo_metadata(
                col,
                col,
                width,
                xf,
                sheet.hidden_columns.contains(&col),
                user_set_width,
            ),
        );
    }
    let dbcell_offsets = write_row_blocks(
        out,
        sheet,
        &physical_rows,
        sst_index,
        caches,
        link_table,
    )?;
    for (patch_at, offset) in index_patch.into_iter().zip(dbcell_offsets) {
        out[patch_at..patch_at + 4].copy_from_slice(&offset.to_le_bytes());
    }
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
    if let Some(drawing_id) = comment_drawing_id {
        write_comments_with_drawing_id(out, &sheet.comments, drawing_id);
    }
    write_charts_with_drawing_ids(
        out,
        &sheet.charts,
        link_table,
        first_chart_drawing_id,
        2,
    );
    {
        let mut data = vec![0u8; 18];
        // options: fDspGrid | fDspRwCol | fDspZeros | fDefaultHdr | fDspGuts |
        // fUnsynced | fSelected | fDspSheet（与 LibreOffice 默认值 0x06B6 一致）
        let options = if selected { 0x06B6_u16 } else { 0x04B6_u16 };
        data[0..2].copy_from_slice(&options.to_le_bytes());
        if let Some((_rows, _cols)) = sheet.freeze.filter(|&(r, c)| r > 0 || c > 0) {
            // MS-XLS Window2：fFrozen(bit3=0x0008) + fFrozenNoSplit(bit8=0x0100)。
            data[0] |= 0x08;
            data[1] |= 0x01;
        }
        record(out, WINDOW2, &data);
        // WINDOW2 之后发射 PANE（xlwt/POI 流序一致）
        if let Some((rows, cols)) = sheet.freeze.filter(|&(r, c)| r > 0 || c > 0) {
            write_pane(out, rows, cols);
        }
    }
    // BIFF8 record order places the Hyperlink Table after the view settings
    // (WINDOW2/SCL/PANE/SELECTION) and merged cells. Apache POI uses WINDOW2 as
    // the row-block terminator, so emitting HLINK before it makes the workbook
    // structurally unreadable by HSSF.
    for hyperlink in &sheet.hyperlinks {
        record(out, HYPERLINK, &hyperlink.encode_record_data());
    }
    record(out, EOF, &[]);
    Ok(())
}

fn physical_rows(sheet: &Biff8Sheet) -> Vec<u16> {
    let mut rows = sheet
        .cells
        .keys()
        .map(|(row, _)| *row)
        .collect::<BTreeSet<_>>();
    rows.extend(sheet.row_heights.keys().copied());
    rows.extend(sheet.row_height_twips.keys().copied());
    rows.extend(sheet.row_xfs.keys().copied());
    rows.extend(sheet.hidden_rows.iter().copied());
    rows.into_iter().collect()
}

fn write_index_placeholder(out: &mut Vec<u8>, rows: &[u16]) -> Vec<usize> {
    let block_count = rows.len().div_ceil(32);
    let mut data = Vec::with_capacity(16 + block_count * 4);
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&u32::from(rows.first().copied().unwrap_or(0)).to_le_bytes());
    data.extend_from_slice(
        &u32::from(rows.last().copied().map_or(0, |row| row.saturating_add(1))).to_le_bytes(),
    );
    data.extend_from_slice(&0u32.to_le_bytes());
    data.resize(16 + block_count * 4, 0);
    let record_start = out.len();
    record(out, INDEX, &data);
    (0..block_count)
        .map(|block| record_start + 4 + 16 + block * 4)
        .collect()
}

fn write_row_blocks(
    out: &mut Vec<u8>,
    sheet: &Biff8Sheet,
    rows: &[u16],
    sst_index: &HashMap<Biff8RichText, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
    link_table: &super::ptg::Biff8LinkTable,
) -> Result<Vec<u32>> {
    let mut dbcell_offsets = Vec::with_capacity(rows.len().div_ceil(32));
    for block in rows.chunks(32) {
        let first_row_record = out.len();
        for &row in block {
            let (first_col, last_col_exclusive) = row_column_span(sheet, row);
            let explicit_height = sheet
                .row_height_twips
                .get(&row)
                .copied()
                .or_else(|| sheet.row_heights.get(&row).map(|height| height.saturating_mul(20)));
            let hidden = sheet.hidden_rows.contains(&row);
            let xf = sheet.row_xfs.get(&row).copied();
            let payload = if explicit_height.is_none() && !hidden && xf.is_none() {
                pack_default_row(row, first_col, last_col_exclusive)
            } else {
                let height_twips = explicit_height
                    .or(sheet.default_row_height_twips)
                    .unwrap_or(15_u16.saturating_mul(20));
                if !(2..=8_192).contains(&height_twips) {
                    return Err(ExcelError::Xls(format!(
                        "BIFF8 row height must be 2..=8192 twips for row {row}, got {height_twips}"
                    )));
                }
                pack_row_metadata(
                    row,
                    first_col,
                    last_col_exclusive,
                    height_twips,
                    explicit_height.is_some(),
                    hidden,
                    xf,
                )
            };
            record(out, ROW, &payload);
        }

        let row_record_bytes = out.len().saturating_sub(first_row_record);
        let mut cell_reference_offset = row_record_bytes.saturating_sub(20);
        let mut cell_offsets = Vec::new();
        for &row in block {
            if sheet
                .cells
                .range((row, 0)..=(row, u8::MAX))
                .next()
                .is_none()
            {
                continue;
            }
            cell_offsets.push(u16::try_from(cell_reference_offset).map_err(|_| {
                ExcelError::Xls(format!(
                    "BIFF8 DBCELL offset exceeds u16 for row {row}"
                ))
            })?);
            let cells_start = out.len();
            write_cells_for_row(out, sheet, row, sst_index, caches, link_table)?;
            cell_reference_offset = out.len().saturating_sub(cells_start);
        }

        let dbcell_position = out.len();
        let row_offset = u32::try_from(dbcell_position.saturating_sub(first_row_record))
            .map_err(|_| ExcelError::Xls("BIFF8 DBCELL row offset overflow".to_owned()))?;
        let mut data = Vec::with_capacity(4 + cell_offsets.len() * 2);
        data.extend_from_slice(&row_offset.to_le_bytes());
        for offset in cell_offsets {
            data.extend_from_slice(&offset.to_le_bytes());
        }
        record(out, DBCELL, &data);
        dbcell_offsets.push(
            u32::try_from(dbcell_position)
                .map_err(|_| ExcelError::Xls("BIFF8 INDEX offset exceeds 4GiB".to_owned()))?,
        );
    }
    Ok(dbcell_offsets)
}

fn row_column_span(sheet: &Biff8Sheet, row: u16) -> (u16, u16) {
    let mut columns = sheet
        .cells
        .range((row, 0)..=(row, u8::MAX))
        .map(|((_, column), _)| u16::from(*column));
    let Some(first) = columns.next() else {
        return (0, 0);
    };
    let last = columns.next_back().unwrap_or(first);
    (first, last.saturating_add(1))
}

fn write_cells_for_row(
    out: &mut Vec<u8>,
    sheet: &Biff8Sheet,
    row: u16,
    sst_index: &HashMap<Biff8RichText, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
    link_table: &super::ptg::Biff8LinkTable,
) -> Result<()> {
    let mut cells = sheet
        .cells
        .range((row, 0)..=(row, u8::MAX))
        .map(|(&(row, column), cell)| (row, column, cell))
        .collect::<Vec<_>>();
    flush_row(out, &mut cells, sst_index, caches, link_table)
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

fn flush_row(
    out: &mut Vec<u8>,
    cells: &mut [(u16, u8, &Biff8Cell)],
    sst_index: &HashMap<Biff8RichText, u32>,
    caches: &HashMap<(u16, u8), Biff8Cached>,
    link_table: &super::ptg::Biff8LinkTable,
) -> Result<()> {
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
                write_cell(
                    out,
                    row,
                    col,
                    cell,
                    sst_index,
                    caches.get(&(row, col)),
                    link_table,
                )?;
                i += 1;
            }
        }
    }
    Ok(())
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
    sst_index: &HashMap<Biff8RichText, u32>,
    cached: Option<&Biff8Cached>,
    link_table: &super::ptg::Biff8LinkTable,
) -> Result<()> {
    match &cell.value {
        Biff8Value::Blank => write_blank(out, row, col, cell.xf),
        Biff8Value::Text(text) => {
            let idx = *sst_index
                .get(&Biff8RichText::plain(text.clone()))
                .unwrap_or(&0);
            write_labelsst(out, row, col, cell.xf, idx);
        }
        Biff8Value::RichText(rich) => {
            let idx = *sst_index.get(rich).unwrap_or(&0);
            write_labelsst(out, row, col, cell.xf, idx);
        }
        Biff8Value::Number(n) => write_number(out, row, col, cell.xf, *n),
        Biff8Value::Bool(b) => write_boolerr(out, row, col, cell.xf, u8::from(*b), false),
        Biff8Value::Error(code) => write_boolerr(out, row, col, cell.xf, *code, true),
        Biff8Value::Formula(expr) => {
            write_formula(out, row, col, cell.xf, expr, cached, link_table)?;
        }
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
    link_table: &super::ptg::Biff8LinkTable,
) -> Result<()> {
    // 空公式表达式：rgce 为空（BIFF8 允许 FORMULA 记录 rgce 长度为 0，
    // 仅存储缓存值，Excel/LibreOffice 打开时不会重算）。
    let rgce = if expr.trim().is_empty() {
        Vec::new()
    } else {
        super::ptg::encode_formula_rpn_with_link_table(expr, link_table)?
    };
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
