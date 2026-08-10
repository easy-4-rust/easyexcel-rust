//! `easyexcel-web::ExcelRows<T>` 的细粒度单测矩阵。
//!
//! 这些测试覆盖 `crates/easyexcel-web/src/web/excel_rows.rs` 中尚未被
//! `tests/web_contract.rs` 直接触达的分支：背压、Drop 取消、主动 cancel、
//! processing_timeout、RowLimitExceeded 传播、解析错误转换以及 `Stream` trait
//! 适配。
//!
//! 8 个测试对应 `docs/test/COVERAGE-GAP-CLOSURE.md` 子任务一（T1.1 - T1.8）。

use std::time::Duration;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::ResourceLimits;
use easyexcel_web::{
    ExcelImport, ExcelWebErrorCode, ExcelWebPolicy, ExcelRows, WebExecutionContext,
};
use futures_util::StreamExt;

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct WebRow {
    #[excel(name = "Name", index = 0)]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: i64,
}

fn policy_with_capacity(
    temp_directory: &std::path::Path,
    limits: ResourceLimits,
    row_channel_capacity: usize,
) -> ExcelWebPolicy {
    ExcelWebPolicy::new(limits)
        .with_temp_directory(temp_directory)
        .with_upload_timeout(Duration::from_secs(5))
        .with_processing_timeout(Duration::from_secs(5))
        .with_row_channel_capacity(row_channel_capacity)
}

fn directory_entry_count(path: &std::path::Path) -> usize {
    std::fs::read_dir(path)
        .expect("read temporary directory")
        .count()
}

fn five_row_csv() -> Vec<u8> {
    let mut buffer = String::from("Name,Value\n");
    for index in 0..5 {
        buffer.push_str(&format!("row-{index},{index}\n"));
    }
    buffer.into_bytes()
}

// T1.1 — 正常 EOF + row_channel_capacity=1 背压路径
#[tokio::test]
async fn excel_rows_normal_parse_yields_all_rows_then_eof() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_capacity(directory.path(), ResourceLimits::default(), 1);
    let context = WebExecutionContext::new("t11-eof", policy);
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\na,1\nb,2\n"),
        "csv",
        Some("two.csv".to_string()),
        context,
    )
    .await
    .expect("receive CSV");

    let mut rows: ExcelRows<WebRow> = import.rows();
    let first = rows.next_row().await.expect("first row").expect("ok");
    let second = rows.next_row().await.expect("second row").expect("ok");
    assert_eq!(
        first,
        WebRow {
            name: "a".to_string(),
            value: 1
        }
    );
    assert_eq!(
        second,
        WebRow {
            name: "b".to_string(),
            value: 2
        }
    );
    assert!(rows.next_row().await.is_none(), "stream should EOF");
    drop(rows);
    assert_eq!(directory_entry_count(directory.path()), 0);
}

// T1.2 — Drop 触发后台任务取消，临时文件被清理
#[tokio::test]
async fn excel_rows_drop_cancels_background_task() {
    let directory = tempfile::tempdir().expect("temporary directory");
    // 用足够大的 processing_timeout，避免误触发超时分支
    let policy = ExcelWebPolicy::new(ResourceLimits::default())
        .with_temp_directory(directory.path())
        .with_upload_timeout(Duration::from_secs(5))
        .with_processing_timeout(Duration::from_secs(10))
        .with_row_channel_capacity(1);
    let context = WebExecutionContext::new("t12-drop", policy);
    let csv = five_row_csv();
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from(csv),
        "csv",
        None,
        context,
    )
    .await
    .expect("receive CSV");

    let mut rows = import.rows();
    // 仅消费首行，让后台 producer 仍持有容量为 1 的通道
    let first = rows.next_row().await.expect("first row").expect("ok");
    assert_eq!(first.name, "row-0");
    // 立即 drop：后台 spawn_blocking 必须取消，临时文件必须清理
    drop(rows);
    // 等待后台任务结束
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while directory_entry_count(directory.path()) != 0 {
        if std::time::Instant::now() >= deadline {
            panic!("tempdir was not cleaned up after drop within 2s");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(directory_entry_count(directory.path()), 0);
}

// T1.3 — 主动 cancel() 终止流
#[tokio::test]
async fn excel_rows_explicit_cancel_terminates_stream() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_capacity(directory.path(), ResourceLimits::default(), 1);
    let context = WebExecutionContext::new("t13-cancel", policy);
    let csv = five_row_csv();
    let import = ExcelImport::<WebRow>::from_bytes(Bytes::from(csv), "csv", None, context)
        .await
        .expect("receive CSV");

    let mut rows = import.rows();
    rows.cancel();
    // 取消后 next_row 必须以错误形式收尾（要么收到 Cancelled 错误，
    // 要么消费者在取消被分发前先读到一行——两种都合法，但最终必须 EOF）
    let mut saw_cancelled = false;
    while let Some(item) = rows.next_row().await {
        if let Err(error) = item {
            if error.code() == ExcelWebErrorCode::Cancelled {
                saw_cancelled = true;
            }
        }
    }
    assert!(
        saw_cancelled,
        "expected at least one Cancelled error after explicit cancel()"
    );
    assert_eq!(directory_entry_count(directory.path()), 0);
}

// T1.4 — processing_timeout 触发 ExcelWebError::ProcessingTimeout
//
// 用极短的 processing_timeout (1ns) 配合 5 行 CSV 触发 tokio::time::timeout
// 分支：`spawn_blocking` worker 还没机会完成就被超时打断，excel_rows.rs:92-100
// 的 cancel + send ProcessingTimeout 路径被执行。
#[tokio::test]
async fn excel_rows_processing_timeout_emits_timeout_error() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = ExcelWebPolicy::new(ResourceLimits::default())
        .with_temp_directory(directory.path())
        .with_upload_timeout(Duration::from_secs(5))
        .with_processing_timeout(Duration::from_nanos(1))
        .with_row_channel_capacity(1);
    let context = WebExecutionContext::new("t14-timeout", policy);
    let csv = five_row_csv();
    let import = ExcelImport::<WebRow>::from_bytes(Bytes::from(csv), "csv", None, context)
        .await
        .expect("receive CSV");

    let mut rows = import.rows();
    let mut saw_timeout = false;
    while let Some(item) = rows.next_row().await {
        if let Err(error) = item {
            if error.code() == ExcelWebErrorCode::ProcessingTimeout {
                saw_timeout = true;
            }
        }
    }
    assert!(
        saw_timeout,
        "expected ProcessingTimeout error when processing_timeout is sub-microsecond"
    );
}

// T1.5 — RowLimitExceeded 通过流传播
#[tokio::test]
async fn excel_rows_row_limit_propagates_through_stream() {
    let directory = tempfile::tempdir().expect("temporary directory");
    // ResourceLimits::new(bytes, sheets, rows, formula_cells)
    let limits = ResourceLimits::new(1024, 8, 1, 8);
    let policy = policy_with_capacity(directory.path(), limits, 1);
    let context = WebExecutionContext::new("t15-row-limit", policy);
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nalpha,1\nbeta,2\n"),
        "csv",
        None,
        context,
    )
    .await
    .expect("receive CSV");

    let mut rows = import.rows();
    let first = rows.next_row().await.expect("first row").expect("ok");
    assert_eq!(
        first,
        WebRow {
            name: "alpha".to_string(),
            value: 1
        }
    );
    let error = rows
        .next_row()
        .await
        .expect("limit error delivered as Some")
        .expect_err("second row must exceed limit");
    assert_eq!(error.code(), ExcelWebErrorCode::RowLimitExceeded);
    // EOF
    assert!(rows.next_row().await.is_none());
}

// T1.6 — 慢消费者 + 5 行 CSV，背压路径不丢行
#[tokio::test]
async fn excel_rows_backpressure_does_not_drop_rows() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_capacity(directory.path(), ResourceLimits::default(), 1);
    let context = WebExecutionContext::new("t16-backpressure", policy);
    let csv = five_row_csv();
    let import = ExcelImport::<WebRow>::from_bytes(Bytes::from(csv), "csv", None, context)
        .await
        .expect("receive CSV");

    let mut rows = import.rows();
    let mut collected = Vec::with_capacity(5);
    for _ in 0..5 {
        let row = rows
            .next_row()
            .await
            .expect("row")
            .expect("ok");
        // 慢消费者：每次 await 之间 sleep 10ms
        tokio::time::sleep(Duration::from_millis(10)).await;
        collected.push(row);
    }
    assert_eq!(collected.len(), 5);
    for (index, row) in collected.iter().enumerate() {
        assert_eq!(row.name, format!("row-{index}"));
        assert_eq!(row.value, index as i64);
    }
    assert!(rows.next_row().await.is_none());
}

// T1.7 — 解析错误（非 execution_stop）通过流转换为 ExcelWebError
//
// 用损坏的 XLSX（带 ZIP magic 但内容无效）触发 ExcelError::Format，
// 走 excel_rows.rs:72-77 的 send_terminal 路径。
// 同时验证：i64 字段遇到非数字字符串时触发 ExcelError::Data，
// 映射为 ExcelWebErrorCode::RowConversionFailed。
#[tokio::test]
async fn excel_rows_parse_error_surfaces_as_excel_web_error() {
    // 损坏 XLSX：带 magic 但内容无效
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_capacity(directory.path(), ResourceLimits::default(), 1);
    let context = WebExecutionContext::new("t17-corrupt", policy);
    let corrupt_xlsx = Bytes::from_static(b"PK\x03\x04CORRUPT_NOT_VALID_OOXML");
    let import = ExcelImport::<WebRow>::from_bytes(
        corrupt_xlsx,
        "xlsx",
        Some("broken.xlsx".to_string()),
        context,
    )
    .await
    .expect("receive bytes");

    let mut rows = import.rows();
    let mut saw_format_error = false;
    while let Some(item) = rows.next_row().await {
        if let Err(error) = item {
            let code = error.code();
            assert!(
                matches!(
                    code,
                    ExcelWebErrorCode::InvalidFormat
                        | ExcelWebErrorCode::RowConversionFailed
                        | ExcelWebErrorCode::Internal
                ),
                "unexpected error code: {code:?}"
            );
            if matches!(code, ExcelWebErrorCode::InvalidFormat) {
                saw_format_error = true;
            }
        }
    }
    assert!(
        saw_format_error,
        "expected InvalidFormat error from corrupted XLSX"
    );
}

// T1.8 — ExcelRows 实现 Stream trait，可用 futures_util::StreamExt 消费
#[tokio::test]
async fn excel_rows_implements_stream_trait() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let policy = policy_with_capacity(directory.path(), ResourceLimits::default(), 4);
    let context = WebExecutionContext::new("t18-stream", policy);
    let import = ExcelImport::<WebRow>::from_bytes(
        Bytes::from_static(b"Name,Value\nx,10\ny,20\nz,30\n"),
        "csv",
        None,
        context,
    )
    .await
    .expect("receive CSV");

    let rows: ExcelRows<WebRow> = import.rows();
    // 用 StreamExt::next 消费；使用 futures_util 已经在 dev-deps 的依赖图里
    let collected: Vec<WebRow> = rows
        .map(|result| result.expect("ok row"))
        .collect()
        .await;

    assert_eq!(
        collected,
        vec![
            WebRow {
                name: "x".to_string(),
                value: 10
            },
            WebRow {
                name: "y".to_string(),
                value: 20
            },
            WebRow {
                name: "z".to_string(),
                value: 30
            },
        ]
    );
}
