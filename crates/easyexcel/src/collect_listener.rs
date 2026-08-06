//! 同步读取用的内建 `ReadListener`，将所有行收集到 `Vec`。
//!
//! 对应 Java：`EasyExcel.readSync(...)` 内部使用的收集型监听器
//! （Java 端无单独公开类型，由 `EasyExcel.readSync` 隐式装配）。

use crate::core::{AnalysisContext, ReadListener, Result};

/// 对应 Java：`EasyExcel.readSync(...)`。 同步读取内部使用的收集型监听器。
///
/// 字段对 crate 内可见以便单元测试直接构造与断言。
pub(crate) struct CollectListener<T>(pub(crate) Vec<T>);

impl<T> ReadListener<T> for CollectListener<T> {
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> Result<()> {
        self.0.push(data);
        Ok(())
    }
}

/// 对应 Java：`EasyExcel.readSync(...)`。 暴露给 [`crate::ExcelSyncReaderBuilder`] 使用的收集入口。
pub(crate) fn collect_listener<T>() -> CollectListener<T> {
    CollectListener(Vec::new())
}

/// 对应 Java：`EasyExcel.readSync(...)`。 取出监听器内部已收集的行。
pub(crate) fn drain_listener<T>(listener: CollectListener<T>) -> Vec<T> {
    listener.0
}
