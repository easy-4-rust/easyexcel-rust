//! 单进程读写场景执行。

use std::path::Path;
use std::{cell::RefCell, rc::Rc};

use easyexcel::{AnalysisContext, EasyExcel, ReadListener};

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
    } else {
        let mut writer = EasyExcel::write::<BenchmarkRow>(path)
            .sheet("Data")
            .constant_memory(true)
            .build();
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
    read(scenario, output)
}

#[derive(Default)]
struct EventState {
    rows: u64,
    checksum: RowChecksum,
}

struct EventListener(Rc<RefCell<EventState>>);

impl ReadListener<BenchmarkRow> for EventListener {
    fn invoke(&mut self, data: BenchmarkRow, _context: &AnalysisContext) -> easyexcel::Result<()> {
        let mut state = self.0.borrow_mut();
        state.rows += 1;
        state.checksum.update(&data);
        Ok(())
    }
}
