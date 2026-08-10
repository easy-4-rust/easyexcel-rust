//! Java 4.0.3 `ExcelBuilder` 七个 public API 的编译、行为与 golden 证据。

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use easyexcel::write::metadata::{WriteTable, WriteWorkbook};
use easyexcel::write::{BuilderFillConfig, ExcelBuilder};
use easyexcel::{
    CellValue, DynamicRow, DynamicValue, EasyExcel, ExcelBuilderImpl, TemplateData, WriteContext,
};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct ExcelBuilderContract {
    authority: String,
    implementation_class: String,
    add_content_two_arg: bool,
    add_content_three_arg: bool,
    write_context_class: String,
    write_context_same: bool,
    finish_false_output_exists: bool,
    merge_after_add_range: String,
    fill_output_exists: bool,
    template_merge_after_fill_range: String,
    finish_true_output_exists: bool,
    finish_true_output_size: u64,
    fill_without_template_error: String,
}

fn contract() -> ExcelBuilderContract {
    serde_json::from_str(include_str!("golden/excel_builder_lifecycle.contract.json"))
        .expect("Java ExcelBuilder contract must be valid JSON")
}

fn dynamic_row(value: &str) -> DynamicRow {
    DynamicRow::new(BTreeMap::from([(
        0,
        DynamicValue::String(value.to_owned()),
    )]))
}

fn contains_text(rows: &[DynamicRow], expected: &str) -> bool {
    rows.iter().any(|row| {
        row.values().values().any(|value| match value {
            DynamicValue::String(value) | DynamicValue::ActualData(CellValue::String(value)) => {
                value == expected
            }
            _ => false,
        })
    })
}

fn xlsx_template() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fill/simple.xlsx")
}

fn xls_template() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xls/fill/simple.xls")
}

fn first_worksheet_xml(path: &Path) -> easyexcel::Result<String> {
    let mut archive = zip::ZipArchive::new(File::open(path)?)
        .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?;
    let mut worksheet = archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?;
    let mut xml = String::new();
    worksheet.read_to_string(&mut xml)?;
    Ok(xml)
}

fn xls_has_merge(path: &Path, expected: (u16, u16, u16, u16)) -> easyexcel::Result<bool> {
    let stream = easyexcel_xls::biff8::record_stream::read_workbook_stream(path)
        .map_err(|error| easyexcel::ExcelError::Format(error.to_string()))?;
    let mut offset = 0_usize;
    while offset + 4 <= stream.len() {
        let record_type = u16::from_le_bytes([stream[offset], stream[offset + 1]]);
        let length = usize::from(u16::from_le_bytes([stream[offset + 2], stream[offset + 3]]));
        let payload_start = offset + 4;
        let payload_end = payload_start.saturating_add(length);
        if payload_end > stream.len() {
            return Err(easyexcel::ExcelError::Format(
                "BIFF8 record extends beyond workbook stream".to_owned(),
            ));
        }
        if record_type == 0x00E5 && length >= 2 {
            let count = usize::from(u16::from_le_bytes([
                stream[payload_start],
                stream[payload_start + 1],
            ]));
            for index in 0..count {
                let start = payload_start + 2 + index * 8;
                if start + 8 > payload_end {
                    break;
                }
                let range = (
                    u16::from_le_bytes([stream[start], stream[start + 1]]),
                    u16::from_le_bytes([stream[start + 2], stream[start + 3]]),
                    u16::from_le_bytes([stream[start + 4], stream[start + 5]]),
                    u16::from_le_bytes([stream[start + 6], stream[start + 7]]),
                );
                if range == expected {
                    return Ok(true);
                }
            }
        }
        offset = payload_end;
    }
    Ok(false)
}

fn assert_excel_builder_trait<T: ExcelBuilder>() {}

#[test]
fn excel_builder_write_context_merge_and_finish_match_java() -> easyexcel::Result<()> {
    assert_excel_builder_trait::<ExcelBuilderImpl>();
    let contract = contract();
    let directory = tempdir()?;
    let output = directory.path().join("excel-builder-api.xlsx");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.options.need_head = false;
    let mut builder = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Builder").need_head(false);
    let mut table = WriteTable::new();
    table.options.need_head = false;

    ExcelBuilder::add_content(&mut builder, [dynamic_row("builder-two-arg")], &sheet)?;
    assert!(contract.add_content_two_arg);
    ExcelBuilder::add_content_with_table(
        &mut builder,
        [dynamic_row("builder-three-arg")],
        &sheet,
        &table,
    )?;
    assert!(contract.add_content_three_arg);

    let first_context = ExcelBuilder::write_context(&builder) as *const dyn WriteContext;
    let second_context = ExcelBuilder::write_context(&builder) as *const dyn WriteContext;
    assert!(contract.write_context_same);
    assert!(std::ptr::addr_eq(first_context, second_context));
    assert_eq!(
        contract.write_context_class,
        "com.alibaba.excel.context.WriteContextImpl"
    );

    ExcelBuilder::merge(&mut builder, 0, 0, 0, 1)?;
    ExcelBuilder::finish(&mut builder, false)?;
    assert_eq!(output.exists(), contract.finish_false_output_exists);
    let xml = first_worksheet_xml(&output)?;
    assert_eq!(contract.merge_after_add_range, "A1:B1");
    assert!(xml.contains("mergeCell ref=\"A1:B1\""));

    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .do_read_sync()?;
    assert!(
        contains_text(&rows, "builder-two-arg"),
        "two-argument row missing from {rows:?}"
    );
    assert!(
        contains_text(&rows, "builder-three-arg"),
        "three-argument row missing from {rows:?}"
    );
    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(
        contract.implementation_class,
        "com.alibaba.excel.write.ExcelBuilderImpl"
    );
    Ok(())
}

#[test]
fn excel_builder_fill_and_error_branch_match_java() -> easyexcel::Result<()> {
    let contract = contract();
    let directory = tempdir()?;
    let output = directory.path().join("excel-builder-fill.xlsx");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.set_template_file(xlsx_template());
    let mut builder = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Sheet1").need_head(false);
    ExcelBuilder::fill(
        &mut builder,
        &TemplateData::new()
            .with("name", "builder-filled")
            .with("number", 7_i32),
        BuilderFillConfig::default(),
        &sheet,
    )?;
    ExcelBuilder::finish(&mut builder, false)?;
    assert_eq!(output.exists(), contract.fill_output_exists);
    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .ignore_empty_row(false)
        .do_read_sync()?;
    assert!(contains_text(&rows, "builder-filled"));

    let missing_output = directory.path().join("missing-template.xlsx");
    let mut missing_workbook = WriteWorkbook::new();
    missing_workbook.set_file(&missing_output);
    let mut missing = ExcelBuilderImpl::from_write_workbook(missing_workbook)?;
    let error = ExcelBuilder::fill(
        &mut missing,
        &TemplateData::new().with("name", "unused"),
        BuilderFillConfig::default(),
        &sheet,
    )
    .expect_err("fill without template must fail");
    assert_eq!(
        error.to_string(),
        format!(
            "unsupported operation: {}",
            contract.fill_without_template_error
        )
    );
    Ok(())
}

#[test]
fn excel_builder_finish_true_matches_java_output_lifecycle() -> easyexcel::Result<()> {
    let contract = contract();
    let directory = tempdir()?;
    let output = directory.path().join("excel-builder-exception.xlsx");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.options.need_head = false;
    let mut builder = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Builder").need_head(false);
    ExcelBuilder::add_content(&mut builder, [dynamic_row("discarded")], &sheet)?;
    ExcelBuilder::finish(&mut builder, true)?;

    assert_eq!(output.exists(), contract.finish_true_output_exists);
    let size = output
        .metadata()
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    assert_eq!(size, contract.finish_true_output_size);
    Ok(())
}

#[test]
fn excel_builder_merge_stays_in_xlsx_and_xls_template_sessions() -> easyexcel::Result<()> {
    let directory = tempdir()?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Sheet1").need_head(false);

    let xlsx_output = directory.path().join("excel-builder-template-merge.xlsx");
    let mut xlsx_workbook = WriteWorkbook::new();
    xlsx_workbook.set_file(&xlsx_output);
    xlsx_workbook.set_template_file(xlsx_template());
    let mut xlsx_builder = ExcelBuilderImpl::from_write_workbook(xlsx_workbook)?;
    ExcelBuilder::fill(
        &mut xlsx_builder,
        &TemplateData::new()
            .with("name", "xlsx-template-merge")
            .with("number", 11_i32),
        BuilderFillConfig::default(),
        &sheet,
    )?;
    ExcelBuilder::merge(&mut xlsx_builder, 0, 0, 0, 1)?;
    ExcelBuilder::finish(&mut xlsx_builder, false)?;
    let xlsx_merge = "A1:B1";
    assert_eq!(xlsx_merge, contract().template_merge_after_fill_range);
    assert!(
        first_worksheet_xml(&xlsx_output)?.contains(&format!("mergeCell ref=\"{xlsx_merge}\""))
    );

    let xls_output = directory.path().join("excel-builder-template-merge.xls");
    let mut xls_workbook = WriteWorkbook::new();
    xls_workbook.set_file(&xls_output);
    xls_workbook.set_excel_type(easyexcel::ExcelTypeEnum::Xls);
    xls_workbook.set_template_file(xls_template());
    let mut xls_builder = ExcelBuilderImpl::from_write_workbook(xls_workbook)?;
    ExcelBuilder::fill(
        &mut xls_builder,
        &TemplateData::new()
            .with("name", "xls-template-merge")
            .with("number", 12_i32),
        BuilderFillConfig::default(),
        &sheet,
    )?;
    ExcelBuilder::merge(&mut xls_builder, 0, 0, 0, 1)?;
    ExcelBuilder::finish(&mut xls_builder, false)?;
    assert!(xls_has_merge(&xls_output, (0, 0, 0, 1))?);

    for (path, expected) in [
        (&xlsx_output, "xlsx-template-merge"),
        (&xls_output, "xls-template-merge"),
    ] {
        let rows = EasyExcel::read_dynamic_sync(path)
            .head_row_number(0)
            .ignore_empty_row(false)
            .do_read_sync()?;
        assert!(contains_text(&rows, expected));
    }
    Ok(())
}

#[test]
fn excel_builder_chain_methods_on_writer_builder_return_self() -> easyexcel::Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("chain-methods.xlsx");

    // ExcelWriterBuilder chain: sheet() + need_head() return Self
    // do_write consumes the builder, producing the file.
    EasyExcel::write::<DynamicRow>(&output)
        .sheet("ChainTest")
        .need_head(false)
        .do_write(vec![dynamic_row("chain-row")])?;
    assert!(output.exists());

    // ExcelWriterSheetBuilder chain: sheet_no + sheet_name + relative_head_row_index + need_head
    // These methods each return Self, enabling fluent construction.
    // The chain compiles and executes without error, proving the return-Self contract.
    let _sheet = EasyExcel::writer_sheet_builder()
        .sheet_no(0)
        .sheet_name("NamedSheet")
        .relative_head_row_index(0)
        .need_head(true);

    Ok(())
}
