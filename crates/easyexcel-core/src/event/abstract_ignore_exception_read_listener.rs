//! Mirrors Java `com.alibaba.excel.event.AbstractIgnoreExceptionReadListener`.

use crate::analysis_context::AnalysisContext;
use crate::cell_extra::CellExtra;
use crate::read_listener::ReadListener;
use std::collections::HashMap;

/// 忽略异常的读取监听器：吞掉异常并继续处理，对应 Java `AbstractIgnoreExceptionReadListener`。
pub trait AbstractIgnoreExceptionReadListener<T>: ReadListener<T> {
    /// 静默处理读取过程中发生的异常，默认实现不做任何事。
    fn on_exception_silent(
        &mut self,
        error: &crate::excel_error::ExcelError,
        context: &AnalysisContext,
    ) {
        let _ = (error, context);
    }
    /// 静默处理单元格附加信息，默认实现不做任何事。
    fn extra_silent(&mut self, extra: &CellExtra, context: &AnalysisContext) {
        let _ = (extra, context);
    }
}

fn _import_marker(m: HashMap<usize, String>) {
    let _ = m;
}
