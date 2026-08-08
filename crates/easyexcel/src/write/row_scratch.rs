//! XLSX 流式写入的可复用行缓冲区。

use crate::core::{
    CellValue, ConverterRegistry, ExcelColumn, ExcelError, ExcelRow, Result, WriteCellData,
};

/// 复用一批行之间的原始值和转换后单元格容器。
///
/// 对应 Java：无直接对应对象；Rust 性能扩展。Java 的
/// `ExcelWriteAddExecutor` 会逐行构造容器；Rust 在不改变转换器、错误位置及
/// Handler 可见值的前提下复用容量。
pub(crate) struct RowScratch {
    original_cells: Vec<CellValue>,
    converted_cells: Vec<WriteCellData>,
}

impl RowScratch {
    /// 按静态 schema 宽度预留容量。
    #[must_use]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            original_cells: Vec::with_capacity(capacity),
            converted_cells: Vec::with_capacity(capacity),
        }
    }

    /// 转换一行并保留 Java 兼容的 sheet/row/column 错误位置。
    pub(crate) fn convert<T>(
        &mut self,
        row: &T,
        converters: &ConverterRegistry,
        sheet_name: &str,
        row_index: u32,
        columns: &[(usize, usize, &'static ExcelColumn)],
        selected_schema_indexes: Option<&[usize]>,
    ) -> Result<()>
    where
        T: ExcelRow,
    {
        row.write_excel_row_into(
            converters,
            selected_schema_indexes,
            &mut self.original_cells,
            &mut self.converted_cells,
        )
        .map_err(|error| remap_write_error(error, sheet_name, row_index, columns))
    }

    /// 返回转换器执行前的字段值，供完整 Handler 生命周期使用。
    #[must_use]
    pub(crate) fn original_cells(&self) -> &[CellValue] {
        &self.original_cells
    }

    /// 返回转换后的写单元格数据。
    #[must_use]
    pub(crate) fn converted_cells(&self) -> &[WriteCellData] {
        &self.converted_cells
    }
}

fn remap_write_error(
    error: ExcelError,
    sheet_name: &str,
    row_index: u32,
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> ExcelError {
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
}
