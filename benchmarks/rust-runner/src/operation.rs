//! 单进程读写场景执行。

use std::path::Path;
use std::{cell::RefCell, rc::Rc};

use easyexcel::{AnalysisContext, EasyExcel, ParallelMapReadListener, ReadListener};

use crate::benchmark_row::BenchmarkRow;
use crate::benchmark_spec::ScenarioSpec;
use crate::checksum::RowChecksum;

const XLS_DATA_ROWS_PER_SHEET: i64 = 65_535;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 被测操作的可观察输出。
pub(crate) struct OperationResult {
    pub(crate) observed_rows: u64,
    pub(crate) checksum: String,
    pub(crate) file_size_bytes: u64,
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。单工作簿纯函数映射并发参数。
#[derive(Clone, Copy)]
pub(crate) struct ParallelMapConfig {
    pub(crate) worker_count: usize,
    pub(crate) queue_capacity: usize,
    pub(crate) work_factor: u32,
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 执行统一写入场景。
pub(crate) fn write(
    scenario: &ScenarioSpec,
    path: &Path,
    rows: u64,
    batch_size: usize,
) -> Result<OperationResult, Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let row_count = i64::try_from(rows)?;
    if scenario.memory == "full" {
        let values = (0..row_count)
            .map(BenchmarkRow::from_id)
            .collect::<Vec<_>>();
        EasyExcel::write::<BenchmarkRow>(path)
            .sheet("Data")
            .constant_memory(false)
            .do_write(values)?;
    } else if scenario.memory == "constant" || scenario.memory == "batched" {
        let builder = EasyExcel::write::<BenchmarkRow>(path).sheet("Data");
        // BIFF8 has a 65,536-row Sheet limit and uses the workbook backend.
        // `batched` bounds runner input allocation and splits Sheets; it must
        // not pretend that the underlying XLS workbook is constant-memory.
        let mut writer = if scenario.memory == "constant" {
            builder.constant_memory(true).build()
        } else {
            builder.build()
        };
        let batch_size = i64::try_from(batch_size)?;
        let sheet_capacity = if scenario.format == "xls" {
            XLS_DATA_ROWS_PER_SHEET
        } else {
            row_count.max(1)
        };
        for (sheet_index, sheet_start) in (0..row_count)
            .step_by(usize::try_from(sheet_capacity)?)
            .enumerate()
        {
            let sheet_end = sheet_start.saturating_add(sheet_capacity).min(row_count);
            let sheet_name = if scenario.format == "xls" {
                format!("Data-{}", sheet_index + 1)
            } else {
                "Data".to_owned()
            };
            let sheet = EasyExcel::writer_sheet::<BenchmarkRow>(sheet_name);
            for start in (sheet_start..sheet_end).step_by(usize::try_from(batch_size)?) {
                let end = start.saturating_add(batch_size).min(sheet_end);
                writer.write((start..end).map(BenchmarkRow::from_id), &sheet)?;
            }
        }
        writer.finish()?;
    } else {
        return Err(format!("unsupported write memory mode: {}", scenario.memory).into());
    }
    Ok(OperationResult {
        observed_rows: rows,
        checksum: crate::checksum::expected_checksum(rows)?,
        file_size_bytes: std::fs::metadata(path)?.len(),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 执行 Event 或 Workbook 读取场景。
pub(crate) fn read(
    scenario: &ScenarioSpec,
    path: &Path,
    parallel_map: Option<ParallelMapConfig>,
) -> Result<OperationResult, Box<dyn std::error::Error>> {
    let (observed_rows, checksum) = if scenario.mode == "workbook" {
        let builder = EasyExcel::read_sync::<BenchmarkRow>(path);
        let rows = if scenario.format == "xls" {
            builder.all_sheets().do_read_sync()?
        } else {
            builder.do_read_sync()?
        };
        let mut checksum = RowChecksum::default();
        for row in &rows {
            checksum.update(row);
        }
        (u64::try_from(rows.len())?, checksum.finish())
    } else if let Some(config) = parallel_map {
        let state = Rc::new(RefCell::new(EventState::default()));
        if config.worker_count == 1 {
            let listener = SerialMapListener {
                downstream: EventListener(Rc::clone(&state)),
                work_factor: config.work_factor,
            };
            EasyExcel::read::<BenchmarkRow, _>(path, listener).do_read()?;
        } else {
            let work_factor = config.work_factor;
            let listener = ParallelMapReadListener::new(
                config.worker_count,
                config.queue_capacity,
                move |row, _context| Ok(apply_benchmark_map(row, work_factor)),
                EventListener(Rc::clone(&state)),
            )?;
            EasyExcel::read::<BenchmarkRow, _>(path, listener).do_read()?;
        }
        let state = Rc::try_unwrap(state)
            .map_err(|_| "parallel-map listener state still shared")?
            .into_inner();
        (state.rows, state.checksum.finish())
    } else {
        let state = Rc::new(RefCell::new(EventState::default()));
        let builder = EasyExcel::read::<BenchmarkRow, _>(path, EventListener(Rc::clone(&state)));
        if scenario.format == "xls" {
            builder.all_sheets().do_read()?;
        } else {
            builder.do_read()?;
        }
        let state = Rc::try_unwrap(state)
            .map_err(|_| "event listener state still shared")?
            .into_inner();
        (state.rows, state.checksum.finish())
    };
    Ok(OperationResult {
        observed_rows,
        checksum,
        file_size_bytes: std::fs::metadata(path)?.len(),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 执行 XLSX Workbook Mode 的读、元数据修改、保存与重新读取校验。
pub(crate) fn roundtrip(
    scenario: &ScenarioSpec,
    input: &Path,
    output: &Path,
) -> Result<OperationResult, Box<dyn std::error::Error>> {
    if scenario.format != "xlsx" || scenario.mode != "workbook" {
        return Err("v1 roundtrip requires XLSX Workbook Mode".into());
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut workbook = easyexcel::xlsx::read_path(input)?;
    workbook.metadata.title = Some("easyexcel-benchmark-roundtrip".to_owned());
    easyexcel::xlsx::write_path(&workbook, output)?;
    let reopened = easyexcel::xlsx::read_path(output)?;
    if reopened.metadata.title.as_deref() != Some("easyexcel-benchmark-roundtrip") {
        return Err("roundtrip metadata marker was not preserved".into());
    }
    read(scenario, output, None)
}

/// 对显式纯函数 mapper 建立确定、可重复且与输出 checksum 绑定的 CPU 工作量。
/// 返回值保持不变，因此串行与并行路径必须产生完全相同的行序列。
fn apply_benchmark_map(row: BenchmarkRow, work_factor: u32) -> BenchmarkRow {
    let mut fingerprint = row.id as u64 ^ row.score.to_bits().rotate_left(17);
    for round in 0..work_factor {
        fingerprint ^= u64::from(round).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        for byte in row.name.as_bytes() {
            fingerprint ^= u64::from(*byte);
            fingerprint = fingerprint
                .rotate_left(9)
                .wrapping_mul(0x1000_0000_01b3);
        }
    }
    std::hint::black_box(fingerprint);
    row
}

#[derive(Default)]
struct EventState {
    rows: u64,
    checksum: RowChecksum,
}

struct EventListener(Rc<RefCell<EventState>>);

struct SerialMapListener {
    downstream: EventListener,
    work_factor: u32,
}

impl ReadListener<BenchmarkRow> for SerialMapListener {
    fn invoke(&mut self, data: BenchmarkRow, context: &AnalysisContext) -> easyexcel::Result<()> {
        self.downstream
            .invoke(apply_benchmark_map(data, self.work_factor), context)
    }
}

impl ReadListener<BenchmarkRow> for EventListener {
    fn invoke(&mut self, data: BenchmarkRow, _context: &AnalysisContext) -> easyexcel::Result<()> {
        let mut state = self.0.borrow_mut();
        state.rows += 1;
        state.checksum.update(&data);
        Ok(())
    }
}
