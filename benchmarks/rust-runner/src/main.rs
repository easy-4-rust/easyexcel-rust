//! `EasyExcel` Java/Rust 共享性能契约的 Rust runner。

mod arguments;
mod benchmark_result;
mod benchmark_row;
mod benchmark_spec;
mod checksum;
mod operation;

use std::time::Instant;

use arguments::Arguments;
use benchmark_result::{BenchmarkResult, CorrectnessResult, EnvironmentResult};
use benchmark_spec::BenchmarkSpec;
use operation::ParallelMapConfig;
use sha2::{Digest, Sha256};

fn main() {
    if let Err(error) = run() {
        eprintln!("easyexcel benchmark runner failed: {error}");
        std::process::exit(1);
    }
}

// 单次结果必须在同一作用域绑定契约、预热、被测操作和 JSON 字段，避免计时边界漂移。
#[allow(clippy::too_many_lines)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::parse()?;
    let spec_bytes = std::fs::read(&arguments.spec)?;
    let spec: BenchmarkSpec = serde_json::from_slice(&spec_bytes)?;
    if spec.schema_version != 1 {
        return Err(format!(
            "unsupported benchmark spec version: {}",
            spec.schema_version
        )
        .into());
    }
    if spec.suite_id.is_empty() || spec.batch_size == 0 {
        return Err("benchmark suite_id and batch_size must be non-empty".into());
    }
    let rows_f64 = f64::from(
        u32::try_from(arguments.rows)
            .map_err(|_| "benchmark rows exceed the v1 contract maximum of 4,294,967,295")?,
    );
    let scenario = spec
        .scenario(&arguments.scenario)
        .ok_or_else(|| format!("unknown scenario: {}", arguments.scenario))?;
    validate_scenario(scenario)?;
    let input = arguments.input.as_deref();
    let output = arguments.output.as_deref();
    validate_paths(&scenario.operation, input, output)?;
    let parallel_map = (arguments.internal_map_work_factor > 0).then(|| ParallelMapConfig {
        worker_count: usize::try_from(arguments.worker_count)
            .expect("u32 worker count always fits usize"),
        queue_capacity: arguments.internal_map_queue_capacity,
        work_factor: arguments.internal_map_work_factor,
    });
    if parallel_map.is_some()
        && (scenario.id != "xlsx-event-read"
            || scenario.operation != "read"
            || scenario.mode != "event")
    {
        return Err("internal parallel-map benchmark requires xlsx-event-read/event mode".into());
    }

    // 稳态预热必须发生在被测 runner 进程内部；启动独立进程不能预热 JVM/JIT。
    for warmup in 0..arguments.warmups {
        let warmup_output = output.map(|path| warmup_path(path, warmup));
        execute(
            scenario,
            input,
            warmup_output.as_deref().or(output),
            arguments.rows,
            spec.batch_size,
            parallel_map,
        )?;
        if let Some(path) = warmup_output {
            let _ = std::fs::remove_file(path);
        }
    }

    let started = Instant::now();
    let operation = execute(
        scenario,
        input,
        output,
        arguments.rows,
        spec.batch_size,
        parallel_map,
    )?;
    let elapsed = started.elapsed();
    let expected = checksum::expected_checksum(arguments.rows)?;
    let success = operation.observed_rows == arguments.rows && operation.checksum == expected;
    let seconds = elapsed.as_secs_f64();
    let bytes_mib = bytes_as_mib(operation.file_size_bytes);
    let result = BenchmarkResult {
        schema_version: 1,
        implementation: "rust",
        phase: if parallel_map.is_some() {
            "internal-parallel-map"
        } else {
            "single"
        },
        temperature: arguments.temperature,
        scenario_id: scenario.id.clone(),
        fixture_origin: None,
        input_sha256: None,
        operation: scenario.operation.clone(),
        rows: arguments.rows,
        cells: arguments.rows.saturating_mul(4),
        wall_time_ns: u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX),
        process_wall_time_ns: None,
        cpu_user_time_ns: None,
        cpu_system_time_ns: None,
        rows_per_second: rows_f64 / seconds,
        cells_per_second: rows_f64 * 4.0 / seconds,
        mib_per_second: bytes_mib / seconds,
        peak_rss_bytes: None,
        java_heap_peak_bytes: None,
        gc_count: None,
        gc_time_ns: None,
        gc_max_pause_ns: None,
        allocator_allocations: None,
        allocator_peak_bytes: None,
        temporary_disk_peak_bytes: None,
        file_size_bytes: operation.file_size_bytes,
        total_written_bytes: matches!(scenario.operation.as_str(), "write" | "roundtrip")
            .then_some(operation.file_size_bytes),
        worker_count: arguments.worker_count,
        internal_map_work_factor: parallel_map.map(|config| config.work_factor),
        internal_map_queue_capacity: parallel_map.map(|config| config.queue_capacity),
        trial: None,
        worker_id: None,
        success,
        errors: u64::from(!success),
        correctness: CorrectnessResult {
            observed_rows: operation.observed_rows,
            checksum: operation.checksum,
            rereadable: matches!(scenario.operation.as_str(), "read" | "roundtrip"),
        },
        environment: EnvironmentResult {
            git_sha: option_env!("EASYEXCEL_GIT_SHA")
                .unwrap_or("unknown")
                .to_owned(),
            runtime: format!(
                "rustc {}",
                option_env!("EASYEXCEL_RUSTC").unwrap_or("unknown")
            ),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            spec_sha256: format!("{:x}", Sha256::digest(&spec_bytes)),
        },
    };
    println!("{}", serde_json::to_string(&result)?);
    if success {
        Ok(())
    } else {
        Err("benchmark correctness check failed".into())
    }
}

fn validate_paths(
    operation: &str,
    input: Option<&std::path::Path>,
    output: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    match operation {
        "read" if input.is_some() => Ok(()),
        "write" if output.is_some() => Ok(()),
        "roundtrip" if input.is_some() && output.is_some() => Ok(()),
        "read" => Err("read scenario requires --input".into()),
        "write" => Err("write scenario requires --output".into()),
        "roundtrip" => Err("roundtrip scenario requires --input and --output".into()),
        other => Err(format!("unsupported operation: {other}").into()),
    }
}

fn execute(
    scenario: &benchmark_spec::ScenarioSpec,
    input: Option<&std::path::Path>,
    output: Option<&std::path::Path>,
    rows: u64,
    batch_size: usize,
    parallel_map: Option<ParallelMapConfig>,
) -> Result<operation::OperationResult, Box<dyn std::error::Error>> {
    match scenario.operation.as_str() {
        "read" => operation::read(scenario, input.ok_or("missing input")?, parallel_map),
        "write" => operation::write(scenario, output.ok_or("missing output")?, rows, batch_size),
        "roundtrip" => operation::roundtrip(
            scenario,
            input.ok_or("missing input")?,
            output.ok_or("missing output")?,
        ),
        other => Err(format!("unsupported operation: {other}").into()),
    }
}

fn warmup_path(path: &std::path::Path, warmup: u32) -> std::path::PathBuf {
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("tmp");
    path.with_extension(format!("warmup-{warmup}.{extension}"))
}

// 性能计数允许超过 f64 的整数精确范围；这里只计算展示用 MiB 吞吐，
// 字节精确值仍保存在 file_size_bytes 中作为正确性和回归依据。
#[allow(clippy::cast_precision_loss)]
fn bytes_as_mib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn validate_scenario(
    scenario: &benchmark_spec::ScenarioSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    if !matches!(scenario.format.as_str(), "xlsx" | "xls" | "csv") {
        return Err(format!("unsupported format: {}", scenario.format).into());
    }
    if !matches!(scenario.mode.as_str(), "event" | "workbook") {
        return Err(format!("unsupported mode: {}", scenario.mode).into());
    }
    if scenario.format == "xls"
        && scenario.operation == "write"
        && (scenario.mode != "workbook" || scenario.memory != "batched")
    {
        return Err(
            "BIFF8 write benchmark must declare workbook mode with batched input delivery"
                .into(),
        );
    }
    Ok(())
}
