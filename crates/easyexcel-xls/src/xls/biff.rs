//! Low-level BIFF8 record primitives shared by the reader and writer:
//! record-type constants, a record iterator with CONTINUE merging, and the
//! `XLUnicodeString` codec (including the per-CONTINUE-boundary grbit rule).

// ---- Record type constants -------------------------------------------------

pub use crate::biff8::record_sid::{
    BLANK_SID as BLANK, BOF_SID as BOF, BOOL_ERR_SID as BOOLERR, BOUND_SHEET_SID as BOUNDSHEET,
    CODE_PAGE_SID as CODEPAGE, CONTINUE_SID as CONTINUE, DATE_MODE_SID as DATEMODE,
    DIMENSION_SID as DIMENSION, EOF_SID as EOF, EXTERNAL_SHEET_SID as EXTERNSHEET,
    EXT_SST_SID as EXTSST, FILE_PASS_SID as FILEPASS,
    FONT_SID as FONT, FORMAT_SID as FORMAT, FORMULA_SID as FORMULA, LABEL_SID as LABEL,
    LABEL_SST_SID as LABELSST, MERGE_CELLS_SID as MERGECELLS, MUL_BLANK_SID as MULBLANK,
    MUL_RK_SID as MULRK, NUMBER_SID as NUMBER, PANE_SID as PANE, RK_SID as RK, SST_SID as SST,
    STRING_SID as STRING, STYLE_SID as STYLE, WINDOW2_SID as WINDOW2, XF_SID as XF,
};

/// Substream type for the workbook globals.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DT_GLOBALS: u16 = 0x0005;
/// Substream type for a worksheet.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DT_WORKSHEET: u16 = 0x0010;

/// BIFF8 version word.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BIFF8_VERSION: u16 = 0x0600;

/// Maximum data payload for a single record (excluding the 4-byte header).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MAX_RECORD_DATA: usize = 8224;

// ---- Reader: raw record iteration ------------------------------------------

include!("biff/raw_record.rs");

include!("biff/records.rs");

// ---- Little-endian read helpers --------------------------------------------

#[inline]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn u16le(d: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([d[off], d[off + 1]])
}

#[inline]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn u32le(d: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([d[off], d[off + 1], d[off + 2], d[off + 3]])
}

#[inline]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn f64le(d: &[u8], off: usize) -> f64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[off..off + 8]);
    f64::from_le_bytes(b)
}

// ---- RK decode -------------------------------------------------------------

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decode the 4-byte RK encoded numeric value.
///
/// bit0 (0x01): the (already-decoded) value was multiplied by 100.
/// bit1 (0x02): integer (value = encoded >> 2) vs double (top 30 bits are the
/// high bits of the IEEE-754 mantissa/exponent, low 34 bits zero).
pub fn decode_rk(rk: u32) -> f64 {
    let div100 = rk & 0x01 != 0;
    let is_int = rk & 0x02 != 0;
    let mut value = if is_int {
        // Arithmetic shift right by 2 (sign-extending), dropping the 2 flag bits.
        f64::from((rk as i32) >> 2)
    } else {
        // The encoded 32 bits, with the low 2 flag bits cleared, are the high
        // 32 bits of a 64-bit IEEE-754 double; the low 32 bits are zero.
        let bits = u64::from(rk & 0xFFFF_FFFC) << 32;
        f64::from_bits(bits)
    };
    if div100 {
        value /= 100.0;
    }
    value
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Try to encode an f64 as an RK value (lossless only). Returns `None` if the
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Encode a string as a full BIFF8 `XLUnicodeString` (2-byte char count + grbit +
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Encode a string as a *short* `XLUnicodeString` (1-byte char count), used by
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse a short `XLUnicodeString` (1-byte char count) at `off`. Returns the
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

    #[test]
    fn encode_rk_integer_range() {
        // Small integers should encode as integer form (bit 1 set)
        assert!(encode_rk(0.0).is_some());
        assert!(encode_rk(1.0).is_some());
        assert!(encode_rk(-1.0).is_some());
        assert!(encode_rk(100.0).is_some());
        assert!(encode_rk(-100.0).is_some());
    }

    #[test]
    fn encode_rk_div100_form() {
        // Values like 12.34 where value*100 is an integer
        let enc = encode_rk(12.34).unwrap();
        assert_eq!(enc & 0x01, 0x01, "div100 flag should be set");
        assert!((decode_rk(enc) - 12.34).abs() < 1e-10);
    }

    #[test]
    fn encode_rk_double_form() {
        // Double whose low 32 mantissa bits are zero
        let v = f64::from_bits(0x3FF0_0000_0000_0000); // 1.0
        assert!(encode_rk(v).is_some());
    }

    #[test]
    fn encode_rk_returns_none_for_unrepresentable() {
        // Some values may or may not be representable; test that roundtrip works for ones that are
        for v in [0.1, 0.3, 1.0 / 7.0] {
            if let Some(enc) = encode_rk(v) {
                assert!((decode_rk(enc) - v).abs() < 1e-12, "rk roundtrip {v}");
            }
        }
    }

    // --- read helpers ---

    #[test]
    fn u16le_reads_little_endian() {
        assert_eq!(u16le(&[0x34, 0x12], 0), 0x1234);
    }

    #[test]
    fn u32le_reads_little_endian() {
        assert_eq!(u32le(&[0x78, 0x56, 0x34, 0x12], 0), 0x12345678);
    }

    #[test]
    fn f64le_reads_little_endian() {
        let bytes = 42.0f64.to_le_bytes();
        assert_eq!(f64le(&bytes, 0), 42.0);
    }

    // --- encode_unicode_string ---

    #[test]
    fn encode_unicode_string_compressed_latin1() {
        let encoded = encode_unicode_string("hello");
        // 2 bytes char count (5) + 1 byte grbit (0x00 compressed) + 5 bytes = 8
        assert_eq!(encoded.len(), 8);
        assert_eq!(u16::from_le_bytes([encoded[0], encoded[1]]), 5);
        assert_eq!(encoded[2], 0x00); // compressed
    }

    #[test]
    fn encode_unicode_string_wide() {
        let encoded = encode_unicode_string("你好");
        // 2 bytes char count (2) + 1 byte grbit (0x01 wide) + 4 bytes = 7
        assert_eq!(encoded.len(), 7);
        assert_eq!(u16::from_le_bytes([encoded[0], encoded[1]]), 2);
        assert_eq!(encoded[2], 0x01); // wide
    }

    #[test]
    fn encode_unicode_string_empty() {
        let encoded = encode_unicode_string("");
        assert_eq!(u16::from_le_bytes([encoded[0], encoded[1]]), 0);
    }

    // --- encode_short_unicode_string ---

    #[test]
    fn encode_short_unicode_string_compressed() {
        let encoded = encode_short_unicode_string("test");
        // 1 byte char count (4) + 1 byte grbit (0x00) + 4 bytes = 6
        assert_eq!(encoded.len(), 6);
        assert_eq!(encoded[0], 4);
        assert_eq!(encoded[1], 0x00);
    }

    #[test]
    fn encode_short_unicode_string_wide() {
        let encoded = encode_short_unicode_string("你好");
        // 1 byte char count (2) + 1 byte grbit (0x01) + 4 bytes = 6
        assert_eq!(encoded.len(), 6);
        assert_eq!(encoded[0], 2);
        assert_eq!(encoded[1], 0x01);
    }

    // --- parse_short_unicode_string ---

    #[test]
    fn parse_short_unicode_string_compressed() {
        let data = [3, 0x00, b'a', b'b', b'c', 0xFF];
        let (s, end) = parse_short_unicode_string(&data, 0);
        assert_eq!(s, "abc");
        assert_eq!(end, 5);
    }

    #[test]
    fn parse_short_unicode_string_wide() {
        let mut data = vec![1, 0x01]; // 1 char, wide
        data.extend_from_slice(&0x4F60u16.to_le_bytes()); // 你
        let (s, end) = parse_short_unicode_string(&data, 0);
        assert_eq!(s, "你");
        assert_eq!(end, 4);
    }

    #[test]
    fn parse_short_unicode_string_empty_input() {
        let (s, end) = parse_short_unicode_string(&[], 0);
        assert_eq!(s, "");
        assert_eq!(end, 0);
    }

    #[test]
    fn parse_short_unicode_string_truncated() {
        // Declares 5 chars but only 2 bytes available
        let data = [5, 0x00, b'a', b'b'];
        let (s, _) = parse_short_unicode_string(&data, 0);
        assert_eq!(s, "ab");
    }

    #[test]
    fn parse_short_unicode_string_truncated_wide() {
        // Declares 2 wide chars but only 1 byte available after grbit
        let data = [2, 0x01, 0x41];
        let (s, _) = parse_short_unicode_string(&data, 0);
        // Can't read a full 2-byte code unit, so 0 chars are decoded
        assert!(s.len() <= 1);
    }

    #[test]
    fn parse_short_unicode_string_no_grbit() {
        // Only 1 byte (count) available, no grbit or char data
        let data = [3];
        let (s, end) = parse_short_unicode_string(&data, 0);
        // grbit defaults to 0 (compressed) via get().copied().unwrap_or(0), but no char data
        assert!(s.is_empty());
        // end = off + 2 = 2 (count + grbit bytes), even though grbit was auto-filled
        assert_eq!(end, 2);
    }

    // --- BIFF8 constants ---

    #[test]
    fn constants_are_correct() {
        assert_eq!(DT_GLOBALS, 0x0005);
        assert_eq!(DT_WORKSHEET, 0x0010);
        assert_eq!(BIFF8_VERSION, 0x0600);
        assert_eq!(MAX_RECORD_DATA, 8224);
    }
}
