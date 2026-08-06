#[test]
fn worksheet_metadata_shifts_ranges_formulas_and_recomputes_dimension() {
    let xml = concat!(
        "<worksheet><dimension ref=\"A1:D3\"/><sheetData>",
        "<row r=\"1\"><c r=\"A1\"></c></row>",
        "<row r=\"4\"><c r=\"D4\"><f>SUM(A1:A3)+$B$3+Sheet1!A3+LOG10(A3)</f></c></row>",
        "</sheetData><mergeCells><mergeCell ref=\"B3:C3\"/></mergeCells>",
        "<hyperlinks><hyperlink ref=\"A3\"/></hyperlinks>",
        "<autoFilter ref=\"A1:D3\"/>",
        "<dataValidations><dataValidation sqref=\"A3 B1:B3\"/></dataValidations>",
        "<conditionalFormatting sqref=\"C3:D3\"></conditionalFormatting></worksheet>"
    );
    let shifted = shift_worksheet_metadata(xml, 3, 1);
    assert!(shifted.contains("mergeCell ref=\"B4:C4\""));
    assert!(shifted.contains("hyperlink ref=\"A4\""));
    assert!(shifted.contains("autoFilter ref=\"A1:D4\""));
    assert!(shifted.contains("dataValidation sqref=\"A4 B1:B4\""));
    assert!(shifted.contains("conditionalFormatting sqref=\"C4:D4\""));
    assert!(shifted.contains("SUM(A1:A4)+$B$4+Sheet1!A4+LOG10(A4)"));

    let dimension = update_worksheet_dimension(&shifted);
    assert!(dimension.contains("dimension ref=\"A1:D4\""));
    assert_eq!(shift_worksheet_metadata(xml, 3, 0), xml);
    assert_eq!(update_worksheet_dimension("<worksheet/>"), "<worksheet/>");
}

#[test]
fn force_new_row_pipeline_shifts_real_formula_merge_and_dimension_metadata() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("metadata-template.xlsx");
    let output = directory.path().join("metadata-output.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Name").map_err(test_error)?;
    worksheet
        .write_string(1, 0, "{.name}")
        .map_err(test_error)?;
    worksheet
        .merge_range(2, 1, 2, 2, "Footer", &Format::new())
        .map_err(test_error)?;
    worksheet.write_formula(2, 3, "=A3").map_err(test_error)?;
    workbook.save(&template).map_err(test_error)?;

    fill_xlsx_template_list(
        &template,
        &output,
        &FillWrapper::new([
            TemplateData::new().with("name", "A"),
            TemplateData::new().with("name", "B"),
            TemplateData::new().with("name", "C"),
        ]),
        FillConfig::new().force_new_row(true),
    )?;

    let entries = load_entries(&output)?;
    let worksheet = entries
        .iter()
        .find(|entry| entry.name == "xl/worksheets/sheet1.xml")
        .ok_or_else(|| ExcelError::Format("worksheet fixture is missing".to_owned()))?;
    let xml = std::str::from_utf8(&worksheet.bytes).map_err(test_error)?;
    assert!(xml.contains("dimension ref=\"A1:D5\""));
    assert!(xml.contains("mergeCell ref=\"B5:C5\""));
    assert!(xml.contains("<c r=\"D5\"><f>A5</f>"));
    Ok(())
}

#[test]
fn a1_reference_and_metadata_parsers_reject_malformed_inputs() {
    assert_eq!(parse_cell_reference("$AA$12"), Some((27, 12)));
    assert_eq!(parse_cell_reference("A"), None);
    assert_eq!(parse_cell_reference("1"), None);
    assert_eq!(parse_cell_reference("A0"), None);
    assert_eq!(parse_cell_reference("A1x"), None);
    assert_eq!(parse_cell_reference("XFE1"), None);
    assert_eq!(parse_cell_reference("ZZZZZZZZZZZZZZZZZZZZ1"), None);
    assert_eq!(shift_a1_reference("A2", 3, 2), "A2");
    assert_eq!(shift_a1_reference("bad", 1, 2), "bad");
    assert_eq!(shift_a1_reference("$A$3", 3, 2), "$A$5");
    assert_eq!(shift_reference_list("A1:A3 C3", 3, 1), "A1:A4 C4");

    assert_eq!(shift_formula_elements("<f", 1, 1), "<f");
    assert_eq!(shift_formula_elements("<f>missing", 1, 1), "<f>missing");
    assert_eq!(shift_formula_references("$", 1, 1), "$");
    assert_eq!(shift_formula_references("A3_name+A3x", 1, 1), "A3_name+A3x");
    assert_eq!(shift_formula_references("Sheet1!A3", 3, 1), "Sheet1!A4");
    assert_eq!(shift_formula_references("LOG10(A3)", 3, 1), "LOG10(A4)");

    assert_eq!(
        shift_tag_references("<mergeCell", "mergeCell", "ref", 1, 1),
        "<mergeCell"
    );
    assert_eq!(
        shift_tag_references("<mergeCell/>", "mergeCell", "ref", 1, 1),
        "<mergeCell/>"
    );
    assert_eq!(
        replace_tag_attribute("<x/>", "dimension", "ref", "A1"),
        "<x/>"
    );
    assert_eq!(
        replace_tag_attribute("<dimension", "dimension", "ref", "A1"),
        "<dimension"
    );
    assert_eq!(
        update_worksheet_dimension(
            "<worksheet><dimension ref=\"A1\"/><c></c><c r=\"bad\"></c></worksheet>"
        ),
        "<worksheet><dimension ref=\"A1\"/><c></c><c r=\"bad\"></c></worksheet>"
    );
}
