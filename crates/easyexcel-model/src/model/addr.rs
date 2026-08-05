//! Cell addresses and ranges, plus A1 / R1C1 conversions.

use std::fmt;

/// A single cell address. Row and column are 0-based internally.
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
    pub const fn new(row: u32, col: u32) -> Self {
        CellAddress {
            row,
            col,
            abs_row: false,
            abs_col: false,
        }
    }

    pub const fn absolute(row: u32, col: u32) -> Self {
        CellAddress {
            row,
            col,
            abs_row: true,
            abs_col: true,
        }
    }

    /// Parse an A1-style reference such as `A1`, `$A$1`, `B$10`.
    ///
    /// Returns `None` if the string is not a valid single-cell reference.
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

    /// Render as A1, honoring the absolute flags (`$A$1`).
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

    /// Render as plain A1 ignoring absolute flags.
    pub fn to_a1_relative(self) -> String {
        format!("{}{}", col_index_to_letters(self.col), self.row + 1)
    }

    /// Render in R1C1 absolute notation, e.g. `R1C1`.
    pub fn to_r1c1(self) -> String {
        format!("R{}C{}", self.row + 1, self.col + 1)
    }

    /// Render in R1C1 notation relative to a base cell, e.g. `R[-1]C[2]`.
    pub fn to_r1c1_relative(self, base: CellAddress) -> String {
        let r = if self.abs_row {
            format!("R{}", self.row + 1)
        } else {
            let d = self.row as i64 - base.row as i64;
            if d == 0 {
                "R".to_string()
            } else {
                format!("R[{d}]")
            }
        };
        let c = if self.abs_col {
            format!("C{}", self.col + 1)
        } else {
            let d = self.col as i64 - base.col as i64;
            if d == 0 {
                "C".to_string()
            } else {
                format!("C[{d}]")
            }
        };
        format!("{r}{c}")
    }

    /// Parse an R1C1 reference, optionally relative to `base`.
    ///
    /// Supports `R1C1`, `R[-1]C[2]`, `RC`, `R[1]C` forms.
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
        let v = base as i64 + num;
        if v < 0 {
            return None;
        }
        Some((v as u32, false))
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

/// A rectangular range of cells, e.g. `A1:B10`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRange {
    pub start: CellAddress,
    pub end: CellAddress,
}

impl CellRange {
    pub fn new(start: CellAddress, end: CellAddress) -> Self {
        // Normalize so start is top-left, end is bottom-right (preserving abs flags).
        let (r0, r1) = (start.row.min(end.row), start.row.max(end.row));
        let (c0, c1) = (start.col.min(end.col), start.col.max(end.col));
        CellRange {
            start: CellAddress {
                row: r0,
                col: c0,
                abs_row: start.abs_row,
                abs_col: start.abs_col,
            },
            end: CellAddress {
                row: r1,
                col: c1,
                abs_row: end.abs_row,
                abs_col: end.abs_col,
            },
        }
    }

    pub fn single(addr: CellAddress) -> Self {
        CellRange {
            start: addr,
            end: addr,
        }
    }

    /// Parse `A1:B10` (or a single `A1`).
    pub fn parse_a1(s: &str) -> Option<CellRange> {
        if let Some((a, b)) = s.split_once(':') {
            Some(CellRange::new(
                CellAddress::parse_a1(a)?,
                CellAddress::parse_a1(b)?,
            ))
        } else {
            Some(CellRange::single(CellAddress::parse_a1(s)?))
        }
    }

    pub fn to_a1(self) -> String {
        if self.start == self.end {
            self.start.to_a1()
        } else {
            format!("{}:{}", self.start.to_a1(), self.end.to_a1())
        }
    }

    pub fn rows(self) -> u32 {
        self.end.row - self.start.row + 1
    }

    pub fn cols(self) -> u32 {
        self.end.col - self.start.col + 1
    }

    pub fn contains(self, row: u32, col: u32) -> bool {
        row >= self.start.row && row <= self.end.row && col >= self.start.col && col <= self.end.col
    }

    /// Iterate (row, col) pairs in row-major order.
    pub fn iter_cells(self) -> impl Iterator<Item = (u32, u32)> {
        let (r0, r1, c0, c1) = (self.start.row, self.end.row, self.start.col, self.end.col);
        (r0..=r1).flat_map(move |r| (c0..=c1).map(move |c| (r, c)))
    }
}

impl fmt::Display for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_a1())
    }
}

/// Convert a column letter string (`A`, `Z`, `AA`, `XFD`) to a 0-based index.
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

/// Convert a 0-based column index to letters (`0 → A`, `26 → AA`).
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
    String::from_utf8(buf).unwrap()
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
}
