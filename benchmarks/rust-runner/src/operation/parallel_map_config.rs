//! 单工作簿纯函数映射并发参数。

/// 单工作簿纯函数映射并发参数，控制并行读取的工作线程数和队列容量。
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
#[derive(Clone, Copy)]
pub(crate) struct ParallelMapConfig {
    /// 工作线程数
    pub(crate) worker_count: usize,
    /// 任务队列容量
    pub(crate) queue_capacity: usize,
    /// 工作因子，控制每次映射的 CPU 计算量
    pub(crate) work_factor: u32,
}
