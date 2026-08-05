//! XLSX 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 XLSX 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::io::Write;
use std::path::Path;

use crate::core::{ExcelRow, Result, WriteHandler, WriteWorkbookContext};
use crate::write::excel_writer_core::{
    HandlerHolderScope, after_workbook, after_workbook_create, before_workbook, save_workbook,
    save_workbook_to_writer, sort_handlers, validate_excel_row_schema,
    with_default_write_converters, write_sheet_to_workbook, write_xlsx_onto_template_package,
};
use crate::write::write_options::WriteOptions;
use easyexcel_xlsx::xlsx::generation::Workbook;

/// 将类型化行写入 XLSX 文件。
///
/// # Errors
///
/// 返回转换、校验、XLSX 格式或 I/O 错误。
pub fn write_xlsx<T, I>(path: &Path, options: &WriteOptions, rows: I) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    write_xlsx_with_handlers(path, options, rows, &mut [])
}

/// Writes typed rows while invoking ordered write handlers.
///
/// # Errors
///
/// Returns a conversion, handler, worksheet-configuration, XLSX-format, or I/O error.
pub fn write_xlsx_with_handlers<T, I>(
    path: &Path,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let effective_options = with_default_write_converters(options);
    let options = &effective_options;
    validate_excel_row_schema::<T>()?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::write::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xlsx_onto_template_package::<T, I>(path, None, options, rows, handlers)?;
    } else {
        let mut workbook = easyexcel_xlsx::xlsx::generation::new_workbook();
        let holder_scope = HandlerHolderScope::new_resolved::<T>(
            path,
            i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
            None,
            options,
        )?;
        write_sheet_to_workbook::<T, I>(
            &mut workbook,
            options,
            rows,
            handlers,
            Some(&holder_scope),
        )?;
        save_workbook(&mut workbook, path, options.password.as_deref())?;
    }
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}

/// Writes typed rows to an arbitrary XLSX byte stream.
///
/// `logical_path` is used only by write-handler contexts. Unlike the path
/// entry point this function writes the OOXML package to `output` itself, so
/// it is suitable for HTTP response bodies and in-memory buffers.
///
/// # Errors
///
/// Returns a conversion, handler, worksheet-configuration, XLSX-format,
/// encryption, or stream I/O error.
pub fn write_xlsx_to_writer<T, I, W>(
    logical_path: &Path,
    mut output: W,
    options: &WriteOptions,
    rows: I,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
    W: Write + Send,
{
    let effective_options = with_default_write_converters(options);
    let options = &effective_options;
    validate_excel_row_schema::<T>()?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(logical_path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::write::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        #[rustfmt::skip]
        write_xlsx_onto_template_package::<T, I>(logical_path, Some(&mut output), options, rows, handlers)?;
    } else {
        let mut workbook = easyexcel_xlsx::xlsx::generation::new_workbook();
        let holder_scope = HandlerHolderScope::new_resolved::<T>(
            logical_path,
            i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
            None,
            options,
        )?;
        write_sheet_to_workbook::<T, I>(
            &mut workbook,
            options,
            rows,
            handlers,
            Some(&holder_scope),
        )?;
        save_workbook_to_writer(&mut workbook, &mut output, options.password.as_deref())?;
    }
    after_workbook(handlers, &workbook_context)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{DynamicRow, DynamicValue};
    use crate::write::write_options::WriteOptions;
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xlsx_to_writer_emits_xlsx_bytes() {
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("write must succeed");
        assert!(!output.get_ref().is_empty());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::{DynamicRow, DynamicValue};
    use crate::write::write_options::WriteOptions;
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xlsx_to_writer_uses_template_when_provided() {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "h").expect("write");
        let template = workbook.save_to_buffer().expect("buffer");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(template),
            ..WriteOptions::default()
        };
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xlsx_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("template write must succeed");
        assert!(!output.get_ref().is_empty());
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    use crate::core::{CellValue, ExcelColumn, ExcelError, ExcelRow, ExcelWriteMetadata};

    /// 两列 typed 行：配合错误的 `dynamic_head` 让 `new_resolved` 校验失败。
    struct WideHeadRow {
        cells: Vec<CellValue>,
    }

    impl ExcelRow for WideHeadRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[
                ExcelColumn::new("first", "First", Some(0), 0, None),
                ExcelColumn::new("second", "Second", Some(1), 0, None),
            ];
            COLUMNS
        }

        fn write_metadata() -> &'static ExcelWriteMetadata {
            const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
            &METADATA
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            Ok(Self { cells: Vec::new() })
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(self.cells.clone())
        }
    }

    #[test]
    fn write_xlsx_to_writer_rejects_mismatched_dynamic_head() {
        // 对应 Java：dynamic_head 路径数少于 schema 列数时 holder 初始化必须失败。
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![vec!["Only".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut output = Vec::new();
        let result = write_xlsx_to_writer::<WideHeadRow, _, _>(
            std::path::Path::new("logical.xlsx"),
            &mut output,
            &options,
            vec![WideHeadRow { cells: Vec::new() }],
            &mut [],
        );
        assert!(matches!(result, Err(ExcelError::Format(_))));
        assert!(output.is_empty());
    }

    /// 直接调用 `ExcelRow` 的三个 trait 方法。
    ///
    /// 对应 Java：`fromRow`/`toRow`/`writeMetadata` 在写入主路径中分别被
    /// 读取侧与转换侧使用；这里直接覆盖调用本身。
    #[test]
    fn wide_head_row_trait_methods_are_reachable() {
        let row = WideHeadRow {
            cells: vec![CellValue::String("v".to_owned())],
        };
        assert_eq!(
            <WideHeadRow as ExcelRow>::write_metadata(),
            &ExcelWriteMetadata::new()
        );
        assert_eq!(
            row.to_row().expect("to_row"),
            vec![CellValue::String("v".to_owned())]
        );
        let restored = WideHeadRow::from_row(&crate::core::RowData::new(
            "s",
            0,
            Vec::new(),
            std::sync::Arc::new(std::collections::HashMap::new()),
        ))
        .expect("from_row");
        assert!(restored.cells.is_empty());
    }
}
