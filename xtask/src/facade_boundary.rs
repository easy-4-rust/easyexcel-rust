//! `easyexcel` 门面与基础引擎 crate 的依赖边界审计。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::TaskResult;

include!("facade_boundary/facade_manifest_to_xlsx_handler_adapters.rs");
include!("facade_boundary/audit.rs");
include!("facade_boundary/read_to_require_tree_absent_case_insensitive.rs");
