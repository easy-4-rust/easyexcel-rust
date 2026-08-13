use easyexcel_web::ExcelWebError;
use poem::http::header::CONTENT_TYPE;
use poem::http::{HeaderValue, StatusCode};
use poem::{Body, IntoResponse, Response};

/// Poem extractor/responder 的统一错误。
#[derive(Debug)]
pub struct ExcelPoemError {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelPoemError {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }

    fn into_poem_error(self) -> poem::Error {
        poem::Error::from_response(self.into_response())
    }
}

impl IntoResponse for ExcelPoemError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.error.status_code().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let problem = self.error.problem_details(self.request_id);
        let body = serde_json::to_vec(&problem).unwrap_or_else(|_| b"{}".to_vec());
        let mut response = Response::builder()
            .status(status)
            .body(Body::from_vec(body));
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json; charset=utf-8"),
        );
        response
    }
}

impl std::fmt::Display for ExcelPoemError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ExcelPoemError {}

impl From<ExcelPoemError> for poem::Error {
    fn from(error: ExcelPoemError) -> Self {
        error.into_poem_error()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_wraps_error() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelPoemError::new(error, "req-1");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn display_delegates() {
        let error = easyexcel_web::ExcelWebError::ProcessingTimeout;
        let err = ExcelPoemError::new(error, "req-2");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn error_trait() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelPoemError::new(error, "req-3");
        let dyn_err: &dyn std::error::Error = &err;
        assert!(!dyn_err.to_string().is_empty());
    }

    #[test]
    fn into_response_has_status() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelPoemError::new(error, "req-4");
        let response = err.into_response();
        assert_eq!(response.status(), poem::http::StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn from_converts_to_poem_error() {
        let error = easyexcel_web::ExcelWebError::Cancelled;
        let err = ExcelPoemError::new(error, "req-5");
        let poem_err: poem::Error = err.into();
        assert_eq!(poem_err.status(), poem::http::StatusCode::REQUEST_TIMEOUT);
    }
}
