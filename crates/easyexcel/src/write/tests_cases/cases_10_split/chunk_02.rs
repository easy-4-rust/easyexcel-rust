/// `WriteFont` nested on strategy styles merges size/color into the XLSX format
/// (Java `WriteCellStyle.setWriteFont` + `WriteFont.merge`).
#[test]
fn write_font_merges_size_and_color_into_strategy_styles() -> Result<()> {
    #[derive(Debug, Clone)]
    struct PlainRow {
        name: String,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("name", "name", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self {
                name: String::new(),
            })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(self.name.clone())])
        }
    }

    let directory = tempdir()?;
    let path = directory.path().join("write-font-strategy.xlsx");
    let strategy = HorizontalCellStyleStrategy::with_head_and_content(
        ExcelCellStyle::new().into(),
        ExcelCellStyle::new().into(),
    )
    .with_head_write_font(
        &WriteFont::new()
            .font_height_in_points(18.0)
            .color(ExcelColor::Rgb(0x00FF_0000)),
    )
    .with_content_write_font(
        &WriteFont::new()
            .font_height_in_points(11.0)
            .color(ExcelColor::Rgb(0x0000_00FF)),
    );
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(strategy)];
    write_xlsx_with_handlers::<PlainRow, _>(
        &path,
        &WriteOptions {
            head_style: CellStyle::new(),
            ..WriteOptions::default()
        },
        vec![PlainRow {
            name: "fonted".to_owned(),
        }],
        &mut handlers,
    )?;

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut styles = String::new();
    archive
        .by_name("xl/styles.xml")
        .map_err(test_error)?
        .read_to_string(&mut styles)
        .map_err(test_error)?;
    assert!(
        styles.contains("sz val=\"18\"") || styles.contains("sz val=\"18.0\""),
        "expected head font size 18 from WriteFont merge: {styles}"
    );
    assert!(
        styles.contains("sz val=\"11\"") || styles.contains("sz val=\"11.0\""),
        "expected content font size 11 from WriteFont merge: {styles}"
    );
    assert!(
        styles.contains("FF0000") || styles.contains("rgb=\"FFFF0000\""),
        "expected red head font color from WriteFont: {styles}"
    );
    assert!(
        styles.contains("0000FF") || styles.contains("rgb=\"FF0000FF\""),
        "expected blue content font color from WriteFont: {styles}"
    );
    Ok(())
}

/// `LongestMatchColumnWidthStyleStrategy` sets column width from content byte
/// length (Java `String.getBytes().length`), not autofit alone.
#[test]
fn longest_match_sets_column_width_from_byte_length() -> Result<()> {
    #[derive(Debug, Clone)]
    struct PlainRow {
        name: String,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("name", "name", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self {
                name: String::new(),
            })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(self.name.clone())])
        }
    }

    let directory = tempdir()?;
    let path = directory.path().join("longest-match-bytes.xlsx");
    // 20 ASCII bytes → character width 20 (head "name" is shorter).
    let content = "abcdefghijklmnopqrst".to_owned();
    assert_eq!(content.len(), 20);
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(LongestMatchColumnWidthStyleStrategy::new())];
    write_xlsx_with_handlers::<PlainRow, _>(
        &path,
        &WriteOptions::default(),
        vec![PlainRow { name: content }],
        &mut handlers,
    )?;

    // Strategy cache must expose the measured Java byte-length width.
    assert_eq!(
        handlers[0].style_column_width(0),
        Some(20),
        "LongestMatch cache should keep max byte length 20"
    );

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(test_error)?
        .read_to_string(&mut sheet)
        .map_err(test_error)?;
    // POI `setColumnWidth(col, chars * 256)` → OOXML width="{chars}".
    assert!(
        sheet.contains("width=\"20\"") || sheet.contains("width=\"20.0\""),
        "expected LongestMatch byte-length width=20, got: {sheet}"
    );
    Ok(())
}

/// `ImageLayout` reads registered handler column widths for image pixel layout
/// (Java `SimpleColumnWidthStyleStrategy` → sheet column width).
///
/// Handler strategy widths override `@ColumnWidth` annotations so image anchors
/// match the final sheet column widths applied by `apply_handler_column_widths`.
#[test]
fn image_layout_reads_handler_column_width() -> Result<()> {
    let columns = selected_columns(AnchoredImageRow::schema(), &WriteOptions::default())?;
    // AnchoredImageRow annotates column_width=20; uniform(30) must still win.
    let handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(SimpleColumnWidthStyleStrategy::uniform(30))];
    let layout = ImageLayout::new(
        &columns,
        &WriteOptions::default(),
        AnchoredImageRow::write_metadata(),
        1,
        &handlers,
    )?;
    // excel_column_width_pixels(30) = 30 * 7 + 5 = 215
    assert_eq!(layout.column_width(0), 215);
    // Columns outside the schema keep the Excel default pixel width.
    assert_eq!(layout.column_width(1), 64);

    // Explicit WriteOptions widths still beat handler strategies.
    let layout_explicit = ImageLayout::new(
        &columns,
        &WriteOptions {
            column_widths: vec![(0, 12)],
            ..WriteOptions::default()
        },
        AnchoredImageRow::write_metadata(),
        1,
        &handlers,
    )?;
    assert_eq!(layout_explicit.column_width(0), 89);
    Ok(())
}

/// Registered `OnceAbsoluteMergeStrategy` applies merge regions
/// (Java `registerWriteHandler(new OnceAbsoluteMergeStrategy(...))`).
#[test]
fn once_absolute_merge_strategy_register_applies_merge() -> Result<()> {
    #[derive(Debug, Clone)]
    struct PlainRow {
        left: String,
        right: String,
    }

    impl ExcelRow for PlainRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("left", "left", Some(0), 0, None),
                ExcelColumn::new("right", "right", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self {
                left: String::new(),
                right: String::new(),
            })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String(self.left.clone()),
                CellValue::String(self.right.clone()),
            ])
        }
    }

    assert!(OnceAbsoluteMergeStrategy::new(-1, 0, 0, 1).is_err());
    assert!(OnceAbsoluteMergeStrategy::new(0, -1, 0, 1).is_err());
    assert!(OnceAbsoluteMergeStrategy::new(0, 0, -1, 1).is_err());
    assert!(OnceAbsoluteMergeStrategy::new(0, 0, 0, -1).is_err());

    let directory = tempdir()?;
    let path = directory.path().join("once-absolute-register.xlsx");
    // Merge head row columns 0..=1 (Java firstRow=0,lastRow=0,firstCol=0,lastCol=1).
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(OnceAbsoluteMergeStrategy::new(0, 0, 0, 1)?)];
    write_xlsx_with_handlers::<PlainRow, _>(
        &path,
        &WriteOptions::default(),
        vec![PlainRow {
            left: "L".to_owned(),
            right: "R".to_owned(),
        }],
        &mut handlers,
    )?;

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(test_error)?
        .read_to_string(&mut sheet)
        .map_err(test_error)?;
    assert!(
        sheet.contains("<mergeCells") && sheet.contains("ref=\"A1:B1\""),
        "expected OnceAbsoluteMergeStrategy merge A1:B1, got: {sheet}"
    );
    Ok(())
}

/// Integration: write styled + merged `.xls` via `write_xls`, read back with calamine.
///
/// Java mapping: `StyleDataTest` / `LoopMergeStrategy` subset for BIFF8 — asserts
/// cell values and MERGECELLS presence (XF colours are write-side only).
#[test]
fn write_xls_style_merge_round_trip() -> Result<()> {
    let directory = tempdir().map_err(test_error)?;
    let path = directory.path().join("style_merge03.xls");
    let options = WriteOptions {
        sheet_name: "Sheet1".to_owned(),
        column_widths: vec![(0, 40), (1, 20)],
        head_style: CellStyle::new().bold(true).background_color(0x00_FF_FF_00),
        content_styles: vec![CellStyle::new().background_color(0x00_00_80_80)],
        merge_ranges: vec![MergeRange::new(3, 4, 0, 0)],
        loop_merges: vec![MirroredLoopMergeStrategy::new(2, 1, 0)?],
        need_head: true,
        ..WriteOptions::default()
    };
    write_xls::<DimensionRow, _>(
        &path,
        &options,
        vec![DimensionRow, DimensionRow, DimensionRow, DimensionRow],
    )?;

    let mut book: Xls<_> = open_workbook(&path).map_err(test_error)?;
    let range = book.worksheet_range("Sheet1").map_err(test_error)?;
    // Header present
    assert!(range.get((0, 0)).is_some());
    let merges = book
        .merge_cells_by_sheet_name("Sheet1")
        .map_err(test_error)?;
    assert!(
        !merges.is_empty(),
        "expected at least one MERGECELLS region, got {merges:?}"
    );
    // Absolute merge (3,0)-(4,0) and/or loop merges on data rows
    assert!(
        merges.iter().any(|d| {
            let Dimensions { start, end } = *d;
            start.1 == 0 && end.1 == 0 && end.0 > start.0
        }),
        "expected a vertical merge in column 0, got {merges:?}"
    );
    Ok(())
}

