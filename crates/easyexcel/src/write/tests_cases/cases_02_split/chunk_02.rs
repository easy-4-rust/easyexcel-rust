#[test]
fn holder_handler_scope_deduplicates_effective_chain_but_runs_each_own_chain() -> Result<()> {
    struct UniqueProbe {
        scope: &'static str,
        events: Arc<Mutex<Vec<String>>>,
    }

    impl UniqueProbe {
        fn push(&self, event: &str) {
            self.events
                .lock()
                .expect("handler event mutex poisoned")
                .push(format!("{}:{event}", self.scope));
        }
    }

    impl NotRepeatExecutor for UniqueProbe {
        fn unique_value(&self) -> &'static str {
            "same-holder-handler"
        }
    }

    impl WriteHandler for UniqueProbe {
        fn as_not_repeat_executor(&self) -> Option<&dyn NotRepeatExecutor> {
            Some(self)
        }

        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.push("workbook");
            Ok(())
        }

        fn before_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            self.push("sheet");
            Ok(())
        }

        fn before_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
            self.push("row");
            Ok(())
        }

        fn after_workbook_dispose(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.push("dispose");
            Ok(())
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("holder-handler-dedup.xlsx");
    let events = Arc::new(Mutex::new(Vec::new()));
    let probe = |scope| {
        Box::new(UniqueProbe {
            scope,
            events: Arc::clone(&events),
        }) as Box<dyn WriteHandler>
    };
    let mut writer = ExcelWriter::with_handlers(&output, vec![probe("workbook")]);
    let sheet = WriteSheet::<EveryCell>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        need_head: false,
        ..WriteOptions::default()
    });
    let mut table = MirroredWriteTable::new();
    table.table_no = 0;

    writer.write_with_table_handlers(
        vec![every_cell()],
        &sheet,
        &table,
        vec![probe("sheet")],
        vec![probe("table")],
    )?;
    writer.finish()?;

    assert_eq!(
        *events.lock().expect("handler event mutex poisoned"),
        vec![
            "workbook:workbook",
            "sheet:workbook",
            "sheet:sheet",
            "table:workbook",
            "table:sheet",
            "table:row",
            "table:dispose",
        ]
    );
    Ok(())
}

#[test]
fn handler_registered_before_empty_finish_participates_in_dispose_chain() -> Result<()> {
    #[derive(Default)]
    struct Counts {
        create: AtomicUsize,
        dispose: AtomicUsize,
    }

    struct WorkbookProbe(Arc<Counts>);

    impl WriteHandler for WorkbookProbe {
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.0.create.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn after_workbook_dispose(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.0.dispose.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("empty-finish-handler.xlsx");
    let counts = Arc::new(Counts::default());
    let mut writer = ExcelWriter::new(&output);
    writer.register_write_handler(Box::new(WorkbookProbe(Arc::clone(&counts))))?;
    writer.finish()?;

    assert_eq!(counts.create.load(Ordering::SeqCst), 1);
    assert_eq!(counts.dispose.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
// 语义敏感：断言 XML 解析出的行高/列宽必须精确等于写入值（浮点往返
// 无损），严格比较即测试意图，故豁免 float_cmp。
#[allow(clippy::float_cmp)]
fn multiple_tables_keep_independent_schema_options_and_single_head() -> Result<()> {
    struct FirstTableRow(&'static str);

    impl ExcelRow for FirstTableRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("first", "First", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self(""))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(self.0.to_owned())])
        }
    }

    struct SecondTableRow(&'static str, i64);

    impl ExcelRow for SecondTableRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("second", "Second", Some(0), 0, None),
                ExcelColumn::new("count", "Count", Some(1), 0, None).with_column_width(31),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
                .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(5, 5, 0, 1));
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self("", 0))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String(self.0.to_owned()),
                CellValue::Int(self.1),
            ])
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("multiple-table-holder-heads.xlsx");
    let first_sheet = WriteSheet::<FirstTableRow>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        need_head: false,
        ..WriteOptions::default()
    });
    let second_sheet = WriteSheet::<SecondTableRow>::from_options(WriteOptions {
        sheet_name: "Data".to_owned(),
        need_head: false,
        ..WriteOptions::default()
    });
    let mut first_table = MirroredWriteTable::with_table_no(0);
    first_table.parameter.need_head = Some(true);
    let mut second_table = MirroredWriteTable::with_table_no(1);
    second_table.parameter.need_head = Some(true);

    let mut writer = ExcelWriter::new(&output);
    writer.write_with_table(vec![FirstTableRow("alpha")], &first_sheet, &first_table)?;
    writer.write_with_table(vec![FirstTableRow("beta")], &first_sheet, &first_table)?;
    writer.write_with_table(
        vec![SecondTableRow("gamma", 3)],
        &second_sheet,
        &second_table,
    )?;
    writer.finish()?;

    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.get((0, 0)), Some(&Data::String("First".to_owned())));
    assert_eq!(range.get((1, 0)), Some(&Data::String("alpha".to_owned())));
    assert_eq!(range.get((2, 0)), Some(&Data::String("beta".to_owned())));
    assert_eq!(range.get((3, 0)), Some(&Data::String("Second".to_owned())));
    assert_eq!(range.get((3, 1)), Some(&Data::String("Count".to_owned())));
    assert_eq!(range.get((4, 0)), Some(&Data::String("gamma".to_owned())));
    assert_eq!(range.get((4, 1)), Some(&Data::Float(3.0)));
    let sheet_xml = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert_eq!(sheet_column_width(&sheet_xml, 2)?, 31.0);
    assert!(sheet_xml.contains("<mergeCell ref=\"A6:B6\"/>"));
    Ok(())
}

