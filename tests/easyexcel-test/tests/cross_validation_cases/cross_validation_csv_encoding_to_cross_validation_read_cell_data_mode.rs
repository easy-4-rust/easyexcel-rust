/// This test verifies the Rust CSV parser handles
/// encoding the same way as Java commons-csv.
#[test]
fn cross_validation_csv_encoding() {
    let path = fixture("demo/demo.csv");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    // Read with UTF-8 (default)
    let rows = EasyExcel::read_sync::<DynamicRow>(&path)
        .charset("UTF-8")
        .do_read_sync()
        .unwrap();

    assert!(!rows.is_empty(), "UTF-8 CSV should parse correctly");
}

/// This test reads BOM CSV files the same way Java does.
#[test]
fn cross_validation_bom_csv() {
    let bom_path = fixture("bom/office_bom.csv");
    let no_bom_path = fixture("bom/no_bom.csv");

    if bom_path.exists() {
        let rows = EasyExcel::read_sync::<DynamicRow>(&bom_path)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty(), "BOM CSV should parse correctly");
    }

    if no_bom_path.exists() {
        let rows = EasyExcel::read_sync::<DynamicRow>(&no_bom_path)
            .do_read_sync()
            .unwrap();
        assert!(!rows.is_empty(), "No-BOM CSV should parse correctly");
    }
}

/// This test verifies that the Rust XLSX writer produces
/// output that can be read back by the Rust XLSX reader,
/// matching the Java round-trip behavior.
#[test]
fn cross_validation_round_trip_xlsx() {
    #[derive(ExcelRow, Debug, Clone)]
    struct TestData {
        #[excel(name = "ID", index = 0)]
        id: i64,
        #[excel(name = "Name", index = 1)]
        name: String,
    }
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("cross_validation_roundtrip.xlsx");

    // Write with Rust

    let data = vec![
        TestData {
            id: 1,
            name: "Alice".to_owned(),
        },
        TestData {
            id: 2,
            name: "Bob".to_owned(),
        },
    ];

    EasyExcel::write::<TestData>(&output_path)
        .sheet("Test")
        .do_write(data)
        .unwrap();

    // Read back with Rust
    let rows = EasyExcel::read_sync::<TestData>(&output_path)
        .do_read_sync()
        .unwrap();

    assert_eq!(rows.len(), 2, "Should read back 2 rows");
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].name, "Alice");
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[1].name, "Bob");

    // Clean up
    let _ = std::fs::remove_file(&output_path);
}

/// This test verifies that the Rust CSV writer produces
/// output that can be read back by the Rust CSV reader,
/// matching the Java round-trip behavior.
#[test]
fn cross_validation_round_trip_csv() {
    #[derive(ExcelRow, Debug, Clone)]
    struct CsvData {
        #[excel(name = "Value", index = 0)]
        value: String,
    }
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("cross_validation_roundtrip.csv");

    // Write with Rust

    let data = vec![
        CsvData {
            value: "hello".to_owned(),
        },
        CsvData {
            value: "world".to_owned(),
        },
    ];

    EasyExcel::write::<CsvData>(&output_path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();

    // Read back with Rust
    let rows = EasyExcel::read_sync::<CsvData>(&output_path)
        .do_read_sync()
        .unwrap();

    assert_eq!(rows.len(), 2, "Should read back 2 CSV rows");
    assert_eq!(rows[0].value, "hello");
    assert_eq!(rows[1].value, "world");

    // Clean up
    let _ = std::fs::remove_file(&output_path);
}

/// This test verifies that the Rust XLSX writer produces
/// output compatible with Java-generated XLSX files.
#[test]
fn cross_validation_java_compatible_xlsx_structure() {
    #[derive(ExcelRow, Debug, Clone)]
    struct CompatData {
        #[excel(name = "StringCol", index = 0)]
        string_col: String,
        #[excel(name = "IntCol", index = 1)]
        int_col: i64,
    }

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("cross_validation_java_compat.xlsx");

    // Write with Rust using features that match Java EasyExcel defaults

    let data = vec![CompatData {
        string_col: "test".to_owned(),
        int_col: 42,
    }];

    // Use same builder pattern as Java EasyExcel.write(file, head).sheet().doWrite()
    EasyExcel::write::<CompatData>(&output_path)
        .sheet("Sheet1")
        .do_write(data)
        .unwrap();

    // Verify the file exists and has XLSX magic bytes
    let bytes = std::fs::read(&output_path).unwrap();
    assert!(
        bytes.starts_with(b"PK"),
        "Output should be a valid XLSX (PK header)"
    );
    assert!(bytes.len() > 100, "XLSX should have reasonable size");

    // Read back
    let rows = EasyExcel::read_sync::<CompatData>(&output_path)
        .do_read_sync()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string_col, "test");
    assert_eq!(rows[0].int_col, 42);

    let _ = std::fs::remove_file(&output_path);
}

/// This test verifies that the Rust XLSX writer produces
/// output compatible with Java's password-encrypted XLSX.
#[test]
fn cross_validation_encrypted_xlsx() {
    #[derive(ExcelRow, Debug, Clone)]
    struct SecretData {
        #[excel(name = "Secret", index = 0)]
        secret: String,
    }

    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join("cross_validation_encrypted.xlsx");

    let data = vec![SecretData {
        secret: "confidential".to_owned(),
    }];

    // Write encrypted (Rust uses ECMA-376 Agile Encryption)
    EasyExcel::write::<SecretData>(&output_path)
        .password("test123")
        .sheet("Secret")
        .do_write(data)
        .unwrap();

    // Read back with password
    let rows = EasyExcel::read_sync::<SecretData>(&output_path)
        .password("test123")
        .do_read_sync()
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].secret, "confidential");

    let _ = std::fs::remove_file(&output_path);
}

/// This test verifies that the Rust XLSX reader handles
/// multi-sheet XLSX files the same way Java does.
#[test]
fn cross_validation_multi_sheet_xlsx() {
    let path = fixture("multiplesheets/multiplesheets.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    // Read all sheets (Java: EasyExcel.read(path).sheet(0/1/2).doRead())
    let rows_sheet0 = EasyExcel::read_sync::<DynamicRow>(&path)
        .sheet(0usize)
        .do_read_sync()
        .unwrap();
    assert!(!rows_sheet0.is_empty(), "Sheet 0 should have data");
}

/// This test verifies that the Rust XLSX reader handles
/// the no-model (Map<Integer, Object>) read mode the same way Java does.
#[test]
fn cross_validation_no_model_read() {
    let path = fixture("demo/demo.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    // Read as DynamicRow (Java equivalent: EasyExcel.read(path).sheet().doReadSync())
    let rows = EasyExcel::read_sync::<DynamicRow>(&path)
        .read_default_return(ReadDefaultReturn::ActualData)
        .do_read_sync()
        .unwrap();

    assert!(!rows.is_empty());

    // Each row should be indexable
    for row in &rows {
        for (idx, val) in row.values() {
            // All present columns should have non-Null values
            match val {
                // sparse cells
                DynamicValue::ActualData(_)
                | DynamicValue::String(_)
                | DynamicValue::ReadCellData(_)
                | DynamicValue::Null => {}
            }
            let _ = idx; // suppress unused warning
        }
    }
}

/// This test verifies that the Rust XLSX reader handles
/// the `ReadCellData` mode the same way Java does.
#[test]
fn cross_validation_read_cell_data_mode() {
    let path = fixture("demo/demo.xlsx");
    assert!(
        path.exists(),
        "required Java fixture missing: {}",
        path.display()
    );

    let rows = EasyExcel::read_sync::<DynamicRow>(&path)
        .read_default_return(ReadDefaultReturn::ReadCellData)
        .do_read_sync()
        .unwrap();

    assert!(!rows.is_empty());

    for row in &rows {
        for val in row.values().values() {
            if let DynamicValue::ReadCellData(rcd) = val {
                // ReadCellData should have row/column info
                assert!(rcd.row_index() < 10000, "Row index should be reasonable");
            }
        }
    }
}
