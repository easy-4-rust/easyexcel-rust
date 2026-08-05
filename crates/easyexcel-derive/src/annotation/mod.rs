//! `#[excel(...)]` 辅助属性的编译期模型与解析器。

mod conditional_parser;
mod data_validation_parser;
mod excel_ignore;
mod excel_ignore_unannotated;
mod excel_property;
mod extension_options;
mod field_options;
mod format;
mod integer;
mod struct_options;
mod style_parser;
mod write;

pub(crate) use field_options::{FieldOptions, parse_field_options};
pub(crate) use integer::SignedInteger;
pub(crate) use struct_options::{StructOptions, parse_struct_options};
pub(crate) use style_parser::number_rounding_mode_tokens;
