//! 薄重导出层：保持 `crate::core::Type` 路径兼容性。
//!
//! 目录重排后，原 core/ 下的类型已拆分到 Java 对应的顶级模块。
//! 本文件通过 glob 重导出保持向后兼容（`use crate::core::*` 仍有效）。

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

// 从新顶级模块重导出（保持 crate::core::Type 路径兼容）
pub use crate::converters::*;
pub use crate::enums::*;
pub use crate::event::*;
pub use crate::metadata::*;
pub use crate::support::*;

// Result 类型别名
pub type Result<T> = std::result::Result<T, crate::support::excel_error::ExcelError>;

// write/ 相关类型
pub use crate::write::write_backend_handle::*;
pub use crate::write::write_cell_context::*;
pub use crate::write::write_context::*;
pub use crate::write::write_fill_executor::*;
pub use crate::write::write_handler::*;
pub use crate::write::write_holder_context::*;
pub use crate::write::write_row_context::*;
pub use crate::write::write_sheet_context::*;
pub use crate::write::write_workbook_context::*;

// metadata 子包重导出
pub use crate::metadata::property::{
    ExcelDataValidationMeta, LoopMergeProperty, NumberRoundingMode, OnceAbsoluteMergeProperty,
};

// 具名类型重导出
pub use crate::converters::converter_registry::ConverterRegistry;
pub use crate::converters::nullable_object_converter::NullableObjectConverter;
pub use crate::enums::enum_cache_location::CacheLocation;
pub use crate::enums::enum_holder::Holder;
pub use crate::event::analysis_context::AnalysisContext;
pub use crate::event::read_listener::ReadListener;
pub use crate::metadata::cell_extra::{CellExtra, CellExtraType};
pub use crate::metadata::cell_value::CellValue;
pub use crate::metadata::excel_cell_style::ExcelCellStyle;
pub use crate::metadata::excel_font_style::ExcelFontStyle;
pub use crate::metadata::excel_row::ExcelRow;
pub use crate::metadata::write_font::WriteFont;
pub use crate::support::Empty;
pub use crate::support::csv_charset::CsvCharset;
pub use crate::support::excel_error::ExcelError;
pub use crate::write::write_cell_data::WriteCellData;

#[cfg(test)]
mod tests;

// 补充缺失的重导出
pub use crate::converters::custom_read_object::CustomReadObject;
pub use crate::enums::enum_read_default_return::ReadDefaultReturn;
pub use crate::write::write_handler::WriteHandler;
pub use crate::write::write_row_context::WriteRowContext;
pub use crate::write::write_sheet_context::WriteSheetContext;
pub use crate::write::write_workbook_context::WriteWorkbookContext;

// 补充 WriteCellContext 及相关类型
pub use crate::converters::read_converter_context::ReadConverterContext;
pub use crate::converters::write_converter_context::WriteConverterContext;
pub use crate::metadata::excel_column::ExcelColumn;
pub use crate::metadata::excel_write_metadata::ExcelWriteMetadata;
pub use crate::write::write_cell_context::WriteCellContext;

/// 任意精度十进制类型，对应 Java `BigDecimal`。
pub use ::bigdecimal::BigDecimal;
/// 任意精度整数类型，对应 Java `BigInteger`。
pub use ::num_bigint::BigInt;
