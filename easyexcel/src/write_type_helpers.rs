//! 文件扩展名 / `ExcelTypeEnum` 解析 helper。
//!
//! 对应 Java：`com.alibaba.excel.support.ExcelTypeEnum` 在写入入口处的判定逻辑。
//! 这些 helper 在 facade 层被多个 builder 共用，集中放在本文件以避免循环依赖。

use std::path::Path;

use crate::core::support::ExcelTypeEnum;
use crate::writer::WriteOptions;

/// 路径是否指向 CSV（不区分大小写）。
pub(crate) fn is_csv_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}

/// 路径是否指向旧版 XLS（不区分大小写）。
pub(crate) fn is_xls_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xls"))
}

/// 写入是否走 CSV：显式 `excel_type` 优先，否则按路径扩展名判断。
pub(crate) fn is_csv_write(path: &Path, options: &WriteOptions) -> bool {
    match options.excel_type {
        Some(excel_type) => excel_type == ExcelTypeEnum::Csv,
        None => is_csv_path(path),
    }
}

/// 写入是否走 XLS：显式 `excel_type` 优先，否则按路径扩展名判断。
pub(crate) fn is_xls_write(path: &Path, options: &WriteOptions) -> bool {
    match options.excel_type {
        Some(excel_type) => excel_type == ExcelTypeEnum::Xls,
        None => is_xls_path(path),
    }
}

/// 计算写入实际使用的 `ExcelTypeEnum`（CSV / XLS / XLSX）。
pub(crate) fn effective_write_type(path: &Path, options: &WriteOptions) -> ExcelTypeEnum {
    if is_csv_write(path, options) {
        ExcelTypeEnum::Csv
    } else if is_xls_write(path, options) {
        ExcelTypeEnum::Xls
    } else {
        ExcelTypeEnum::Xlsx
    }
}
