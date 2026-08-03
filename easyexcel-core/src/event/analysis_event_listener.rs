//! 对应 Java：`com.alibaba.excel.event.AnalysisEventListener`.

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

#[cfg(test)]
mod tests_extra {
    use super::*;

    /// 对应 Java：仅实现必需方法的监听器，验证默认回调
    struct DefaultListener;

    impl crate::ReadListener<crate::CellValue> for DefaultListener {
        fn invoke(
            &mut self,
            _data: crate::CellValue,
            _context: &crate::AnalysisContext,
        ) -> crate::Result<()> {
            Ok(())
        }
    }

    impl AnalysisEventListener<crate::CellValue> for DefaultListener {}

    #[test]
    fn default_callbacks_are_noops() {
        // 对应 Java：AnalysisEventListener 默认回调不做任何事
        let mut listener = DefaultListener;
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        listener.invoke_head_map(
            std::collections::HashMap::from([(0, "Name".to_owned())]),
            &context,
        );
        listener.do_after_all_analysed(&context).expect("after ok");
        _import_marker(crate::CellValue::Int(1));
    }

    #[test]
    fn invoke_callback_returns_ok() {
        // 对应 Java：ReadListener.invoke 数据行回调返回 Ok
        use crate::read_listener::ReadListener;
        let mut listener = DefaultListener;
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        listener
            .invoke(crate::CellValue::Int(1), &context)
            .expect("invoke ok");
    }
}
