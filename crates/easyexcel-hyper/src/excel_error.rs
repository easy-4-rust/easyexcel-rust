use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, Response};
use http_body_util::{BodyExt, Full};

use crate::ResponseBody;
use easyexcel_web::ExcelWebError;

/// Hyper 桥接层的统一错误。
#[derive(Debug)]
pub struct ExcelHyperError {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelHyperError {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }

    /// 转换为 Hyper JSON 错误响应。
    #[must_use]
    pub fn into_response(self) -> Response<ResponseBody> {
        let status = self.error.status_code();
        let problem = self.error.problem_details(self.request_id);
        let bytes = serde_json::to_vec(&problem).unwrap_or_else(|_| b"{}".to_vec());
        let body = Full::new(Bytes::from(bytes))
            .map_err(|never| match never {})
            .boxed();
        let mut response = Response::new(body);
        *response.status_mut() = status;
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json; charset=utf-8"),
        );
        response
    }
}

impl std::fmt::Display for ExcelHyperError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ExcelHyperError {}
