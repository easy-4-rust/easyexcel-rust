#[test]
#[allow(clippy::too_many_lines)]
fn xlsx_stream_matches_java_cell_types_cached_formulas_dates_and_trimming() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("mixed-cells.xlsx");
    write_xlsx_package(
        &path,
        &[
            (
                "[Content_Types].xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#,
            ),
            (
                "_rels/.rels",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
            ),
            (
                "xl/workbook.xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <workbookPr date1904="1"/>
  <sheets><sheet name="Mixed" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#,
            ),
            (
                "xl/sharedStrings.xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
  <si><r><t xml:space="preserve"><![CDATA[  shared_x000D_]]></t></r><rPh><t>ignored</t></rPh><r><t xml:space="preserve">value  </t></r></si>
</sst>"#,
            ),
            (
                "xl/styles.xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <cellXfs count="2"><xf numFmtId="0"/><xf numFmtId="14"/></cellXfs>
</styleSheet>"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:I2"/>
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Shared</t></is></c>
      <c r="B1" t="inlineStr"><is><t>Inline</t></is></c>
      <c r="C1" t="inlineStr"><is><t>Boolean</t></is></c>
      <c r="D1" t="inlineStr"><is><t>Integer</t></is></c>
      <c r="E1" t="inlineStr"><is><t>Float</t></is></c>
      <c r="F1" t="inlineStr"><is><t>Formula number</t></is></c>
      <c r="G1" t="inlineStr"><is><t>Formula string</t></is></c>
      <c r="H1" t="inlineStr"><is><t>Error</t></is></c>
      <c r="I1" t="inlineStr"><is><t>Date</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="s"><v>0</v></c>
      <c r="B2" t="inlineStr"><is><r><t xml:space="preserve"><![CDATA[  inline ]]></t></r><rPh><t>ignored</t></rPh><r><t xml:space="preserve">value  </t></r></is></c>
      <c r="C2" t="b"><v>1</v></c>
      <c r="D2" t="n"><v>42</v></c>
      <c r="E2" t="n"><v>3.5</v></c>
      <c r="F2"><f><![CDATA[SUM(D2:E2)]]></f><v>45.5</v></c>
      <c r="G2" t="str"><f>CONCAT("cache","d")</f><v>cached</v></c>
      <c r="H2" t="e"><v>#DIV/0!</v></c>
      <c r="I2" s="1"><v>1</v></c>
    </row>
  </sheetData>
</worksheet>"#,
            ),
        ],
    )?;

    let mut probe = RawProbe::default();
    read_xlsx::<RawRow, _>(
        &path,
        &ReadOptions {
            use_1904_windowing: true,
            ..options()
        },
        &mut probe,
    )?;
    assert_eq!(probe.0.len(), 1);
    assert_eq!(
        probe.0[0].cells[0],
        CellValue::String("shared\rvalue".to_owned())
    );
    assert_eq!(
        probe.0[0].cells[1],
        CellValue::String("inline value".to_owned())
    );
    assert_eq!(probe.0[0].cells[2], CellValue::Bool(true));
    assert_eq!(probe.0[0].cells[3], CellValue::Float(42.0));
    assert_eq!(probe.0[0].cells[4], CellValue::Float(3.5));
    assert_eq!(probe.0[0].cells[5], CellValue::Float(45.5));
    assert_eq!(probe.0[0].cells[6], CellValue::String("cached".to_owned()));
    assert_eq!(probe.0[0].cells[7], CellValue::Error("#DIV/0!".to_owned()));
    assert_eq!(probe.0[0].cells[8].as_text(), "1904-01-02 00:00:00");
    assert_eq!(
        probe.0[0].formulas,
        vec![
            None,
            None,
            None,
            None,
            None,
            Some("SUM(D2:E2)".to_owned()),
            Some("CONCAT(\"cache\",\"d\")".to_owned()),
            None,
            None,
        ]
    );

    let expected = probe.0[0].clone();
    for read_cache in [
        ReadCacheMode::Memory,
        ReadCacheMode::Moka,
        ReadCacheMode::File,
    ] {
        let mut cached = RawProbe::default();
        read_xlsx::<RawRow, _>(
            &path,
            &ReadOptions {
                use_1904_windowing: true,
                read_cache,
                ..options()
            },
            &mut cached,
        )?;
        assert_eq!(cached.0.as_slice(), std::slice::from_ref(&expected));
    }

    let mut untrimmed = RawProbe::default();
    read_xlsx::<RawRow, _>(
        &path,
        &ReadOptions {
            auto_trim: false,
            ..options()
        },
        &mut untrimmed,
    )?;
    assert_eq!(
        untrimmed.0[0].cells[0],
        CellValue::String("  shared\rvalue  ".to_owned())
    );
    assert_eq!(
        untrimmed.0[0].cells[1],
        CellValue::String("  inline value  ".to_owned())
    );

    let mut java_default = DynamicProbe::default();
    read_xlsx::<DynamicRow, _>(&path, &options(), &mut java_default)?;
    // BuiltinFormats ALL_LANGUAGES id=14 is `yyyy/m/d` (Java BuiltinFormats).
    assert_eq!(
        java_default.0[0].get(8),
        Some(&DynamicValue::String("1900/1/1".to_owned()))
    );

    let mut strings = DynamicProbe::default();
    read_xlsx::<DynamicRow, _>(
        &path,
        &ReadOptions {
            use_1904_windowing: true,
            ..options()
        },
        &mut strings,
    )?;
    assert_eq!(
        strings.0[0].get(3),
        Some(&DynamicValue::String("42".to_owned()))
    );
    assert_eq!(
        strings.0[0].get(8),
        Some(&DynamicValue::String("1904/1/2".to_owned()))
    );

    let mut actual = DynamicProbe::default();
    read_xlsx::<DynamicRow, _>(
        &path,
        &ReadOptions {
            read_default_return: ReadDefaultReturn::ActualData,
            use_1904_windowing: true,
            ..options()
        },
        &mut actual,
    )?;
    assert_eq!(
        actual.0[0].get(2),
        Some(&DynamicValue::ActualData(CellValue::Bool(true)))
    );
    assert_eq!(
        actual.0[0].get(5),
        Some(&DynamicValue::ActualData(CellValue::Decimal(
            "45.5".parse().map_err(test_error)?
        )))
    );
    assert_eq!(
        actual.0[0].get(7),
        Some(&DynamicValue::ActualData(CellValue::String(
            "#DIV/0!".to_owned()
        )))
    );

    let mut cell_data = DynamicProbe::default();
    read_xlsx::<DynamicRow, _>(
        &path,
        &ReadOptions {
            read_default_return: ReadDefaultReturn::ReadCellData,
            use_1904_windowing: true,
            ..options()
        },
        &mut cell_data,
    )?;
    let DynamicValue::ReadCellData(formula_cell) =
        cell_data.0[0].get(5).expect("formula cell data")
    else {
        panic!("expected formula read cell data");
    };
    let expected_decimal = CellValue::Decimal("45.5".parse().map_err(test_error)?);
    assert_eq!(formula_cell.raw_value(), &expected_decimal);
    assert_eq!(formula_cell.data(), &expected_decimal);
    assert_eq!(
        formula_cell.formula().map(FormulaData::formula_value),
        Some("SUM(D2:E2)")
    );
    Ok(())
}

#[test]
fn xlsx_primary_cell_stream_rejects_malformed_xml() -> Result<()> {
    let (directory, base) = workbook_fixture()?;
    let cases = [
        (
            "display-cell-xml-error.xlsx",
            r#"<worksheet><sheetData><row r="1"><c r="A1"><v><"#,
        ),
        (
            "display-tail-xml-error.xlsx",
            r#"<worksheet><sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>Value</t></is></c></row>
<row r="2"><c r="A2" t="inlineStr"><is><t>one</t></is></c></row><"#,
        ),
    ];

    for (name, replacement) in cases {
        let metadata_path = directory.path().join(name);
        rewrite_first_sheet(&base, &metadata_path, replacement)?;
        let mut listener = DynamicProbe::default();
        assert!(read_xlsx::<DynamicRow, _>(&metadata_path, &options(), &mut listener).is_err());
    }
    Ok(())
}

#[test]
fn dynamic_xlsx_reports_display_stream_initialization_errors() -> Result<()> {
    let (directory, base) = workbook_fixture()?;
    let malformed = directory.path().join("missing-sheet-data.xlsx");
    rewrite_first_sheet(&base, &malformed, "<worksheet/>")?;
    let mut listener = DynamicProbe::default();
    assert!(read_xlsx::<DynamicRow, _>(&malformed, &options(), &mut listener).is_err());
    Ok(())
}

#[test]
fn dynamic_rows_preserve_xlsx_gaps_and_csv_scalar_contracts() -> Result<()> {
    let (directory, base) = workbook_fixture()?;
    let sparse = directory.path().join("dynamic-sparse.xlsx");
    rewrite_first_sheet(
        &base,
        &sparse,
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>First</t></is></c>
      <c r="C1" t="inlineStr"><is><t>Tail</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>value</t></is></c>
      <c r="C2" t="n"><v>3</v></c>
    </row>
  </sheetData>
</worksheet>"#,
    )?;
    let mut xlsx = DynamicProbe::default();
    read_xlsx::<DynamicRow, _>(&sparse, &options(), &mut xlsx)?;
    assert_eq!(xlsx.0[0].values().len(), 3);
    assert_eq!(
        xlsx.0[0].get(0),
        Some(&DynamicValue::String("value".to_owned()))
    );
    assert_eq!(xlsx.0[0].get(1), Some(&DynamicValue::Null));
    assert_eq!(
        xlsx.0[0].get(2),
        Some(&DynamicValue::String("3".to_owned()))
    );

    let csv_path = directory.path().join("dynamic.csv");
    fs::write(&csv_path, "Text,Number,Empty,Tail\r\nvalue,109,,last\r\n")?;
    let mut csv_strings = DynamicProbe::default();
    read_csv::<DynamicRow, _>(&csv_path, &options(), &mut csv_strings)?;
    assert_eq!(
        csv_strings.0[0].get(1),
        Some(&DynamicValue::String("109".to_owned()))
    );
    assert_eq!(
        csv_strings.0[0].get(2),
        Some(&DynamicValue::String(String::new()))
    );

    let mut csv_actual = DynamicProbe::default();
    read_csv::<DynamicRow, _>(
        &csv_path,
        &ReadOptions {
            read_default_return: ReadDefaultReturn::ActualData,
            ..options()
        },
        &mut csv_actual,
    )?;
    assert_eq!(
        csv_actual.0[0].get(1),
        Some(&DynamicValue::ActualData(CellValue::String(
            "109".to_owned()
        )))
    );
    assert_eq!(
        csv_actual.0[0].get(2),
        Some(&DynamicValue::ActualData(CellValue::String(String::new())))
    );
    Ok(())
}

