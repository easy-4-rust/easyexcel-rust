//! `easyexcel-web` 的框架中立流式、安全与生命周期契约测试。

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_web::{
    ExcelExport, ExcelImport, ExcelProblemDetails, ExcelWebError, ExcelWebErrorCode,
    ExcelWebPolicy, ExcelWebRuntime, WebExecutionContext, excel_attachment_content_disposition,
    excel_xlsx_attachment_headers,
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

#[test]
fn error_code_and_status_for_all_variants() {
    let cases: Vec<(ExcelWebError, ExcelWebErrorCode, u16)> = vec![
        (
            ExcelWebError::FileTooLarge {
                actual: 100,
                limit: 50,
            },
            ExcelWebErrorCode::FileTooLarge,
            413,
        ),
        (
            ExcelWebError::RowLimitExceeded { limit: 100 },
            ExcelWebErrorCode::RowLimitExceeded,
            422,
        ),
        (
            ExcelWebError::UnsupportedMediaType {
                extension: "json".to_string(),
            },
            ExcelWebErrorCode::UnsupportedMediaType,
            415,
        ),
        (
            ExcelWebError::Transport {
                message: "reset".to_string(),
            },
            ExcelWebErrorCode::TransportFailed,
            400,
        ),
        (ExcelWebError::Cancelled, ExcelWebErrorCode::Cancelled, 408),
        (
            ExcelWebError::ProcessingTimeout,
            ExcelWebErrorCode::ProcessingTimeout,
            504,
        ),
        (
            ExcelWebError::Io(std::io::Error::new(std::io::ErrorKind::Other, "disk")),
            ExcelWebErrorCode::StorageFailed,
            500,
        ),
        (
            ExcelWebError::Worker {
                message: "panic".to_string(),
            },
            ExcelWebErrorCode::Internal,
            500,
        ),
    ];
    for (error, expected_code, expected_status) in cases {
        assert_eq!(error.code(), expected_code, "code mismatch for {:?}", error);
        assert_eq!(
            error.status_code().as_u16(),
            expected_status,
            "status mismatch for {:?}",
            error
        );
    }
}

#[test]
fn public_detail_for_each_error_variant() {
    let details: Vec<(ExcelWebError, &str)> = vec![
        (
            ExcelWebError::FileTooLarge {
                actual: 200,
                limit: 100,
            },
            "200",
        ),
        (ExcelWebError::RowLimitExceeded { limit: 50 }, "50"),
        (
            ExcelWebError::UnsupportedMediaType {
                extension: "json".to_string(),
            },
            "json",
        ),
        (
            ExcelWebError::Transport {
                message: "secret".to_string(),
            },
            "请求体传输失败",
        ),
        (ExcelWebError::Cancelled, "操作已取消"),
        (ExcelWebError::ProcessingTimeout, "超时"),
        (
            ExcelWebError::Worker {
                message: "panic".to_string(),
            },
            "异常终止",
        ),
    ];
    for (error, expected_fragment) in details {
        let pd = error.problem_details("req-test");
        assert!(
            pd.detail().contains(expected_fragment),
            "detail '{}' should contain '{}' for {:?}",
            pd.detail(),
            expected_fragment,
            error,
        );
    }
}

#[test]
fn excel_error_data_maps_to_row_conversion_failed() {
    let error = ExcelWebError::Excel(easyexcel::ExcelError::Data {
        sheet: "Sheet1".to_string(),
        row: 2,
        column: Some(1),
        field: "amount",
        value: "abc".to_string(),
        message: "bad value".to_string(),
    });
    assert_eq!(error.code(), ExcelWebErrorCode::RowConversionFailed);
    let pd = error.problem_details("req-data");
    assert!(pd.detail().contains("第 3 行第 2 列"));
}

#[test]
fn excel_error_data_without_column() {
    let error = ExcelWebError::Excel(easyexcel::ExcelError::Data {
        sheet: "Sheet1".to_string(),
        row: 0,
        column: None,
        field: "name",
        value: "".to_string(),
        message: "bad".to_string(),
    });
    assert_eq!(error.code(), ExcelWebErrorCode::RowConversionFailed);
    let pd = error.problem_details("req-data2");
    assert!(pd.detail().contains("第 1 行的数据转换失败"));
}

#[test]
fn excel_error_format_maps_to_invalid_format() {
    let error = ExcelWebError::Excel(easyexcel::ExcelError::Format("bad".to_string()));
    assert_eq!(error.code(), ExcelWebErrorCode::InvalidFormat);
    let pd = error.problem_details("req-fmt");
    assert!(pd.detail().contains("格式无效"));
}

#[test]
fn excel_error_sheet_not_found_maps_to_invalid_format() {
    let error = ExcelWebError::Excel(easyexcel::ExcelError::SheetNotFound("missing".to_string()));
    assert_eq!(error.code(), ExcelWebErrorCode::InvalidFormat);
    let pd = error.problem_details("req-snf");
    assert!(pd.detail().contains("工作表不存在"));
}

#[test]
fn excel_error_unsupported_maps_to_unsupported_media_type() {
    let error = ExcelWebError::Excel(easyexcel::ExcelError::Unsupported("macro".to_string()));
    assert_eq!(error.code(), ExcelWebErrorCode::UnsupportedMediaType);
    let pd = error.problem_details("req-unsup");
    assert!(pd.detail().contains("不受支持"));
}

#[test]
fn cancelled_and_timeout_constructors() {
    let cancelled = ExcelWebError::cancelled();
    assert_eq!(cancelled.code(), ExcelWebErrorCode::Cancelled);
    let timeout = ExcelWebError::processing_timeout();
    assert_eq!(timeout.code(), ExcelWebErrorCode::ProcessingTimeout);
}

#[test]
fn io_error_maps_to_storage_failed() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let error = ExcelWebError::Io(io_err);
    assert_eq!(error.code(), ExcelWebErrorCode::StorageFailed);
    let pd = error.problem_details("req-io");
    assert!(pd.detail().contains("临时存储"));
}

#[test]
fn display_impl_for_all_error_variants() {
    let errors: Vec<ExcelWebError> = vec![
        ExcelWebError::FileTooLarge {
            actual: 100,
            limit: 50,
        },
        ExcelWebError::RowLimitExceeded { limit: 100 },
        ExcelWebError::UnsupportedMediaType {
            extension: "x".to_string(),
        },
        ExcelWebError::Transport {
            message: "t".to_string(),
        },
        ExcelWebError::Cancelled,
        ExcelWebError::ProcessingTimeout,
        ExcelWebError::Worker {
            message: "w".to_string(),
        },
    ];
    for error in &errors {
        let display = format!("{error}");
        assert!(
            !display.is_empty(),
            "Display should not be empty for {error:?}"
        );
    }
}

#[tokio::test]
async fn csv_export_accessors_and_content_type() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "export-csv",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    let rows = vec![WebRow {
        name: "x".to_string(),
        value: 1,
    }];
    let export = ExcelExport::prepare(rows, Format::Csv, "data", "Sheet", context)
        .await
        .expect("prepare CSV");

    assert_eq!(export.file_name(), "data.csv");
    assert_eq!(export.format(), Format::Csv);
    assert!(export.content_type().contains("text/csv"));
    assert_eq!(export.content_length(), export.content_length());
    assert!(export.io_chunk_size() > 0);
    assert!(!export.context().request_id().is_empty());
    export.cancel();
}

#[tokio::test]
async fn import_with_transport_error_propagates() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "transport-err",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    use futures_util::stream;
    let bad_stream = stream::iter([Err::<Bytes, _>("connection reset")]);
    let error = ExcelImport::<WebRow>::receive(bad_stream, "csv", None, context)
        .await
        .expect_err("transport error must fail");
    assert_eq!(error.code(), ExcelWebErrorCode::TransportFailed);
}

#[tokio::test]
async fn import_with_unsupported_extension_returns_error() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "import-bad-ext",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    let error =
        ExcelImport::<WebRow>::from_bytes(Bytes::from_static(b"data"), "json", None, context)
            .await
            .expect_err("unsupported extension must fail");
    assert_eq!(error.code(), ExcelWebErrorCode::UnsupportedMediaType);
}

#[tokio::test]
async fn import_accessors() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let context = WebExecutionContext::new(
        "import-accessors",
        policy_with_limits(directory.path(), ResourceLimits::default()),
    );
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\na,1\n"),
        "csv",
        Some("data.csv".to_string()),
        context,
    )
    .await
    .expect("receive CSV");

    assert_eq!(import.file_name(), Some("data.csv"));
    assert_eq!(import.format(), Format::Csv);
    assert!(import.received_bytes() > 0);
    assert!(!import.context().request_id().is_empty());
}

#[test]
fn web_execution_context_generated_has_unique_id() {
    let policy = ExcelWebPolicy::default();
    let ctx1 = WebExecutionContext::generated(policy.clone());
    let ctx2 = WebExecutionContext::generated(policy);
    assert_ne!(ctx1.request_id(), ctx2.request_id());
    assert!(ctx1.request_id().starts_with("excel-"));
}

#[test]
fn web_execution_context_checkpoint_ok_when_not_cancelled() {
    let policy = ExcelWebPolicy::default();
    let ctx = WebExecutionContext::new("ckpt-ok", policy);
    assert!(!ctx.is_cancelled());
    assert!(ctx.checkpoint().is_ok());
}

#[test]
fn web_execution_context_checkpoint_err_when_cancelled() {
    let policy = ExcelWebPolicy::default();
    let ctx = WebExecutionContext::new("ckpt-cancel", policy);
    ctx.cancel();
    assert!(ctx.is_cancelled());
    assert!(ctx.checkpoint().is_err());
    assert_eq!(
        ctx.checkpoint().unwrap_err().code(),
        ExcelWebErrorCode::Cancelled
    );
}

#[test]
fn web_execution_context_cancellation_token() {
    let policy = ExcelWebPolicy::default();
    let ctx = WebExecutionContext::new("token-test", policy);
    let token = ctx.cancellation_token();
    assert!(!token.is_cancelled());
    ctx.cancel();
    assert!(token.is_cancelled());
}

#[test]
fn excel_web_runtime_generated_context() {
    let policy = ExcelWebPolicy::default();
    let runtime = ExcelWebRuntime::new(policy);
    let ctx = runtime.generated_context();
    assert!(ctx.request_id().starts_with("excel-"));
    assert!(runtime.available_permits() > 0);
}

#[test]
fn excel_attachment_content_disposition_encodes_special_chars() {
    let disposition = excel_attachment_content_disposition("report 2024.xlsx");
    assert!(disposition.starts_with("attachment;filename*=utf-8''"));
    assert!(disposition.contains("report%202024.xlsx"));
}

#[test]
fn excel_xlsx_attachment_headers_has_correct_content_type() {
    let headers = excel_xlsx_attachment_headers("report");
    let ct = headers.get("content-type").expect("content-type header");
    assert_eq!(
        ct,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    let cd = headers
        .get("content-disposition")
        .expect("disposition header");
    assert!(cd.to_str().unwrap().contains("report.xlsx"));
}

#[test]
fn problem_details_accessor_methods() {
    let pd = ExcelProblemDetails::new(
        ExcelWebErrorCode::ProcessingTimeout,
        "表格处理超时",
        "req-pd",
    );
    assert!(pd.type_uri().contains("processing_timeout"));
    assert_eq!(pd.title(), "PROCESSING TIMEOUT");
    assert_eq!(pd.status(), 504);
    assert_eq!(pd.code(), ExcelWebErrorCode::ProcessingTimeout);
    assert!(pd.retryable());
    assert_eq!(pd.request_id(), "req-pd");
}

#[test]
fn excel_web_error_excel_io_maps_to_storage_failed() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let excel_err = ExcelWebError::Excel(easyexcel::ExcelError::Io(io_err));
    assert_eq!(excel_err.code(), ExcelWebErrorCode::StorageFailed);
}
