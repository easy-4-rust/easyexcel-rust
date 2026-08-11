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

#[cfg(test)]
mod tests {
    use super::*;

    // --- as_str 测试 ---------------------------------------------------

    #[test]
    fn as_str_null() {
        assert_eq!(CellError::Null.as_str(), "#NULL!");
    }

    #[test]
    fn as_str_div0() {
        assert_eq!(CellError::Div0.as_str(), "#DIV/0!");
    }

    #[test]
    fn as_str_value() {
        assert_eq!(CellError::Value.as_str(), "#VALUE!");
    }

    #[test]
    fn as_str_ref() {
        assert_eq!(CellError::Ref.as_str(), "#REF!");
    }

    #[test]
    fn as_str_name() {
        assert_eq!(CellError::Name.as_str(), "#NAME?");
    }

    #[test]
    fn as_str_num() {
        assert_eq!(CellError::Num.as_str(), "#NUM!");
    }

    #[test]
    fn as_str_na() {
        assert_eq!(CellError::NA.as_str(), "#N/A");
    }

    #[test]
    fn as_str_getting_data() {
        assert_eq!(CellError::GettingData.as_str(), "#GETTING_DATA");
    }

    #[test]
    fn as_str_spill() {
        assert_eq!(CellError::Spill.as_str(), "#SPILL!");
    }

    #[test]
    fn as_str_calc() {
        assert_eq!(CellError::Calc.as_str(), "#CALC!");
    }

    // --- biff_code 测试 ------------------------------------------------

    #[test]
    fn biff_code_null() {
        assert_eq!(CellError::Null.biff_code(), 0x00);
    }

    #[test]
    fn biff_code_div0() {
        assert_eq!(CellError::Div0.biff_code(), 0x07);
    }

    #[test]
    fn biff_code_value() {
        assert_eq!(CellError::Value.biff_code(), 0x0F);
    }

    #[test]
    fn biff_code_ref() {
        assert_eq!(CellError::Ref.biff_code(), 0x17);
    }

    #[test]
    fn biff_code_name() {
        assert_eq!(CellError::Name.biff_code(), 0x1D);
    }

    #[test]
    fn biff_code_num() {
        assert_eq!(CellError::Num.biff_code(), 0x24);
    }

    #[test]
    fn biff_code_na() {
        assert_eq!(CellError::NA.biff_code(), 0x2A);
    }

    #[test]
    fn biff_code_spill() {
        // Spill maps to same code as NA
        assert_eq!(CellError::Spill.biff_code(), 0x2A);
    }

    #[test]
    fn biff_code_calc() {
        // Calc maps to same code as NA
        assert_eq!(CellError::Calc.biff_code(), 0x2A);
    }

    #[test]
    fn biff_code_getting_data() {
        assert_eq!(CellError::GettingData.biff_code(), 0x2B);
    }

    // --- from_biff_code 测试 -------------------------------------------

    #[test]
    fn from_biff_0x00_is_null() {
        assert_eq!(CellError::from_biff_code(0x00), CellError::Null);
    }

    #[test]
    fn from_biff_0x07_is_div0() {
        assert_eq!(CellError::from_biff_code(0x07), CellError::Div0);
    }

    #[test]
    fn from_biff_0x17_is_ref() {
        assert_eq!(CellError::from_biff_code(0x17), CellError::Ref);
    }

    #[test]
    fn from_biff_0x1d_is_name() {
        assert_eq!(CellError::from_biff_code(0x1D), CellError::Name);
    }

    #[test]
    fn from_biff_0x24_is_num() {
        assert_eq!(CellError::from_biff_code(0x24), CellError::Num);
    }

    #[test]
    fn from_biff_0x2a_is_na() {
        assert_eq!(CellError::from_biff_code(0x2A), CellError::NA);
    }

    #[test]
    fn from_biff_0x2b_is_getting_data() {
        assert_eq!(CellError::from_biff_code(0x2B), CellError::GettingData);
    }

    #[test]
    fn from_biff_unknown_is_value() {
        // Unknown codes default to Value
        assert_eq!(CellError::from_biff_code(0xFF), CellError::Value);
        assert_eq!(CellError::from_biff_code(0x01), CellError::Value);
    }

    // --- parse 测试 ----------------------------------------------------

    #[test]
    fn parse_null() {
        assert_eq!(CellError::parse("#NULL!"), Some(CellError::Null));
    }

    #[test]
    fn parse_div0() {
        assert_eq!(CellError::parse("#DIV/0!"), Some(CellError::Div0));
    }

    #[test]
    fn parse_value() {
        assert_eq!(CellError::parse("#VALUE!"), Some(CellError::Value));
    }

    #[test]
    fn parse_ref() {
        assert_eq!(CellError::parse("#REF!"), Some(CellError::Ref));
    }

    #[test]
    fn parse_name() {
        assert_eq!(CellError::parse("#NAME?"), Some(CellError::Name));
    }

    #[test]
    fn parse_num() {
        assert_eq!(CellError::parse("#NUM!"), Some(CellError::Num));
    }

    #[test]
    fn parse_na() {
        assert_eq!(CellError::parse("#N/A"), Some(CellError::NA));
    }

    #[test]
    fn parse_getting_data() {
        assert_eq!(
            CellError::parse("#GETTING_DATA"),
            Some(CellError::GettingData)
        );
    }

    #[test]
    fn parse_spill() {
        assert_eq!(CellError::parse("#SPILL!"), Some(CellError::Spill));
    }

    #[test]
    fn parse_calc() {
        assert_eq!(CellError::parse("#CALC!"), Some(CellError::Calc));
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(CellError::parse("#UNKNOWN!"), None);
        assert_eq!(CellError::parse("hello"), None);
        assert_eq!(CellError::parse(""), None);
    }

    // --- Display 测试 --------------------------------------------------

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", CellError::Null), "#NULL!");
    }

    #[test]
    fn display_div0() {
        assert_eq!(format!("{}", CellError::Div0), "#DIV/0!");
    }

    #[test]
    fn display_value() {
        assert_eq!(format!("{}", CellError::Value), "#VALUE!");
    }

    #[test]
    fn display_ref() {
        assert_eq!(format!("{}", CellError::Ref), "#REF!");
    }

    #[test]
    fn display_name() {
        assert_eq!(format!("{}", CellError::Name), "#NAME?");
    }

    #[test]
    fn display_num() {
        assert_eq!(format!("{}", CellError::Num), "#NUM!");
    }

    #[test]
    fn display_na() {
        assert_eq!(format!("{}", CellError::NA), "#N/A");
    }

    #[test]
    fn display_getting_data() {
        assert_eq!(format!("{}", CellError::GettingData), "#GETTING_DATA");
    }

    #[test]
    fn display_spill() {
        assert_eq!(format!("{}", CellError::Spill), "#SPILL!");
    }

    #[test]
    fn display_calc() {
        assert_eq!(format!("{}", CellError::Calc), "#CALC!");
    }

    // --- Debug / Clone / PartialEq 测试 --------------------------------

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", CellError::Null), "Null");
        assert_eq!(format!("{:?}", CellError::Div0), "Div0");
    }

    #[test]
    fn clone_preserves_value() {
        let e = CellError::Num;
        let cloned = e;
        assert_eq!(e, cloned);
    }

    #[test]
    fn equality_works() {
        assert_eq!(CellError::Num, CellError::Num);
        assert_ne!(CellError::Num, CellError::NA);
    }

    #[test]
    fn hash_consistency() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(CellError::Num, "num");
        map.insert(CellError::NA, "na");
        assert_eq!(map.get(&CellError::Num), Some(&"num"));
        assert_eq!(map.get(&CellError::NA), Some(&"na"));
    }
}

