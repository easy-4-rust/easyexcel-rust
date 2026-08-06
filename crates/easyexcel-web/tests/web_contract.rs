//! `easyexcel-web` 的框架中立流式、安全与生命周期契约测试。

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_web::{
    ExcelExport, ExcelImport, ExcelWebError, ExcelWebErrorCode, ExcelWebPolicy, ExcelWebRuntime,
    WebExecutionContext,
};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct WebRow {
    #[excel(name = "Name", index = 0)]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: i64,
}

struct GatedRows {
    entered: Arc<Barrier>,
    released: Arc<AtomicBool>,
    emitted: bool,
}

impl Iterator for GatedRows {
    type Item = WebRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted {
            return None;
        }
        self.emitted = true;
        self.entered.wait();
        while !self.released.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(1));
        }
        Some(WebRow {
            name: "first".to_string(),
            value: 1,
        })
    }
}

struct CountingRows {
    started: Arc<AtomicUsize>,
    emitted: bool,
}

impl Iterator for CountingRows {
    type Item = WebRow;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted {
            return None;
        }
        self.emitted = true;
        self.started.fetch_add(1, Ordering::AcqRel);
        Some(WebRow {
            name: "second".to_string(),
            value: 2,
        })
    }
}

fn policy_with_limits(temp_directory: &std::path::Path, limits: ResourceLimits) -> ExcelWebPolicy {
    ExcelWebPolicy::new(limits)
        .with_temp_directory(temp_directory)
        .with_upload_timeout(Duration::from_secs(5))
        .with_processing_timeout(Duration::from_secs(5))
        .with_row_channel_capacity(1)
}

fn directory_entry_count(path: &std::path::Path) -> usize {
    std::fs::read_dir(path)
        .expect("read temporary directory")
        .count()
}

#[tokio::test]
async fn csv_upload_streams_typed_rows_and_cleans_artifact() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_limits(directory.path(), ResourceLimits::default());
    let context = WebExecutionContext::new("import-success", policy);
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nalpha,1\nbeta,2\n"),
        "csv",
        Some("rows.csv".to_string()),
        context,
    )
    .await
    .expect("receive CSV");

    assert_eq!(import.received_bytes(), 26);
    assert_eq!(directory_entry_count(directory.path()), 1);
    let mut rows = import.rows();
    assert_eq!(
        rows.next_row().await.expect("first row").expect("row"),
        WebRow {
            name: "alpha".to_string(),
            value: 1,
        }
    );
    assert_eq!(
        rows.next_row().await.expect("second row").expect("row"),
        WebRow {
            name: "beta".to_string(),
            value: 2,
        }
    );
    assert!(rows.next_row().await.is_none());
    drop(rows);
    assert_eq!(directory_entry_count(directory.path()), 0);
}

#[tokio::test]
async fn upload_limit_fails_before_oversized_chunk_is_written() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let limits = ResourceLimits::new(8, 8, 8, 8);
    let context =
        WebExecutionContext::new("upload-limit", policy_with_limits(directory.path(), limits));
    let error = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nalpha,1\n"),
        "csv",
        None,
        context,
    )
    .await
    .expect_err("oversized upload must fail");

    assert_eq!(error.code(), ExcelWebErrorCode::FileTooLarge);
    assert_eq!(directory_entry_count(directory.path()), 0);
}

#[tokio::test]
async fn row_limit_is_reported_through_bounded_row_stream() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let limits = ResourceLimits::new(1024, 8, 1, 8);
    let context =
        WebExecutionContext::new("row-limit", policy_with_limits(directory.path(), limits));
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nalpha,1\nbeta,2\n"),
        "csv",
        None,
        context,
    )
    .await
    .expect("receive CSV");
    let mut rows = import.rows();

    assert!(rows.next_row().await.expect("first row").is_ok());
    let error = rows
        .next_row()
        .await
        .expect("limit error")
        .expect_err("second row must exceed limit");
    assert_eq!(error.code(), ExcelWebErrorCode::RowLimitExceeded);
    drop(rows);
}

#[tokio::test]
async fn cancellation_before_parsing_is_not_silent_success() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "cancel-before-read",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nalpha,1\n"),
        "csv",
        None,
        context.clone(),
    )
    .await
    .expect("receive CSV");
    context.cancel();
    let mut rows = import.rows();
    let error = rows
        .next_row()
        .await
        .expect("cancellation result")
        .expect_err("cancelled parser must report error");

    assert_eq!(error.code(), ExcelWebErrorCode::Cancelled);
}

#[tokio::test]
async fn xlsx_export_is_async_readable_and_removed_on_drop() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "export-success",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    let rows = vec![WebRow {
        name: "alpha".to_string(),
        value: 1,
    }];
    let mut export = ExcelExport::prepare(rows, Format::Xlsx, "report", "Data", context)
        .await
        .expect("prepare XLSX");

    assert_eq!(export.file_name(), "report.xlsx");
    assert_eq!(directory_entry_count(directory.path()), 1);
    let mut bytes = Vec::new();
    export.read_to_end(&mut bytes).await.expect("read export");
    assert!(bytes.starts_with(b"PK"));
    assert_eq!(
        u64::try_from(bytes.len()).expect("length"),
        export.content_length()
    );
    drop(export);
    assert_eq!(directory_entry_count(directory.path()), 0);
}

#[tokio::test]
async fn shared_runtime_bounds_blocking_excel_tasks() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_limits(directory.path(), ResourceLimits::default())
        .with_max_concurrent_tasks(1);
    let runtime = ExcelWebRuntime::new(policy);
    let entered = Arc::new(Barrier::new(2));
    let released = Arc::new(AtomicBool::new(false));
    let first_context = runtime.context("concurrency-first");
    let first_entered = Arc::clone(&entered);
    let first_released = Arc::clone(&released);
    let first = tokio::spawn(async move {
        ExcelExport::prepare(
            GatedRows {
                entered: first_entered,
                released: first_released,
                emitted: false,
            },
            Format::Csv,
            "first.csv",
            "Data",
            first_context,
        )
        .await
    });

    let wait_for_first = Arc::clone(&entered);
    tokio::task::spawn_blocking(move || wait_for_first.wait())
        .await
        .expect("wait for first worker");
    assert_eq!(runtime.available_permits(), 0);

    let started = Arc::new(AtomicUsize::new(0));
    let second_started = Arc::clone(&started);
    let second_context = runtime.context("concurrency-second");
    let second = tokio::spawn(async move {
        ExcelExport::prepare(
            CountingRows {
                started: second_started,
                emitted: false,
            },
            Format::Csv,
            "second.csv",
            "Data",
            second_context,
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(started.load(Ordering::Acquire), 0);
    released.store(true, Ordering::Release);

    let first_export = first.await.expect("first task").expect("first export");
    let second_export = second.await.expect("second task").expect("second export");
    assert_eq!(started.load(Ordering::Acquire), 1);
    drop((first_export, second_export));
    assert_eq!(runtime.available_permits(), 1);
}

#[test]
fn problem_details_has_stable_code_status_and_no_internal_message() {
    let error = ExcelWebError::Transport {
        message: "/private/tmp/secret.xlsx: connection reset".to_string(),
    };
    let problem = error.problem_details("request-42");
    let json = serde_json::to_value(problem).expect("serialize problem details");

    assert_eq!(json["code"], "TRANSPORT_FAILED");
    assert_eq!(json["status"], 400);
    assert_eq!(json["request_id"], "request-42");
    assert_eq!(json["detail"], "请求体传输失败");
    assert!(!json.to_string().contains("secret.xlsx"));
}
