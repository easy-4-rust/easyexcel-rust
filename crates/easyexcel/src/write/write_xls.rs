//! XLS 写入功能。
//!
//! 对应 Java：`com.alibaba.excel.ExcelWriter` 的 XLS 写入路径。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/ExcelWriter.java

use std::io::Write;
use std::path::Path;

use crate::core::{ExcelError, ExcelRow, Result, WriteHandler, WriteWorkbookContext};
use crate::write::xls_adapter::Biff8Book;
use crate::write::excel_writer_core::{
    HandlerHolderScope, after_workbook, after_workbook_create, before_workbook, sort_handlers,
    validate_excel_row_schema, validate_xls_options, with_default_write_converters,
    write_sheet_to_biff8_book, write_xls_onto_template,
};
use crate::write::write_options::WriteOptions;

/// 将类型化行写入 BIFF8 (`.xls`) 文件。
///
/// # Errors
///
/// 返回转换、校验、BIFF8 格式或 I/O 错误。
pub fn write_xls<T, I>(path: &Path, options: &WriteOptions, rows: I) -> Result<()>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    write_xls_with_handlers(path, options, rows, &mut [])
}

/// Writes typed rows to a BIFF8 file while invoking ordered write handlers.
///
/// When [`WriteOptions`] carries a template, uses
/// the `easyexcel-xls` BIFF8 template engine (Java `withTemplate` + `doWrite` on HSSF).
/// Password protection remains [`ExcelError::Unsupported`].
///
/// # Errors
///
/// Returns a conversion, handler, BIFF8-format, template, or I/O error.
pub fn write_xls_with_handlers<T, I>(
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
    validate_xls_options(options)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::write::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xls_onto_template::<T, I>(path, None, options, rows, handlers)?;
        after_workbook(handlers, &workbook_context)?;
        return Ok(());
    }

    let mut book = Biff8Book::default();
    #[rustfmt::skip]
    let holder_scope = HandlerHolderScope::new_resolved::<T>(path, i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX), None, options)?;
    write_sheet_to_biff8_book::<T, I>(&mut book, options, rows, handlers, Some(&holder_scope))?;
    book.save_to_path_with_password(path, options.password.as_deref())
        .map_err(ExcelError::from)?;
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}

/// Writes typed rows as BIFF8 bytes to an arbitrary writer.
///
/// # Errors
///
/// Returns a conversion, handler, BIFF8-format, or stream I/O error.
pub fn write_xls_to_writer<T, I, W>(
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
    validate_xls_options(options)?;
    sort_handlers(handlers);
    let workbook_context = WriteWorkbookContext::new(logical_path);
    before_workbook(handlers, &workbook_context)?;
    after_workbook_create(handlers, &workbook_context)?;

    if crate::write::template_write::has_template(
        options.template_file.as_deref(),
        options.template_bytes.as_deref(),
    ) {
        write_xls_onto_template::<T, I>(logical_path, Some(&mut output), options, rows, handlers)?;
        after_workbook(handlers, &workbook_context)?;
        return Ok(());
    }

    let mut book = Biff8Book::default();
    let holder_scope = HandlerHolderScope::new_resolved::<T>(
        logical_path,
        i32::try_from(options.sheet_index.unwrap_or(0)).unwrap_or(i32::MAX),
        None,
        options,
    )?;
    write_sheet_to_biff8_book::<T, I>(&mut book, options, rows, handlers, Some(&holder_scope))?;
    book.write_to_and_flush(&mut output)?;
    after_workbook(handlers, &workbook_context)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{DynamicRow, DynamicValue};
    use crate::write::write_options::WriteOptions;
    use std::collections::BTreeMap;
    use std::io::{Cursor, Read};

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xls_to_writer_emits_biff8_bytes() {
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            ..WriteOptions::default()
        };
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xls_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("write must succeed");
        assert!(!output.get_ref().is_empty());
    }

    #[test]
    fn freeze_head_emits_pane_record_in_written_xls() {
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            freeze_head: true,
            need_head: true,
            // DynamicRow 无静态 schema：显式表头 → head_rows=1 → 冻结首行
            dynamic_head: Some(vec![vec!["Name".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xls_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xls"),
            &mut output,
            &options,
            vec![dynamic_row()],
            &mut [],
        )
        .expect("write must succeed");
        // 从 CFB 容器提取 Workbook 流并扫描记录
        let mut cfb =
            cfb::CompoundFile::open(Cursor::new(output.get_ref().as_slice())).expect("valid cfb");
        let mut stream = cfb.open_stream("Workbook").expect("Workbook stream");
        let mut stream_bytes = Vec::new();
        stream
            .read_to_end(&mut stream_bytes)
            .expect("read Workbook stream");
        let stream = stream_bytes;
        let mut pane = None;
        let mut window2 = None;
        let mut i = 0;
        while i + 4 <= stream.len() {
            let typ = u16::from_le_bytes([stream[i], stream[i + 1]]);
            let len = u16::from_le_bytes([stream[i + 2], stream[i + 3]]) as usize;
            let data = &stream[i + 4..i + 4 + len];
            if typ == 0x0041 {
                pane = Some(data.to_vec());
            }
            if typ == 0x023E {
                window2 = Some(data.to_vec());
            }
            i += 4 + len;
        }
        // 冻结表头 1 行: PANE = px=0 py=1 rwTop=1 colLeft=0 pnnAct=2
        assert_eq!(
            pane.as_deref(),
            Some(&[0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00][..]),
            "freeze_head 写出 PANE 记录"
        );
        let w2 = window2.expect("WINDOW2 必须存在");
        let window_options = u16::from_le_bytes([w2[0], w2[1]]);
        assert_eq!(window_options & 0x0008, 0x0008, "WINDOW2 fFrozen 位置位");
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::{DynamicRow, DynamicValue};
    use crate::write::xls_adapter::Biff8Book;
    use crate::write::write_options::WriteOptions;
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn dynamic_row() -> DynamicRow {
        let mut values = BTreeMap::new();
        values.insert(0, DynamicValue::String("alice".to_owned()));
        DynamicRow::new(values)
    }

    #[test]
    fn write_xls_to_writer_uses_template_when_provided() {
        let mut book = Biff8Book::default();
        book.sheet_mut("Sheet1");
        let template = book.to_cfb_bytes().expect("template");
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            template_bytes: Some(template),
            ..WriteOptions::default()
        };
        let mut output = Cursor::new(Vec::<u8>::new());
        write_xls_to_writer::<DynamicRow, _, _>(
            std::path::Path::new("logical.xls"),
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
    fn write_xls_to_writer_rejects_mismatched_dynamic_head() {
        // 对应 Java：dynamic_head 路径数少于 schema 列数时
        // `WriteContextImpl.initSheet` 前的 head 校验必须失败。
        let options = WriteOptions {
            sheet_name: "Sheet1".to_owned(),
            dynamic_head: Some(vec![vec!["Only".to_owned()]]),
            ..WriteOptions::default()
        };
        let mut output = Vec::new();
        let result = write_xls_to_writer::<WideHeadRow, _, _>(
            std::path::Path::new("logical.xls"),
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
