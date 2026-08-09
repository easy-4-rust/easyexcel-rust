include!("biff8cell_to_write_bof/biff8cell.rs");



include!("biff8cell_to_write_bof/biff8value.rs");

include!("biff8cell_to_write_bof/biff8merge.rs");

include!("biff8cell_to_write_bof/biff8_hyperlink_kind.rs");
include!("biff8cell_to_write_bof/biff8hyperlink.rs");

include!("biff8cell_to_write_bof/biff8comment.rs");

include!("biff8cell_to_write_bof/biff8rich_text.rs");

include!("biff8cell_to_write_bof/generated_biff8_cell_value.rs");

include!("biff8cell_to_write_bof/biff8_chart_kind.rs");
include!("biff8cell_to_write_bof/biff8_chart_range.rs");
include!("biff8cell_to_write_bof/biff8_chart_series.rs");
include!("biff8cell_to_write_bof/biff8_chart.rs");



include!("biff8cell_to_write_bof/biff8sheet.rs");



fn checked_row_index(row: u32) -> Result<u16> {
    u16::try_from(row).map_err(|_| ExcelError::Xls("BIFF8 supports at most 65536 rows".to_owned()))
}

fn checked_column_index(col: usize) -> Result<u8> {
    u8::try_from(col).map_err(|_| ExcelError::Xls("BIFF8 supports at most 256 columns".to_owned()))
}

include!("biff8cell_to_write_bof/biff8book.rs");



/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts a calendar date to an Excel 1900-date-system serial number.
#[must_use]
pub fn date_to_excel_serial(date: NaiveDate) -> f64 {
    date_to_excel_serial_with_windowing(date, false)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts a calendar date using either the 1900 or 1904 date windowing system.
///
/// # Panics
///
/// Never in practice; the 1899/1900/1904 epoch constants are statically valid.
#[must_use]
pub fn date_to_excel_serial_with_windowing(date: NaiveDate, use_1904_windowing: bool) -> f64 {
    easyexcel_model::date_to_excel_serial(date, use_1904_windowing)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts a naive date-time to an Excel serial (date + fraction of day).
#[must_use]
pub fn datetime_to_excel_serial(value: NaiveDateTime) -> f64 {
    datetime_to_excel_serial_with_windowing(value, false)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Converts a naive date-time using either the 1900 or 1904 date windowing system.
#[must_use]
pub fn datetime_to_excel_serial_with_windowing(
    value: NaiveDateTime,
    use_1904_windowing: bool,
) -> f64 {
    easyexcel_model::datetime_to_excel_serial(value, use_1904_windowing)
}

/// Builds the BIFF8 `Workbook` stream (globals + worksheet substreams).
// 语义敏感：BOUNDSHEET 偏移为 BIFF8 规范的 u32 绝对偏移，文件流不可能
// 超过 4GiB，usize->u32 在此场景不可能截断。
#[allow(clippy::cast_possible_truncation)]
fn build_workbook_stream(book: &Biff8Book, caches: &[HashMap<(u16, u8), Biff8Cached>]) -> Vec<u8> {
    build_workbook_stream_result(book, caches)
        .expect("generated BIFF8 workbook stream must satisfy structural limits")
}

fn build_workbook_stream_result(
    book: &Biff8Book,
    caches: &[HashMap<(u16, u8), Biff8Cached>],
) -> Result<Vec<u8>> {
    build_workbook_stream_with_filepass(book, caches, None)
}

#[allow(clippy::cast_possible_truncation)]
fn build_workbook_stream_with_filepass(
    book: &Biff8Book,
    caches: &[HashMap<(u16, u8), Biff8Cached>],
    filepass_payload: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    write_bof(&mut out, DT_GLOBALS);
    if let Some(payload) = filepass_payload {
        record(&mut out, FILEPASS, payload);
    }
    record(&mut out, CODEPAGE, &1200u16.to_le_bytes());
    record(&mut out, INTERFACEHDR, &0x04B0u16.to_le_bytes());
    // BIFF8 MMS stores both the added-menu and deleted-menu counters. POI's
    // MMSRecord parser consumes two bytes, so a one-byte payload produces a
    // structurally truncated workbook even when permissive readers accept it.
    record(&mut out, MMS, &[0x00, 0x00]);
    record(&mut out, INTERFACEEND, &[]);
    let mut write_access = vec![0u8; 112];
    write_access[..14].copy_from_slice(b"easyexcel-rust");
    record(&mut out, WRITEACCESS, &write_access);
    record(&mut out, CODENAME, &encode_unicode_string("easyexcel"));
    let date_mode = u16::from(book.use_1904_windowing);
    record(&mut out, DATEMODE, &date_mode.to_le_bytes());
    // CALCMODE：自动重算（0x0001），确保公式在打开时重新计算
    record(&mut out, CALCMODE, &1u16.to_le_bytes());

    for _ in 0..5 {
        write_default_font(&mut out);
    }
    for font in book.styles.custom_fonts() {
        record(&mut out, FONT, &font);
    }
    // FORMAT：自定义数字格式（BIFF8 规范位于 FONT 之后、XF 之前）
    for (ifmt, code) in book.styles.custom_formats() {
        let mut data = Vec::new();
        data.extend_from_slice(&ifmt.to_le_bytes());
        data.extend_from_slice(&encode_short_unicode_string(code));
        record(&mut out, FORMAT, &data);
    }
    if book.styles.needs_palette() {
        write_palette_record(&mut out, book.styles.palette_overrides());
    }
    for _ in 0..16 {
        write_style_xf(&mut out);
    }
    write_cell_xf(&mut out, 14); // XF_DATE
    write_cell_xf(&mut out, 22); // XF_DATETIME
    for xf in book.styles.custom_xfs() {
        record(&mut out, XF, xf);
    }

    {
        let mut data = Vec::new();
        data.extend_from_slice(&0x8000u16.to_le_bytes());
        data.push(0x00);
        data.push(0xFF);
        record(&mut out, STYLE, &data);
    }

    let sheets = if book.sheets.is_empty() {
        vec![Biff8Sheet::new("Sheet1")]
    } else {
        book.sheets.clone()
    };
    if book.active_sheet >= sheets.len() {
        return Err(ExcelError::Xls(format!(
            "BIFF8 active sheet index {} exceeds sheet count {}",
            book.active_sheet,
            sheets.len()
        )));
    }
    let active_sheet = u16::try_from(book.active_sheet)
        .map_err(|_| ExcelError::Xls("BIFF8 active sheet index exceeds u16".to_owned()))?;
    record(&mut out, WINDOW1, &pack_window1(active_sheet, 1));
    let (sst_strings, sst_index, total_refs) = build_sst(&sheets);
    let sheet_names = sheets.iter().map(|sheet| sheet.name.clone()).collect::<Vec<_>>();
    let formulas = sheets
        .iter()
        .flat_map(|sheet| sheet.cells.values())
        .filter_map(|cell| match &cell.value {
            Biff8Value::Formula(formula) => Some(formula.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let chart_references = sheets
        .iter()
        .flat_map(|sheet| sheet.charts.iter())
        .flat_map(|chart| chart.series.iter())
        .flat_map(|series| {
            series
                .categories
                .iter()
                .map(|range| (range.sheet_name.as_str(), range.sheet_name.as_str()))
                .chain(std::iter::once((
                    series.values.sheet_name.as_str(),
                    series.values.sheet_name.as_str(),
                )))
        })
        .collect::<Vec<_>>();
    let link_table = super::ptg::Biff8LinkTable::from_formulas_and_references(
        &sheet_names,
        &formulas,
        &chart_references,
    );
    // DGG 是 Workbook 全局唯一的 drawing/shape 分配表。每个含批注的工作表
    // 使用一个 drawing，每个图表沿用当前独立 drawing 的编码，但所有 id 均
    // 从同一序列分配，避免跨 Sheet 及批注/图表之间重复使用 1/1024。
    let mut next_drawing_id = 1_u16;
    let mut drawing_clusters = Vec::new();
    let mut sheet_drawing_plans = Vec::with_capacity(sheets.len());
    for sheet in &sheets {
        let comment_drawing_id = if sheet.comments.is_empty() {
            None
        } else {
            let drawing_id = next_drawing_id;
            next_drawing_id = next_drawing_id.saturating_add(1);
            drawing_clusters.push((
                drawing_id,
                u32::try_from(sheet.comments.len())
                    .unwrap_or(u32::MAX)
                    .saturating_add(1),
            ));
            Some(drawing_id)
        };
        let first_chart_drawing_id = next_drawing_id;
        for _ in &sheet.charts {
            drawing_clusters.push((next_drawing_id, 3_u32));
            next_drawing_id = next_drawing_id.saturating_add(1);
        }
        sheet_drawing_plans.push((comment_drawing_id, first_chart_drawing_id));
    }
    if !drawing_clusters.is_empty() {
        record(
            &mut out,
            MSODRAWINGGROUP,
            &drawing_group_for_clusters(&drawing_clusters),
        );
    }

    let mut boundsheet_patches = Vec::with_capacity(sheets.len());
    for sheet in &sheets {
        boundsheet_patches.push(write_boundsheet_placeholder(&mut out, sheet));
    }

    if !sst_strings.is_empty() {
        out.extend_from_slice(&build_sst_records(&sst_strings, total_refs));
        record(&mut out, EXTSST, &[0, 0]);
    }
    if !link_table.is_empty() {
        record(&mut out, SUPBOOK, &link_table.supbook_payload());
        record(&mut out, EXTERNSHEET, &link_table.externsheet_payload());
    }
    record(&mut out, EOF, &[]);

    let mut sheet_offsets = Vec::with_capacity(sheets.len());
    for (sheet_idx, sheet) in sheets.iter().enumerate() {
        sheet_offsets.push(out.len() as u32);
        // 默认 sheet 场景（book.sheets 为空）无缓存表，回退空表
        let cache = caches.get(sheet_idx).cloned().unwrap_or_default();
        let (comment_drawing_id, first_chart_drawing_id) = sheet_drawing_plans[sheet_idx];
        write_worksheet(
            &mut out,
            sheet,
            &sst_index,
            &cache,
            &link_table,
            comment_drawing_id,
            first_chart_drawing_id,
            sheet_idx == book.active_sheet,
        )?;
    }
    for (patch_off, pos) in boundsheet_patches.into_iter().zip(sheet_offsets) {
        out[patch_off..patch_off + 4].copy_from_slice(&pos.to_le_bytes());
    }
    Ok(out)
}

fn write_bof(out: &mut Vec<u8>, dt: u16) {
    let mut data = Vec::new();
    data.extend_from_slice(&BIFF8_VERSION.to_le_bytes());
    data.extend_from_slice(&dt.to_le_bytes());
    data.extend_from_slice(&0x0DBBu16.to_le_bytes());
    data.extend_from_slice(&0x07CCu16.to_le_bytes());
    data.extend_from_slice(&0x0000_00C1u32.to_le_bytes());
    data.extend_from_slice(&0x0000_0006u32.to_le_bytes());
    record(out, BOF, &data);
}
