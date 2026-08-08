//! Java 4.0.3 `ExcelWriter` 构造、链式写入、填充与生命周期三证据用例。

use std::any::Any;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use easyexcel::write::BuilderFillConfig;
use easyexcel::write::metadata::{WriteTable, WriteWorkbook};
use easyexcel::{
    CellValue, DynamicRow, DynamicValue, EasyExcel, ExcelBuilderImpl, ExcelTypeEnum, TemplateData,
    WriteContext,
};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct ExcelWriterContract {
    authority: String,
    direct_constructor: String,
    write_collection_returns_self: bool,
    write_supplier_returns_self: bool,
    write_table_returns_self: bool,
    write_supplier_table_returns_self: bool,
    write_supplier_calls: usize,
    write_context_class: String,
    write_context_same: bool,
    finish_then_close: bool,
    fill_object_returns_self: bool,
    fill_config_returns_self: bool,
    fill_supplier_returns_self: bool,
    fill_supplier_config_returns_self: bool,
    fill_supplier_calls: usize,
}

fn contract() -> ExcelWriterContract {
    serde_json::from_str(include_str!("golden/excel_writer_lifecycle.contract.json"))
        .expect("Java ExcelWriter contract must be valid JSON")
}

fn dynamic_row(value: &str) -> DynamicRow {
    DynamicRow::new(BTreeMap::from([(
        0,
        DynamicValue::String(value.to_owned()),
    )]))
}

fn xls_template() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xls/fill/simple.xls")
}

fn xlsx_template() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fill/simple.xlsx")
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

#[test]
fn excel_writer_constructor_write_overloads_context_finish_and_close_match_java()
-> easyexcel::Result<()> {
    let contract = contract();
    let directory = tempdir()?;
    let output = directory.path().join("excel-writer-api.xlsx");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.options.need_head = false;

    let mut writer = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Data").need_head(false);
    let mut table = WriteTable::new();
    table.options.need_head = false;
    let original = std::ptr::from_mut(&mut writer);

    let returned = std::ptr::from_mut(writer.write([dynamic_row("collection")], &sheet)?);
    assert!(contract.write_collection_returns_self);
    assert_eq!(original, returned);

    let supplier_calls = Cell::new(0_usize);
    let returned = std::ptr::from_mut(writer.write_with_supplier(
        || {
            supplier_calls.set(supplier_calls.get() + 1);
            [dynamic_row("supplier")]
        },
        &sheet,
    )?);
    assert!(contract.write_supplier_returns_self);
    assert_eq!(original, returned);

    let returned =
        std::ptr::from_mut(writer.write_with_table([dynamic_row("table")], &sheet, &table)?);
    assert!(contract.write_table_returns_self);
    assert_eq!(original, returned);

    let returned = std::ptr::from_mut(writer.write_with_table_supplier(
        || {
            supplier_calls.set(supplier_calls.get() + 1);
            [dynamic_row("supplier-table")]
        },
        &sheet,
        &table,
    )?);
    assert!(contract.write_supplier_table_returns_self);
    assert_eq!(original, returned);
    assert_eq!(supplier_calls.get(), contract.write_supplier_calls);

    let first_context = writer.write_context() as *const dyn WriteContext;
    let second_context = writer.write_context() as *const dyn WriteContext;
    assert_eq!(
        contract.write_context_class,
        "com.alibaba.excel.context.WriteContextImpl"
    );
    assert!(contract.write_context_same);
    assert!(std::ptr::addr_eq(first_context, second_context));
    assert_eq!(
        writer.write_context().current_write_holder().table_no(),
        Some(0)
    );

    writer.finish()?;
    writer.close()?;
    assert!(contract.finish_then_close);

    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .do_read_sync()?;
    for expected in ["collection", "supplier", "table", "supplier-table"] {
        assert!(
            contains_text(&rows, expected),
            "missing written value {expected}"
        );
    }
    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(contract.direct_constructor, "com.alibaba.excel.ExcelWriter");
    Ok(())
}

#[test]
fn excel_writer_fill_overloads_and_suppliers_execute_real_biff8_fill() -> easyexcel::Result<()> {
    let contract = contract();
    let directory = tempdir()?;
    let output = directory.path().join("excel-writer-fill.xls");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.set_excel_type(ExcelTypeEnum::Xls);
    workbook.set_template_file(xls_template());

    let mut writer = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Sheet1").need_head(false);
    let original = std::ptr::from_mut(&mut writer);

    let first = TemplateData::new().with("name", "writer-fill-xls");
    let returned = std::ptr::from_mut(writer.fill_default(&first, &sheet)?);
    assert!(contract.fill_object_returns_self);
    assert_eq!(original, returned);

    let configured = TemplateData::new().with("number", 7_i32);
    let returned = std::ptr::from_mut(writer.fill(
        &configured,
        BuilderFillConfig::new().auto_style(true),
        &sheet,
    )?);
    assert!(contract.fill_config_returns_self);
    assert_eq!(original, returned);

    let supplier_calls = Cell::new(0_usize);
    let returned = std::ptr::from_mut(writer.fill_with_supplier(
        || {
            supplier_calls.set(supplier_calls.get() + 1);
            Box::new(TemplateData::new().with("unused", "one")) as Box<dyn Any>
        },
        &sheet,
    )?);
    assert!(contract.fill_supplier_returns_self);
    assert_eq!(original, returned);

    let returned = std::ptr::from_mut(writer.fill_with_config_supplier(
        || {
            supplier_calls.set(supplier_calls.get() + 1);
            Box::new(TemplateData::new().with("unused", "two")) as Box<dyn Any>
        },
        BuilderFillConfig::default(),
        &sheet,
    )?);
    assert!(contract.fill_supplier_config_returns_self);
    assert_eq!(original, returned);
    assert_eq!(supplier_calls.get(), contract.fill_supplier_calls);

    // Java 的同一个 ExcelWriter 允许 fill 后继续 write；BIFF8 也必须共享
    // 同一个模板会话，不能在 finish 时丢掉普通写入数据。
    writer.write([dynamic_row("writer-after-fill-xls")], &sheet)?;

    writer.close()?;
    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .ignore_empty_row(false)
        .do_read_sync()?;
    assert!(contains_text(&rows, "writer-fill-xls"));
    assert!(contains_text(&rows, "writer-after-fill-xls"));
    Ok(())
}

#[test]
fn excel_writer_fill_then_write_share_one_template_lifecycle() -> easyexcel::Result<()> {
    let directory = tempdir()?;
    let output = directory.path().join("excel-writer-mixed.xlsx");
    let mut workbook = WriteWorkbook::new();
    workbook.set_file(&output);
    workbook.set_template_file(xlsx_template());

    let mut writer = ExcelBuilderImpl::from_write_workbook(workbook)?;
    let sheet = EasyExcel::writer_sheet::<DynamicRow>("Sheet1").need_head(false);
    writer.fill_default(
        &TemplateData::new()
            .with("name", "mixed-fill")
            .with("number", 9_i32),
        &sheet,
    )?;
    writer.write([dynamic_row("mixed-write")], &sheet)?;
    writer.finish()?;

    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .ignore_empty_row(false)
        .do_read_sync()?;
    assert!(contains_text(&rows, "mixed-fill"));
    assert!(contains_text(&rows, "mixed-write"));
    Ok(())
}
