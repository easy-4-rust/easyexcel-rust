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
pub use holder::{ExcelHolder, HolderEnum};

pub mod anchor_type;
pub use anchor_type::*;
pub mod cell_extra;
pub use cell_extra::*;
pub mod cell_value;
pub use cell_value::*;
pub mod client_anchor_data;
pub use client_anchor_data::*;
pub mod comment_data;
pub use comment_data::*;
pub mod coordinate_data;
pub use coordinate_data::*;
pub mod dynamic_row;
pub use dynamic_row::*;
pub mod dynamic_value;
pub use dynamic_value::*;
pub mod excel_border_style;
pub use excel_border_style::*;
pub mod excel_cell_style;
pub use excel_cell_style::*;
pub mod excel_color;
pub use excel_color::*;
pub mod excel_column;
pub use excel_column::*;
pub mod excel_data_format;
pub use excel_data_format::*;
pub mod excel_fill_pattern;
pub use excel_fill_pattern::*;
pub mod excel_font_script;
pub use excel_font_script::*;
pub mod excel_font_style;
pub use excel_font_style::*;
pub mod excel_horizontal_alignment;
pub use excel_horizontal_alignment::*;
pub mod excel_row;
pub use excel_row::*;
pub mod excel_underline;
pub use excel_underline::*;
pub mod excel_vertical_alignment;
pub use excel_vertical_alignment::*;
pub mod excel_write_head_property;
pub use excel_write_head_property::*;
pub mod excel_write_metadata;
pub use excel_write_metadata::*;
pub mod formula_data;
pub use formula_data::*;
pub mod hyperlink_data;
pub use hyperlink_data::*;
pub mod image_data;
pub use image_data::*;
pub mod image_type;
pub use image_type::*;
pub mod interval_font;
pub use interval_font::*;
pub mod read_cell_data;
pub use read_cell_data::*;
pub mod rich_text_string_data;
pub use rich_text_string_data::*;
pub mod row_data;
pub use row_data::*;
pub mod write_font;
pub use write_font::*;
