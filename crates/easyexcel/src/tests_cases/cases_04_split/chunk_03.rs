/// Java `ExcelWriterBuilder.withTemplate` + `sheet().doWrite` appends onto the template.
#[test]
fn with_template_do_write_appends_and_preserves_other_sheets() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("template.xlsx");
    let output = directory.path().join("template-write.xlsx");

    let mut writer = EasyExcel::write::<Value>(&template).build();
    writer.write(
        [Value("kept".to_owned())],
        &EasyExcel::writer_sheet::<Value>("Sheet1").need_head(false),
    )?;
    writer.write(
        [Value("other".to_owned())],
        &EasyExcel::writer_sheet::<Value>("Sheet2").need_head(false),
    )?;
    writer.finish()?;

    EasyExcel::write::<Value>(&output)
        .with_template(&template)
        .sheet_index(0)
        .need_head(false)
        .do_write([Value("appended".to_owned())])?;

    let sheet1 = EasyExcel::read_sync::<Value>(&output)
        .sheet(0usize)
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(
        sheet1,
        vec![Value("kept".to_owned()), Value("appended".to_owned())]
    );
    let sheet2 = EasyExcel::read_sync::<Value>(&output)
        .sheet(1usize)
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(sheet2, vec![Value("other".to_owned())]);

    let csv = directory.path().join("template-write.csv");
    let error = EasyExcel::write::<Value>(&csv)
        .with_template(&template)
        .do_write([Value("x".to_owned())])
        .expect_err("csv cannot use template");
    assert!(error.to_string().contains("csv cannot use template"));
    Ok(())
}

/// Java `ExcelWriterBuilder.withTemplate(InputStream)` → Rust `with_template_bytes`.
#[test]
fn with_template_bytes_do_write_matches_file_template() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("template-bytes.xlsx");
    let output = directory.path().join("from-bytes.xlsx");

    EasyExcel::write::<Value>(&template)
        .need_head(false)
        .do_write([Value("seed".to_owned())])?;
    let bytes = fs::read(&template)?;

    EasyExcel::write::<Value>(&output)
        .with_template_bytes(bytes)
        .need_head(false)
        .do_write([Value("from-bytes".to_owned())])?;

    let rows = EasyExcel::read_sync::<Value>(&output)
        .head_row_number(0)
        .do_read_sync()?;
    assert_eq!(
        rows,
        vec![Value("seed".to_owned()), Value("from-bytes".to_owned())]
    );
    Ok(())
}

