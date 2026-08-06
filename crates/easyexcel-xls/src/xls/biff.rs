//! Low-level BIFF8 record primitives shared by the reader and writer:
//! record-type constants, a record iterator with CONTINUE merging, and the
//! XLUnicodeString codec (including the per-CONTINUE-boundary grbit rule).

// ---- Record type constants -------------------------------------------------

pub use crate::biff8::record_sid::{
    BLANK_SID as BLANK, BOF_SID as BOF, BOOL_ERR_SID as BOOLERR, BOUND_SHEET_SID as BOUNDSHEET,
    CODE_PAGE_SID as CODEPAGE, CONTINUE_SID as CONTINUE, DATE_MODE_SID as DATEMODE,
    DIMENSION_SID as DIMENSION, EOF_SID as EOF, EXT_SST_SID as EXTSST, FILE_PASS_SID as FILEPASS,
    FONT_SID as FONT, FORMAT_SID as FORMAT, FORMULA_SID as FORMULA, LABEL_SID as LABEL,
    LABEL_SST_SID as LABELSST, MERGE_CELLS_SID as MERGECELLS, MUL_BLANK_SID as MULBLANK,
    MUL_RK_SID as MULRK, NUMBER_SID as NUMBER, PANE_SID as PANE, RK_SID as RK, SST_SID as SST,
    STRING_SID as STRING, STYLE_SID as STYLE, WINDOW2_SID as WINDOW2, XF_SID as XF,
};

/// Substream type for the workbook globals.
pub const DT_GLOBALS: u16 = 0x0005;
/// Substream type for a worksheet.
pub const DT_WORKSHEET: u16 = 0x0010;

/// BIFF8 version word.
pub const BIFF8_VERSION: u16 = 0x0600;

/// Maximum data payload for a single record (excluding the 4-byte header).
pub const MAX_RECORD_DATA: usize = 8224;

// ---- Reader: raw record iteration ------------------------------------------

/// A single parsed BIFF record (CONTINUE records already merged in by
/// [`Records`] for records that carry overflow data).
pub struct RawRecord {
    pub typ: u16,
    pub data: Vec<u8>,
    /// For SST specifically we need to know where each CONTINUE boundary fell
    /// in the merged byte stream, because the grbit byte restarts there. This
    /// holds the byte offset (within `data`) at which each continuation block
    /// began. Empty for records with no continuation.
    pub continue_breaks: Vec<usize>,
}

/// Iterator over BIFF records in a byte buffer. CONTINUE (0x003C) records are
/// automatically appended to the preceding record's data, and their boundary
/// offsets recorded in `continue_breaks`.
pub struct Records<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Records<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Records { buf, pos: 0 }
    }

    fn read_header(&self, at: usize) -> Option<(u16, usize)> {
        if at + 4 > self.buf.len() {
            return None;
        }
        let typ = u16::from_le_bytes([self.buf[at], self.buf[at + 1]]);
        let len = u16::from_le_bytes([self.buf[at + 2], self.buf[at + 3]]) as usize;
        Some((typ, len))
    }
}

impl<'a> Iterator for Records<'a> {
    type Item = RawRecord;

    fn next(&mut self) -> Option<RawRecord> {
        let (typ, len) = self.read_header(self.pos)?;
        let data_start = self.pos + 4;
        let data_end = (data_start + len).min(self.buf.len());
        let mut data = self.buf[data_start..data_end].to_vec();
        self.pos = data_end;

        // Merge any following CONTINUE records.
        let mut continue_breaks = Vec::new();
        while let Some((next_typ, next_len)) = self.read_header(self.pos) {
            if next_typ != CONTINUE {
                break;
            }
            let cstart = self.pos + 4;
            let cend = (cstart + next_len).min(self.buf.len());
            continue_breaks.push(data.len());
            data.extend_from_slice(&self.buf[cstart..cend]);
            self.pos = cend;
        }

        Some(RawRecord {
            typ,
            data,
            continue_breaks,
        })
    }
}

// ---- Little-endian read helpers --------------------------------------------

#[inline]
pub fn u16le(d: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([d[off], d[off + 1]])
}

#[inline]
pub fn u32le(d: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([d[off], d[off + 1], d[off + 2], d[off + 3]])
}

#[inline]
pub fn f64le(d: &[u8], off: usize) -> f64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[off..off + 8]);
    f64::from_le_bytes(b)
}

// ---- RK decode -------------------------------------------------------------

/// Decode the 4-byte RK encoded numeric value.
///
/// bit0 (0x01): the (already-decoded) value was multiplied by 100.
/// bit1 (0x02): integer (value = encoded >> 2) vs double (top 30 bits are the
/// high bits of the IEEE-754 mantissa/exponent, low 34 bits zero).
pub fn decode_rk(rk: u32) -> f64 {
    let div100 = rk & 0x01 != 0;
    let is_int = rk & 0x02 != 0;
    let mut value = if is_int {
        // Arithmetic shift right by 2 (sign-extending), dropping the 2 flag bits.
        ((rk as i32) >> 2) as f64
    } else {
        // The encoded 32 bits, with the low 2 flag bits cleared, are the high
        // 32 bits of a 64-bit IEEE-754 double; the low 32 bits are zero.
        let bits = ((rk & 0xFFFF_FFFC) as u64) << 32;
        f64::from_bits(bits)
    };
    if div100 {
        value /= 100.0;
    }
    value
}

/// Try to encode an f64 as an RK value (lossless only). Returns `None` if the
/// number cannot be represented exactly in RK form (the caller should then emit
/// a full NUMBER record).
pub fn encode_rk(v: f64) -> Option<u32> {
    // Integer that fits in 30 signed bits.
    if v.fract() == 0.0 && v >= -(1i64 << 29) as f64 && v < (1i64 << 29) as f64 {
        let iv = v as i32;
        return Some(((iv << 2) as u32) | 0x02);
    }
    // v*100 integer that fits in 30 signed bits (div100 form).
    let v100 = v * 100.0;
    if v100.fract() == 0.0 && v100 >= -(1i64 << 29) as f64 && v100 < (1i64 << 29) as f64 {
        let iv = v100 as i32;
        return Some(((iv << 2) as u32) | 0x02 | 0x01);
    }
    // Double whose low 32 mantissa bits are zero.
    let bits = v.to_bits();
    if bits & 0x0000_0000_FFFF_FFFF == 0 {
        let hi = (bits >> 32) as u32;
        if hi & 0x03 == 0 {
            return Some(hi);
        }
    }
    None
}

// ---- XLUnicodeString codec -------------------------------------------------

/// Encode a string as a full BIFF8 XLUnicodeString (2-byte char count + grbit +
/// chars). Chooses compressed (latin1) when every char is <= 0xFF.
pub fn encode_unicode_string(s: &str) -> Vec<u8> {
    let chars: Vec<u16> = s.encode_utf16().collect();
    let compressed = chars.iter().all(|&c| c <= 0xFF);
    let mut out = Vec::new();
    out.extend_from_slice(&(chars.len() as u16).to_le_bytes());
    if compressed {
        out.push(0x00);
        for &c in &chars {
            out.push(c as u8);
        }
    } else {
        out.push(0x01);
        for &c in &chars {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}

/// Encode a string as a *short* XLUnicodeString (1-byte char count), used by
/// BOUNDSHEET sheet names. Char count is clamped to 255.
pub fn encode_short_unicode_string(s: &str) -> Vec<u8> {
    let chars: Vec<u16> = s.encode_utf16().take(255).collect();
    let compressed = chars.iter().all(|&c| c <= 0xFF);
    let mut out = Vec::new();
    out.push(chars.len() as u8);
    if compressed {
        out.push(0x00);
        for &c in &chars {
            out.push(c as u8);
        }
    } else {
        out.push(0x01);
        for &c in &chars {
            out.extend_from_slice(&c.to_le_bytes());
        }
    }
    out
}

/// Parse a short XLUnicodeString (1-byte char count) at `off`. Returns the
/// string and the offset just past it.
pub fn parse_short_unicode_string(d: &[u8], off: usize) -> (String, usize) {
    if off >= d.len() {
        return (String::new(), off);
    }
    let cch = d[off] as usize;
    let grbit = d.get(off + 1).copied().unwrap_or(0);
    let compressed = grbit & 0x01 == 0;
    let mut p = off + 2;
    let mut s = String::new();
    if compressed {
        for _ in 0..cch {
            if p >= d.len() {
                break;
            }
            s.push(d[p] as char);
            p += 1;
        }
    } else {
        let mut units = Vec::with_capacity(cch);
        for _ in 0..cch {
            if p + 1 >= d.len() {
                break;
            }
            units.push(u16::from_le_bytes([d[p], d[p + 1]]));
            p += 2;
        }
        s = String::from_utf16_lossy(&units);
    }
    (s, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk_decode_known() {
        // Integer 0 with int flag.
        assert_eq!(decode_rk(0x0000_0002), 0.0);
        // Integer 1 << 2 | int flag => 1.
        assert_eq!(decode_rk((1 << 2) | 0x02), 1.0);
        // Integer with div100: 1234 stored as (1234<<2)|3 => 12.34
        let enc = ((1234i32 << 2) as u32) | 0x03;
        assert!((decode_rk(enc) - 12.34).abs() < 1e-9);
        // Double form: 1.0 == 0x3FF0000000000000; high 32 bits = 0x3FF00000.
        assert_eq!(decode_rk(0x3FF0_0000), 1.0);
        // Negative integer: -5 => ((-5)<<2)|2.
        let enc = (((-5i32) << 2) as u32) | 0x02;
        assert_eq!(decode_rk(enc), -5.0);
    }

    #[test]
    fn rk_roundtrip() {
        for v in [0.0, 1.0, -5.0, 12.34, 100.0, -0.25, 1234.5, 3.0] {
            if let Some(enc) = encode_rk(v) {
                assert!((decode_rk(enc) - v).abs() < 1e-12, "rk roundtrip {v}");
            }
        }
        // 0.25 has low mantissa bits zero -> double form is fine.
        assert!(
            encode_rk(0.1).is_none() || (decode_rk(encode_rk(0.1).unwrap()) - 0.1).abs() < 1e-12
        );
    }
}
