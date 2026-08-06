#[test]
fn facade_template_stream_factories_write_real_archives_and_close_owned_outputs() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("stream-template.xlsx");
    write_minimal_template(
        &template,
        "<sst><si><t>{name}</t></si></sst>",
        "<worksheet><sheetData><row r=\"1\"><c r=\"A1\" t=\"s\"><v>0</v></c></row></sheetData></worksheet>",
    )?;
    let bytes = fs::read(&template)?;

    let path_output = directory.path().join("reader-path.xlsx");
    EasyExcel::template_writer_from_reader(Cursor::new(bytes.clone()), &path_output)?.finish()?;
    assert!(fs::read(path_output)?.starts_with(b"PK"));

    let mut borrowed_path = Cursor::new(Vec::new());
    EasyExcel::template_writer_to_writer(&template, &mut borrowed_path)?.finish()?;
    assert!(borrowed_path.get_ref().starts_with(b"PK"));

    let mut borrowed_reader = Cursor::new(Vec::new());
    EasyExcel::template_writer_from_reader_to_writer(
        Cursor::new(bytes.clone()),
        &mut borrowed_reader,
    )?
    .finish()?;
    assert!(borrowed_reader.get_ref().starts_with(b"PK"));

    let path_stream = ExcelOutputStream::new(FacadeProbeWrite::default());
    let path_observer = path_stream.clone();
    EasyExcel::template_writer_to_output_stream(&template, path_stream)?.finish()?;
    assert!(path_observer.is_closed());

    let reader_stream = ExcelOutputStream::new(FacadeProbeWrite::default());
    let reader_observer = reader_stream.clone();
    EasyExcel::template_writer_from_reader_to_output_stream(Cursor::new(bytes), reader_stream)?
        .finish()?;
    assert!(reader_observer.is_closed());

    let missing = directory.path().join("missing-template.xlsx");
    assert!(
        EasyExcel::template_writer_from_reader(
            Cursor::new(b"invalid".to_vec()),
            directory.path().join("invalid-reader.xlsx")
        )
        .is_err()
    );
    let mut missing_borrowed = Cursor::new(Vec::new());
    assert!(EasyExcel::template_writer_to_writer(&missing, &mut missing_borrowed).is_err());
    assert!(
        EasyExcel::template_writer_from_reader_to_writer(
            Cursor::new(b"invalid".to_vec()),
            &mut missing_borrowed
        )
        .is_err()
    );
    assert!(
        EasyExcel::template_writer_to_output_stream(
            &missing,
            ExcelOutputStream::new(FacadeProbeWrite::default())
        )
        .is_err()
    );
    assert!(
        EasyExcel::template_writer_from_reader_to_output_stream(
            Cursor::new(b"invalid".to_vec()),
            ExcelOutputStream::new(FacadeProbeWrite::default())
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn sheet_selector_inputs_map_indices_borrowed_and_owned_names() {
    assert_eq!(0_usize.into_sheet_selector(), SheetSelector::Index(0));
    assert_eq!(
        "Users".into_sheet_selector(),
        SheetSelector::Name("Users".to_owned())
    );
    assert_eq!(
        "Owned".to_owned().into_sheet_selector(),
        SheetSelector::Name("Owned".to_owned())
    );
    assert!(easyexcel_io::path_has_extension(
        Path::new("legacy.XLS"),
        "xls"
    ));
    assert!(!easyexcel_io::path_has_extension(
        Path::new("modern.xlsx"),
        "xls"
    ));
}

#[test]
#[allow(clippy::too_many_lines)]
fn factories_and_builder_options_match_java_style_chaining() {
    let read = EasyExcel::read::<Value, _>("input.xlsx", Listener::default())
        .sheet(2_usize)
        .all_sheets()
        .head_row_number(3)
        .ignore_empty_row(false)
        .auto_trim(false)
        .use_1904_windowing(true)
        .use_scientific_format(true)
        .use_scientific_format(false)
        .use_scientific_format(true)
        .locale(ExcelLocale::from_name("de-DE").expect("German locale"))
        .start_row(4)
        .end_row(8)
        .read_rows(5, 7)
        .header_alias("Source", "Value")
        .custom_object("event-context".to_owned())
        .read_cache(ReadCacheMode::File)
        .read_default_return(ReadDefaultReturn::ActualData)
        .extra_read(CellExtraType::Comment)
        .extra_read(CellExtraType::Merge)
        .password("read-secret")
        .charset("GBK");
    assert_eq!(read.path, PathBuf::from("input.xlsx"));
    assert_eq!(read.options.sheet, SheetSelector::All);
    assert_eq!(read.options.head_row_number, 3);
    assert!(!read.options.ignore_empty_row);
    assert!(!read.options.auto_trim);
    assert!(read.options.use_1904_windowing);
    assert_eq!(
        read.options.scientific_format,
        ScientificFormatMode::Scientific
    );
    assert_eq!(read.options.locale.language_tag(), "de_DE");
    assert_eq!(read.options.start_row, Some(5));
    assert_eq!(read.options.end_row, Some(7));
    assert_eq!(
        read.options
            .header_aliases
            .get("Source")
            .map(String::as_str),
        Some("Value")
    );
    assert_eq!(
        read.options
            .custom_object
            .as_ref()
            .and_then(|value| value.downcast_ref::<String>())
            .map(String::as_str),
        Some("event-context")
    );
    assert_eq!(
        read.options.read_default_return,
        ReadDefaultReturn::ActualData
    );
    assert_eq!(read.options.read_cache, ReadCacheMode::File);
    assert!(read.options.extra_read.contains(&CellExtraType::Comment));
    assert!(read.options.extra_read.contains(&CellExtraType::Merge));
    assert_eq!(read.options.password.as_deref(), Some("read-secret"));
    assert_eq!(read.options.charset.name(), "GBK");

    let sync = EasyExcel::read_sync::<Value>("sync.xlsx")
        .sheet("Values")
        .all_sheets()
        .head_row_number(2)
        .ignore_empty_row(false)
        .auto_trim(false)
        .use_1904_windowing(true)
        .use_scientific_format(true)
        .use_scientific_format(false)
        .use_scientific_format(true)
        .locale(ExcelLocale::from_name("zh-CN").expect("Chinese locale"))
        .start_row(3)
        .end_row(9)
        .read_rows(4, 6)
        .header_alias("Original", "Value")
        .custom_object(42_u32)
        .read_cache(ReadCacheMode::Memory)
        .read_default_return(ReadDefaultReturn::ReadCellData)
        .extra_read(CellExtraType::Hyperlink)
        .password("sync-secret")
        .charset(CsvCharset::new("UTF-16BE"));
    assert_eq!(sync.path, PathBuf::from("sync.xlsx"));
    assert_eq!(sync.options.sheet, SheetSelector::All);
    assert_eq!(sync.options.head_row_number, 2);
    assert!(!sync.options.ignore_empty_row);
    assert!(!sync.options.auto_trim);
    assert!(sync.options.use_1904_windowing);
    assert_eq!(
        sync.options.scientific_format,
        ScientificFormatMode::Scientific
    );
    assert_eq!(sync.options.locale.language_tag(), "zh_CN");
    assert_eq!(sync.options.start_row, Some(4));
    assert_eq!(sync.options.end_row, Some(6));
    assert_eq!(
        sync.options
            .header_aliases
            .get("Original")
            .map(String::as_str),
        Some("Value")
    );
    assert_eq!(
        sync.options
            .custom_object
            .as_ref()
            .and_then(|value| value.downcast_ref::<u32>()),
        Some(&42)
    );
    assert_eq!(
        sync.options.read_default_return,
        ReadDefaultReturn::ReadCellData
    );
    assert_eq!(sync.options.read_cache, ReadCacheMode::Memory);
    assert!(sync.options.extra_read.contains(&CellExtraType::Hyperlink));
    assert_eq!(sync.options.password.as_deref(), Some("sync-secret"));
    assert_eq!(sync.options.charset.name(), "UTF-16BE");

    let write = EasyExcel::write::<Value>("output.xlsx")
        .sheet("Values")
        .need_head(false)
        .freeze_head(true)
        .freeze_panes(2, 1)
        .include_column_indexes([2, 0])
        .include_column_field_names(["value"])
        .exclude_column_indexes([3])
        .exclude_column_field_names(["ignored".to_owned()])
        .order_by_include_column(true)
        .merge_cells(MergeRange::new(0, 0, 0, 1))
        .auto_width()
        .column_width(0, 24)
        .head_style(CellStyle::new().italic(true))
        .content_style(CellStyle::new().bold(true))
        .content_styles([CellStyle::new().wrap_text(true)])
        .loop_merge(LoopMergeStrategy::new(2, 1, 0).unwrap())
        .head([["Group", "Value"]])
        .password("write-secret")
        .charset("GBK")
        .with_bom(false)
        .register_write_handler(NoopWriteHandler)
        .constant_memory(true)
        .compress_temp_files(true);
    assert_eq!(write.path, PathBuf::from("output.xlsx"));
    assert_eq!(write.options.sheet_name, "Values");
    assert_eq!(write.options.sheet_index, None);
    assert!(!write.options.need_head);
    assert!(write.options.freeze_head);
    assert_eq!(write.options.freeze_panes, Some((2, 1)));
    assert_eq!(write.options.include_column_indexes, Some(vec![2, 0]));
    assert_eq!(
        write.options.include_column_field_names,
        Some(vec!["value".to_owned()])
    );
    assert_eq!(write.options.exclude_column_indexes, vec![3]);
    assert_eq!(
        write.options.exclude_column_field_names,
        vec!["ignored".to_owned()]
    );
    assert!(write.options.order_by_include_column);
    assert_eq!(
        write.options.merge_ranges,
        vec![MergeRange::new(0, 0, 0, 1)]
    );
    assert!(write.options.auto_width);
    assert_eq!(write.options.column_widths, vec![(0, 24)]);
    assert!(write.options.head_style.italic);
    assert_eq!(write.options.content_styles.len(), 1);
    assert!(write.options.content_styles[0].wrap_text);
    assert_eq!(write.options.loop_merges.len(), 1);
    assert_eq!(
        write.options.dynamic_head,
        Some(vec![vec!["Group".to_owned(), "Value".to_owned()]])
    );
    assert_eq!(write.handlers.len(), 1);
    assert!(write.options.constant_memory);
    assert!(write.options.compress_temp_files);
    assert_eq!(write.options.password.as_deref(), Some("write-secret"));
    assert_eq!(write.options.charset.name(), "GBK");
    assert!(!write.options.with_bom);

    let indexed_write = EasyExcel::write::<Value>("indexed.xlsx").sheet_index(4);
    assert_eq!(indexed_write.options.sheet_index, Some(4));
    assert_eq!(indexed_write.options.sheet_name, "4");
    let indexed_sheet = EasyExcel::writer_sheet_index::<Value>(5);
    assert_eq!(indexed_sheet.options().sheet_index, Some(5));
    assert_eq!(indexed_sheet.options().sheet_name, "5");

    let dynamic = EasyExcel::read_dynamic("dynamic.xlsx", DynamicListener::default());
    assert_eq!(dynamic.path, PathBuf::from("dynamic.xlsx"));
    assert_eq!(
        dynamic.options.read_default_return,
        ReadDefaultReturn::String
    );
    let dynamic_sync = EasyExcel::read_dynamic_sync("dynamic-sync.xlsx");
    assert_eq!(dynamic_sync.path, PathBuf::from("dynamic-sync.xlsx"));
}

