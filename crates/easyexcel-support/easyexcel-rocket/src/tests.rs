//! 边界用例单元测试：对应 Java `WebTest.downloadFailedUsingJson`
//! 中降级分支的不可达性验证（Rocket 适配，镜像 axum 适配器的 `tests_extra2`）。

use easyexcel::ExcelRow;
use easyexcel_core::ExcelDownloadErrorBody;
use rocket::http::Status;
use rocket::response::Response;
use serde_json::{Value, json};

use crate::{
    excel_download_error_response, excel_download_or_json_response,
    excel_download_response_from_bytes,
};

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

/// 对应 Java：尝试触发 `excel_download_error_response` 的 JSON 序列化失败回退分支
/// （`write_response.rs` 的 `unwrap_or_else` 回退文案）。
///
/// `ExcelDownloadErrorBody` 仅由两个 `String` 字段派生 `Serialize`，序列化在数学上
/// 不可能失败，因此 `unwrap_or_else` 的回退文案分支不可达。此处用边界字符串
/// （换行 / 制表符 / emoji）再次确认回退文案「JSON序列化错误」不会被输出。
#[test]
fn error_response_serialization_fallback_is_unreachable() {
    let mut response = excel_download_error_response(ExcelDownloadErrorBody::download_failed(
        "尝试触发回退分支\n\t🎉",
    ));
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json; charset=utf-8")
    );
    let value: Value = serde_json::from_slice(&body_bytes(&mut response)).expect("json");
    assert_eq!(value["status"], json!("failure"));
    assert_eq!(
        value["message"],
        json!("下载文件失败尝试触发回退分支\n\t🎉")
    );
}

/// 对应 Java：尝试让 `excel_download_response_from_bytes` 失败以触发
/// `excel_download_or_json_response` 成功分支的降级闭包。
///
/// 文件名经 `urlencoding` 百分号编码后必为合法 ASCII 值，Rocket `Header::new`
/// 亦不做值校验，函数恒返回 Ok，降级闭包在数学上不可达。此处以边界文件名
/// （中文 / 空格 / emoji / 制表符 / 百分号）逐一确认恒成功，
/// 且 Content-Disposition 始终为 RFC 5987 `filename*` 形态。
#[test]
fn bytes_response_never_fails_for_edge_case_file_names() {
    for name in ["edge case 文件", "emoji🎉", "a\tb", "100%"] {
        let response =
            excel_download_response_from_bytes(name, vec![1, 2, 3]).expect("must succeed");
        assert_eq!(response.status(), Status::Ok);
        let expected = format!(
            "attachment;filename*=utf-8''{}.xlsx",
            urlencoding::encode(name).replace('+', "%20")
        );
        assert_eq!(
            response.headers().get_one("Content-Disposition"),
            Some(expected.as_str())
        );
    }
}

/// 对应 Java：端到端尝试触发 `excel_download_or_json_response` 的字节响应降级闭包。
///
/// 写入成功路径恒走到 `excel_download_response_from_bytes`（恒 Ok），
/// 与 `tests/adapter_contract.rs` 的 `excel_download_or_json_response_success_path` 互补：
/// 此处特意使用边界文件名，确认即便文件名含空格也走附件响应而非 JSON 错误体。
#[derive(Debug, Clone, ExcelRow)]
struct AttemptRow {
    #[excel(name = "值", order = 1)]
    value: String,
}

#[test]
fn or_json_success_path_stays_attachment_for_edge_case_names() {
    let response = excel_download_or_json_response(
        "edge case 文件",
        "模板",
        [AttemptRow {
            value: "attempt".to_owned(),
        }],
    );
    assert_eq!(response.status(), Status::Ok);
    let headers = response.headers();
    assert!(
        headers
            .get_one("Content-Type")
            .expect("content-type")
            .contains("spreadsheetml")
    );
    assert!(
        headers
            .get_one("Content-Disposition")
            .expect("content-disposition")
            .contains("utf-8''")
    );
}
