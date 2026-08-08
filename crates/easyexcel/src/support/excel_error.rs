//! Mirrors the union of Java `com.alibaba.excel.exception.*` classes.

use thiserror::Error;

/// 对应 Java：com.alibaba.excel.exception.*。 All public easyexcel errors with row and column diagnostics where applicable.
///
/// Java uses seven `RuntimeException` subclasses (`ExcelCommonException`,
/// `ExcelAnalysisException`, `ExcelAnalysisStopException`, etc.). Rust
/// collapses them into a single `Error` enum with `thiserror` for
/// ergonomic `Display` / `From<io::Error>` integration.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExcelError {
    /// A cell-to-field conversion error. (Java `ExcelDataConvertException`)
    #[error(
        "sheet={sheet}, row={row}, column={column:?}, field={field}, value={value:?}: {message}"
    )]
    Data {
        /// Sheet name.
        sheet: String,
        /// Zero-based row index.
        row: u32,
        /// Zero-based column index.
        column: Option<usize>,
        /// Rust field name.
        field: &'static str,
        /// Original cell text.
        value: String,
        /// Human-readable failure reason.
        message: String,
    },
    /// A requested worksheet does not exist. (Java `SheetNotFoundException`)
    #[error("worksheet not found: {0}")]
    SheetNotFound(String),
    /// The workbook or OOXML package is invalid. (Java `ExcelAnalysisException`)
    #[error("excel format error: {0}")]
    Format(String),
    /// The requested operation is not supported by the selected engine. (Java `ExcelCommonException`)
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    /// Callers requested a normal early stop of the entire analysis.
    /// (Java `ExcelAnalysisStopException`)
    #[error("analysis stopped: {0}")]
    AnalysisStop(String),
    /// Callers requested a normal early stop of the current worksheet.
    /// (Java `ExcelAnalysisStopSheetException`)
    #[error("sheet analysis stopped: {0}")]
    AnalysisStopSheet(String),
    /// 输入、输出或计算超过配置的资源上限。
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
    /// An I/O operation failed. (Java `ExcelCommonException` wrapping `IOException`)
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Clone for ExcelError {
    fn clone(&self) -> Self {
        match self {
            Self::Data {
                sheet,
                row,
                column,
                field,
                value,
                message,
            } => Self::Data {
                sheet: sheet.clone(),
                row: *row,
                column: *column,
                field,
                value: value.clone(),
                message: message.clone(),
            },
            Self::SheetNotFound(s) => Self::SheetNotFound(s.clone()),
            Self::Format(s) => Self::Format(s.clone()),
            Self::Unsupported(s) => Self::Unsupported(s.clone()),
            Self::AnalysisStop(s) => Self::AnalysisStop(s.clone()),
            Self::AnalysisStopSheet(s) => Self::AnalysisStopSheet(s.clone()),
            Self::ResourceLimit(s) => Self::ResourceLimit(s.clone()),
            Self::Io(e) => Self::Io(std::io::Error::new(e.kind(), e.to_string())),
        }
    }
}

impl PartialEq for ExcelError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Data {
                    sheet: s1,
                    row: r1,
                    column: c1,
                    field: f1,
                    value: v1,
                    message: m1,
                },
                Self::Data {
                    sheet: s2,
                    row: r2,
                    column: c2,
                    field: f2,
                    value: v2,
                    message: m2,
                },
            ) => s1 == s2 && r1 == r2 && c1 == c2 && f1 == f2 && v1 == v2 && m1 == m2,
            // 三个携带 String 负载的变体比较内容是否一致（合并同体分支）
            (Self::SheetNotFound(a), Self::SheetNotFound(b))
            | (Self::Format(a), Self::Format(b))
            | (Self::Unsupported(a), Self::Unsupported(b))
            | (Self::AnalysisStop(a), Self::AnalysisStop(b))
            | (Self::AnalysisStopSheet(a), Self::AnalysisStopSheet(b))
            | (Self::ResourceLimit(a), Self::ResourceLimit(b)) => a == b,
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind() && a.to_string() == b.to_string(),
            _ => false,
        }
    }
}

impl Eq for ExcelError {}

impl From<easyexcel_io::Error> for ExcelError {
    fn from(error: easyexcel_io::Error) -> Self {
        match error {
            easyexcel_io::Error::Io(error) => Self::Io(error),
            easyexcel_io::Error::SheetNotFound(sheet) => Self::SheetNotFound(sheet),
            easyexcel_io::Error::Unsupported(message) => Self::Unsupported(message),
            easyexcel_io::Error::ResourceLimit {
                resource,
                limit,
                actual,
            } => Self::ResourceLimit(format!(
                "resource limit exceeded: {resource} limit={limit} actual={actual}"
            )),
            other => Self::Format(other.to_string()),
        }
    }
}

impl From<easyexcel_format::NumberFormatError> for ExcelError {
    fn from(error: easyexcel_format::NumberFormatError) -> Self {
        Self::Format(error.to_string())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    fn data_error() -> ExcelError {
        ExcelError::Data {
            sheet: "Sheet1".to_string(),
            row: 2,
            column: Some(3),
            field: "name",
            value: "abc".to_string(),
            message: "convert failed".to_string(),
        }
    }

    #[test]
    fn clone_preserves_every_variant() {
        // 对应 Java：异常对象的深拷贝语义
        let data = data_error();
        assert_eq!(data.clone(), data);

        let sheet = ExcelError::SheetNotFound("s".to_string());
        assert_eq!(sheet.clone(), sheet);

        let format = ExcelError::Format("f".to_string());
        assert_eq!(format.clone(), format);

        let stop = ExcelError::AnalysisStop("done".to_string());
        assert_eq!(stop.clone(), stop);

        let stop_sheet = ExcelError::AnalysisStopSheet("done".to_string());
        assert_eq!(stop_sheet.clone(), stop_sheet);

        let unsupported = ExcelError::Unsupported("u".to_string());
        assert_eq!(unsupported.clone(), unsupported);

        let io = ExcelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "no file"));
        let cloned = io.clone();
        assert!(
            matches!(cloned, ExcelError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound)
        );
    }

    #[test]
    fn partial_eq_matches_same_variant_and_rejects_different() {
        // 对应 Java：异常相等性判断
        assert_eq!(data_error(), data_error());
        // 仅 column 不同的 Data 不相等
        assert_ne!(
            data_error(),
            ExcelError::Data {
                sheet: "Sheet1".to_string(),
                row: 2,
                column: None,
                field: "name",
                value: "abc".to_string(),
                message: "convert failed".to_string(),
            }
        );

        assert_eq!(
            ExcelError::SheetNotFound("a".to_string()),
            ExcelError::SheetNotFound("a".to_string())
        );
        assert_ne!(
            ExcelError::SheetNotFound("a".to_string()),
            ExcelError::SheetNotFound("b".to_string())
        );
        assert_eq!(
            ExcelError::Format("f".to_string()),
            ExcelError::Format("f".to_string())
        );
        assert_eq!(
            ExcelError::Unsupported("u".to_string()),
            ExcelError::Unsupported("u".to_string())
        );
        assert_ne!(
            ExcelError::Unsupported("u".to_string()),
            ExcelError::Format("u".to_string())
        );
        // Io 比较 kind 与消息
        let io1 = ExcelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "x"));
        let io2 = ExcelError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "x"));
        assert_eq!(io1, io2);
        // 不同变体不相等
        assert_ne!(data_error(), ExcelError::Format("f".to_string()));
    }
}
