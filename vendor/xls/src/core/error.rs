//! Error types for the `xls` library.

use thiserror::Error;

/// Excel cell error values. These are first-class values that can be stored in
/// cells and propagated through formula evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellError {
    /// `#NULL!` — intersection of two ranges that don't intersect.
    Null,
    /// `#DIV/0!` — division by zero (or by blank).
    Div0,
    /// `#VALUE!` — wrong type of argument or operand.
    Value,
    /// `#REF!` — invalid cell reference.
    Ref,
    /// `#NAME?` — unrecognized function or defined name.
    Name,
    /// `#NUM!` — invalid numeric value.
    Num,
    /// `#N/A` — value not available (lookups, etc.).
    NA,
    /// `#GETTING_DATA` — placeholder used while external data loads. Rare.
    GettingData,
    /// `#SPILL!` — a dynamic array could not spill (post-v1; kept for parity).
    Spill,
    /// `#CALC!` — a calculation engine error (e.g. empty array). Post-v1.
    Calc,
}

impl CellError {
    /// The user-facing display string, e.g. `#DIV/0!`.
    pub const fn as_str(self) -> &'static str {
        match self {
            CellError::Null => "#NULL!",
            CellError::Div0 => "#DIV/0!",
            CellError::Value => "#VALUE!",
            CellError::Ref => "#REF!",
            CellError::Name => "#NAME?",
            CellError::Num => "#NUM!",
            CellError::NA => "#N/A",
            CellError::GettingData => "#GETTING_DATA",
            CellError::Spill => "#SPILL!",
            CellError::Calc => "#CALC!",
        }
    }

    /// The BIFF8 error code byte (used by the XLS reader/writer).
    pub const fn biff_code(self) -> u8 {
        match self {
            CellError::Null => 0x00,
            CellError::Div0 => 0x07,
            CellError::Value => 0x0F,
            CellError::Ref => 0x17,
            CellError::Name => 0x1D,
            CellError::Num => 0x24,
            CellError::NA => 0x2A,
            // The following have no classic BIFF code; map to #N/A on write.
            CellError::GettingData => 0x2B,
            CellError::Spill => 0x2A,
            CellError::Calc => 0x2A,
        }
    }

    /// Decode a BIFF8 error code byte.
    pub const fn from_biff_code(code: u8) -> CellError {
        match code {
            0x00 => CellError::Null,
            0x07 => CellError::Div0,
            0x0F => CellError::Value,
            0x17 => CellError::Ref,
            0x1D => CellError::Name,
            0x24 => CellError::Num,
            0x2A => CellError::NA,
            0x2B => CellError::GettingData,
            _ => CellError::Value,
        }
    }

    /// Parse a display string like `#DIV/0!` into a [`CellError`].
    pub fn parse(s: &str) -> Option<CellError> {
        Some(match s {
            "#NULL!" => CellError::Null,
            "#DIV/0!" => CellError::Div0,
            "#VALUE!" => CellError::Value,
            "#REF!" => CellError::Ref,
            "#NAME?" => CellError::Name,
            "#NUM!" => CellError::Num,
            "#N/A" => CellError::NA,
            "#GETTING_DATA" => CellError::GettingData,
            "#SPILL!" => CellError::Spill,
            "#CALC!" => CellError::Calc,
            _ => return None,
        })
    }
}

impl std::fmt::Display for CellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Top-level error type for fallible library operations (file I/O, parsing).
#[derive(Debug, Error)]
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

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Error::Zip(e.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::Csv(e.to_string())
    }
}

impl From<quick_xml::Error> for Error {
    fn from(e: quick_xml::Error) -> Self {
        Error::Xml(e.to_string())
    }
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, Error>;
