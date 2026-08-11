use serde::{Deserialize, Serialize};

use super::ExcelWebErrorCode;

/// 跨框架一致的 RFC 9457 风格错误响应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcelProblemDetails {
    #[serde(rename = "type")]
    type_uri: String,
    title: String,
    status: u16,
    code: ExcelWebErrorCode,
    detail: String,
    request_id: String,
    retryable: bool,
}

impl ExcelProblemDetails {
    /// 创建稳定错误响应。
    #[must_use]
    pub fn new(
        code: ExcelWebErrorCode,
        detail: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            type_uri: format!(
                "https://easyexcel.rs/problems/{}",
                code.as_str().to_lowercase()
            ),
            title: code.as_str().replace('_', " "),
            status: code.status_code().as_u16(),
            code,
            detail: detail.into(),
            request_id: request_id.into(),
            retryable: code.retryable(),
        }
    }

    /// 返回问题类型 URI。
    #[must_use]
    pub fn type_uri(&self) -> &str {
        &self.type_uri
    }

    /// 返回人类可读标题。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 返回 HTTP 状态码数值。
    #[must_use]
    pub const fn status(&self) -> u16 {
        self.status
    }

    /// 返回稳定机器错误码。
    #[must_use]
    pub const fn code(&self) -> ExcelWebErrorCode {
        self.code
    }

    /// 返回经过脱敏的错误说明。
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// 返回错误是否建议重试。
    #[must_use]
    pub const fn retryable(&self) -> bool {
        self.retryable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_all_fields() {
        let pd = ExcelProblemDetails::new(
            ExcelWebErrorCode::FileTooLarge,
            "file exceeded 10MB",
            "req-123",
        );
        assert_eq!(pd.type_uri(), "https://easyexcel.rs/problems/file_too_large");
        assert_eq!(pd.title(), "FILE TOO LARGE");
        assert_eq!(pd.status(), 413);
        assert_eq!(pd.code(), ExcelWebErrorCode::FileTooLarge);
        assert_eq!(pd.detail(), "file exceeded 10MB");
        assert_eq!(pd.request_id(), "req-123");
        assert!(!pd.retryable());
    }

    #[test]
    fn new_retryable_error() {
        let pd = ExcelWebErrorCode::ProcessingTimeout;
        let details = ExcelProblemDetails::new(pd, "took too long", "req-456");
        assert!(details.retryable());
        assert_eq!(details.status(), 504);
    }

    #[test]
    fn new_internal_error() {
        let pd = ExcelProblemDetails::new(
            ExcelWebErrorCode::Internal,
            "something broke",
            "req-789",
        );
        assert_eq!(pd.status(), 500);
        assert!(!pd.retryable());
        assert_eq!(pd.type_uri(), "https://easyexcel.rs/problems/internal");
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let pd = ExcelProblemDetails::new(
            ExcelWebErrorCode::InvalidFormat,
            "bad xlsx",
            "req-abc",
        );
        let json = serde_json::to_string(&pd).unwrap();
        let restored: ExcelProblemDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(pd, restored);
    }
}
