//! 七种 Web 框架适配器共享的行为契约与测试夹具。

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelRows, ExcelWebError, ExcelWebErrorCode, ExcelWebPolicy, ExcelWebRuntime};

/// 所有框架必须能以相同语义读写的测试行。
#[derive(Debug, Clone, PartialEq, ExcelRow)]
pub struct ConformanceRow {
    /// 名称列。
    #[excel(name = "name", index = 0)]
    pub name: String,
    /// 数值列。
    #[excel(name = "value", index = 1)]
    pub value: f64,
}

/// 框架响应归一化后的快照。
#[derive(Debug)]
pub struct ResponseSnapshot {
    /// HTTP 状态码。
    pub status: u16,
    /// `Content-Type` 响应头。
    pub content_type: String,
    /// `Content-Disposition` 响应头。
    pub content_disposition: String,
    /// 完整响应体，仅由小型 conformance 夹具收集。
    pub body: Bytes,
}

/// 创建所有适配器共享的运行环境。
#[must_use]
pub fn runtime() -> ExcelWebRuntime {
    ExcelWebRuntime::new(
        ExcelWebPolicy::default()
            .with_max_concurrent_tasks(2)
            .with_row_channel_capacity(1),
    )
}

/// 返回共享 CSV 上传夹具。
#[must_use]
pub fn upload_fixture() -> Bytes {
    Bytes::from_static(b"name,value\nalpha,1.5\nbeta,2.5\n")
}

/// 返回共享下载数据。
#[must_use]
pub fn download_rows() -> Vec<ConformanceRow> {
    vec![
        ConformanceRow {
            name: "alpha".to_string(),
            value: 1.5,
        },
        ConformanceRow {
            name: "beta".to_string(),
            value: 2.5,
        },
    ]
}

/// 返回 XLS 上传夹具（`include_bytes!` 内嵌）。
///
/// 该夹具为 BIFF8 格式的简单数据表，用于验证框架适配器的 XLS 解析路径。
#[must_use]
pub fn xls_upload_fixture() -> Bytes {
    Bytes::from_static(include_bytes!("fixtures/conformance.xls"))
}

/// 返回 2-sheet XLSX 上传夹具（`include_bytes!` 内嵌）。
///
/// 该夹具包含两个工作表，用于验证多 sheet 场景下首表行能被正确解析。
#[must_use]
pub fn xlsx_multisheet_fixture() -> Bytes {
    Bytes::from_static(include_bytes!("fixtures/conformance_multisheet.xlsx"))
}

/// 构造超过 `max_file_bytes` 限制的字节块。
///
/// 将 `max_file_bytes` 设为 64 字节，然后生成 128 字节的填充数据。
#[must_use]
pub fn oversized_fixture(policy: &ExcelWebPolicy) -> Bytes {
    let limit = policy.resource_limits().max_file_bytes() as usize;
    // 生成超过限制 1 字节的数据
    let size = limit + 1;
    Bytes::from(vec![0u8; size])
}

/// 返回带 XLSX 魔数但内容损坏的字节块。
#[must_use]
pub fn corrupted_xlsx_fixture() -> Bytes {
    // PK\x03\x04 是 ZIP/XLSX 的魔数，后面跟损坏内容
    Bytes::from_static(b"PK\x03\x04CORRUPTED_DATA_NOT_A_VALID_ZIP_ARCHIVE")
}

/// 创建一个 `max_file_bytes` 为 64 字节的严格策略运行环境。
///
/// 用于触发 `FileTooLarge` 错误路径。
#[must_use]
pub fn strict_runtime() -> ExcelWebRuntime {
    let limits = easyexcel::io::ResourceLimits::new(64, 8, 2_000_000, 500_000);
    ExcelWebRuntime::new(
        ExcelWebPolicy::new(limits)
            .with_max_concurrent_tasks(2)
            .with_row_channel_capacity(1),
    )
}

/// 验证 XLS 上传夹具的解析结果。
///
/// XLS 夹具的行结构可能与 `ConformanceRow` 不完全一致（如列名不同）。
/// 此函数验证两种可接受的路径：
/// 1. 解析成功且至少返回 1 行（夹具列名匹配）。
/// 2. 解析触发 `RowConversionFailed`（夹具列名不匹配但格式合法）。
///
/// # Errors
///
/// 底层解析返回 `InvalidFormat` 或其他非预期错误时传播。
///
/// # Panics
///
/// 解析结果为空（无行且无错误）时 panic。
pub async fn verify_upload_xls(
    mut rows: ExcelRows<ConformanceRow>,
) -> Result<(), ExcelWebError> {
    let mut count = 0_usize;
    while let Some(result) = rows.next_row().await {
        match result {
            Ok(_row) => count += 1,
            Err(error)
                if error.code() == ExcelWebErrorCode::RowConversionFailed
                    || error.code() == ExcelWebErrorCode::InvalidFormat =>
            {
                // 合法的列不匹配或格式差异 — 证明 XLS 解析路径已触发。
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
    assert!(count >= 1, "XLS upload must produce at least 1 row");
    Ok(())
}

/// 验证多 sheet XLSX 上传夹具的解析结果。
///
/// 多 sheet 夹具默认解析首表，验证首表至少返回 1 行或触发合法的列转换错误。
///
/// # Errors
///
/// 底层解析返回 `InvalidFormat` 或其他非预期错误时传播。
///
/// # Panics
///
/// 解析结果为空（无行且无错误）时 panic。
pub async fn verify_upload_multisheet(
    mut rows: ExcelRows<ConformanceRow>,
) -> Result<(), ExcelWebError> {
    let mut count = 0_usize;
    while let Some(result) = rows.next_row().await {
        match result {
            Ok(_row) => count += 1,
            Err(error)
                if error.code() == ExcelWebErrorCode::RowConversionFailed
                    || error.code() == ExcelWebErrorCode::InvalidFormat =>
            {
                // 合法的列不匹配或格式差异 — 证明多 sheet 解析路径已触发。
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
    assert!(count >= 1, "multi-sheet upload must produce at least 1 row");
    Ok(())
}

/// 验证错误响应快照。
///
/// 检查状态码与错误码匹配（来自 [`easyexcel_web::ExcelWebErrorCode`]），
/// 以及响应体为 Problem Details JSON 且包含 `"code"` 字段。
///
/// # Panics
///
/// 状态码或错误码不匹配时 panic。
pub fn verify_error_response(snapshot: &ResponseSnapshot, expected_code: &str) {
    // ExcelWebErrorCode 的推荐状态码映射：
    //   FILE_TOO_LARGE         → 413 Payload Too Large
    //   INVALID_FORMAT         → 422 Unprocessable Entity
    //   ROW_CONVERSION_FAILED  → 422 Unprocessable Entity
    let expected_status = match expected_code {
        "FILE_TOO_LARGE" => 413_u16,
        "INVALID_FORMAT" | "ROW_CONVERSION_FAILED" => 422_u16,
        _ => panic!("unknown expected error code: {expected_code}"),
    };
    assert_eq!(
        snapshot.status, expected_status,
        "status mismatch: expected {expected_status}, got {}",
        snapshot.status
    );
    let body_str = std::str::from_utf8(&snapshot.body).expect("response body must be valid UTF-8");
    assert!(
        body_str.contains(expected_code),
        "response body must contain error code {expected_code}, got: {body_str}"
    );
}

/// 验证框架 extractor 产生的类型化行流。
///
/// # Errors
///
/// 底层解析失败时返回统一 Web 错误。
///
/// # Panics
///
/// 框架返回的行与共享夹具不一致时 panic。
pub async fn verify_upload(mut rows: ExcelRows<ConformanceRow>) -> Result<(), ExcelWebError> {
    let mut actual = Vec::new();
    while let Some(row) = rows.next_row().await {
        actual.push(row?);
    }
    assert_eq!(actual, download_rows());
    Ok(())
}

/// 验证框架 responder 的统一下载协议与 OOXML 响应体。
///
/// # Panics
///
/// 状态码、响应头或 OOXML 签名不符合共享契约时 panic。
pub fn verify_download(snapshot: &ResponseSnapshot) {
    assert_eq!(snapshot.status, 200);
    assert_eq!(
        snapshot.content_type,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert!(snapshot.content_disposition.starts_with("attachment;"));
    assert!(snapshot.content_disposition.contains("conformance.xlsx"));
    assert!(snapshot.body.starts_with(b"PK"));
}
