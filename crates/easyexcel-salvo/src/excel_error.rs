use easyexcel_web::ExcelWebError;
use salvo::http::StatusCode;
use salvo::writing::Json;
use salvo::{Depot, Request, Response, Writer, async_trait};

/// Salvo extractor/writer 的统一错误。
#[derive(Debug)]
pub struct ExcelSalvoError {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelSalvoError {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }
}

impl std::fmt::Display for ExcelSalvoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ExcelSalvoError {}

#[async_trait]
impl Writer for ExcelSalvoError {
    async fn write(self, _request: &mut Request, _depot: &mut Depot, response: &mut Response) {
        response.status_code = Some(
            StatusCode::from_u16(self.error.status_code().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
        response.render(Json(self.error.problem_details(self.request_id)));
    }
}
