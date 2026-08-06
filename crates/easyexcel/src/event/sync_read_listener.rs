//! 对应 Java：`com.alibaba.excel.event.SyncReadListener`.

use crate::core::analysis_context::AnalysisContext;
use crate::core::read_listener::ReadListener;

/// 对应 Java：com.alibaba.excel.event.SyncReadListener。 Synchronous data reading.
///
/// Rust port of Java `SyncReadListener extends AnalysisEventListener<Object>`.
/// Java collects every row into a `List<Object>`. The Rust port mirrors
/// the same buffer so `doReadAllSync()` callers can retrieve the list.
pub struct SyncReadListener {
    list: Vec<crate::CellValue>,
}

impl SyncReadListener {
    /// Creates an empty listener.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.event.SyncReadListener。
    pub const fn new() -> Self {
        Self { list: Vec::new() }
    }

    /// 对应 Java：com.alibaba.excel.event.SyncReadListener。 Returns the collected list. (Java `getList()`)
    #[must_use]
    pub fn list(&self) -> &[crate::CellValue] {
        &self.list
    }

    /// 对应 Java：com.alibaba.excel.event.SyncReadListener。 Sets the list. (Java `setList(List)`)
    pub fn set_list(&mut self, list: Vec<crate::CellValue>) {
        self.list = list;
    }
}

impl Default for SyncReadListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadListener<crate::CellValue> for SyncReadListener {
    fn invoke(
        &mut self,
        data: crate::CellValue,
        _context: &AnalysisContext,
    ) -> crate::core::analysis_context::Result<()> {
        self.list.push(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::CellValue;

    #[test]
    fn new_default_list_and_set_list() {
        // 对应 Java：SyncReadListener 构造与 getList/setList
        let listener = SyncReadListener::new();
        assert!(listener.list().is_empty());

        let default = SyncReadListener::default();
        assert!(default.list().is_empty());

        let mut listener = SyncReadListener::new();
        listener.set_list(vec![CellValue::Int(1), CellValue::Int(2)]);
        assert_eq!(listener.list(), &[CellValue::Int(1), CellValue::Int(2)]);
    }

    #[test]
    fn invoke_collects_rows_in_order() {
        // 对应 Java：SyncReadListener.invoke 收集每一行
        let mut listener = SyncReadListener::new();
        let context = AnalysisContext::new("Sheet1", 0, 0);
        listener
            .invoke(CellValue::String("a".to_owned()), &context)
            .expect("invoke ok");
        listener
            .invoke(CellValue::Int(7), &context)
            .expect("invoke ok");
        assert_eq!(
            listener.list(),
            &[CellValue::String("a".to_owned()), CellValue::Int(7)]
        );
    }
}
