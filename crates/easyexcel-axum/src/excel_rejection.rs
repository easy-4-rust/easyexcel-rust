use axum::Json;
use axum::response::{IntoResponse, Response};
use easyexcel_web::ExcelWebError;

/// Axum extractor/responder 的统一拒绝响应。
#[derive(Debug)]
pub struct ExcelRejection {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelRejection {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }

    /// 返回稳定错误码。
    #[must_use]
    pub const fn error(&self) -> &ExcelWebError {
        &self.error
    }
}

impl IntoResponse for ExcelRejection {
    fn into_response(self) -> Response {
        let status = self.error.status_code();
        let problem = self.error.problem_details(self.request_id);
        (status, Json(problem)).into_response()
    }
}
