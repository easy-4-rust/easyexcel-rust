//! Java `annotation.write.style` 注解解析入口。

mod column_width;
mod content_font_style;
mod content_loop_merge;
mod content_row_height;
mod content_style;
mod head_font_style;
mod head_row_height;
mod head_style;
mod once_absolute_merge;

pub(crate) use column_width::{
    parse_field as parse_field_column_width, parse_struct as parse_struct_column_width,
};
pub(crate) use content_font_style::{
    parse_field as parse_field_content_font_style, parse_struct as parse_struct_content_font_style,
};
pub(crate) use content_loop_merge::parse as parse_content_loop_merge;
pub(crate) use content_row_height::parse as parse_content_row_height;
pub(crate) use content_style::{
    parse_field as parse_field_content_style, parse_struct as parse_struct_content_style,
};
pub(crate) use head_font_style::{
    parse_field as parse_field_head_font_style, parse_struct as parse_struct_head_font_style,
};
pub(crate) use head_row_height::parse as parse_head_row_height;
pub(crate) use head_style::{
    parse_field as parse_field_head_style, parse_struct as parse_struct_head_style,
};
pub(crate) use once_absolute_merge::parse as parse_once_absolute_merge;
