/// 对应 Java：无直接对应对象；Rust 架构扩展。 Excel cell error values. These are first-class values that can be stored in
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
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
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
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn biff_code(self) -> u8 {
        match self {
            CellError::Null => 0x00,
            CellError::Div0 => 0x07,
            CellError::Value => 0x0F,
            CellError::Ref => 0x17,
            CellError::Name => 0x1D,
            CellError::Num => 0x24,
            CellError::NA | CellError::Spill | CellError::Calc => 0x2A,
            // The following have no classic BIFF code; map to #N/A on write.
            CellError::GettingData => 0x2B,
        }
    }

    /// Decode a BIFF8 error code byte.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn from_biff_code(code: u8) -> CellError {
        match code {
            0x00 => CellError::Null,
            0x07 => CellError::Div0,
            0x17 => CellError::Ref,
            0x1D => CellError::Name,
            0x24 => CellError::Num,
            0x2A => CellError::NA,
            0x2B => CellError::GettingData,
            _ => CellError::Value,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse a display string like `#DIV/0!` into a [`CellError`].
    #[must_use]
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

