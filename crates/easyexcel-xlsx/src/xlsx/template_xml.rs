//! XLSX 模板工作表 XML 修改原语。
//!
//! 输入使用中立值和坐标，不依赖 `EasyExcel` builder、handler 或 annotation。

use std::fmt::Write as _;

use easyexcel_io::{Error, Result};

include!("template_xml/template_rich_text.rs");
include!("template_xml/templatecellvalue_to_remove_attribute.rs");
include!("template_xml/cell_references.rs");
