//! 对应 Java：`com.alibaba.excel.metadata.*` sub-packages.

pub mod abstract_cell;
pub mod abstract_holder;
pub mod abstract_parameter_builder;
pub mod basic_parameter;
pub mod cell;
pub mod cell_range;
pub mod configuration_holder;
pub mod csv;
pub mod data;
pub mod field_cache;
pub mod field_wrapper;
pub mod fill;
pub mod font;
pub mod format;
pub mod global_configuration;
pub mod head;
pub mod holder;
pub mod null_object;
pub mod property;

#[cfg(test)]
mod tests;

pub use abstract_cell::AbstractCell;
pub use abstract_holder::AbstractHolder;
pub use abstract_parameter_builder::{AbstractParameterBuilder, BasicParameterBuilder};
pub use basic_parameter::BasicParameter;
pub use cell::Cell;
pub use cell_range::CellRange;
pub use configuration_holder::{ConfigurationHolder, MetadataHolder};
pub use field_cache::FieldCache;
pub use field_wrapper::FieldWrapper;
pub use fill::AnalysisCell;
pub use global_configuration::GlobalConfiguration;
pub use head::Head;
pub use null_object::NullObject;

pub use property::{
    ColumnWidthProperty, DateTimeFormatProperty, ExcelContentProperty, ExcelHeadProperty,
    ExcelReadHeadProperty, FontProperty, LoopMergeProperty, NumberFormatProperty,
    NumberRoundingMode, OnceAbsoluteMergeProperty, RowHeightProperty, StyleProperty,
};

pub use data::{CellData, DataFormatData};
pub use font::Font;
pub use holder::Holder;

pub use data::anchor_type::AnchorType;
pub mod cell_extra;
pub use cell_extra::{CellExtra, CellExtraType};
pub use data::cell_value::CellValue;
pub use data::client_anchor_data::ClientAnchorData;
pub use data::comment_data::CommentData;
pub use data::coordinate_data::CoordinateData;
pub use data::dynamic_row::DynamicRow;
pub use data::dynamic_value::DynamicValue;
pub mod excel_border_style;
pub use excel_border_style::ExcelBorderStyle;
pub mod excel_cell_style;
pub use excel_cell_style::ExcelCellStyle;
pub mod excel_color;
pub use excel_color::ExcelColor;
pub mod excel_column;
pub use excel_column::ExcelColumn;
pub mod excel_data_format;
pub use excel_data_format::ExcelDataFormat;
pub mod excel_fill_pattern;
pub use excel_fill_pattern::ExcelFillPattern;
pub mod excel_font_script;
pub use excel_font_script::ExcelFontScript;
pub mod excel_font_style;
pub use excel_font_style::ExcelFontStyle;
pub mod excel_horizontal_alignment;
pub use excel_horizontal_alignment::ExcelHorizontalAlignment;
pub mod excel_row;
pub use excel_row::ExcelRow;
pub mod excel_underline;
pub use excel_underline::ExcelUnderline;
pub mod excel_vertical_alignment;
pub use excel_vertical_alignment::ExcelVerticalAlignment;
pub mod excel_write_head_property;
pub use excel_write_head_property::ExcelWriteHeadProperty;
pub mod excel_write_metadata;
pub use data::formula_data::FormulaData;
pub use data::hyperlink_data::{HyperlinkData, HyperlinkType};
pub use data::image_data::ImageData;
pub use data::image_type::ImageType;
pub use data::interval_font::IntervalFont;
pub use data::read_cell_data::ReadCellData;
pub use data::rich_text_string_data::RichTextStringData;
pub use data::row_data::RowData;
pub use data::write_font::WriteFont;
pub use excel_write_metadata::ExcelWriteMetadata;
