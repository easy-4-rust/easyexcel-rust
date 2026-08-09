//! 对应 Java：`com.alibaba.excel.read.listener.PageReadListener<T>`.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::analysis_context::{AnalysisContext, Result};
use crate::core::read_listener::ReadListener;

/// Java `PageReadListener.BATCH_COUNT` 的线程安全 Rust 载体。
///
/// Java 字段允许调用方在构造默认 listener 前修改；Rust 使用原子值保留相同的
/// 进程级默认配置，同时避免可变全局变量的数据竞争。
pub static BATCH_COUNT: AtomicUsize = AtomicUsize::new(100);

/// A listener that buffers rows and invokes a callback page by page.
///
/// 对应 Java：`PageReadListener<T>(Consumer<List<T>> consumer, int batchCount)`
/// with `BATCH_COUNT = 100`.
pub struct PageReadListener<T> {
    batch_size: usize,
    batch_index: usize,
    rows: Vec<T>,
    callback: Box<PageCallback<T>>,
}

/// Callback signature for [`PageReadListener`].
type PageCallback<T> = dyn FnMut(Vec<T>, &AnalysisContext) -> Result<()>;

impl<T> PageReadListener<T> {
    /// 使用当前全局 `BATCH_COUNT` 和 Java `Consumer<List<T>>` 形状创建 listener。
    #[must_use]
    pub fn from_consumer(mut consumer: impl FnMut(Vec<T>) + 'static) -> Self {
        Self::from_consumer_with_batch_count(
            BATCH_COUNT.load(Ordering::Relaxed),
            move |rows| consumer(rows),
        )
    }

    /// 使用显式批量大小和 Java `Consumer<List<T>>` 形状创建 listener。
    #[must_use]
    pub fn from_consumer_with_batch_count(
        batch_size: usize,
        mut consumer: impl FnMut(Vec<T>) + 'static,
    ) -> Self {
        Self::new(batch_size, move |rows, _context| {
            consumer(rows);
            Ok(())
        })
    }

    /// 对应 Java：com.alibaba.excel.read.listener.`PageReadListener<T>`。 Creates a paged listener. A zero size is normalized to one row. (Java `PageReadListener(Consumer, int)`)
    #[must_use]
    pub fn new(
        batch_size: usize,
        callback: impl FnMut(Vec<T>, &AnalysisContext) -> Result<()> + 'static,
    ) -> Self {
        let batch_size = batch_size.max(1);
        Self {
            batch_size,
            batch_index: 0,
            rows: Vec::with_capacity(batch_size),
            callback: Box::new(callback),
        }
    }

    fn flush(&mut self, context: &AnalysisContext) -> Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let rows = std::mem::replace(&mut self.rows, Vec::with_capacity(self.batch_size));
        let context = context.with_batch_index(self.batch_index);
        complete_page(&mut self.batch_index, (self.callback)(rows, &context))
    }
}

impl<T> ReadListener<T> for PageReadListener<T> {
    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        self.rows.push(data);
        if self.rows.len() >= self.batch_size {
            return self.flush(context);
        }
        Ok(())
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        self.flush(context)
    }
}

fn complete_page(batch_index: &mut usize, result: Result<()>) -> Result<()> {
    result.map(|()| {
        *batch_index += 1;
    })
}
