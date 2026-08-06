//! Error types for the `xls` library.

use thiserror::Error;

include!("error/cell_error.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Top-level error type for fallible library operations (file I/O, parsing).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid XLSX file: {0}")]
    Xlsx(String),

    #[error("invalid XLS file: {0}")]
    Xls(String),

    #[error("invalid CSV: {0}")]
    Csv(String),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("OLE2/CFB error: {0}")]
    Cfb(String),

    #[error("XML error: {0}")]
    Xml(String),

    #[error("file is password-protected ({0}); supply a password to decrypt")]
    PasswordProtected(String),

    #[error("incorrect password (decryption produced invalid data)")]
    WrongPassword,

    #[error("unsupported feature: {0}")]
    Unsupported(String),

    #[error("formula parse error: {0}")]
    FormulaParse(String),

    #[error("{0}")]
    Other(String),
}

include!("error/result.rs");
