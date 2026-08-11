//! 显式 opt-in 的有界并行行转换 Listener。

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

use crate::core::{
    AnalysisContext, CellExtra, ErrorAction, ExcelError, ReadListener, Result,
};

struct MapJob<T> {
    sequence: u64,
    data: T,
    context: AnalysisContext,
}

struct MapResult<U> {
    sequence: u64,
    context: AnalysisContext,
    result: Result<U>,
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic payload")
}

/// 对用户显式声明为纯函数的行转换执行有界并行化，并按输入顺序串行提交。
///
/// 对应 Java：无直接对应对象；Rust 性能扩展。XML 解码和下游
/// [`ReadListener`] 回调始终保持单线程、有序；只有 `mapper` 在 worker 中并发。
/// 队列容量是硬上限，首个 worker/下游错误会触发取消。
pub struct ParallelMapReadListener<T, U, L> {
    downstream: L,
    job_tx: Option<mpsc::SyncSender<MapJob<T>>>,
    result_rx: mpsc::Receiver<MapResult<U>>,
    workers: Vec<JoinHandle<()>>,
    ready: BTreeMap<u64, MapResult<U>>,
    cancel: Arc<AtomicBool>,
    next_sequence: u64,
    next_commit: u64,
    in_flight: usize,
    queue_capacity: usize,
    marker: PhantomData<fn(T) -> U>,
}

impl<T, U, L> ParallelMapReadListener<T, U, L>
where
    T: Send + 'static,
    U: Send + 'static,
    L: ReadListener<U>,
{
    /// 创建并行映射 Listener。
    ///
    /// `worker_count` 和 `queue_capacity` 必须大于零。`mapper` 必须是无状态
    /// 或自行同步的纯转换；副作用应放在 `downstream` 中以保持顺序。
    pub fn new<F>(
        worker_count: usize,
        queue_capacity: usize,
        mapper: F,
        downstream: L,
    ) -> Result<Self>
    where
        F: Fn(T, &AnalysisContext) -> Result<U> + Send + Sync + 'static,
    {
        if worker_count == 0 || queue_capacity == 0 {
            return Err(ExcelError::Format(
                "parallel_map worker_count and queue_capacity must be greater than zero"
                    .to_owned(),
            ));
        }
        let (job_tx, job_rx) = mpsc::sync_channel::<MapJob<T>>(queue_capacity);
        // 结果数始终受 in_flight/queue_capacity 限制；使用非阻塞发送避免取消
        // 阶段主线程 join 与 worker 等待结果队列空间形成死锁。
        let (result_tx, result_rx) = mpsc::channel::<MapResult<U>>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let mapper = Arc::new(mapper);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(worker_count);
        for index in 0..worker_count {
            let jobs = Arc::clone(&job_rx);
            let results = result_tx.clone();
            let mapper = Arc::clone(&mapper);
            let cancelled = Arc::clone(&cancel);
            workers.push(
                thread::Builder::new()
                    .name(format!("easyexcel-map-{index}"))
                    .spawn(move || {
                        loop {
                            let job = {
                                let Ok(receiver) = jobs.lock() else {
                                    cancelled.store(true, Ordering::Release);
                                    break;
                                };
                                receiver.recv()
                            };
                            let Ok(job) = job else {
                                break;
                            };
                            let MapJob {
                                sequence,
                                data,
                                context,
                            } = job;
                            if cancelled.load(Ordering::Acquire) {
                                let _ = results.send(MapResult {
                                    sequence,
                                    context,
                                    result: Err(ExcelError::AnalysisStop(
                                        "parallel_map pipeline cancelled after worker error"
                                            .to_owned(),
                                    )),
                                });
                                continue;
                            }
                            // 用户 mapper 是扩展边界，panic 不能让某个 sequence 永久
                            // 消失。否则主解析线程在队列背压期间会等待一个永远不会
                            // 到达的结果，而其余 worker 又因发送端仍开放持续等待任务。
                            // 将 panic 转换成普通的有序错误，取消、排空和 join 才能沿
                            // 与 mapper 返回 Err 相同的协议收敛。
                            let result = match catch_unwind(AssertUnwindSafe(|| {
                                mapper(data, &context)
                            })) {
                                Ok(result) => result,
                                Err(payload) => Err(ExcelError::Format(format!(
                                    "parallel_map mapper panicked: {}",
                                    panic_message(payload.as_ref())
                                ))),
                            }
                            .map_err(|error| error.with_parallel_row_context(&context));
                            if result.is_err() {
                                // 首个 worker 错误立即阻止主线程继续提交新任务；已经进入
                                // 队列的任务仍返回有序取消结果，避免 drain/join 死锁。
                                cancelled.store(true, Ordering::Release);
                            }
                            if results
                                .send(MapResult {
                                    sequence,
                                    context,
                                    result,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    })
                    .map_err(ExcelError::Io)?,
            );
        }
        drop(result_tx);
        Ok(Self {
            downstream,
            job_tx: Some(job_tx),
            result_rx,
            workers,
            ready: BTreeMap::new(),
            cancel,
            next_sequence: 0,
            next_commit: 0,
            in_flight: 0,
            queue_capacity,
            marker: PhantomData,
        })
    }

    /// 返回下游 Listener 的共享引用。
    #[must_use]
    pub const fn downstream(&self) -> &L {
        &self.downstream
    }

    /// 返回下游 Listener 的可变引用。
    pub const fn downstream_mut(&mut self) -> &mut L {
        &mut self.downstream
    }

    fn receive_one(&mut self) -> Result<()> {
        let result = self.result_rx.recv().map_err(|_| {
            ExcelError::Format("parallel_map worker terminated before returning a row".to_owned())
        })?;
        self.ready.insert(result.sequence, result);
        self.commit_ready()
    }

    fn commit_ready(&mut self) -> Result<()> {
        while let Some(result) = self.ready.remove(&self.next_commit) {
            self.in_flight = self.in_flight.saturating_sub(1);
            match result.result {
                Ok(data) => self.downstream.invoke(data, &result.context)?,
                Err(error) => {
                    self.cancel.store(true, Ordering::Release);
                    return Err(error);
                }
            }
            self.next_commit = self.next_commit.saturating_add(1);
        }
        Ok(())
    }

    fn drain_all(&mut self) -> Result<()> {
        while self.in_flight > 0 {
            self.receive_one()?;
        }
        Ok(())
    }

    fn close_jobs(&mut self) {
        self.job_tx.take();
    }

    fn join_workers(&mut self) -> Result<()> {
        for worker in self.workers.drain(..) {
            if worker.join().is_err() {
                return Err(ExcelError::Format("parallel_map worker panicked".to_owned()));
            }
        }
        Ok(())
    }
}

impl<T, U, L> ReadListener<T> for ParallelMapReadListener<T, U, L>
where
    T: Send + 'static,
    U: Send + 'static,
    L: ReadListener<U>,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        self.cancel.store(true, Ordering::Release);
        self.downstream.on_exception(error, context)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        self.drain_all()?;
        self.downstream.invoke_head(head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        // 乱序完成的结果进入 ready 后不一定立刻释放最早 sequence；必须持续
        // 接收直到真正提交至少一行，才能保证 in_flight 永不超过硬上限。
        while self.in_flight >= self.queue_capacity {
            self.receive_one()?;
        }
        if self.cancel.load(Ordering::Acquire) {
            // worker 可能在主线程观察取消前已返回带真实坐标的错误；先按序
            // 排空到该错误，不能用一个泛化的 AnalysisStop 把首错覆盖掉。
            self.drain_all()?;
            return Err(ExcelError::AnalysisStop(
                "parallel_map pipeline cancelled".to_owned(),
            ));
        }
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.job_tx
            .as_ref()
            .ok_or_else(|| ExcelError::AnalysisStop("parallel_map pipeline closed".to_owned()))?
            .send(MapJob {
                sequence,
                data,
                context: context.clone(),
            })
            .map_err(|_| ExcelError::AnalysisStop("parallel_map worker queue closed".to_owned()))?;
        self.in_flight = self.in_flight.saturating_add(1);
        Ok(())
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        self.drain_all()?;
        self.downstream.extra(extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        self.close_jobs();
        self.drain_all()?;
        self.join_workers()?;
        self.downstream.do_after_all_analysed(context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        !self.cancel.load(Ordering::Acquire) && self.downstream.has_next(context)
    }
}

impl<T, U, L> Drop for ParallelMapReadListener<T, U, L> {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.job_tx.take();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    /// 简单的下游 ReadListener，记录所有收到的 invoke 值。
    #[derive(Default)]
    struct CollectListener {
        values: Rc<RefCell<Vec<i32>>>,
        heads: usize,
        afters: usize,
        extras: usize,
        exception_action: ErrorAction,
    }

    impl CollectListener {
        fn new(values: Rc<RefCell<Vec<i32>>>) -> Self {
            Self {
                values,
                ..Self::default()
            }
        }
        fn with_error_action(values: Rc<RefCell<Vec<i32>>>, action: ErrorAction) -> Self {
            Self {
                values,
                exception_action: action,
                ..Self::default()
            }
        }
    }

    impl ReadListener<i32> for CollectListener {
        fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
            self.exception_action
        }

        fn invoke_head(
            &mut self,
            _head: &HashMap<String, usize>,
            _context: &AnalysisContext,
        ) -> Result<()> {
            self.heads += 1;
            Ok(())
        }

        fn invoke(&mut self, data: i32, _context: &AnalysisContext) -> Result<()> {
            self.values.borrow_mut().push(data);
            Ok(())
        }

        fn extra(&mut self, _extra: &CellExtra, _context: &AnalysisContext) -> Result<()> {
            self.extras += 1;
            Ok(())
        }

        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
            self.afters += 1;
            Ok(())
        }
    }

    fn ctx() -> AnalysisContext {
        AnalysisContext::new("Sheet1", 0, 0)
    }

    // ── 构造与参数校验 ──

    #[test]
    fn zero_worker_count_returns_error() {
        // 对应 Java：worker_count 必须大于零
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(values);
        let result = ParallelMapReadListener::<i32, i32, _>::new(
            0,
            4,
            |data, _ctx| Ok(data),
            downstream,
        );
        assert!(result.is_err());
    }

    #[test]
    fn zero_queue_capacity_returns_error() {
        // 对应 Java：queue_capacity 必须大于零
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(values);
        let result = ParallelMapReadListener::<i32, i32, _>::new(
            2,
            0,
            |data, _ctx| Ok(data),
            downstream,
        );
        assert!(result.is_err());
    }

    // ── 基本并发映射 ──

    #[test]
    fn identity_mapper_preserves_order() {
        // 对应 Java：mapper 并发执行但结果按输入顺序提交
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            2,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");

        let context = ctx();
        for i in 0..10 {
            ReadListener::invoke(&mut listener, i, &context).expect("invoke");
        }
        ReadListener::do_after_all_analysed(&mut listener, &context).expect("finalized");

        let result = values.borrow();
        assert_eq!(*result, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn double_mapper_transforms_values() {
        // 对应 Java：mapper 转换数据
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            2,
            8,
            |data, _ctx| Ok(data * 2),
            downstream,
        )
        .expect("new");

        let context = ctx();
        for i in 1..=5 {
            ReadListener::invoke(&mut listener, i, &context).expect("invoke");
        }
        ReadListener::do_after_all_analysed(&mut listener, &context).expect("finalized");

        let result = values.borrow();
        assert_eq!(*result, vec![2, 4, 6, 8, 10]);
    }

    // ── 错误传播与取消 ──

    #[test]
    fn mapper_error_propagates_and_cancels() {
        // 对应 Java：mapper 返回 Err 时取消 pipeline 并传播错误
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| {
                if data == 3 {
                    Err(ExcelError::Format("bad value".to_owned()))
                } else {
                    Ok(data)
                }
            },
            downstream,
        )
        .expect("new");

        let context = ctx();
        for i in 0..5 {
            let result = ReadListener::invoke(&mut listener, i, &context);
            if result.is_err() {
                break;
            }
        }
        // 后续调用应该返回 pipeline cancelled
        let result = ReadListener::invoke(&mut listener, 99, &context);
        assert!(result.is_err());
    }

    // ── mapper panic 转为错误 ──

    #[test]
    fn mapper_panic_is_converted_to_error() {
        // 对应 Java：用户 mapper panic 不应崩溃整个进程
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |_data, _ctx| -> Result<i32> {
                panic!("intentional panic");
            },
            downstream,
        )
        .expect("new");

        let context = ctx();
        // 第一次 invoke 发送任务到 worker
        let result = ReadListener::invoke(&mut listener, 1, &context);
        // 可能在 invoke 阶段（背压）或 do_after_all_analysed 阶段获得错误
        let finalize = ReadListener::do_after_all_analysed(&mut listener, &context);
        assert!(result.is_err() || finalize.is_err());
    }

    // ── on_exception 取消 ──

    #[test]
    fn on_exception_sets_cancel_flag() {
        // 对应 Java：外部异常触发取消
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream =
            CollectListener::with_error_action(Rc::clone(&values), ErrorAction::Stop);
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");

        let context = ctx();
        let error = ExcelError::Format("external error".to_owned());
        ReadListener::on_exception(&mut listener, &error, &context);
        // has_next 应返回 false（cancel 已设置）
        assert!(!ReadListener::has_next(&mut listener, &context));
    }

    // ── has_next ──

    #[test]
    fn has_next_delegates_to_downstream() {
        // 对应 Java：has_next 转发到下游并检查取消状态
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");

        let context = ctx();
        // 未取消时 has_next 取决于下游（默认 true）
        assert!(ReadListener::has_next(&mut listener, &context));
    }

    // ── invoke_head 和 extra 先排空 ──

    #[test]
    fn invoke_head_drains_pending() {
        // 对应 Java：invoke_head 先排空所有待处理结果
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");

        let context = ctx();
        for i in 0..3 {
            ReadListener::invoke(&mut listener, i, &context).expect("invoke");
        }
        let head = HashMap::from([("col".to_owned(), 0)]);
        ReadListener::invoke_head(&mut listener, &head, &context).expect("head");
        // 所有值应在 invoke_head 之前排空
        assert_eq!(values.borrow().len(), 3);
    }

    #[test]
    fn extra_drains_pending() {
        // 对应 Java：extra 先排空所有待处理结果
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");

        let context = ctx();
        ReadListener::invoke(&mut listener, 42, &context).expect("invoke");
        let extra = CellExtra::new(
            crate::core::CellExtraType::Comment,
            Some("note".to_owned()),
            0, 0, 1, 1,
        );
        ReadListener::extra(&mut listener, &extra, &context).expect("extra");
        assert_eq!(values.borrow().len(), 1);
    }

    // ── downstream 和 downstream_mut 访问器 ──

    #[test]
    fn downstream_accessor() {
        // 对应 Java：downstream() 返回下游引用
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");
        let _downstream_ref: &CollectListener = listener.downstream();
    }

    #[test]
    fn downstream_mut_accessor() {
        // 对应 Java：downstream_mut() 返回下游可变引用
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            1,
            4,
            |data, _ctx| Ok(data),
            downstream,
        )
        .expect("new");
        let _downstream_mut: &mut CollectListener = listener.downstream_mut();
    }

    // ── 多行高并发 ──

    #[test]
    fn many_rows_with_multiple_workers() {
        // 对应 Java：多 worker 高并发场景保持有序
        let values = Rc::new(RefCell::new(Vec::new()));
        let downstream = CollectListener::new(Rc::clone(&values));
        let mut listener = ParallelMapReadListener::<i32, i32, _>::new(
            4,
            16,
            |data, _ctx| Ok(data + 1),
            downstream,
        )
        .expect("new");

        let context = ctx();
        for i in 0..100 {
            ReadListener::invoke(&mut listener, i, &context).expect("invoke");
        }
        ReadListener::do_after_all_analysed(&mut listener, &context).expect("finalized");

        let result = values.borrow();
        let expected: Vec<i32> = (1..=100).collect();
        assert_eq!(*result, expected);
    }

    // ── panic_message ──

    #[test]
    fn panic_message_handles_string_payload() {
        // 对应 Java：panic_message 提取字符串消息
        let payload: Box<dyn std::any::Any + Send> = Box::new("test panic msg".to_owned());
        let msg = panic_message(payload.as_ref());
        assert_eq!(msg, "test panic msg");
    }

    #[test]
    fn panic_message_handles_str_ref_payload() {
        // 对应 Java：panic_message 处理 &str 类型 payload
        let payload: Box<dyn std::any::Any + Send> = Box::new("static str panic");
        let msg = panic_message(payload.as_ref());
        assert_eq!(msg, "static str panic");
    }

    #[test]
    fn panic_message_handles_non_string_payload() {
        // 对应 Java：非字符串 panic payload 回退到默认消息
        let payload: Box<dyn std::any::Any + Send> = Box::new(42i32);
        let msg = panic_message(payload.as_ref());
        assert_eq!(msg, "non-string panic payload");
    }
}
