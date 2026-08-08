#[test]
fn default_options_and_helpers_are_deterministic() {
    assert_eq!(
        WriteOptions::default(),
        WriteOptions {
            excel_type: None,
            sheet_name: "Sheet1".to_owned(),
            sheet_index: None,
            auto_trim: true,
            use_1904_windowing: false,
            locale: "default".to_owned(),
            use_scientific_format: false,
            filed_cache_location: crate::core::CacheLocation::ThreadLocal,
            constant_memory: false,
            compress_temp_files: false,
            need_head: true,
            use_default_style: true,
            freeze_head: false,
            freeze_panes: None,
            include_column_indexes: None,
            include_column_field_names: None,
            exclude_column_indexes: Vec::new(),
            exclude_column_field_names: Vec::new(),
            order_by_include_column: false,
            relative_head_row_index: 0,
            automatic_merge_head: true,
            merge_ranges: Vec::new(),
            auto_width: false,
            column_widths: Vec::new(),
            head_style: CellStyle::new().bold(true),
            content_styles: Vec::new(),
            loop_merges: Vec::new(),
            dynamic_head: None,
            password: None,
            biff8_macro_policy: crate::Biff8MacroPolicy::Preserve,
            charset: CsvCharset::default(),
            with_bom: true,
            auto_close_stream: true,
            write_excel_on_exception: false,
            converters: ConverterRegistry::default(),
            template_file: None,
            template_bytes: None,
            use_legacy_template_seed: false,
        }
    );
    assert_eq!(to_column(0).expect("column"), 0);
    assert_eq!(to_column(usize::from(u16::MAX)).expect("column"), u16::MAX);
    assert!(to_column(usize::from(u16::MAX) + 1).is_err());
    assert_eq!(
        format_error("broken").to_string(),
        "excel format error: broken"
    );
    assert!(MirroredLoopMergeStrategy::new(0, 1, 0).is_err());
    assert!(MirroredLoopMergeStrategy::new(1, 0, 0).is_err());
    assert!(MirroredLoopMergeStrategy::new(1, 1, 0).is_err());
    let strategy = MirroredLoopMergeStrategy::new(2, 3, 4).expect("valid loop merge");
    assert_eq!(strategy.each_rows(), 2);
    assert_eq!(strategy.column_extend(), 3);
    assert_eq!(strategy.column_index(), 4);
    let dynamic = WriteSheet::<EveryCell>::new("Dynamic").head([["User", "Name"], ["User", "Age"]]);
    assert_eq!(
        dynamic.options().dynamic_head,
        Some(vec![
            vec!["User".to_owned(), "Name".to_owned()],
            vec!["User".to_owned(), "Age".to_owned()],
        ])
    );
    let indexed = WriteSheet::<EveryCell>::new_index(5);
    assert_eq!(indexed.options().sheet_index, Some(5));
    assert_eq!(indexed.options().sheet_name, "5");
    let indexed_name = WriteSheet::<EveryCell>::new("Named").sheet_index(6);
    assert_eq!(indexed_name.options().sheet_index, Some(6));
    assert_eq!(indexed_name.options().sheet_name, "Named");
}

#[test]
fn stateful_writer_installs_java_default_handlers_by_effective_type() {
    let xlsx = ExcelWriter::with_handlers_and_options(
        "default-handlers.xlsx",
        Vec::new(),
        WriteOptions::default(),
    );
    assert_eq!(xlsx.workbook_handlers.len(), 4);

    let xls = ExcelWriter::with_handlers_and_options(
        "default-handlers.xls",
        Vec::new(),
        WriteOptions {
            use_default_style: false,
            ..WriteOptions::default()
        },
    );
    assert_eq!(xls.workbook_handlers.len(), 2);

    let csv = ExcelWriter::with_handlers_and_options(
        "default-handlers.csv",
        Vec::new(),
        WriteOptions::default(),
    );
    assert_eq!(csv.workbook_handlers.len(), 2);
}

#[test]
fn stateful_sheet_handlers_are_isolated_and_reused_by_holder() -> Result<()> {
    #[derive(Default)]
    struct Counts {
        workbook: AtomicUsize,
        sheet: AtomicUsize,
        row: AtomicUsize,
    }

    struct ScopeProbe(Arc<Counts>);

    impl WriteHandler for ScopeProbe {
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.0.workbook.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn before_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            self.0.sheet.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn before_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
            self.0.row.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("sheet-handler-isolation.xlsx");
    let workbook_counts = Arc::new(Counts::default());
    let first_counts = Arc::new(Counts::default());
    let second_counts = Arc::new(Counts::default());
    let mut writer = ExcelWriter::with_handlers(
        &output,
        vec![Box::new(ScopeProbe(Arc::clone(&workbook_counts)))],
    );
    let first = WriteSheet::<EveryCell>::from_options(WriteOptions {
        sheet_name: "First".to_owned(),
        need_head: false,
        ..WriteOptions::default()
    });
    let second = WriteSheet::<EveryCell>::from_options(WriteOptions {
        sheet_name: "Second".to_owned(),
        need_head: false,
        ..WriteOptions::default()
    });

    writer.write_with_sheet_handlers(
        vec![every_cell()],
        &first,
        vec![Box::new(ScopeProbe(Arc::clone(&first_counts)))],
    )?;
    writer.write_with_sheet_handlers(
        vec![every_cell()],
        &second,
        vec![Box::new(ScopeProbe(Arc::clone(&second_counts)))],
    )?;
    writer.write(vec![every_cell()], &first)?;
    writer.finish()?;

    assert_eq!(workbook_counts.workbook.load(Ordering::SeqCst), 1);
    assert_eq!(workbook_counts.sheet.load(Ordering::SeqCst), 2);
    assert_eq!(workbook_counts.row.load(Ordering::SeqCst), 3);
    assert_eq!(first_counts.workbook.load(Ordering::SeqCst), 1);
    assert_eq!(first_counts.sheet.load(Ordering::SeqCst), 1);
    assert_eq!(first_counts.row.load(Ordering::SeqCst), 2);
    assert_eq!(second_counts.workbook.load(Ordering::SeqCst), 1);
    assert_eq!(second_counts.sheet.load(Ordering::SeqCst), 1);
    assert_eq!(second_counts.row.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn table_holder_runs_supplementary_callbacks_then_own_parent_row_chain() -> Result<()> {
    struct LogProbe {
        scope: &'static str,
        events: Arc<Mutex<Vec<String>>>,
    }

    impl LogProbe {
        fn push(&self, event: &str) {
            self.events
                .lock()
                .expect("handler event mutex poisoned")
                .push(format!("{}:{event}", self.scope));
        }
    }

    impl WriteHandler for LogProbe {
        fn before_workbook_create(&mut self, context: &WriteWorkbookContext) -> Result<()> {
            assert_eq!(
                context
                    .write_workbook_holder()
                    .path()
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str),
                Some("table-handler-holder-order.xlsx")
            );
            let current_holder = context.write_context().current_write_holder();
            assert_eq!(current_holder.holder_type(), Holder::Workbook);
            assert_eq!(current_holder.path(), context.path());
            self.push("workbook");
            Ok(())
        }

        fn before_sheet_create(&mut self, context: &WriteSheetContext) -> Result<()> {
            assert_eq!(
                context
                    .write_workbook_holder()
                    .and_then(|holder| holder.path().file_name())
                    .and_then(std::ffi::OsStr::to_str),
                Some("table-handler-holder-order.xlsx")
            );
            assert_eq!(context.write_sheet_holder().sheet_name(), "Data");
            assert_eq!(context.write_sheet_holder().sheet_no(), Some(0));
            assert!(context.write_table_holder().is_none());
            self.push("sheet");
            Ok(())
        }

        fn before_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            assert_eq!(
                context
                    .write_workbook_holder()
                    .and_then(|holder| holder.path().file_name())
                    .and_then(std::ffi::OsStr::to_str),
                Some("table-handler-holder-order.xlsx")
            );
            assert_eq!(context.write_sheet_holder().sheet_name(), "Data");
            assert_eq!(context.write_sheet_holder().sheet_no(), Some(0));
            assert_eq!(
                context
                    .write_table_holder()
                    .map(crate::core::WriteTableHolderView::table_no),
                Some(0)
            );
            let current_holder = context.write_context().current_write_holder();
            assert_eq!(current_holder.holder_type(), Holder::Table);
            assert!(!current_holder.need_head());
            assert!(!current_holder.automatic_merge_head());
            assert!(current_holder.order_by_include_column());
            assert_eq!(current_holder.include_column_indexes(), Some(&[1, 0][..]));
            assert_eq!(
                current_holder.exclude_column_field_names(),
                &["decimal".to_owned()]
            );
            let head_map = current_holder.excel_write_head_property().head_map();
            assert_eq!(head_map.len(), 2);
            assert_eq!(
                head_map.get(&0).and_then(|head| head.field_name()),
                Some("string")
            );
            assert_eq!(
                head_map.get(&1).and_then(|head| head.field_name()),
                Some("empty")
            );
            self.push("row");
            Ok(())
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            assert_eq!(
                context
                    .write_workbook_holder()
                    .and_then(|holder| holder.path().file_name())
                    .and_then(std::ffi::OsStr::to_str),
                Some("table-handler-holder-order.xlsx")
            );
            assert_eq!(context.write_sheet_holder().last_row_index(), Some(0));
            assert_eq!(
                context
                    .write_table_holder()
                    .map(crate::core::WriteTableHolderView::table_no),
                Some(0)
            );
            let current_holder = context.write_context().current_write_holder();
            assert_eq!(current_holder.holder_type(), Holder::Table);
            assert_eq!(
                current_holder.excel_write_head_property().head_map().len(),
                2
            );
            Ok(())
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("table-handler-holder-order.xlsx");
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut writer = ExcelWriter::with_handlers(
        &output,
        vec![Box::new(LogProbe {
            scope: "workbook",
            events: Arc::clone(&events),
        })],
    );
    let sheet = WriteSheet::<EveryCell>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        need_head: false,
        automatic_merge_head: false,
        order_by_include_column: true,
        include_column_indexes: Some(vec![1, 0]),
        exclude_column_field_names: vec!["decimal".to_owned()],
        ..WriteOptions::default()
    });
    let mut table = MirroredWriteTable::new();
    table.table_no = 0;

    writer.write_with_table_handlers(
        vec![every_cell()],
        &sheet,
        &table,
        vec![Box::new(LogProbe {
            scope: "sheet",
            events: Arc::clone(&events),
        })],
        vec![Box::new(LogProbe {
            scope: "table",
            events: Arc::clone(&events),
        })],
    )?;
    writer.finish()?;

    assert_eq!(
        *events.lock().expect("handler event mutex poisoned"),
        vec![
            "workbook:workbook",
            "sheet:workbook",
            "sheet:sheet",
            "workbook:sheet",
            "table:workbook",
            "table:sheet",
            "table:row",
            "sheet:row",
            "workbook:row",
        ]
    );
    Ok(())
}
