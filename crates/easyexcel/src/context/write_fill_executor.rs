//! 对应 Java：`com.alibaba.excel.write.executor.ExcelWriteFillExecutor`.
//!
//! The trait lives in `easyexcel-core` so `easyexcel-writer` can hold an optional
//! hook without depending on `easyexcel-template`, and the template crate can
//! provide the concrete fill implementation.

use std::any::Any;

use crate::{CellValue, ExcelError, MergeRange, Result, WriteDirection};

include!("write_fill_executor/write_fill_config.rs");

include!("write_fill_executor/write_fill_sheet.rs");

/// Hook implemented by `easyexcel-template` and wired from the `easyexcel` facade.
///
/// 对应 Java：`ExcelWriteFillExecutor.fill(Object, FillConfig)`.
pub trait WriteFillExecutor {
    /// Accumulates one scalar or collection fill against the loaded template.
    ///
    /// # Errors
    ///
    /// Returns a format error when `data` is not a supported fill payload, or a
    /// template I/O / OOXML error from the underlying engine.
    fn fill(
        &mut self,
        data: &dyn Any,
        fill_config: WriteFillConfig,
        sheet: WriteFillSheet,
    ) -> Result<()>;

    /// 在同一个模板会话中追加普通写入行。
    ///
    /// Java 的 `ExcelWriter` 允许在 `fill` 前后继续调用 `write`；因此模板
    /// executor 必须同时持有两类操作，不能让它们分别落到两个工作簿实例。
    ///
    /// 对应 Java：`ExcelWriteAddExecutor.add(...)` 与
    /// `ExcelWriteFillExecutor.fill(...)` 共享同一个 `WriteContext`。
    ///
    /// # Errors
    ///
    /// 当具体模板后端不支持普通行追加时返回兼容性错误。
    fn write_rows(&mut self, rows: Vec<Vec<CellValue>>, sheet: WriteFillSheet) -> Result<()> {
        let _ = (rows, sheet);
        Err(ExcelError::Unsupported(
            "template executor does not support ordinary row writes".to_owned(),
        ))
    }

    /// 在同一模板会话的当前工作表上增加一个绝对合并区域。
    ///
    /// 对应 Java：`ExcelBuilderImpl#merge` 直接修改当前
    /// `WriteSheetHolder` 持有的工作表；fill、普通 write 与 merge 必须共享
    /// 同一个模板对象，不能在 finish 时由另一个 writer 覆盖。
    ///
    /// # Errors
    ///
    /// 当具体模板后端不支持合并区域时返回兼容性错误。
    fn add_merge(&mut self, range: MergeRange, sheet: WriteFillSheet) -> Result<()> {
        let _ = (range, sheet);
        Err(ExcelError::Unsupported(
            "template executor does not support merged regions".to_owned(),
        ))
    }

    /// Persists accumulated fill results to the configured output target.
    ///
    /// 对应 Java：`WriteContext.finish(boolean onException)` for fill-only
    /// sessions.
    ///
    /// # Errors
    ///
    /// Returns an output, close, or package-format error.
    fn finish(&mut self, on_exception: bool) -> Result<()>;
}

/// Returns a descriptive error when no template stream is configured.
///
/// 对应 Java：`ExcelGenerateException("Calling the 'fill' method must use a template.")`.
#[must_use]
pub fn fill_requires_template_error() -> ExcelError {
    ExcelError::Unsupported("Calling the 'fill' method must use a template.".to_owned())
}

/// Returns a descriptive error when CSV fill is requested.
///
/// 对应 Java：`ExcelGenerateException("csv does not support filling data.")`.
#[must_use]
pub fn csv_fill_unsupported_error() -> ExcelError {
    ExcelError::Unsupported("csv does not support filling data.".to_owned())
}
