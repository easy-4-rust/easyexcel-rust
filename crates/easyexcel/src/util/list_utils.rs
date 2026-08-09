//! Java `ListUtils` 门面重导出。

pub use easyexcel_utils::list_utils::{
    new_array_list, new_array_list_from_iter, new_array_list_from_slice,
    new_array_list_with_capacity, new_array_list_with_expected_size,
};

// Java 将该校验放在 ListUtils；Rust 复用统一的 ExcelError 校验入口，
// 避免在基础集合 crate 中反向依赖 facade 错误类型。
pub use super::validate::check_not_null;
