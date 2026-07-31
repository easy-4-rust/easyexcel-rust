//! Mirrors Java `com.alibaba.excel.event.AnalysisEventListener`.

use crate::CellValue;

/// 分析事件监听器：表头与全部数据行分析完成后回调，对应 Java `AnalysisEventListener`。
pub trait AnalysisEventListener<T>: crate::ReadListener<T> {
    /// 表头信息回调，默认实现不做任何事。
    fn invoke_head_map(
        &mut self,
        head_map: std::collections::HashMap<usize, String>,
        context: &crate::AnalysisContext,
    ) {
        let _ = (head_map, context);
    }
    /// 全部数据分析完成后的回调，默认实现不做任何事。
    fn do_after_all_analysed(&mut self, context: &crate::AnalysisContext) -> crate::Result<()> {
        let _ = context;
        Ok(())
    }
}

fn _import_marker(v: CellValue) {
    let _ = v;
}
