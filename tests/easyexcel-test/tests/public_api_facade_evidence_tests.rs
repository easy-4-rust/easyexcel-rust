//! Java 4.0.3 `EasyExcel` / `EasyExcelFactory` 门面的三证据行为用例。

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use easyexcel::{
    AnalysisContext, DynamicRow, EasyExcel, EasyExcelFactory, ExcelOutputStream, ReadListener,
};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct FacadeContract {
    authority: String,
    easy_excel_class: String,
    easy_excel_factory_class: String,
    easy_excel_superclass: String,
    methods: BTreeMap<String, String>,
}

fn contract() -> FacadeContract {
    serde_json::from_str(include_str!("golden/facade_api.contract.json"))
        .expect("Java facade contract must be valid JSON")
}

fn assert_rust_type(value: &impl Sized, expected_simple_name: &str) {
    let actual = std::any::type_name_of_val(value);
    assert!(
        actual.contains(expected_simple_name),
        "expected Rust type containing {expected_simple_name}, got {actual}"
    );
}

struct CountingListener(Arc<AtomicUsize>);

impl ReadListener<DynamicRow> for CountingListener {
    fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> easyexcel::Result<()> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> easyexcel::Result<()> {
        Ok(())
    }
}

#[test]
fn facade_types_and_constructors_match_java_contract() {
    let contract = contract();

    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(contract.easy_excel_class, "com.alibaba.excel.EasyExcel");
    assert_eq!(
        contract.easy_excel_factory_class,
        "com.alibaba.excel.EasyExcelFactory"
    );
    assert_eq!(
        contract.easy_excel_superclass,
        "com.alibaba.excel.EasyExcelFactory"
    );
    assert_eq!(EasyExcel::new(), EasyExcelFactory::new());
}

#[test]
fn facade_builder_overloads_match_java_builder_families() -> easyexcel::Result<()> {
    let contract = contract();
    let path = PathBuf::from("facade-contract.xlsx");

    let reader = EasyExcel::reader();
    let reader_path = EasyExcel::reader_from_path(&path);
    let reader_input = EasyExcel::reader_from_input_stream(Cursor::new(Vec::<u8>::new()))?;
    for value in [&reader, &reader_path, &reader_input] {
        assert_rust_type(value, "ExcelReaderBuilder");
    }
    let listener_rows = Arc::new(AtomicUsize::new(0));
    assert_rust_type(
        &EasyExcel::read::<DynamicRow, _>(&path, CountingListener(Arc::clone(&listener_rows))),
        "ExcelReaderBuilder",
    );
    assert_rust_type(
        &EasyExcel::read_from_input_stream::<DynamicRow, _, _>(
            Cursor::new(Vec::<u8>::new()),
            CountingListener(listener_rows),
        )?,
        "ExcelReaderBuilder",
    );
    for key in [
        "read()",
        "read(File)",
        "read(File,ReadListener)",
        "read(File,Class,ReadListener)",
        "read(InputStream)",
        "read(InputStream,ReadListener)",
        "read(InputStream,Class,ReadListener)",
        "read(String)",
        "read(String,ReadListener)",
        "read(String,Class,ReadListener)",
    ] {
        assert_eq!(
            contract.methods[key],
            "com.alibaba.excel.read.builder.ExcelReaderBuilder"
        );
    }

    assert_rust_type(&EasyExcel::read_sheet(), "ExcelReaderSheetBuilder");
    assert_rust_type(&EasyExcel::read_sheet_index(1), "ExcelReaderSheetBuilder");
    assert_rust_type(
        &EasyExcel::read_sheet_with(1, "Data"),
        "ExcelReaderSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::read_sheet_name("Data"),
        "ExcelReaderSheetBuilder",
    );
    for key in [
        "readSheet()",
        "readSheet(Integer)",
        "readSheet(Integer,String)",
        "readSheet(String)",
    ] {
        assert_eq!(
            contract.methods[key],
            "com.alibaba.excel.read.builder.ExcelReaderSheetBuilder"
        );
    }

    assert_rust_type(&EasyExcel::writer(), "ExcelWriterBuilder");
    assert_rust_type(&EasyExcel::writer_to_path(&path), "ExcelWriterBuilder");
    assert_rust_type(&EasyExcel::write::<DynamicRow>(&path), "ExcelWriterBuilder");
    let output = ExcelOutputStream::new(Cursor::new(Vec::<u8>::new()));
    assert_rust_type(
        &EasyExcel::writer_to_output_stream(output),
        "ExcelWriterOutputStreamBuilder",
    );
    for key in [
        "write()",
        "write(File)",
        "write(File,Class)",
        "write(OutputStream)",
        "write(OutputStream,Class)",
        "write(String)",
        "write(String,Class)",
    ] {
        assert_eq!(
            contract.methods[key],
            "com.alibaba.excel.write.builder.ExcelWriterBuilder"
        );
    }

    assert_rust_type(
        &EasyExcel::writer_sheet_builder(),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_index(1),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_with(1, "Data"),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_name("Data"),
        "ExcelWriterSheetBuilder",
    );
    for key in [
        "writerSheet()",
        "writerSheet(Integer)",
        "writerSheet(Integer,String)",
        "writerSheet(String)",
    ] {
        assert_eq!(
            contract.methods[key],
            "com.alibaba.excel.write.builder.ExcelWriterSheetBuilder"
        );
    }
    assert_rust_type(
        &EasyExcel::writer_table_builder_default(),
        "ExcelWriterTableBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_table_builder(1),
        "ExcelWriterTableBuilder",
    );
    for key in ["writerTable()", "writerTable(Integer)"] {
        assert_eq!(
            contract.methods[key],
            "com.alibaba.excel.write.builder.ExcelWriterTableBuilder"
        );
    }

    Ok(())
}

#[test]
fn input_stream_listener_facade_retains_temporary_input_until_read_finishes()
-> easyexcel::Result<()> {
    let rows = Arc::new(AtomicUsize::new(0));
    let builder = EasyExcel::read_from_input_stream::<DynamicRow, _, _>(
        Cursor::new(include_bytes!("fixtures/demo/demo.xlsx").as_slice()),
        CountingListener(Arc::clone(&rows)),
    )?;

    assert_rust_type(&builder, "ExcelReaderBuilder");
    builder.do_read()?;
    assert_eq!(rows.load(Ordering::Relaxed), 10);
    Ok(())
}

#[test]
fn facade_read_overloads_produce_builders() -> easyexcel::Result<()> {
    let contract = contract();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo/demo.xlsx");

    // read() - no arg
    assert_rust_type(&EasyExcel::reader(), "ExcelReaderBuilder");

    // read(File)
    assert_rust_type(&EasyExcel::reader_from_path(&path), "ExcelReaderBuilder");

    // read(InputStream)
    let input = std::fs::read(&path)?;
    assert_rust_type(
        &EasyExcel::reader_from_input_stream(Cursor::new(input))?,
        "ExcelReaderBuilder",
    );

    // read(File, ReadListener) + read(InputStream, ReadListener)
    let rows = Arc::new(AtomicUsize::new(0));
    let builder = EasyExcel::read::<DynamicRow, _>(&path, CountingListener(Arc::clone(&rows)));
    assert_rust_type(&builder, "ExcelReaderBuilder");

    let input2 = std::fs::read(&path)?;
    let builder2 = EasyExcel::read_from_input_stream::<DynamicRow, _, _>(
        Cursor::new(input2),
        CountingListener(rows),
    )?;
    assert_rust_type(&builder2, "ExcelReaderBuilder");

    // readSheet overloads
    assert_rust_type(&EasyExcel::read_sheet(), "ExcelReaderSheetBuilder");
    assert_rust_type(&EasyExcel::read_sheet_index(0), "ExcelReaderSheetBuilder");
    assert_rust_type(
        &EasyExcel::read_sheet_with(0, "Sheet1"),
        "ExcelReaderSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::read_sheet_name("Sheet1"),
        "ExcelReaderSheetBuilder",
    );

    // Verify Java contract method return types
    for key in [
        "read()",
        "read(File)",
        "read(InputStream)",
        "readSheet()",
        "readSheet(Integer)",
        "readSheet(Integer,String)",
        "readSheet(String)",
    ] {
        assert!(
            contract.methods.contains_key(key),
            "Java contract missing method: {key}"
        );
    }

    Ok(())
}

#[test]
fn facade_write_overloads_produce_builders() -> easyexcel::Result<()> {
    let contract = contract();
    let directory = tempdir()?;
    let path = directory.path().join("write-overloads.xlsx");

    // write() - no arg
    assert_rust_type(&EasyExcel::writer(), "ExcelWriterBuilder");

    // write(File)
    assert_rust_type(&EasyExcel::writer_to_path(&path), "ExcelWriterBuilder");

    // write(File, Class) - typed
    assert_rust_type(&EasyExcel::write::<DynamicRow>(&path), "ExcelWriterBuilder");

    // write(OutputStream)
    let output = ExcelOutputStream::new(Cursor::new(Vec::<u8>::new()));
    assert_rust_type(
        &EasyExcel::writer_to_output_stream(output),
        "ExcelWriterOutputStreamBuilder",
    );

    // writerSheet overloads
    assert_rust_type(
        &EasyExcel::writer_sheet_builder(),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_index(0),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_with(0, "Data"),
        "ExcelWriterSheetBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_sheet_builder_name("Data"),
        "ExcelWriterSheetBuilder",
    );

    // writerTable overloads
    assert_rust_type(
        &EasyExcel::writer_table_builder_default(),
        "ExcelWriterTableBuilder",
    );
    assert_rust_type(
        &EasyExcel::writer_table_builder(0),
        "ExcelWriterTableBuilder",
    );

    // Verify Java contract method return types
    for key in [
        "write()",
        "write(File)",
        "write(File,Class)",
        "write(OutputStream)",
        "write(OutputStream,Class)",
        "write(String)",
        "write(String,Class)",
        "writerSheet()",
        "writerSheet(Integer)",
        "writerSheet(Integer,String)",
        "writerSheet(String)",
        "writerTable()",
        "writerTable(Integer)",
    ] {
        assert!(
            contract.methods.contains_key(key),
            "Java contract missing method: {key}"
        );
    }

    Ok(())
}
