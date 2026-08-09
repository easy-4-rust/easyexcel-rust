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
