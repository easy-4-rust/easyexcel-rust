//! 面向上层框架的 XLSX 单元格事件读取引擎。
//!
//! 本模块拥有 OOXML 包关系、XML 事件、共享字符串、样式格式和工作表附加
//! 信息解析。上层门面只需把中立事件映射到自己的 metadata/listener 类型。

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Seek};

use bigdecimal::BigDecimal;
use easyexcel_cache::{
    ReadCacheMode, SharedStringCache, SharedStringCacheReader, create_cache, memory_cache,
};
use easyexcel_format::{
    CompiledExcelFormat, SpreadsheetLocale, compile_format_code, excel_display_number,
    format_with_compiled, is_date_format_code, is_scientific_magnitude, java_plain_extreme_format,
    java_scientific_format, resolve_builtin_format_code,
};
use easyexcel_io::{Error, Result};
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Decoder, Reader as XmlReader, XmlVersion};

use super::package::{RawRelationships, Relationships, relationship_part_name, resolve_target};
use super::package_reader::XlsxPackageReader;
use super::{
    decode_ooxml_escape, dimension_last_row as parse_dimension_last_row, parse_a1_cell_range,
    parse_a1_cell_reference, parse_xlsx_index, parse_xlsx_row_number as parse_row_number,
};

include!("event_reader/readseek_to_read_comments.rs");
include!("event_reader/parse_comments_to_xlsx_error.rs");
