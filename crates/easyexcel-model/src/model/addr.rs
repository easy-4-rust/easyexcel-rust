//! Cell addresses and ranges, plus A1 / R1C1 conversions.

use std::fmt;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 A single cell address. Row and column are 0-based internally.
///
/// `abs_row` / `abs_col` record whether the original A1 reference pinned that
/// component with `$` (needed for round-trip fidelity and shared-formula offset
/// maths).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellAddress {
    pub row: u32,
    pub col: u32,
    pub abs_row: bool,
    pub abs_col: bool,
}

impl CellAddress {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(row: u32, col: u32) -> Self {
        CellAddress {
            row,
            col,
            abs_row: false,
            abs_col: false,
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn absolute(row: u32, col: u32) -> Self {
        CellAddress {
            row,
            col,
            abs_row: true,
            abs_col: true,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse an A1-style reference such as `A1`, `$A$1`, `B$10`.
    ///
    /// Returns `None` if the string is not a valid single-cell reference.
    #[must_use]
    pub fn parse_a1(s: &str) -> Option<CellAddress> {
        let bytes = s.as_bytes();
        let mut i = 0;
        let abs_col = bytes.get(i) == Some(&b'$');
        if abs_col {
            i += 1;
        }
        let col_start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        if i == col_start {
            return None;
        }
        let col = col_letters_to_index(&s[col_start..i])?;

        let abs_row = bytes.get(i) == Some(&b'$');
        if abs_row {
            i += 1;
        }
        let row_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == row_start || i != bytes.len() {
            return None;
        }
        let row1: u32 = s[row_start..i].parse().ok()?;
        if row1 == 0 {
            return None;
        }
        Some(CellAddress {
            row: row1 - 1,
            col,
            abs_row,
            abs_col,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Render as A1, honoring the absolute flags (`$A$1`).
    #[must_use]
    pub fn to_a1(self) -> String {
        let mut out = String::new();
        if self.abs_col {
            out.push('$');
        }
        out.push_str(&col_index_to_letters(self.col));
        if self.abs_row {
            out.push('$');
        }
        out.push_str(&(self.row + 1).to_string());
        out
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Render as plain A1 ignoring absolute flags.
    #[must_use]
    pub fn to_a1_relative(self) -> String {
        format!("{}{}", col_index_to_letters(self.col), self.row + 1)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Render in R1C1 absolute notation, e.g. `R1C1`.
    #[must_use]
    pub fn to_r1c1(self) -> String {
        format!("R{}C{}", self.row + 1, self.col + 1)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Render in R1C1 notation relative to a base cell, e.g. `R[-1]C[2]`.
    #[must_use]
    pub fn to_r1c1_relative(self, base: CellAddress) -> String {
        let r = if self.abs_row {
            format!("R{}", self.row + 1)
        } else {
            let d = i64::from(self.row) - i64::from(base.row);
            if d == 0 {
                "R".to_string()
            } else {
                format!("R[{d}]")
            }
        };
        let c = if self.abs_col {
            format!("C{}", self.col + 1)
        } else {
            let d = i64::from(self.col) - i64::from(base.col);
            if d == 0 {
                "C".to_string()
            } else {
                format!("C[{d}]")
            }
        };
        format!("{r}{c}")
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse an R1C1 reference, optionally relative to `base`.
    ///
    /// Supports `R1C1`, `R[-1]C[2]`, `RC`, `R[1]C` forms.
    #[must_use]
    pub fn parse_r1c1(s: &str, base: CellAddress) -> Option<CellAddress> {
        let bytes = s.as_bytes();
        if bytes.first() != Some(&b'R') && bytes.first() != Some(&b'r') {
            return None;
        }
        let mut i = 1;
        let (row, abs_row) = parse_r1c1_component(bytes, &mut i, base.row)?;
        if bytes.get(i) != Some(&b'C') && bytes.get(i) != Some(&b'c') {
            return None;
        }
        i += 1;
        let (col, abs_col) = parse_r1c1_component(bytes, &mut i, base.col)?;
        if i != bytes.len() {
            return None;
        }
        Some(CellAddress {
            row,
            col,
            abs_row,
            abs_col,
        })
    }
}

fn parse_r1c1_component(bytes: &[u8], i: &mut usize, base: u32) -> Option<(u32, bool)> {
    // After the R or C marker. Forms: "" (relative same), "[n]" (relative), "n" (absolute 1-based)
    if *i >= bytes.len() || bytes[*i] == b'C' || bytes[*i] == b'c' {
        // bare R or C → relative offset 0
        return Some((base, false));
    }
    if bytes[*i] == b'[' {
        *i += 1;
        let start = *i;
        if bytes.get(*i) == Some(&b'-') || bytes.get(*i) == Some(&b'+') {
            *i += 1;
        }
        while *i < bytes.len() && bytes[*i].is_ascii_digit() {
            *i += 1;
        }
        let num: i64 = std::str::from_utf8(&bytes[start..*i]).ok()?.parse().ok()?;
        if bytes.get(*i) != Some(&b']') {
            return None;
        }
        *i += 1;
        let value = i64::from(base).checked_add(num)?;
        Some((u32::try_from(value).ok()?, false))
    } else if bytes[*i].is_ascii_digit() {
        let start = *i;
        while *i < bytes.len() && bytes[*i].is_ascii_digit() {
            *i += 1;
        }
        let num: u32 = std::str::from_utf8(&bytes[start..*i]).ok()?.parse().ok()?;
        if num == 0 {
            return None;
        }
        Some((num - 1, true))
    } else {
        None
    }
}

impl fmt::Display for CellAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_a1())
    }
}

include!("addr/cell_range.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Convert a column letter string (`A`, `Z`, `AA`, `XFD`) to a 0-based index.
#[must_use]
pub fn col_letters_to_index(letters: &str) -> Option<u32> {
    if letters.is_empty() {
        return None;
    }
    let mut idx: u32 = 0;
    for ch in letters.chars() {
        let c = ch.to_ascii_uppercase();
        if !c.is_ascii_uppercase() {
            return None;
        }
        idx = idx
            .checked_mul(26)?
            .checked_add((c as u32 - 'A' as u32) + 1)?;
    }
    Some(idx - 1)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Convert a 0-based column index to letters (`0 → A`, `26 → AA`).
#[must_use]
pub fn col_index_to_letters(mut index: u32) -> String {
    let mut buf = Vec::new();
    loop {
        let rem = (index % 26) as u8;
        buf.push(b'A' + rem);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    buf.reverse();
    buf.into_iter().map(char::from).collect()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 从 A1 单元格引用读取零基行号；无效输入返回 `0`。
#[must_use]
pub fn row_from_a1(reference: &str) -> u32 {
    reference
        .chars()
        .skip_while(char::is_ascii_alphabetic)
        .collect::<String>()
        .parse::<u32>()
        .map_or(0, |row| row.saturating_sub(1))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 从 A1 单元格引用读取零基列号；无效输入返回 `0`。
#[must_use]
pub fn column_from_a1(reference: &str) -> u32 {
    let letters = reference
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect::<String>();
    col_letters_to_index(&letters).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_roundtrip() {
        for (idx, s) in [
            (0u32, "A"),
            (25, "Z"),
            (26, "AA"),
            (27, "AB"),
            (701, "ZZ"),
            (702, "AAA"),
            (16383, "XFD"),
        ] {
            assert_eq!(col_index_to_letters(idx), s);
            assert_eq!(col_letters_to_index(s), Some(idx));
        }
    }

    #[test]
    fn a1_parse() {
        let a = CellAddress::parse_a1("A1").unwrap();
        assert_eq!((a.row, a.col), (0, 0));
        let b = CellAddress::parse_a1("$C$10").unwrap();
        assert_eq!((b.row, b.col, b.abs_row, b.abs_col), (9, 2, true, true));
        let c = CellAddress::parse_a1("B$5").unwrap();
        assert_eq!((c.row, c.col, c.abs_row, c.abs_col), (4, 1, true, false));
        assert_eq!(CellAddress::parse_a1("A0"), None);
        assert_eq!(CellAddress::parse_a1("1A"), None);
        assert_eq!(CellAddress::parse_a1(""), None);
    }

    #[test]
    fn a1_display() {
        let a = CellAddress {
            row: 9,
            col: 2,
            abs_row: true,
            abs_col: true,
        };
        assert_eq!(a.to_a1(), "$C$10");
        assert_eq!(a.to_a1_relative(), "C10");
    }

    #[test]
    fn range_parse() {
        let r = CellRange::parse_a1("A1:B3").unwrap();
        assert_eq!(r.rows(), 3);
        assert_eq!(r.cols(), 2);
        assert_eq!(r.iter_cells().count(), 6);
        assert!(r.contains(2, 1));
        assert!(!r.contains(3, 0));
    }

    #[test]
    fn r1c1_roundtrip() {
        let base = CellAddress::new(5, 5);
        let a = CellAddress::parse_r1c1("R[-1]C[2]", base).unwrap();
        assert_eq!((a.row, a.col), (4, 7));
        let b = CellAddress::parse_r1c1("R1C1", base).unwrap();
        assert_eq!((b.row, b.col, b.abs_row), (0, 0, true));
        let c = CellAddress::parse_r1c1("RC", base).unwrap();
        assert_eq!((c.row, c.col), (5, 5));
        assert_eq!(a.to_r1c1_relative(base), "R[-1]C[2]");
    }

    #[test]
    fn r1c1_relative_offsets_reject_u32_overflow() {
        let base = CellAddress::new(u32::MAX, u32::MAX);
        assert!(CellAddress::parse_r1c1("R[1]C", base).is_none());
        assert!(CellAddress::parse_r1c1("RC[1]", base).is_none());
        assert!(CellAddress::parse_r1c1("R[4294967296]C", CellAddress::new(0, 0)).is_none());
    }

    #[test]
    fn cell_address_new_has_no_absolute_flags() {
        let a = CellAddress::new(5, 10);
        assert_eq!(a.row, 5);
        assert_eq!(a.col, 10);
        assert!(!a.abs_row);
        assert!(!a.abs_col);
    }

    #[test]
    fn cell_address_absolute_sets_flags() {
        let a = CellAddress::absolute(5, 10);
        assert!(a.abs_row);
        assert!(a.abs_col);
    }

    #[test]
    fn parse_a1_with_dollar_col_only() {
        let a = CellAddress::parse_a1("$A1").unwrap();
        assert_eq!((a.row, a.col, a.abs_row, a.abs_col), (0, 0, false, true));
    }

    #[test]
    fn to_a1_all_combinations() {
        let a = CellAddress {
            row: 0,
            col: 0,
            abs_row: false,
            abs_col: false,
        };
        assert_eq!(a.to_a1(), "A1");
        let b = CellAddress {
            row: 0,
            col: 0,
            abs_row: true,
            abs_col: true,
        };
        assert_eq!(b.to_a1(), "$A$1");
        let c = CellAddress {
            row: 0,
            col: 0,
            abs_row: false,
            abs_col: true,
        };
        assert_eq!(c.to_a1(), "$A1");
        let d = CellAddress {
            row: 0,
            col: 0,
            abs_row: true,
            abs_col: false,
        };
        assert_eq!(d.to_a1(), "A$1");
    }

    #[test]
    fn to_r1c1_absolute() {
        let a = CellAddress::new(0, 0);
        assert_eq!(a.to_r1c1(), "R1C1");
        let b = CellAddress::new(9, 2);
        assert_eq!(b.to_r1c1(), "R10C3");
    }

    #[test]
    fn to_r1c1_relative_same_cell() {
        let a = CellAddress::new(5, 5);
        assert_eq!(a.to_r1c1_relative(a), "RC");
    }

    #[test]
    fn to_r1c1_relative_with_absolute_components() {
        let base = CellAddress::new(5, 5);
        let a = CellAddress {
            row: 3,
            col: 7,
            abs_row: true,
            abs_col: false,
        };
        assert_eq!(a.to_r1c1_relative(base), "R4C[2]");
    }

    #[test]
    fn parse_r1c1_invalid_start() {
        assert!(CellAddress::parse_r1c1("X1C1", CellAddress::new(0, 0)).is_none());
    }

    #[test]
    fn parse_r1c1_missing_c() {
        assert!(CellAddress::parse_r1c1("R1X1", CellAddress::new(0, 0)).is_none());
    }

    #[test]
    fn parse_r1c1_invalid_row_zero() {
        assert!(CellAddress::parse_r1c1("R0C1", CellAddress::new(0, 0)).is_none());
    }

    #[test]
    fn parse_r1c1_trailing_chars() {
        assert!(CellAddress::parse_r1c1("R1C1X", CellAddress::new(0, 0)).is_none());
    }

    #[test]
    fn parse_r1c1_negative_offset() {
        let base = CellAddress::new(10, 10);
        let a = CellAddress::parse_r1c1("R[-5]C[-3]", base).unwrap();
        assert_eq!((a.row, a.col), (5, 7));
    }

    #[test]
    fn parse_r1c1_plus_sign() {
        let base = CellAddress::new(0, 0);
        let a = CellAddress::parse_r1c1("R[+1]C[+2]", base).unwrap();
        assert_eq!((a.row, a.col), (1, 2));
    }

    #[test]
    fn display_cell_address() {
        let a = CellAddress::new(9, 2);
        assert_eq!(format!("{a}"), "C10");
    }

    #[test]
    fn col_letters_to_index_invalid() {
        assert_eq!(col_letters_to_index(""), None);
        assert_eq!(col_letters_to_index("1"), None);
    }

    #[test]
    fn row_from_a1_basic() {
        assert_eq!(row_from_a1("A1"), 0);
        assert_eq!(row_from_a1("B10"), 9);
        assert_eq!(row_from_a1("invalid"), 0);
    }

    #[test]
    fn column_from_a1_basic() {
        assert_eq!(column_from_a1("A1"), 0);
        assert_eq!(column_from_a1("B10"), 1);
        assert_eq!(column_from_a1("Z1"), 25);
    }
}
