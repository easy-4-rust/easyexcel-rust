use easyexcel_web::ExcelWebError;
use http::StatusCode;
use warp::{Rejection, Reply};

/// Warp filter 使用的 `EasyExcel` 拒绝原因。
#[derive(Debug)]
pub struct ExcelWarpRejection {
    error: ExcelWebError,
    request_id: String,
}

impl ExcelWarpRejection {
    /// 使用请求标识包装 Web 内核错误。
    #[must_use]
    pub fn new(error: ExcelWebError, request_id: impl Into<String>) -> Self {
        Self {
            error,
            request_id: request_id.into(),
        }
    }
}

impl warp::reject::Reject for ExcelWarpRejection {}

/// 将 `EasyExcel` rejection 恢复为稳定 Problem Details 响应。
///
/// # Errors
///
/// 非 `EasyExcel` rejection 原样返回给后续 recovery 链。
pub async fn recover_excel_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    let Some(error) = rejection.find::<ExcelWarpRejection>() else {
        return Err(rejection);
    };
    let status = StatusCode::from_u16(error.error.status_code().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let problem = error.error.problem_details(&error.request_id);
    Ok(warp::reply::with_status(
        warp::reply::json(&problem),
        status,
    ))
}
