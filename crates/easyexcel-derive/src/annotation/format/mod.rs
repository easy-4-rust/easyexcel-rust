//! Java `annotation.format` 注解解析入口。

mod date_time_format;
mod number_format;

pub(crate) use date_time_format::parse as parse_date_time_format;
pub(crate) use number_format::parse as parse_number_format;
