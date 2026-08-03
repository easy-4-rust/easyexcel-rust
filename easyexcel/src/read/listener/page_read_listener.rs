//! 对应 Java：`com.alibaba.excel.read.listener.PageReadListener`.

use std::collections::VecDeque;

use crate::core::{AnalysisContext, ReadListener};

/// 对应 Java：`PageReadListener<T> implements ReadListener<T>`.
///
/// Java batches rows in a list and invokes a `Consumer<List<T>>` when the
/// batch is full or on `doAfterAllAnalysed`. Rust mirrors with a
/// `VecDeque<T>` and an injected callback.
pub struct PageReadListener<T> {
    batch_size: usize,
    rows: VecDeque<T>,
    callback: Box<dyn FnMut(Vec<T>) + Send>,
}

impl<T> PageReadListener<T> {
    /// 对应 Java：`PageReadListener(Consumer<List<T>>)`.
    pub fn new(callback: impl FnMut(Vec<T>) + Send + 'static) -> Self {
        Self {
            batch_size: 100,
            rows: VecDeque::new(),
            callback: Box::new(callback),
        }
    }

    /// 对应 Java：`PageReadListener(Consumer<List<T>>, int)`.
    #[must_use]
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// Mirrors the static `BATCH_COUNT` constant.
    pub const BATCH_COUNT: usize = 100;

    /// Flushes any remaining rows. (Java `doAfterAllAnalysed` step)
    pub fn flush(&mut self) {
        if !self.rows.is_empty() {
            let drained: Vec<T> = self.rows.drain(..).collect();
            (self.callback)(drained);
        }
    }
}

impl<T: Send + 'static> ReadListener<T> for PageReadListener<T> {
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> crate::core::Result<()> {
        self.rows.push_back(data);
        if self.rows.len() >= self.batch_size {
            self.flush();
        }
        Ok(())
    }

    fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> crate::core::Result<()> {
        self.flush();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::*;

    fn context() -> AnalysisContext {
        AnalysisContext::new("Sheet1", 0, 0)
    }

    #[test]
    fn batches_rows_and_flushes_on_batch_size() {
        // 对应 Java：PageReadListener 满批触发回调，结束后 flush 剩余行
        let batches = Arc::new(Mutex::new(Vec::new()));
        let mut listener = PageReadListener::new({
            let batches = Arc::clone(&batches);
            move |batch: Vec<i32>| batches.lock().unwrap().push(batch)
        });
        for value in 0..105 {
            listener.invoke(value, &context()).expect("invoke");
        }
        assert_eq!(batches.lock().unwrap().len(), 1);
        assert_eq!(batches.lock().unwrap()[0].len(), 100);

        listener.do_after_all_analysed(&context()).expect("finish");
        let batches = batches.lock().unwrap();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[1], (100..105).collect::<Vec<_>>());
    }

    #[test]
    fn with_batch_size_clamps_to_at_least_one() {
        // 对应 Java：batchSize=0 收敛为 1
        let batches = Arc::new(Mutex::new(Vec::new()));
        let mut listener = PageReadListener::new({
            let batches = Arc::clone(&batches);
            move |batch: Vec<i32>| batches.lock().unwrap().push(batch)
        })
        .with_batch_size(0);
        listener.invoke(1, &context()).expect("invoke");
        assert_eq!(*batches.lock().unwrap(), vec![vec![1]]);
        assert_eq!(PageReadListener::<i32>::BATCH_COUNT, 100);
    }

    #[test]
    fn flush_with_empty_queue_is_noop() {
        // 对应 Java：无剩余行时 doAfterAllAnalysed 不触发回调
        let calls = Arc::new(Mutex::new(0usize));
        let mut listener = PageReadListener::new({
            let calls = Arc::clone(&calls);
            move |_: Vec<i32>| *calls.lock().unwrap() += 1
        });
        listener.flush();
        assert_eq!(*calls.lock().unwrap(), 0);
    }
}
