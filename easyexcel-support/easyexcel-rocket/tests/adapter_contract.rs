//! Web 适配器契约测试：对应 Java `com.alibaba.easyexcel.test.demo.web.WebTest`
//! 的下载 / 上传 / 失败降级语义（Rocket 适配，镜像 axum 适配器的 tests.rs 断言）。

use std::sync::{Arc, Mutex};

use easyexcel::{ExcelRow, ReadListener};
use easyexcel_core::{
    AnalysisContext, CellValue, ExcelColumn, ExcelDownloadErrorBody, ExcelError,
    ExcelWriteMetadata, Result, RowData,
};
use rocket::http::Status;
use rocket::response::Response;
use serde_json::{Value, json};

use easyexcel_rocket::{
    excel_download_error_response, excel_download_or_json_response, excel_download_response,
    excel_download_response_from_bytes, excel_xlsx_attachment_headers, extension_from_path,
    read_upload_sync, read_upload_with_listener, write_rows_to_bytes, write_upload_temp,
};

/// 对应 Java `demo.write.WriteDemoData`。
#[derive(Debug, Clone, ExcelRow)]
struct WriteDemoData {
    #[excel(name = "字符串标题", order = 1)]
    string: String,
    #[excel(name = "数字标题", order = 2)]
    double_data: f64,
}

/// 行转换必然失败的行类型：`to_row` 注入数据转换错误
/// （对应 Java 写入时抛出 `ExcelDataConvertException`）。
struct FailingRow;

impl ExcelRow for FailingRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
        &METADATA
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Err(ExcelError::Data {
            sheet: String::new(),
            row: 0,
            column: Some(0),
            field: "field",
            value: "bad".to_owned(),
            message: "injected conversion failure".to_owned(),
        })
    }
}

/// 对应 Java `demo.read.DemoData`。
#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct DemoData {
    #[excel(name = "字符串标题", order = 1)]
    string: String,
    #[excel(name = "数字标题", order = 2)]
    double_data: Option<f64>,
}

fn write_demo_data() -> Vec<WriteDemoData> {
    vec![WriteDemoData {
        string: "字符串0".to_owned(),
        double_data: 0.56,
    }]
}

/// 收集读取行的简单监听器（对应 Java `AnalysisEventListener` 匿名实现）。
struct CollectListener<T> {
    rows: Arc<Mutex<Vec<T>>>,
}

impl<T> CollectListener<T> {
    fn new(rows: Arc<Mutex<Vec<T>>>) -> Self {
        Self { rows }
    }
}

impl<T: Send + 'static> ReadListener<T> for CollectListener<T> {
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> Result<()> {
        self.rows.lock().expect("lock").push(data);
        Ok(())
    }
}

/// 读取 Rocket 响应体为字节（测试辅助）。
///
/// Rocket 的 `Body` 仅实现异步读取，故通过 rocket 重导出的 tokio
/// 构建 current-thread 运行时阻塞驱动 `to_bytes()`。
fn body_bytes(response: &mut Response<'static>) -> Vec<u8> {
    let mut body = response.body_mut().take();
    rocket::tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(async { body.to_bytes().await.expect("collect") })
}

/// 在 `excel_xlsx_attachment_headers` 的返回值中查找指定名称的 Header 值。
fn header_value<'a>(headers: &'a [rocket::http::Header<'_>], name: &str) -> &'a str {
    headers
        .iter()
        .find(|header| header.name().as_str().eq_ignore_ascii_case(name))
        .expect("header")
        .value()
}

#[test]
fn excel_download_error_body_serializes_java_keys() {
    let body = ExcelDownloadErrorBody::download_failed("stream closed");
    let value: Value = serde_json::to_value(&body).expect("serialize");
    assert_eq!(value["status"], json!("failure"));
    assert_eq!(value["message"], json!("下载文件失败stream closed"));
    assert!(value.get("status").is_some());
    assert!(value.get("message").is_some());
}

#[test]
fn excel_download_error_body_deserializes_java_keys() {
    let raw = r#"{"status":"failure","message":"下载文件失败测试"}"#;
    let body: ExcelDownloadErrorBody = serde_json::from_str(raw).expect("deserialize");
    assert_eq!(body.status, "failure");
    assert_eq!(body.message, "下载文件失败测试");
}

#[test]
fn attachment_headers_match_java_web_test() {
    let headers = excel_xlsx_attachment_headers("测试");
    assert_eq!(
        header_value(&headers, "Content-Type"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    // RFC 5987: filename*=utf-8'' + percent-encoded 名称 + .xlsx
    assert_eq!(
        header_value(&headers, "Content-Disposition"),
        "attachment;filename*=utf-8''%E6%B5%8B%E8%AF%95.xlsx"
    );
}

#[test]
fn attachment_headers_replace_plus_with_percent20() {
    // 空格经 urlencoding 编码为 `+`，Java 侧需替换为 %20
    let headers = excel_xlsx_attachment_headers("a b");
    assert_eq!(
        header_value(&headers, "Content-Disposition"),
        "attachment;filename*=utf-8''a%20b.xlsx"
    );
}

#[test]
fn write_rows_to_bytes_produces_ooxml_zip() {
    let bytes = write_rows_to_bytes("模板", write_demo_data()).expect("write");
    assert!(!bytes.is_empty());
    // XLSX 是 ZIP 容器，magic 为 PK\x03\x04
    assert_eq!(&bytes[..2], b"PK");
}

#[test]
fn excel_download_response_from_bytes_sets_headers_and_status() {
    let bytes = write_rows_to_bytes("模板", write_demo_data()).expect("write");
    let mut response = excel_download_response_from_bytes("测试", bytes.clone()).expect("response");
    assert_eq!(response.status(), Status::Ok);
    let headers = response.headers();
    assert_eq!(
        headers.get_one("Content-Type"),
        Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    );
    assert_eq!(
        headers.get_one("Content-Disposition"),
        Some("attachment;filename*=utf-8''%E6%B5%8B%E8%AF%95.xlsx")
    );
    assert_eq!(body_bytes(&mut response).len(), bytes.len());
}

#[test]
fn excel_download_response_end_to_end() {
    let response = excel_download_response("测试", "模板", write_demo_data()).expect("response");
    assert_eq!(response.status(), Status::Ok);
    let headers = response.headers();
    assert!(
        headers
            .get_one("Content-Disposition")
            .expect("content-disposition")
            .contains("utf-8''")
    );
}

#[test]
fn excel_download_error_response_returns_json_500() {
    let mut response =
        excel_download_error_response(ExcelDownloadErrorBody::download_failed("stream closed"));
    assert_eq!(response.status(), Status::InternalServerError);
    let headers = response.headers();
    assert_eq!(
        headers.get_one("Content-Type"),
        Some("application/json; charset=utf-8")
    );
    let value: Value = serde_json::from_slice(&body_bytes(&mut response)).expect("json");
    assert_eq!(value["status"], json!("failure"));
    assert_eq!(value["message"], json!("下载文件失败stream closed"));
}

#[test]
fn excel_download_or_json_response_success_path() {
    let response = excel_download_or_json_response("测试", "模板", write_demo_data());
    assert_eq!(response.status(), Status::Ok);
    let headers = response.headers();
    assert!(
        headers
            .get_one("Content-Type")
            .expect("content-type")
            .contains("spreadsheetml")
    );
}

#[test]
fn excel_download_or_json_response_write_error_degrades_to_json() {
    // 行转换失败 → 降级为 JSON 错误体（Java downloadFailedUsingJson 的 catch 分支）
    let mut response = excel_download_or_json_response("测试", "模板", [FailingRow]);
    assert_eq!(response.status(), Status::InternalServerError);
    let value: Value = serde_json::from_slice(&body_bytes(&mut response)).expect("json");
    assert_eq!(value["status"], json!("failure"));
    assert!(value["message"].as_str().unwrap().contains("下载文件失败"));
}

/// 临时文件后缀断言与 axum 适配器 tests.rs 完全一致（`ends_with` 文本比较）。
#[allow(clippy::case_sensitive_file_extension_comparisons)]
#[test]
fn write_upload_temp_persists_bytes_with_dot_extension() {
    let bytes = b"file-content";
    let (path, _temp) = write_upload_temp(bytes, ".csv").expect("temp");
    assert!(std::path::Path::new(path.to_str().unwrap())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv")));
    assert_eq!(std::fs::read(&path).expect("read"), bytes);
}

/// 临时文件后缀断言与 axum 适配器 tests.rs 完全一致（`ends_with` 文本比较）。
#[allow(clippy::case_sensitive_file_extension_comparisons)]
#[test]
fn write_upload_temp_normalizes_extension_without_dot() {
    let bytes = b"file-content";
    let (path, _temp) = write_upload_temp(bytes, "xls").expect("temp");
    assert!(std::path::Path::new(path.to_str().unwrap())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xls")));
    assert_eq!(std::fs::read(&path).expect("read"), bytes);
}

#[test]
fn extension_from_path_maps_supported_formats() {
    assert_eq!(extension_from_path(std::path::Path::new("a.csv")), ".csv");
    assert_eq!(extension_from_path(std::path::Path::new("a.XLS")), ".xls");
    assert_eq!(extension_from_path(std::path::Path::new("a.xlsx")), ".xlsx");
    assert_eq!(
        extension_from_path(std::path::Path::new("a.unknown")),
        ".xlsx"
    );
    assert_eq!(extension_from_path(std::path::Path::new("noext")), ".xlsx");
}

fn xlsx_bytes_with_one_row() -> Vec<u8> {
    write_rows_to_bytes("模板", write_demo_data()).expect("write")
}

#[test]
fn read_upload_with_listener_event_driven_roundtrip() {
    let rows: Arc<Mutex<Vec<DemoData>>> = Arc::new(Mutex::new(Vec::new()));
    let listener = CollectListener::new(rows.clone());
    read_upload_with_listener::<DemoData, _>(&xlsx_bytes_with_one_row(), ".xlsx", listener)
        .expect("read");
    let rows = rows.lock().expect("lock");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "字符串0");
    assert_eq!(rows[0].double_data, Some(0.56));
}

#[test]
fn read_upload_sync_roundtrip() {
    let rows: Vec<DemoData> =
        read_upload_sync::<DemoData>(&xlsx_bytes_with_one_row(), "xlsx").expect("read");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].string, "字符串0");
    assert_eq!(rows[0].double_data, Some(0.56));
}

#[test]
fn read_upload_sync_rejects_garbage_bytes() {
    let error = read_upload_sync::<DemoData>(b"not-an-excel-file", ".xlsx").expect_err("error");
    assert!(
        matches!(
            error,
            ExcelError::Unsupported(_) | ExcelError::Format(_) | ExcelError::Io(_)
        ),
        "unexpected error: {error:?}"
    );
}

#[test]
fn read_upload_with_listener_propagates_parse_error() {
    let rows: Arc<Mutex<Vec<DemoData>>> = Arc::new(Mutex::new(Vec::new()));
    let listener = CollectListener::new(rows.clone());
    let error =
        read_upload_with_listener::<DemoData, _>(b"garbage", ".xlsx", listener).expect_err("error");
    assert!(
        matches!(
            error,
            ExcelError::Unsupported(_) | ExcelError::Format(_) | ExcelError::Io(_)
        ),
        "unexpected error: {error:?}"
    );
}
