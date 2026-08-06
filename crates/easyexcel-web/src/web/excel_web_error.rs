use std::io;

use easyexcel::ExcelError;
use http::StatusCode;
use thiserror::Error;

use super::{ExcelProblemDetails, ExcelWebErrorCode};

/// `EasyExcel` Web 管线的统一错误。
///
/// 该错误保留服务端诊断来源，同时通过 [`Self::problem_details`] 对外输出
/// 不包含临时路径和内部实现细节的稳定协议。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExcelWebError {
    /// 上传文件超过字节限制。
    #[error("uploaded file contains {actual} bytes, limit is {limit} bytes")]
    FileTooLarge {
        /// 已接收或预计写入的字节数。
        actual: u64,
        /// 配置的最大字节数。
        limit: u64,
    },
    /// 读取或写出的行数超过限制。
    #[error("row count exceeds limit {limit}")]
    RowLimitExceeded {
        /// 配置的最大总行数。
        limit: u64,
    },
    /// 上传格式不受支持。
    #[error("unsupported spreadsheet format: {extension}")]
    UnsupportedMediaType {
        /// 调用方提供的扩展名。
        extension: String,
    },
    /// 请求体传输失败。
    #[error("request body transport failed: {message}")]
    Transport {
        /// 服务端诊断信息；不会原样返回客户端。
        message: String,
    },
    /// 操作已取消。
    #[error("excel operation was cancelled")]
    Cancelled,
    /// 操作超过时间限制。
    #[error("excel operation exceeded its time limit")]
    ProcessingTimeout,
    /// `EasyExcel` 解析、转换或写出失败。
    #[error(transparent)]
    Excel(#[from] ExcelError),
    /// 临时存储或异步 I/O 失败。
    #[error(transparent)]
    Io(#[from] io::Error),
    /// 阻塞 Excel 任务异常终止。
    #[error("excel worker failed: {message}")]
    Worker {
        /// 服务端诊断信息；不会原样返回客户端。
        message: String,
    },
}

impl ExcelWebError {
    /// 创建取消错误。
    #[must_use]
    pub const fn cancelled() -> Self {
        Self::Cancelled
    }

    /// 创建处理超时错误。
    #[must_use]
    pub const fn processing_timeout() -> Self {
        Self::ProcessingTimeout
    }

    /// 返回稳定机器错误码。
    #[must_use]
    pub const fn code(&self) -> ExcelWebErrorCode {
        match self {
            Self::FileTooLarge { .. } => ExcelWebErrorCode::FileTooLarge,
            Self::RowLimitExceeded { .. } => ExcelWebErrorCode::RowLimitExceeded,
            Self::UnsupportedMediaType { .. } | Self::Excel(ExcelError::Unsupported(_)) => {
                ExcelWebErrorCode::UnsupportedMediaType
            }
            Self::Transport { .. } => ExcelWebErrorCode::TransportFailed,
            Self::Cancelled => ExcelWebErrorCode::Cancelled,
            Self::ProcessingTimeout => ExcelWebErrorCode::ProcessingTimeout,
            Self::Excel(ExcelError::Data { .. }) => ExcelWebErrorCode::RowConversionFailed,
            Self::Excel(ExcelError::Format(_) | ExcelError::SheetNotFound(_)) => {
                ExcelWebErrorCode::InvalidFormat
            }
            Self::Excel(ExcelError::Io(_)) | Self::Io(_) => ExcelWebErrorCode::StorageFailed,
            Self::Excel(_) | Self::Worker { .. } => ExcelWebErrorCode::Internal,
        }
    }

    /// 返回推荐 HTTP 状态码。
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        self.code().status_code()
    }

    /// 将内部错误转换为跨框架一致且脱敏的错误响应。
    #[must_use]
    pub fn problem_details(&self, request_id: impl Into<String>) -> ExcelProblemDetails {
        ExcelProblemDetails::new(self.code(), self.public_detail(), request_id)
    }

    fn public_detail(&self) -> String {
        match self {
            Self::FileTooLarge { actual, limit } => {
                format!("上传文件大小为 {actual} 字节，超过 {limit} 字节限制")
            }
            Self::RowLimitExceeded { limit } => format!("表格行数超过 {limit} 行限制"),
            Self::UnsupportedMediaType { extension } => {
                format!("不支持扩展名为 {extension} 的表格文件")
            }
            Self::Transport { .. } => "请求体传输失败".to_string(),
            Self::Cancelled => "操作已取消".to_string(),
            Self::ProcessingTimeout => "表格处理超时".to_string(),
            Self::Excel(ExcelError::Data { row, column, .. }) => column.map_or_else(
                || format!("第 {} 行的数据转换失败", row + 1),
                |column| format!("第 {} 行第 {} 列的数据转换失败", row + 1, column + 1),
            ),
            Self::Excel(ExcelError::SheetNotFound(_)) => "请求的工作表不存在".to_string(),
            Self::Excel(ExcelError::Format(_)) => "工作簿格式无效或已损坏".to_string(),
            Self::Excel(ExcelError::Unsupported(_)) => "当前表格操作不受支持".to_string(),
            Self::Excel(ExcelError::Io(_)) | Self::Io(_) => "临时存储操作失败".to_string(),
            Self::Excel(_) => "表格处理失败".to_string(),
            Self::Worker { .. } => "表格处理任务异常终止".to_string(),
        }
    }
}
