//! 与 Java runner 对齐的结构化结果。

use serde::Serialize;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 单次、单进程性能测试结果。
#[derive(Debug, Serialize)]
pub(crate) struct BenchmarkResult {
    pub(crate) schema_version: u32,
    pub(crate) implementation: &'static str,
    pub(crate) phase: &'static str,
    pub(crate) temperature: String,
    pub(crate) scenario_id: String,
    pub(crate) fixture_origin: Option<&'static str>,
    pub(crate) input_sha256: Option<String>,
    pub(crate) operation: String,
    pub(crate) rows: u64,
    pub(crate) cells: u64,
    pub(crate) wall_time_ns: u64,
    pub(crate) process_wall_time_ns: Option<u64>,
    pub(crate) cpu_user_time_ns: Option<u64>,
    pub(crate) cpu_system_time_ns: Option<u64>,
    pub(crate) rows_per_second: f64,
    pub(crate) cells_per_second: f64,
    pub(crate) mib_per_second: f64,
    pub(crate) peak_rss_bytes: Option<u64>,
    pub(crate) java_heap_peak_bytes: Option<u64>,
    pub(crate) gc_count: Option<u64>,
    pub(crate) gc_time_ns: Option<u64>,
    pub(crate) gc_max_pause_ns: Option<u64>,
    pub(crate) allocator_allocations: Option<u64>,
    pub(crate) allocator_peak_bytes: Option<u64>,
    pub(crate) temporary_disk_peak_bytes: Option<u64>,
    pub(crate) file_size_bytes: u64,
    pub(crate) total_written_bytes: Option<u64>,
    pub(crate) worker_count: u32,
    pub(crate) trial: Option<u32>,
    pub(crate) worker_id: Option<u32>,
    pub(crate) success: bool,
    pub(crate) errors: u64,
    pub(crate) correctness: CorrectnessResult,
    pub(crate) environment: EnvironmentResult,
}

include!("benchmark_result/correctness_result.rs");

include!("benchmark_result/environment_result.rs");
