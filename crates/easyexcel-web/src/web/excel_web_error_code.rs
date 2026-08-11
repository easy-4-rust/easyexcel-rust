use http::StatusCode;
use serde::{Deserialize, Serialize};

/// 面向 Web 客户端的稳定错误码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ExcelWebErrorCode {
    /// 请求体超过允许的最大字节数。
    FileTooLarge,
    /// 解析或生成的行数超过限制。
    RowLimitExceeded,
    /// 文件扩展名或媒体类型不受支持。
    UnsupportedMediaType,
    /// 工作簿格式无效或已损坏。
    InvalidFormat,
    /// 单元格无法转换为目标行类型。
    RowConversionFailed,
    /// 操作超过配置的时间限制。
    ProcessingTimeout,
    /// 客户端断开或调用方主动取消。
    Cancelled,
    /// 请求体传输失败。
    TransportFailed,
    /// 临时文件或底层 I/O 操作失败。
    StorageFailed,
    /// 未分类的内部错误。
    Internal,
}

impl ExcelWebErrorCode {
    /// 返回跨框架稳定的字符串错误码。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileTooLarge => "FILE_TOO_LARGE",
            Self::RowLimitExceeded => "ROW_LIMIT_EXCEEDED",
            Self::UnsupportedMediaType => "UNSUPPORTED_MEDIA_TYPE",
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::RowConversionFailed => "ROW_CONVERSION_FAILED",
            Self::ProcessingTimeout => "PROCESSING_TIMEOUT",
            Self::Cancelled => "CANCELLED",
            Self::TransportFailed => "TRANSPORT_FAILED",
            Self::StorageFailed => "STORAGE_FAILED",
            Self::Internal => "INTERNAL",
        }
    }

    /// 返回推荐 HTTP 状态码。
    #[must_use]
    pub const fn status_code(self) -> StatusCode {
        match self {
            Self::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::RowLimitExceeded | Self::InvalidFormat | Self::RowConversionFailed => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::ProcessingTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::Cancelled => StatusCode::REQUEST_TIMEOUT,
            Self::TransportFailed => StatusCode::BAD_REQUEST,
            Self::StorageFailed | Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 返回调用方是否适合在修复输入之外直接重试。
    #[must_use]
    pub const fn retryable(self) -> bool {
        matches!(
            self,
            Self::ProcessingTimeout | Self::Cancelled | Self::TransportFailed | Self::StorageFailed
        )
    }
}

impl std::fmt::Display for ExcelWebErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_all_variants() {
        assert_eq!(ExcelWebErrorCode::FileTooLarge.as_str(), "FILE_TOO_LARGE");
        assert_eq!(
            ExcelWebErrorCode::RowLimitExceeded.as_str(),
            "ROW_LIMIT_EXCEEDED"
        );
        assert_eq!(
            ExcelWebErrorCode::UnsupportedMediaType.as_str(),
            "UNSUPPORTED_MEDIA_TYPE"
        );
        assert_eq!(ExcelWebErrorCode::InvalidFormat.as_str(), "INVALID_FORMAT");
        assert_eq!(
            ExcelWebErrorCode::RowConversionFailed.as_str(),
            "ROW_CONVERSION_FAILED"
        );
        assert_eq!(
            ExcelWebErrorCode::ProcessingTimeout.as_str(),
            "PROCESSING_TIMEOUT"
        );
        assert_eq!(ExcelWebErrorCode::Cancelled.as_str(), "CANCELLED");
        assert_eq!(
            ExcelWebErrorCode::TransportFailed.as_str(),
            "TRANSPORT_FAILED"
        );
        assert_eq!(ExcelWebErrorCode::StorageFailed.as_str(), "STORAGE_FAILED");
        assert_eq!(ExcelWebErrorCode::Internal.as_str(), "INTERNAL");
    }

    #[test]
    fn status_code_mapping() {
        assert_eq!(
            ExcelWebErrorCode::FileTooLarge.status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            ExcelWebErrorCode::RowLimitExceeded.status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            ExcelWebErrorCode::InvalidFormat.status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            ExcelWebErrorCode::RowConversionFailed.status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            ExcelWebErrorCode::UnsupportedMediaType.status_code(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
        assert_eq!(
            ExcelWebErrorCode::ProcessingTimeout.status_code(),
            StatusCode::GATEWAY_TIMEOUT
        );
        assert_eq!(
            ExcelWebErrorCode::Cancelled.status_code(),
            StatusCode::REQUEST_TIMEOUT
        );
        assert_eq!(
            ExcelWebErrorCode::TransportFailed.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ExcelWebErrorCode::StorageFailed.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            ExcelWebErrorCode::Internal.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn retryable_flags() {
        assert!(ExcelWebErrorCode::ProcessingTimeout.retryable());
        assert!(ExcelWebErrorCode::Cancelled.retryable());
        assert!(ExcelWebErrorCode::TransportFailed.retryable());
        assert!(ExcelWebErrorCode::StorageFailed.retryable());
        assert!(!ExcelWebErrorCode::FileTooLarge.retryable());
        assert!(!ExcelWebErrorCode::RowLimitExceeded.retryable());
        assert!(!ExcelWebErrorCode::UnsupportedMediaType.retryable());
        assert!(!ExcelWebErrorCode::InvalidFormat.retryable());
        assert!(!ExcelWebErrorCode::RowConversionFailed.retryable());
        assert!(!ExcelWebErrorCode::Internal.retryable());
    }

    #[test]
    fn display_impl() {
        assert_eq!(format!("{}", ExcelWebErrorCode::Internal), "INTERNAL");
        assert_eq!(
            format!("{}", ExcelWebErrorCode::FileTooLarge),
            "FILE_TOO_LARGE"
        );
    }
}
