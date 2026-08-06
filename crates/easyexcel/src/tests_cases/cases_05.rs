/// Java `EasyExcel.write(...).withTemplate(...).build()` + `write(data, sheet)` + `finish()`.
#[test]
fn with_template_stateful_writer_appends_on_named_sheet() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("stateful-template.xlsx");
    let output = directory.path().join("stateful-out.xlsx");

    let mut seed = EasyExcel::write::<Value>(&template).build();
    seed.write(
        [Value("alpha".to_owned())],
        &EasyExcel::writer_sheet::<Value>("Data").need_head(false),
    )?;
    seed.write(
        [Value("beta".to_owned())],
        &EasyExcel::writer_sheet::<Value>("Other").need_head(false),
    )?;
    seed.finish()?;

    let mut writer = EasyExcel::write::<Value>(&output)
        .with_template(&template)
        .build();
    writer.write(
        [Value("gamma".to_owned())],
        &EasyExcel::writer_sheet::<Value>("Data").need_head(false),
    )?;
    writer.finish()?;

    let data = EasyExcel::read_sync::<Value>(&output)
        .sheet("Data")
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(
        data,
        vec![Value("alpha".to_owned()), Value("gamma".to_owned())]
    );
    let other = EasyExcel::read_sync::<Value>(&output)
        .sheet("Other")
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(other, vec![Value("beta".to_owned())]);
    Ok(())
}

/// Two-column row used to seed merge + style templates for `with_template` asserts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PairRow {
    left: String,
    right: String,
}

impl ExcelRow for PairRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("left", "left", Some(0), 0, None),
            ExcelColumn::new("right", "right", Some(1), 0, None),
        ];
        COLUMNS
    }

    fn from_row(row: &RowData) -> Result<Self> {
        Ok(Self {
            left: row
                .cell(&Self::schema()[0])
                .map_or_else(String::new, CellValue::as_text),
            right: row
                .cell(&Self::schema()[1])
                .map_or_else(String::new, CellValue::as_text),
        })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![
            CellValue::String(self.left.clone()),
            CellValue::String(self.right.clone()),
        ])
    }
}

/// Default ZIP path keeps template `styles.xml` / `mergeCells` after `do_write` append.
///
/// Java: `ExcelWriterBuilder.withTemplate` + append — POI keeps styles/merges on the
/// workbook; Rust mirrors that via `TemplatePackage` (not the legacy value-replay seed).
#[test]
fn with_template_do_write_preserves_styles_and_merges() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("styled-template.xlsx");
    let output = directory.path().join("styled-out.xlsx");

    EasyExcel::write::<PairRow>(&template)
        .merge_cells(MergeRange::new(0, 0, 0, 1))
        .head_style(CellStyle::new().bold(true).italic(true))
        .do_write([PairRow {
            left: "seed-l".to_owned(),
            right: "seed-r".to_owned(),
        }])?;

    let styles_before = zip_entry_text(&template, "xl/styles.xml")?;
    let sheet_before = zip_entry_text(&template, "xl/worksheets/sheet1.xml")?;
    assert!(
        styles_before.contains("<b")
            || styles_before.contains("<b/>")
            || styles_before.contains("<b "),
        "template must carry bold style marker: {styles_before}"
    );
    assert!(
        sheet_before.contains("mergeCell") || sheet_before.contains("mergeCells"),
        "template must carry mergeCells: {sheet_before}"
    );

    EasyExcel::write::<Value>(&output)
        .with_template(&template)
        .need_head(false)
        .do_write([Value("appended".to_owned())])?;

    let styles_after = zip_entry_text(&output, "xl/styles.xml")?;
    let sheet_after = zip_entry_text(&output, "xl/worksheets/sheet1.xml")?;
    assert_eq!(
        styles_before, styles_after,
        "ZIP preserve path must leave xl/styles.xml byte-identical"
    );
    assert!(
        sheet_after.contains("mergeCell") || sheet_after.contains("mergeCells"),
        "mergeCells must survive append: {sheet_after}"
    );
    assert!(
        sheet_after.contains("appended"),
        "appended row must be present: {sheet_after}"
    );
    Ok(())
}

/// Creating a sheet absent from the template must not rewrite existing styles/merges.
#[test]
fn with_template_new_sheet_keeps_existing_styles_and_merges() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("base-template.xlsx");
    let output = directory.path().join("new-sheet-out.xlsx");

    EasyExcel::write::<PairRow>(&template)
        .sheet("Styled")
        .merge_cells(MergeRange::new(0, 0, 0, 1))
        .head_style(CellStyle::new().bold(true))
        .do_write([PairRow {
            left: "a".to_owned(),
            right: "b".to_owned(),
        }])?;

    let styles_before = zip_entry_text(&template, "xl/styles.xml")?;
    let sheet_before = zip_entry_text(&template, "xl/worksheets/sheet1.xml")?;

    EasyExcel::write::<Value>(&output)
        .with_template(&template)
        .sheet("Fresh")
        .need_head(false)
        .do_write([Value("on-new".to_owned())])?;

    let styles_after = zip_entry_text(&output, "xl/styles.xml")?;
    let sheet_after = zip_entry_text(&output, "xl/worksheets/sheet1.xml")?;
    assert_eq!(
        styles_before, styles_after,
        "styles.xml must stay untouched"
    );
    assert_eq!(
        sheet_before, sheet_after,
        "existing Styled sheet (incl. mergeCells) must stay byte-identical"
    );

    let fresh = EasyExcel::read_sync::<Value>(&output)
        .sheet("Fresh")
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(fresh, vec![Value("on-new".to_owned())]);

    let styled = EasyExcel::read_sync::<PairRow>(&output)
        .sheet("Styled")
        .do_read_sync()?;
    assert_eq!(
        styled,
        vec![PairRow {
            left: "a".to_owned(),
            right: "b".to_owned(),
        }]
    );
    Ok(())
}
