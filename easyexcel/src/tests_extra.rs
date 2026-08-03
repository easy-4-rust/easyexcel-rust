//! `lib.rs` facade 的补充集成测试。
//!
//! 这些用例原本内联在 `lib.rs` 的 `mod tests_extra` 中；facade 拆分后集中在本文件。
//! `use crate::*` 让测试可以无障碍访问 facade 公开 API。

use std::collections::BTreeMap;
use std::io::Cursor;

use tempfile::tempdir;

use crate::EasyExcel;
use crate::core::{
    AnalysisContext, Converter, DynamicRow, DynamicValue, ExcelError, NullableObjectConverter,
    ReadListener, Result,
};
use crate::read::{EternalReadCacheSelector, SimpleReadCacheSelector, StoredReadCacheSelector};
use crate::template::{FillConfig, FillDirection, FillWrapper, TemplateData};
use crate::write::ExcelOutputStream;

/// 对应 Java：测试用可空转换器（NullableObjectConverter 标记）。
#[derive(Debug, Clone, Copy)]
struct TestNullableStringConverter;

impl Converter<String> for TestNullableStringConverter {}

impl NullableObjectConverter<String> for TestNullableStringConverter {}

/// 对应 Java：测试用事件监听器，统计收到的行数。
#[derive(Debug, Default)]
struct CountingListener {
    rows: usize,
}

impl ReadListener<DynamicRow> for CountingListener {
    fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
        self.rows += 1;
        Ok(())
    }
}

fn dynamic_row(column: usize, text: &str) -> DynamicRow {
    let mut cells = BTreeMap::new();
    cells.insert(column, DynamicValue::String(text.to_owned()));
    DynamicRow::new(cells)
}

/// 对应 Java：`EasyExcel.read(File)`（无监听器）与 `write(OutputStream)` 入口。
#[test]
fn reader_from_path_and_writer_to_output_stream_execute_real_io() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("path-read.xlsx");
    EasyExcel::write::<DynamicRow>(&path)
        .need_head(false)
        .do_write([dynamic_row(0, "path-read")])?;

    // reader_from_path：对应 Java `EasyExcel.read(file)`。
    let mut listener = CountingListener::default();
    EasyExcel::reader_from_path(&path)
        .head_row_number(0)
        .sheet_name("Sheet1")
        .build(&mut listener)?
        .read_all()?;
    assert_eq!(listener.rows, 1);

    // writer_to_output_stream：对应 Java `EasyExcel.write(outputStream)`，
    // 默认 autoCloseStream(true) 在写完后续流被关闭。
    let output = ExcelOutputStream::new(Cursor::new(Vec::new()));
    let inspect = output.clone();
    EasyExcel::writer_to_output_stream(output)
        .sheet_name("Stream")
        .need_head(false)
        .do_write([dynamic_row(0, "stream-write")])?;
    assert!(inspect.is_closed());
    Ok(())
}

/// 对应 Java：事件与同步读取构建器接受可空转换器和缓存选择器。
#[test]
fn reader_builders_register_nullable_converter_and_cache_selector() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("converters.xlsx");
    EasyExcel::write::<DynamicRow>(&path)
        .need_head(false)
        .do_write([dynamic_row(0, "value")])?;

    // 事件路径：ExcelReaderBuilder.registerNullableConverter / readCacheSelector。
    let mut listener = CountingListener::default();
    let event_builder = EasyExcel::read::<DynamicRow, _>(&path, &mut listener)
        .register_nullable_converter::<String, _>(TestNullableStringConverter)
        .read_cache_selector(StoredReadCacheSelector::Simple(
            SimpleReadCacheSelector::new(),
        ));
    let _ = event_builder;

    // 同步收集路径：普通读取得到 1 行；注册可空转换器与缓存选择器
    // （对应 Java：registerNullableConverter / readCacheSelector）。
    assert_eq!(
        EasyExcel::read_dynamic_sync(&path)
            .head_row_number(0)
            .do_read_sync()?
            .len(),
        1
    );
    let _sync_builder = EasyExcel::read_sync::<DynamicRow>(&path)
        .register_nullable_converter::<String, _>(TestNullableStringConverter)
        .read_cache_selector(StoredReadCacheSelector::Eternal(
            EternalReadCacheSelector::map_cache(),
        ));
    Ok(())
}

/// 对应 Java：writer 构建器开启默认样式、自动合并表头、相对表头行号与 legacy 模板种子。
#[test]
fn writer_builder_chains_style_head_and_legacy_seed_options() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("styled.xlsx");
    EasyExcel::write::<DynamicRow>(&path)
        .use_default_style(true)
        .automatic_merge_head(true)
        .relative_head_row_index(1)
        .use_legacy_template_seed(true)
        .do_write([dynamic_row(0, "styled")])?;
    assert!(path.exists());

    let plain_path = directory.path().join("plain.xlsx");
    EasyExcel::write::<DynamicRow>(&plain_path)
        .use_default_style(false)
        .do_write([dynamic_row(0, "plain")])?;
    assert!(plain_path.exists());
    Ok(())
}

/// 对应 Java：`doFill(Supplier<Object>, FillConfig)` 水平展开集合。
#[test]
fn do_fill_with_config_supplier_expands_horizontal_collection() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("fill-template.xlsx");
    EasyExcel::write::<DynamicRow>(&template)
        .need_head(false)
        .do_write([dynamic_row(0, "{.name}")])?;

    let output = directory.path().join("fill-output.xlsx");
    EasyExcel::write::<DynamicRow>(&output)
        .with_template(&template)
        .need_head(false)
        .do_fill_with_config_supplier(
            || {
                FillWrapper::new([
                    TemplateData::new().with("name", "H1"),
                    TemplateData::new().with("name", "H2"),
                ])
            },
            FillConfig::new().direction(FillDirection::Horizontal),
        )?;

    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .do_read_sync()?;
    let rendered = rows
        .iter()
        .flat_map(|row| row.values().values())
        .filter_map(|value| match value {
            DynamicValue::String(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(rendered.contains(&"H1"), "{rendered:?}");
    assert!(rendered.contains(&"H2"), "{rendered:?}");
    Ok(())
}

/// 对应 Java：CSV 输出流配模板时返回 Unsupported（`csv cannot use template.`）。
#[test]
fn csv_stream_write_with_template_returns_unsupported() -> Result<()> {
    let directory = tempdir()?;
    let template = directory.path().join("csv-template.xlsx");
    EasyExcel::write::<DynamicRow>(&template)
        .need_head(false)
        .do_write([dynamic_row(0, "{name}")])?;

    let csv_path = directory.path().join("out.csv");
    let mut cursor = Cursor::new(Vec::new());
    let error = EasyExcel::write::<DynamicRow>(&csv_path)
        .with_template(&template)
        .to_writer(&mut cursor)
        .do_write([dynamic_row(0, "csv")])
        .expect_err("csv 模板写入必须失败");
    assert!(
        matches!(error, ExcelError::Unsupported(ref message) if message.contains("csv")),
        "unexpected error: {error}"
    );
    Ok(())
}

/// 对应 Java：`excelType(ExcelTypeEnum.XLS)` 显式写出真实 BIFF8 文件。
#[test]
fn xls_write_via_explicit_excel_type_emits_biff8() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("explicit-type.xls");
    EasyExcel::write::<DynamicRow>(&path)
        .excel_type(crate::support::ExcelTypeEnum::Xls)
        .need_head(false)
        .do_write([dynamic_row(0, "xls-row")])?;
    let bytes = std::fs::read(&path)?;
    // OLE2 复合文档魔数（D0 CF 11 E0）。
    assert_eq!(&bytes[..4], &[0xD0, 0xCF, 0x11, 0xE0]);
    Ok(())
}
