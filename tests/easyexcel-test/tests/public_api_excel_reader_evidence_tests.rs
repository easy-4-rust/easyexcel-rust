//! Java 4.0.3 `ExcelReader` 生命周期的三证据行为用例。

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use easyexcel::read::ReadOptions;
use easyexcel::{AnalysisContext, DynamicRow, EasyExcel, ExcelReader, ReadListener};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ExcelReaderContract {
    authority: String,
    direct_constructor: String,
    reader_class: String,
    analysis_context_class: String,
    get_analysis_context_same: bool,
    excel_executor_class: String,
    finish_then_close: bool,
    deprecated_read: bool,
    read_all: bool,
    read_list_returns_self: bool,
    read_varargs_returns_self: bool,
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

fn contract() -> ExcelReaderContract {
    serde_json::from_str(include_str!("golden/excel_reader_lifecycle.contract.json"))
        .expect("Java ExcelReader contract must be valid JSON")
}

fn workbook() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/artifacts/simple_data.xlsx")
}

fn reader(rows: &Arc<AtomicUsize>) -> easyexcel::Result<ExcelReader<DynamicRow, CountingListener>> {
    ExcelReader::new(
        workbook(),
        ReadOptions::default(),
        CountingListener(Arc::clone(rows)),
    )
}

#[test]
#[allow(deprecated)]
fn excel_reader_context_executor_finish_and_close_match_java() -> easyexcel::Result<()> {
    let contract = contract();
    let rows = Arc::new(AtomicUsize::new(0));
    let mut reader = reader(&rows)?;

    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(contract.direct_constructor, "com.alibaba.excel.ExcelReader");
    assert_eq!(contract.reader_class, "com.alibaba.excel.ExcelReader");
    assert!(
        contract
            .analysis_context_class
            .ends_with("DefaultXlsxReadContext")
    );
    assert!(contract.excel_executor_class.ends_with("XlsxSaxAnalyser"));
    assert!(contract.get_analysis_context_same);
    assert!(std::ptr::eq(
        reader.analysis_context(),
        reader.get_analysis_context()
    ));
    assert!(std::any::type_name_of_val(reader.excel_executor()).contains("ExcelReadExecutorKind"));
    reader.finish();
    reader.close();
    assert!(contract.finish_then_close);
    Ok(())
}

#[test]
#[allow(deprecated)]
fn excel_reader_read_aliases_and_sheet_overloads_match_java() -> easyexcel::Result<()> {
    let contract = contract();

    let deprecated_rows = Arc::new(AtomicUsize::new(0));
    let mut deprecated_reader = reader(&deprecated_rows)?;
    deprecated_reader.read_deprecated()?;
    deprecated_reader.finish();
    assert!(contract.deprecated_read);
    assert_eq!(deprecated_rows.load(Ordering::Relaxed), 10);

    let all_rows = Arc::new(AtomicUsize::new(0));
    let mut all_reader = reader(&all_rows)?;
    all_reader.read_all()?;
    all_reader.finish();
    assert!(contract.read_all);
    assert_eq!(all_rows.load(Ordering::Relaxed), 10);

    let sheet = EasyExcel::read_sheet_index(0).build();
    let list_rows = Arc::new(AtomicUsize::new(0));
    let mut list_reader = reader(&list_rows)?;
    let original = std::ptr::from_mut(&mut list_reader);
    let returned = std::ptr::from_mut(list_reader.read(std::slice::from_ref(&sheet))?);
    assert!(contract.read_list_returns_self);
    assert_eq!(original, returned);
    list_reader.finish();
    assert_eq!(list_rows.load(Ordering::Relaxed), 10);

    let varargs_rows = Arc::new(AtomicUsize::new(0));
    let mut varargs_reader = reader(&varargs_rows)?;
    let original = std::ptr::from_mut(&mut varargs_reader);
    let returned = std::ptr::from_mut(varargs_reader.read(&[sheet])?);
    assert!(contract.read_varargs_returns_self);
    assert_eq!(original, returned);
    varargs_reader.finish();
    assert_eq!(varargs_rows.load(Ordering::Relaxed), 10);
    Ok(())
}
