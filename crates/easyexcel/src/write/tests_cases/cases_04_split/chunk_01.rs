#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn handler_context_exposes_real_pre_converter_value_across_write_backends() -> Result<()> {
    struct ConvertedContextRow(i64);

    impl ExcelRow for ConvertedContextRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] =
                &[ExcelColumn::new("amount", "Amount", Some(0), 0, None).with_field_type("i64")];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self(0))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::Int(self.0)])
        }

        fn to_row_with_converters(
            &self,
            _converters: &crate::core::ConverterRegistry,
        ) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(format!("converted:{}", self.0))])
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Snapshot {
        original: Option<CellValue>,
        field_type: Option<&'static str>,
        converted: CellValue,
        first_data: Option<CellValue>,
        target_type: Option<crate::core::CellDataType>,
        workbook_file: Option<String>,
        sheet_name: String,
        sheet_no: Option<i32>,
        last_row_index: Option<u32>,
        table_no: Option<i32>,
    }

    struct ConversionProbe {
        snapshots: Arc<std::sync::Mutex<Vec<Snapshot>>>,
    }

    impl WriteHandler for ConversionProbe {
        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            if context.is_head {
                assert_eq!(context.original_value, None);
                assert_eq!(context.original_field_type, None);
            } else {
                self.snapshots
                    .lock()
                    .map_err(|_| test_error("snapshot poisoned"))?
                    .push(Snapshot {
                        original: context.original_value.clone(),
                        field_type: context.original_field_type,
                        converted: context.value.clone(),
                        first_data: context.first_cell_data().cloned(),
                        target_type: context.target_cell_data_type,
                        workbook_file: context
                            .write_workbook_holder()
                            .and_then(|holder| holder.path().file_name())
                            .and_then(std::ffi::OsStr::to_str)
                            .map(str::to_owned),
                        sheet_name: context.write_sheet_holder().sheet_name().to_owned(),
                        sheet_no: context.write_sheet_holder().sheet_no(),
                        last_row_index: context.write_sheet_holder().last_row_index(),
                        table_no: context
                            .write_table_holder()
                            .map(crate::core::WriteTableHolderView::table_no),
                    });
            }
            Ok(())
        }

        fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                self.snapshots
                    .lock()
                    .map_err(|_| test_error("snapshot poisoned"))?
                    .push(Snapshot {
                        original: context.original_value.clone(),
                        field_type: context.original_field_type,
                        converted: context.value.clone(),
                        first_data: context.first_cell_data().cloned(),
                        target_type: context.target_cell_data_type,
                        workbook_file: context
                            .write_workbook_holder()
                            .and_then(|holder| holder.path().file_name())
                            .and_then(std::ffi::OsStr::to_str)
                            .map(str::to_owned),
                        sheet_name: context.write_sheet_holder().sheet_name().to_owned(),
                        sheet_no: context.write_sheet_holder().sheet_no(),
                        last_row_index: context.write_sheet_holder().last_row_index(),
                        table_no: context
                            .write_table_holder()
                            .map(crate::core::WriteTableHolderView::table_no),
                    });
            }
            Ok(())
        }
    }

    fn assert_snapshots(
        snapshots: &Arc<std::sync::Mutex<Vec<Snapshot>>>,
        expected_file: &str,
        expected_row: u32,
    ) -> Result<()> {
        let snapshots = snapshots
            .lock()
            .map_err(|_| test_error("snapshot poisoned"))?;
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].original, None);
        assert_eq!(snapshots[0].field_type, None);
        assert_eq!(
            snapshots[0].converted,
            CellValue::String("converted:42".to_owned())
        );
        assert_eq!(snapshots[0].first_data, None);
        assert_eq!(snapshots[0].target_type, None);
        for snapshot in snapshots.iter() {
            assert_eq!(snapshot.workbook_file.as_deref(), Some(expected_file));
            assert_eq!(snapshot.sheet_name, "Sheet1");
            assert_eq!(snapshot.sheet_no, Some(0));
            assert_eq!(snapshot.last_row_index, Some(expected_row));
            assert_eq!(snapshot.table_no, None);
        }
        assert_eq!(snapshots[1].original, Some(CellValue::Int(42)));
        assert_eq!(snapshots[1].field_type, Some("i64"));
        assert_eq!(
            snapshots[1].first_data,
            Some(CellValue::String("converted:42".to_owned()))
        );
        assert_eq!(
            snapshots[1].target_type,
            Some(crate::core::CellDataType::String)
        );
        Ok(())
    }

    let directory = tempdir()?;
    for extension in ["xlsx", "xls", "csv"] {
        let output = directory.path().join(format!("converted.{extension}"));
        let snapshots = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(ConversionProbe {
            snapshots: Arc::clone(&snapshots),
        })];
        match extension {
            "xlsx" => write_xlsx_with_handlers::<ConvertedContextRow, _>(
                &output,
                &WriteOptions::default(),
                [ConvertedContextRow(42)],
                &mut handlers,
            )?,
            "xls" => write_xls_with_handlers::<ConvertedContextRow, _>(
                &output,
                &WriteOptions::default(),
                [ConvertedContextRow(42)],
                &mut handlers,
            )?,
            "csv" => write_csv_with_handlers::<ConvertedContextRow, _>(
                &output,
                &WriteOptions::default(),
                [ConvertedContextRow(42)],
                &mut handlers,
            )?,
            _ => unreachable!(),
        }
        assert_snapshots(&snapshots, &format!("converted.{extension}"), 1)?;
    }

    let template = directory.path().join("source-template.xlsx");
    write_xlsx::<ConvertedContextRow, _>(&template, &WriteOptions::default(), std::iter::empty())?;
    let output = directory.path().join("converted-template.xlsx");
    let snapshots = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(ConversionProbe {
        snapshots: Arc::clone(&snapshots),
    })];
    write_xlsx_with_handlers::<ConvertedContextRow, _>(
        &output,
        &WriteOptions {
            template_file: Some(template),
            ..WriteOptions::default()
        },
        [ConvertedContextRow(42)],
        &mut handlers,
    )?;
    assert_snapshots(&snapshots, "converted-template.xlsx", 2)
}

#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn logical_row_and_cell_handles_commit_real_backend_mutations() -> Result<()> {
    struct HandleRow;

    impl ExcelRow for HandleRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String("source".to_owned())])
        }
    }

    struct MutationProbe;

    impl WriteHandler for MutationProbe {
        fn after_row_dispose(&mut self, context: &WriteRowContext) -> Result<()> {
            if !context.is_head {
                context.row().set_height(31);
            }
            Ok(())
        }

        fn after_cell_dispose(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                context
                    .cell()
                    .set_value(CellValue::String("mutated".to_owned()));
                context.cell().set_style(ExcelCellStyle {
                    fill_pattern: Some(ExcelFillPattern::Solid),
                    fill_foreground_color: Some(ExcelColor::Rgb(0x0044_72C4)),
                    ..ExcelCellStyle::new()
                });
            }
            Ok(())
        }
    }

    struct MutationObserver;

    impl WriteHandler for MutationObserver {
        fn order(&self) -> i32 {
            1
        }

        fn after_cell_dispose(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                assert_eq!(
                    context.cell().value(),
                    CellValue::String("mutated".to_owned())
                );
            }
            Ok(())
        }
    }

    let directory = tempdir()?;
    for extension in ["xlsx", "xls", "csv"] {
        let output = directory.path().join(format!("handle.{extension}"));
        let mut handlers: Vec<Box<dyn WriteHandler>> =
            vec![Box::new(MutationProbe), Box::new(MutationObserver)];
        match extension {
            "xlsx" => write_xlsx_with_handlers::<HandleRow, _>(
                &output,
                &WriteOptions::default(),
                [HandleRow],
                &mut handlers,
            )?,
            "xls" => write_xls_with_handlers::<HandleRow, _>(
                &output,
                &WriteOptions::default(),
                [HandleRow],
                &mut handlers,
            )?,
            "csv" => write_csv_with_handlers::<HandleRow, _>(
                &output,
                &WriteOptions::default(),
                [HandleRow],
                &mut handlers,
            )?,
            _ => unreachable!(),
        }
        if extension == "csv" {
            assert!(std::fs::read_to_string(&output)?.contains("mutated"));
        } else if extension == "xls" {
            let mut workbook: Xls<_> = open_workbook(&output).map_err(test_error)?;
            let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
            assert_eq!(
                range.get_value((1, 0)),
                Some(&Data::String("mutated".to_owned()))
            );
        } else {
            let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
            let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
            assert_eq!(
                range.get_value((1, 0)),
                Some(&Data::String("mutated".to_owned()))
            );
            let sheet = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
            assert!((sheet_row_height(&sheet, 2)? - 31.0).abs() <= 0.25);
            assert!(cell_style_id(&sheet, "A2").is_some());
        }
    }

    let template = directory.path().join("handle-template-source.xlsx");
    write_xlsx::<HandleRow, _>(&template, &WriteOptions::default(), std::iter::empty())?;
    let output = directory.path().join("handle-template.xlsx");
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(MutationProbe), Box::new(MutationObserver)];
    write_xlsx_with_handlers::<HandleRow, _>(
        &output,
        &WriteOptions {
            template_file: Some(template),
            ..WriteOptions::default()
        },
        [HandleRow],
        &mut handlers,
    )?;
    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Sheet1").map_err(test_error)?;
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("mutated".to_owned()))
    );
    let sheet = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!((sheet_row_height(&sheet, 3)? - 31.0).abs() <= 0.25);
    assert!(cell_style_id(&sheet, "A3").is_some());
    Ok(())
}

