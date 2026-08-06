include!("cases_01_split/chunk_01.rs");









include!("cases_01_split/chunk_02.rs");

fn java_compatibility_fixture(directory: &TempDir, name: &str) -> Result<std::path::PathBuf> {
    let encoded = match name {
        "t01.xls" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t01.xls.gz.b64"
        )),
        "t02.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t02.xlsx.gz.b64"
        )),
        "t03.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t03.xlsx.gz.b64"
        )),
        "t04.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t04.xlsx.gz.b64"
        )),
        "t05.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t05.xlsx.gz.b64"
        )),
        "t06.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t06.xlsx.gz.b64"
        )),
        "t07.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t07.xlsx.gz.b64"
        )),
        "t09.xlsx" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-compat-t09.xlsx.gz.b64"
        )),
        _ => {
            return Err(ExcelError::Format(format!(
                "unknown compatibility fixture: {name}"
            )));
        }
    };
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(test_error)?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut workbook = Vec::new();
    decoder.read_to_end(&mut workbook).map_err(test_error)?;
    let path = directory.path().join(name);
    fs::write(&path, workbook).map_err(test_error)?;
    Ok(path)
}

fn read_java_compatibility_rows(
    directory: &TempDir,
    name: &str,
    head_row_number: u32,
    read_default_return: ReadDefaultReturn,
) -> Result<Vec<DynamicRow>> {
    let path = java_compatibility_fixture(directory, name)?;
    let options = ReadOptions {
        head_row_number,
        read_default_return,
        ..ReadOptions::default()
    };
    let mut listener = DynamicProbe::default();
    if path.extension().is_some_and(|extension| extension == "xls") {
        read_xls::<DynamicRow, _>(&path, &options, &mut listener)?;
    } else {
        read_xlsx::<DynamicRow, _>(&path, &options, &mut listener)?;
    }
    Ok(listener.0)
}





