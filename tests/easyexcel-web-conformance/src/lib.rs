//! 七种 Web 框架适配器共享的行为契约与测试夹具。

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelRows, ExcelWebError, ExcelWebPolicy, ExcelWebRuntime};

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
