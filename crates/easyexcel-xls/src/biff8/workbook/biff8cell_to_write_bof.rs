include!("biff8cell_to_write_bof/biff8cell.rs");



include!("biff8cell_to_write_bof/biff8value.rs");

include!("biff8cell_to_write_bof/biff8merge.rs");



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
    let mut out: Vec<u8> = Vec::new();
    write_bof(&mut out, DT_GLOBALS);
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
    let (sst_strings, sst_index, total_refs) = build_sst(&sheets);

    let mut boundsheet_patches = Vec::with_capacity(sheets.len());
    for sheet in &sheets {
        boundsheet_patches.push(write_boundsheet_placeholder(&mut out, &sheet.name));
    }

    if !sst_strings.is_empty() {
        out.extend_from_slice(&build_sst_records(&sst_strings, total_refs));
        record(&mut out, EXTSST, &[0, 0]);
    }
    record(&mut out, EOF, &[]);

    let mut sheet_offsets = Vec::with_capacity(sheets.len());
    for (sheet_idx, sheet) in sheets.iter().enumerate() {
        sheet_offsets.push(out.len() as u32);
        // 默认 sheet 场景（book.sheets 为空）无缓存表，回退空表
        let cache = caches.get(sheet_idx).cloned().unwrap_or_default();
        write_worksheet(&mut out, sheet, &sst_index, &cache);
    }
    for (patch_off, pos) in boundsheet_patches.into_iter().zip(sheet_offsets) {
        out[patch_off..patch_off + 4].copy_from_slice(&pos.to_le_bytes());
    }
    out
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
