use thiserror::Error as ThisError;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 表格读取、写入和协议执行错误。
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// 文件系统或流错误。
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// XLSX 结构错误。
    #[error("invalid XLSX file: {0}")]
    Xlsx(String),
    /// XLS 结构错误。
    #[error("invalid XLS file: {0}")]
    Xls(String),
    /// CSV 结构错误。
    #[error("invalid CSV: {0}")]
    Csv(String),
    /// ZIP 容器错误。
    #[error("ZIP error: {0}")]
    Zip(String),
    /// OLE2/CFB 容器错误。
    #[error("OLE2/CFB error: {0}")]
    Cfb(String),
    /// XML 解析错误。
    #[error("XML error: {0}")]
    Xml(String),
    /// 请求的工作表不存在。
    #[error("worksheet not found: {0}")]
    SheetNotFound(String),
    /// 文件已加密但未提供密码。
    #[error("file is password-protected ({0}); supply a password to decrypt")]
    PasswordProtected(String),
    /// 密码不正确。
    #[error("incorrect password")]
    WrongPassword,
    /// 当前实现不支持请求的能力。
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    /// 公式解析错误。
    #[error("formula parse error: {0}")]
    FormulaParse(String),
    /// Markdown 语法或投影错误。
    #[error("invalid Markdown at line {line:?}: {message}")]
    Markdown {
        /// 可用时提供一基行号。
        line: Option<usize>,
        /// 稳定的人类可读诊断。
        message: String,
    },
    /// 输入、输出或计算超过资源限制。
    #[error("resource limit exceeded for {resource}: limit={limit}, actual={actual}")]
    ResourceLimit {
        /// 资源维度名称。
        resource: &'static str,
        /// 配置上限。
        limit: u64,
        /// 实际观测值。
        actual: u64,
    },
    /// 其他带上下文的错误。
    #[error("{0}")]
    Other(String),
}

impl From<zip::result::ZipError> for Error {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Self::Csv(error.to_string())
    }
}

impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Self::Xml(error.to_string())
    }
}

include!("error/result.rs");
