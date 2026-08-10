#[test]
#[allow(clippy::too_many_lines)]
fn image_anchor_layout_and_validation_cover_java_coordinate_boundaries() -> Result<()> {
    let columns = selected_columns(AnchoredImageRow::schema(), &WriteOptions::default())?;
    let layout = ImageLayout::new(
        &columns,
        &WriteOptions {
            column_widths: vec![(0, 12), (2, 0)],
            ..WriteOptions::default()
        },
        AnchoredImageRow::write_metadata(),
        1,
        &[],
    )?;
    assert_eq!(layout.column_width(0), 89);
    assert_eq!(layout.column_width(1), 64);
    assert_eq!(layout.column_width(2), 0);
    assert_eq!(layout.row_height(0), 24);
    assert_eq!(layout.row_height(1), 40);
    assert_eq!(easyexcel_xlsx::xlsx::generation::column_width_pixels(0), 0);
    assert_eq!(
        easyexcel_xlsx::xlsx::generation::row_height_pixels(None),
        20
    );
    let bytes = tiny_png();
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let invalid_anchors = [
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_first_row_index(-1)),
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_first_column_index(-1)),
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_last_row_index(-1)),
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_last_column_index(-1)),
        ClientAnchorData::new()
            .coordinates(CoordinateData::new().first_row_index(2).last_row_index(1)),
        ClientAnchorData::new().coordinates(
            CoordinateData::new()
                .relative_first_column_index(70_000)
                .relative_last_column_index(70_000),
        ),
        ClientAnchorData::new()
            .coordinates(CoordinateData::new().relative_last_column_index(70_000)),
        ClientAnchorData::new().coordinates(
            CoordinateData::new()
                .first_row_index(1_048_576)
                .last_row_index(1_048_576),
        ),
        ClientAnchorData::new().left(64),
        ClientAnchorData::new().top(20),
    ];
    for anchor in invalid_anchors {
        assert!(
            insert_image_data(
                worksheet,
                0,
                0,
                &ImageData::new(bytes.clone()).anchor(anchor),
                &ImageLayout::default(),
            )
            .is_err()
        );
    }
    assert!(
        insert_image_data(
            worksheet,
            0,
            0,
            &ImageData::new([1, 2, 3]),
            &ImageLayout::default(),
        )
        .is_err()
    );

    let width_overflow = ImageLayout {
        column_widths: HashMap::from([(0, u32::MAX)]),
        ..ImageLayout::default()
    };
    let two_columns =
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_last_column_index(1));
    assert!(
        insert_image_data(
            worksheet,
            0,
            0,
            &ImageData::new(bytes.clone()).anchor(two_columns),
            &width_overflow,
        )
        .is_err()
    );
    let height_overflow = ImageLayout {
        content_row_height: u32::MAX,
        ..ImageLayout::default()
    };
    let two_rows =
        ClientAnchorData::new().coordinates(CoordinateData::new().relative_last_row_index(1));
    assert!(
        insert_image_data(
            worksheet,
            0,
            0,
            &ImageData::new(bytes.clone()).anchor(two_rows),
            &height_overflow,
        )
        .is_err()
    );
    let metadata = ExcelWriteMetadata::new();
    let style = SheetStyleContext::content(None, &metadata, WriteGlobalFlags::default())
        .column(&TEST_COLUMN);
    assert!(
        write_cell(
            worksheet,
            0,
            0,
            &TEST_COLUMN,
            &CellValue::Images {
                value: Box::new(CellValue::Decimal(
                    "9".repeat(400).parse().expect("valid huge decimal"),
                )),
                images: Vec::new(),
            },
            style.clone(),
            &ImageLayout::default(),
        )
        .is_err()
    );
    assert!(
        write_cell(
            worksheet,
            0,
            0,
            &TEST_COLUMN,
            &CellValue::Images {
                value: Box::new(CellValue::Empty),
                images: vec![ImageData::new([1, 2, 3])],
            },
            style,
            &ImageLayout::default(),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn decimal_writer_rejects_values_outside_xlsx_numeric_range() {
    let huge: BigDecimal = "9".repeat(400).parse().expect("valid large decimal");
    let metadata = ExcelWriteMetadata::new();
    let style = SheetStyleContext::content(None, &metadata, WriteGlobalFlags::default())
        .column(&TEST_COLUMN);
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    assert!(
        write_cell(
            worksheet,
            0,
            0,
            &TEST_COLUMN,
            &CellValue::Decimal(huge),
            style.clone(),
            &ImageLayout::default(),
        )
        .is_err()
    );
    assert!(
        write_cell(
            worksheet,
            u32::MAX,
            0,
            &TEST_COLUMN,
            &CellValue::Decimal("1.5".parse().expect("valid decimal")),
            style,
            &ImageLayout::default(),
        )
        .is_err()
    );
}

#[test]
fn constant_memory_writer_can_omit_headers_and_freeze_request() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("stream.xlsx");
    write_xlsx::<AutoStateRow, _>(
        &path,
        &WriteOptions {
            sheet_name: "Stream".to_owned(),
            constant_memory: true,
            need_head: false,
            freeze_head: true,
            freeze_panes: None,
            ..WriteOptions::default()
        },
        vec![AutoStateRow { value: 1 }, AutoStateRow { value: 2 }],
    )?;
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(test_error)?;
    let range = workbook.worksheet_range("Stream").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::Float(1.0))
    );
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::Float(2.0))
    );
    Ok(())
}

#[test]
fn constant_memory_rejects_advanced_cells_with_a_stable_error() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("advanced-stream.xlsx");
    let error = write_xlsx::<EveryCell, _>(
        &path,
        &WriteOptions {
            constant_memory: true,
            ..WriteOptions::default()
        },
        vec![every_cell()],
    )
    .expect_err("advanced cells must not be silently dropped in constant-memory mode");
    assert!(matches!(error, ExcelError::Unsupported(_)));
    assert_eq!(
        error.to_string(),
        "unsupported operation: constant-memory XLSX does not support comment, image, images, or rich-text cells"
    );
    Ok(())
}

/// Java `SXSSFWorkbook.setCompressTempFiles(true)` → `WriteOptions.compress_temp_files`.
///
/// Verifies the flag forces constant-memory spill, mirrors rows into a gzip tempfile
/// (magic `1f 8b`), and multi-batch writes succeed without OOM.
#[test]
fn compress_temp_files_forces_constant_memory_spill() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("compress_temp.xlsx");
    let sheet = WriteSheet::<AutoStateRow>::new("Spill").compress_temp_files(true);
    assert!(sheet.options().compress_temp_files);
    assert!(sheet.options().constant_memory);

    let mut writer = ExcelWriter::with_handlers_and_options(
        &path,
        Vec::new(),
        WriteOptions {
            compress_temp_files: true,
            ..WriteOptions::default()
        },
    );
    assert!(writer.compress_temp_files_enabled());
    for _ in 0..5 {
        writer.write(
            vec![AutoStateRow { value: 1 }, AutoStateRow { value: 2 }],
            &sheet,
        )?;
    }
    // Late toggle mirrors Java afterWorkbookCreate (no-op once sheets exist).
    writer.set_compress_temp_files(true);
    writer.finish()?;

    let snap = writer
        .last_gzip_spill_snapshot()
        .expect("gzip spill snapshot after finish");
    assert!(snap.is_gzip, "spill must start with gzip magic 1f 8b");
    assert!(snap.uncompressed_len > 0);
    assert!(snap.compressed_len > 0);

    let mut workbook: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    let range = workbook.worksheet_range("Spill").map_err(test_error)?;
    // Header + 10 data rows.
    assert_eq!(range.height(), 11);
    assert_eq!(
        range.get_value((1, 0)),
        Some(&Data::Float(1.0))
    );
    Ok(())
}
