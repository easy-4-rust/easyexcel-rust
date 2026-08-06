include!("state_and_conversion/write_global_flags.rs");



/// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the worksheet name after applying [`WriteOptions::auto_trim`].
pub(crate) fn effective_sheet_name(options: &WriteOptions) -> String {
    if options.auto_trim {
        easyexcel_utils::string_utils::java_trim(&options.sheet_name).to_owned()
    } else {
        options.sheet_name.clone()
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn validate_stateful_backend(is_csv: bool, password: Option<&str>) -> Result<()> {
    match (is_csv, password.is_some()) {
        (true, true) => Err(ExcelError::Unsupported(
            "password protection is not supported for CSV".to_owned(),
        )),
        // XLS password is now supported via BIFF8 RC4 (Phase 5.3)
        _ => Ok(()),
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn uses_constant_memory_spill(options: &WriteOptions) -> bool {
    options.constant_memory || options.compress_temp_files
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn validate_stateful_schema(
    sheet_name: &str,
    state: &StatefulSheetState,
    schema: &'static [ExcelColumn],
) -> Result<()> {
    if state.schema == schema {
        Ok(())
    } else {
        Err(ExcelError::Format(format!(
            "worksheet schema changed between writes: {sheet_name}"
        )))
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn with_default_write_converters(options: &WriteOptions) -> WriteOptions {
    let mut effective = options.clone();
    effective.converters =
        crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters);
    effective
}

/// 线程安全的内存输出缓冲区，用于兼容无文件目标的写入编排。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) type CapturedOutput = easyexcel_io::SharedByteBuffer;
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn take_captured_output(output: &CapturedOutput) -> Result<Vec<u8>> {
    output.take().map_err(ExcelError::Io)
}

include!("state_and_conversion/prepared_write_row.rs");
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn convert_row_at<T>(
    row: &T,
    converters: &ConverterRegistry,
    sheet_name: &str,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<(Vec<CellValue>, Vec<WriteCellData>)>
where
    T: ExcelRow,
{
    let selected_schema_indexes = (!T::schema().is_empty()).then(|| {
        columns
            .iter()
            .map(|(_, schema_index, _)| *schema_index)
            .collect::<Vec<_>>()
    });
    row.to_excel_write_row_selected(converters, selected_schema_indexes.as_deref())
        .map_err(|error| {
            let ExcelError::Data {
                column,
                field,
                value,
                message,
                ..
            } = error
            else {
                return error;
            };
            let physical_column = columns
                .iter()
                .find(|(_, _, candidate)| candidate.field == field)
                .map(|(physical, _, _)| *physical)
                .or(column);
            ExcelError::Data {
                sheet: sheet_name.to_owned(),
                row: row_index,
                column: physical_column,
                field,
                value,
                message,
            }
        })
}

// 泛型行对象按值传入是宏生成的调用惯例，改引用会改变泛型约束
#[allow(clippy::needless_pass_by_value)]
fn prepare_write_row<T>(
    row: T,
    converters: &ConverterRegistry,
    sheet_name: &str,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<PreparedWriteRow>
where
    T: ExcelRow,
{
    if row.is_absent_row() {
        return Ok(PreparedWriteRow {
            absent: true,
            original_cells: Vec::new(),
            cells: Vec::new(),
        });
    }
    let (original_cells, cells) = convert_row_at(&row, converters, sheet_name, row_index, columns)?;
    Ok(PreparedWriteRow {
        absent: false,
        original_cells,
        cells,
    })
}
