#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
// 语义敏感：断言 XML 解析出的行高/列宽必须精确等于写入值（浮点往返
// 无损），严格比较即测试意图，故豁免 float_cmp。
#[allow(clippy::float_cmp)]
fn template_annotation_layout_stays_absolute_and_preserves_package() -> Result<()> {
    struct TemplateRow(&'static str, i64);

    impl ExcelRow for TemplateRow {
        fn schema() -> &'static [ExcelColumn] {
            const FIELD_STYLE: ExcelCellStyle = ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(14)),
                ..ExcelCellStyle::new()
            };
            const FIELD_FONT: ExcelFontStyle = ExcelFontStyle {
                font_height_in_points: Some(18.0),
                ..ExcelFontStyle::new()
            };
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("name", "Name", Some(0), 0, None)
                    .with_content_style(FIELD_STYLE)
                    .with_content_font_style(FIELD_FONT),
                ExcelColumn::new("count", "Count", Some(1), 0, None).with_column_width(29),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const HEAD_STYLE: ExcelCellStyle = ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(13)),
                ..ExcelCellStyle::new()
            };
            const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
                italic: Some(true),
                font_height_in_points: Some(16.0),
                ..ExcelFontStyle::new()
            };
            const CONTENT_STYLE: ExcelCellStyle = ExcelCellStyle {
                border_bottom: Some(ExcelBorderStyle::Thin),
                ..ExcelCellStyle::new()
            };
            const CONTENT_FONT: ExcelFontStyle = ExcelFontStyle {
                bold: Some(true),
                color: Some(ExcelColor::Indexed(10)),
                ..ExcelFontStyle::new()
            };
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
                .head_style(HEAD_STYLE)
                .head_font_style(HEAD_FONT)
                .content_row_height(26)
                .content_style(CONTENT_STYLE)
                .content_font_style(CONTENT_FONT)
                .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(0, 0, 0, 1));
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
    let template = directory.path().join("absolute-layout-template.xlsx");
    let output = directory.path().join("absolute-layout-output.xlsx");
    let mut workbook = Workbook::new();
    let seed_format = Format::new()
        .set_bold()
        .set_background_color(0x0000_00ff)
        .set_pattern(FormatPattern::Solid);
    workbook
        .add_worksheet()
        .set_name("Data")
        .map_err(test_error)?
        .write_string_with_format(0, 0, "seed", &seed_format)
        .map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;
    let template_sheet_xml = zip_entry(&template, "xl/worksheets/sheet1.xml")?;
    let seed_style = cell_style_id(&template_sheet_xml, "A1").expect("template seed style");

    let handler_style = ExcelCellStyle {
        vertical_alignment: Some(crate::core::ExcelVerticalAlignment::Center),
        data_format: Some(ExcelDataFormat::Custom("0.000")),
        ..ExcelCellStyle::new()
    };
    let mut handlers: Vec<Box<dyn WriteHandler>> =
        vec![Box::new(HorizontalCellStyleStrategy::new(vec![
            handler_style.into(),
        ]))];
    write_xlsx_with_handlers::<TemplateRow, _>(
        &output,
        &WriteOptions {
            sheet_name: "Data".to_owned(),
            template_file: Some(template),
            ..WriteOptions::default()
        },
        vec![TemplateRow("appended", 7)],
        &mut handlers,
    )?;

    let sheet_xml = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!(sheet_xml.contains("<mergeCell ref=\"A1:B1\"/>"));
    assert!(!sheet_xml.contains("<mergeCell ref=\"A2:B2\"/>"));
    assert_eq!(
        cell_style_id(&sheet_xml, "A1").as_deref(),
        Some(seed_style.as_str())
    );
    assert_eq!(sheet_column_width(&sheet_xml, 2)?, 29.0);
    assert!((sheet_row_height(&sheet_xml, 3)? - 26.0).abs() < f64::EPSILON);
    assert!(sheet_xml.contains("appended"));
    let head_style = cell_style_id(&sheet_xml, "A2").expect("head annotation style");
    let first_style = cell_style_id(&sheet_xml, "A3").expect("field annotation style");
    let second_style = cell_style_id(&sheet_xml, "B3").expect("type annotation style");
    assert_ne!(head_style, first_style);
    assert_ne!(first_style, second_style);
    let styles_xml = zip_entry(&output, "xl/styles.xml")?;
    assert!(styles_xml.contains("rgb=\"FFFF00FF\""));
    assert!(styles_xml.contains("rgb=\"FFFF0000\""));
    assert!(styles_xml.contains("rgb=\"FFFFFF00\""));
    assert!(styles_xml.contains("<sz val=\"16\"/>"));
    assert!(styles_xml.contains("<sz val=\"18\"/>"));
    assert!(styles_xml.contains("<i/>"));
    assert!(styles_xml.contains("<b/>"));
    assert!(styles_xml.contains("style=\"thin\""));
    assert!(styles_xml.contains("formatCode=\"0.000\""));
    assert!(styles_xml.contains("vertical=\"center\""));
    assert!(styles_xml.contains("rgb=\"FF0000FF\""));
    let mut workbook: Xlsx<_> = open_workbook(&output).map_err(test_error)?;
    let range = workbook.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.get((0, 0)), Some(&Data::String("seed".to_owned())));
    assert_eq!(range.get((1, 0)), Some(&Data::String("Name".to_owned())));
    assert_eq!(
        range.get((2, 0)),
        Some(&Data::String("appended".to_owned()))
    );
    Ok(())
}

#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn template_zip_path_runs_row_cell_lifecycle_and_applies_mutation_and_skip() -> Result<()> {
    struct TemplateRow;

    impl ExcelRow for TemplateRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("name", "Name", Some(0), 0, None),
                ExcelColumn::new("count", "Count", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const STYLE: ExcelCellStyle = ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(10)),
                ..ExcelCellStyle::new()
            };
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new().content_style(STYLE);
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![
                CellValue::String("original".to_owned()),
                CellValue::Int(7),
            ])
        }
    }

    struct MutatingHandler {
        events: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl WriteHandler for MutatingHandler {
        fn before_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("row-before:{}", context.row_index));
            Ok(())
        }

        fn after_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("row-created:{}", context.row_index));
            Ok(())
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            if context.column_index == 0 {
                context.value = CellValue::String("mutated".to_owned());
                context.ignore_fill_style = true;
            } else {
                context.skip = true;
            }
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("cell-before:{}", context.column_index));
            Ok(())
        }

        fn after_cell_create(&mut self, context: &WriteCellContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("cell-created:{}", context.column_index));
            Ok(())
        }

        fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("cell-converted:{}", context.column_index));
            Ok(())
        }

        fn after_cell_dispose(&mut self, context: &WriteCellContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("cell-disposed:{}", context.column_index));
            Ok(())
        }

        fn after_row_dispose(&mut self, context: &WriteRowContext) -> Result<()> {
            self.events
                .lock()
                .map_err(|_| test_error("events poisoned"))?
                .push(format!("row-disposed:{}", context.row_index));
            Ok(())
        }
    }

    let directory = tempdir()?;
    let template = directory.path().join("handler-template.xlsx");
    let output = directory.path().join("handler-output.xlsx");
    let mut workbook = Workbook::new();
    workbook
        .add_worksheet()
        .set_name("Data")
        .map_err(test_error)?
        .write_string(0, 0, "seed")
        .map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(MutatingHandler {
        events: Arc::clone(&events),
    })];
    write_xlsx_with_handlers::<TemplateRow, _>(
        &output,
        &WriteOptions {
            sheet_name: "Data".to_owned(),
            need_head: false,
            template_file: Some(template),
            ..WriteOptions::default()
        },
        vec![TemplateRow],
        &mut handlers,
    )?;

    let sheet = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!(sheet.contains("mutated"), "sheet XML: {sheet}");
    assert!(!sheet.contains("original"));
    assert!(!sheet.contains("<c r=\"B2\""));
    assert!(cell_style_id(&sheet, "A2").is_none());
    assert_eq!(
        events
            .lock()
            .map_err(|_| test_error("events poisoned"))?
            .as_slice(),
        [
            "row-before:1",
            "row-created:1",
            "cell-before:0",
            "cell-created:0",
            "cell-converted:0",
            "cell-disposed:0",
            "cell-before:1",
            "cell-created:1",
            "cell-converted:1",
            "cell-disposed:1",
            "row-disposed:1",
        ]
    );
    Ok(())
}

#[test]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn handler_context_matches_java_conversion_stages_and_ignore_fill_style() -> Result<()> {
    struct ContextRow;

    impl ExcelRow for ContextRow {
        fn schema() -> &'static [ExcelColumn] {
            const STYLE: ExcelCellStyle = ExcelCellStyle {
                fill_pattern: Some(ExcelFillPattern::Solid),
                fill_foreground_color: Some(ExcelColor::Indexed(10)),
                ..ExcelCellStyle::new()
            };
            const COLUMNS: &[ExcelColumn] =
                &[ExcelColumn::new("name", "Name", Some(0), 0, None).with_content_style(STYLE)];
            COLUMNS
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String("value".to_owned())])
        }
    }

    #[derive(Default)]
    struct ObservedContext {
        rows: Vec<(bool, Option<usize>)>,
        before_content_original: Option<CellValue>,
        before_content_data_len: usize,
        after_create_content_data_len: usize,
        head_converted_calls: usize,
        content_converted_calls: usize,
        converted_original: Option<CellValue>,
        converted_first: Option<CellValue>,
        converted_target: Option<crate::core::CellDataType>,
        head_disposed_first: Option<CellValue>,
    }

    struct ContextProbe {
        observed: Arc<std::sync::Mutex<ObservedContext>>,
    }

    impl WriteHandler for ContextProbe {
        fn before_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            self.observed
                .lock()
                .map_err(|_| test_error("context poisoned"))?
                .rows
                .push((context.is_head, context.relative_row_index));
            Ok(())
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            if !context.is_head {
                let mut observed = self
                    .observed
                    .lock()
                    .map_err(|_| test_error("context poisoned"))?;
                observed.before_content_original = context.original_value.clone();
                observed.before_content_data_len = context.cell_data_list.len();
                context.ignore_fill_style = true;
            }
            Ok(())
        }

        fn after_cell_create(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                self.observed
                    .lock()
                    .map_err(|_| test_error("context poisoned"))?
                    .after_create_content_data_len = context.cell_data_list.len();
            }
            Ok(())
        }

        fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
            let mut observed = self
                .observed
                .lock()
                .map_err(|_| test_error("context poisoned"))?;
            if context.is_head {
                observed.head_converted_calls += 1;
            } else {
                observed.content_converted_calls += 1;
                observed.converted_original = context.original_value.clone();
                observed.converted_first = context.first_cell_data().cloned();
                observed.converted_target = context.target_cell_data_type;
            }
            Ok(())
        }

        fn after_cell_dispose(&mut self, context: &WriteCellContext) -> Result<()> {
            if context.is_head {
                self.observed
                    .lock()
                    .map_err(|_| test_error("context poisoned"))?
                    .head_disposed_first = context.first_cell_data().cloned();
            }
            Ok(())
        }
    }

    let directory = tempdir()?;
    let output = directory.path().join("handler-context.xlsx");
    let observed = Arc::new(std::sync::Mutex::new(ObservedContext::default()));
    let mut handlers: Vec<Box<dyn WriteHandler>> = vec![Box::new(ContextProbe {
        observed: Arc::clone(&observed),
    })];
    write_xlsx_with_handlers::<ContextRow, _>(
        &output,
        &WriteOptions::default(),
        vec![ContextRow],
        &mut handlers,
    )?;

    let observed = observed
        .lock()
        .map_err(|_| test_error("context poisoned"))?;
    assert_eq!(observed.rows, [(true, Some(0)), (false, Some(0))]);
    assert_eq!(observed.before_content_original, None);
    assert_eq!(observed.before_content_data_len, 0);
    assert_eq!(observed.after_create_content_data_len, 0);
    assert_eq!(observed.head_converted_calls, 0);
    assert_eq!(observed.content_converted_calls, 1);
    assert_eq!(
        observed.converted_original,
        Some(CellValue::String("value".to_owned()))
    );
    assert_eq!(
        observed.converted_first,
        Some(CellValue::String("value".to_owned()))
    );
    assert_eq!(
        observed.converted_target,
        Some(crate::core::CellDataType::String)
    );
    assert_eq!(
        observed.head_disposed_first,
        Some(CellValue::String("Name".to_owned()))
    );
    drop(observed);

    let sheet = zip_entry(&output, "xl/worksheets/sheet1.xml")?;
    assert!(
        cell_style_id(&sheet, "A2").is_none(),
        "ignoreFillStyle must suppress annotation style: {sheet}"
    );
    Ok(())
}

