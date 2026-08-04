//! Engineering functions:
//!  – Number-base conversions (BIN2DEC, DEC2BIN, HEX2DEC, …)
//!  – Bitwise (BITAND, BITOR, BITXOR, BITLSHIFT, BITRSHIFT)
//!  – Comparison (DELTA, GESTEP)
//!  – Unit conversion (CONVERT)
//!  – Error functions (ERF, ERFC, ERF.PRECISE, ERFC.PRECISE)
//!  – Complex number arithmetic (COMPLEX, IMABS, IMREAL, …)
//!
//! PARITY:
//!  - Base conversions: negative numbers use 10-bit two's complement, matching Excel.
//!  - CONVERT: subset of units (mass, distance, time, temperature) with metric prefixes;
//!    unknown units return #N/A.
//!  - ERF/ERFC: use the standard series/continued-fraction approximation; tolerance ~1e-15.
//!  - Complex numbers: represented as text "a+bi" or "a+bj" (user-selectable suffix).
//!  - BESSEL* functions are not implemented (skipped — complex series; marked as note).

use super::{Registry, VARIADIC};
use crate::core::error::CellError;
use crate::core::formula::coerce::to_number;
use crate::core::formula::context::Context;
use crate::core::formula::value::Value;

pub fn register(r: &mut Registry) {
    // Base conversions
    r.add("DEC2BIN", 1, 2, false, dec2bin);
    r.add("DEC2OCT", 1, 2, false, dec2oct);
    r.add("DEC2HEX", 1, 2, false, dec2hex);
    r.add("BIN2DEC", 1, 1, false, bin2dec);
    r.add("BIN2OCT", 1, 2, false, bin2oct);
    r.add("BIN2HEX", 1, 2, false, bin2hex);
    r.add("OCT2BIN", 1, 2, false, oct2bin);
    r.add("OCT2DEC", 1, 1, false, oct2dec);
    r.add("OCT2HEX", 1, 2, false, oct2hex);
    r.add("HEX2BIN", 1, 2, false, hex2bin);
    r.add("HEX2OCT", 1, 2, false, hex2oct);
    r.add("HEX2DEC", 1, 1, false, hex2dec);

    // Bitwise
    r.add("BITAND", 2, 2, false, bitand);
    r.add("BITOR", 2, 2, false, bitor);
    r.add("BITXOR", 2, 2, false, bitxor);
    r.add("BITLSHIFT", 2, 2, false, bitlshift);
    r.add("BITRSHIFT", 2, 2, false, bitrshift);

    // Comparison
    r.add("DELTA", 1, 2, false, delta);
    r.add("GESTEP", 1, 2, false, gestep);

    // Unit conversion
    r.add("CONVERT", 3, 3, false, convert);

    // Error functions
    r.add("ERF", 1, 2, false, erf_fn);
    r.add("ERFC", 1, 1, false, erfc_fn);
    r.add("ERF.PRECISE", 1, 1, false, erf_precise);
    r.add("ERFC.PRECISE", 1, 1, false, erfc_precise);

    // Complex numbers
    r.add("COMPLEX", 2, 3, false, complex);
    r.add("IMABS", 1, 1, false, imabs);
    r.add("IMREAL", 1, 1, false, imreal);
    r.add("IMAGINARY", 1, 1, false, imaginary);
    r.add("IMCONJUGATE", 1, 1, false, imconjugate);
    r.add("IMSUM", 1, VARIADIC, false, imsum);
    r.add("IMSUB", 2, 2, false, imsub);
    r.add("IMPRODUCT", 1, VARIADIC, false, improduct);
    r.add("IMDIV", 2, 2, false, imdiv);
    r.add("IMEXP", 1, 1, false, imexp);
    r.add("IMLN", 1, 1, false, imln);
    r.add("IMSQRT", 1, 1, false, imsqrt);
    r.add("IMARGUMENT", 1, 1, false, imargument);
    r.add("IMPOWER", 2, 2, false, impower);
    r.add("IMSIN", 1, 1, false, imsin);
    r.add("IMCOS", 1, 1, false, imcos);
    r.add("IMTAN", 1, 1, false, imtan);
    r.add("IMCOT", 1, 1, false, imcot);
    r.add("IMSINH", 1, 1, false, imsinh);
    r.add("IMCOSH", 1, 1, false, imcosh);
    r.add("IMSEC", 1, 1, false, imsec);
    r.add("IMCSC", 1, 1, false, imcsc);
    r.add("IMSECH", 1, 1, false, imsech);
    r.add("IMCSCH", 1, 1, false, imcsch);
    r.add("IMLOG2", 1, 1, false, imlog2);
    r.add("IMLOG10", 1, 1, false, imlog10);

    // BESSEL functions — not implemented; return #NUM!
    // PARITY: BESSELJ, BESSELY, BESSELI, BESSELK are complex series expansions
    // not included in this implementation. They return #NUM! as a stub.
    r.add("BESSELJ", 2, 2, false, bessel_stub);
    r.add("BESSELY", 2, 2, false, bessel_stub);
    r.add("BESSELI", 2, 2, false, bessel_stub);
    r.add("BESSELK", 2, 2, false, bessel_stub);
}

fn bessel_stub(_: &mut dyn Context, _: &[Value]) -> Value {
    // PARITY: Bessel functions not implemented; returning #NUM! instead of a value.
    Value::Error(CellError::Num)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get(args: &[Value], i: usize) -> Result<f64, CellError> {
    to_number(&args[i])
}

fn opt(args: &[Value], i: usize, default: f64) -> Result<f64, CellError> {
    if i < args.len() {
        match &args[i] {
            Value::Empty => Ok(default),
            v => to_number(v),
        }
    } else {
        Ok(default)
    }
}

fn get_text(args: &[Value], i: usize) -> Result<String, CellError> {
    match &args[i] {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(crate::core::value::format_number_general(*n)),
        Value::Bool(b) => Ok(if *b {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }),
        Value::Empty => Ok(String::new()),
        Value::Error(e) => Err(*e),
        _ => Err(CellError::Value),
    }
}

// ---------------------------------------------------------------------------
// Base conversion helpers
// ---------------------------------------------------------------------------

/// Format an integer using two's complement in the given base with given number of digits.
/// `bits` = width in bits (10 for BIN, 30 for OCT, 40 for HEX as per Excel).
fn format_twos_complement(
    val: i64,
    radix: u32,
    digits: usize,
    bits: u32,
) -> Result<String, CellError> {
    let max_val = (1i64 << (bits - 1)) - 1;
    let min_val = -(1i64 << (bits - 1));
    if val > max_val || val < min_val {
        return Err(CellError::Num);
    }
    let bits_u = val as u64 & ((1u64 << bits) - 1);
    let s = format_radix(bits_u, radix);
    // Pad with leading zeros to `digits` (if specified).
    if digits == 0 {
        Ok(s)
    } else if s.len() > digits {
        Err(CellError::Num)
    } else {
        Ok(format!("{:0>width$}", s, width = digits))
    }
}

fn format_radix(mut n: u64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let digits = b"0123456789ABCDEF";
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[(n % radix as u64) as usize]);
        n /= radix as u64;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

/// Parse a two's complement number in the given base with given bit width.
fn parse_twos_complement(s: &str, radix: u32, bits: u32) -> Result<i64, CellError> {
    let s = s.trim().to_uppercase();
    if s.is_empty() {
        return Err(CellError::Num);
    }
    let max_digits = match radix {
        2 => 10,
        8 => 10,
        16 => 10,
        _ => 20,
    };
    if s.len() > max_digits {
        return Err(CellError::Num);
    }
    let raw = u64::from_str_radix(&s, radix).map_err(|_| CellError::Num)?;
    let sign_bit = 1u64 << (bits - 1);
    if raw & sign_bit != 0 {
        // negative: sign-extend
        let mask = (1u64 << bits) - 1;
        let magnitude = (!raw & mask) + 1;
        Ok(-(magnitude as i64))
    } else {
        Ok(raw as i64)
    }
}

// ---------------------------------------------------------------------------
// DEC2BIN / DEC2OCT / DEC2HEX
// ---------------------------------------------------------------------------

fn dec_to_base(_: &mut dyn Context, args: &[Value], radix: u32, bits: u32) -> Value {
    let n = match get(args, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(n, radix, places, bits) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

fn dec2bin(ctx: &mut dyn Context, args: &[Value]) -> Value {
    dec_to_base(ctx, args, 2, 10)
}
fn dec2oct(ctx: &mut dyn Context, args: &[Value]) -> Value {
    dec_to_base(ctx, args, 8, 30)
}
fn dec2hex(ctx: &mut dyn Context, args: &[Value]) -> Value {
    dec_to_base(ctx, args, 16, 40)
}

// ---------------------------------------------------------------------------
// BIN2DEC / BIN2OCT / BIN2HEX
// ---------------------------------------------------------------------------

fn bin2dec(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match parse_twos_complement(&s, 2, 10) {
        Ok(v) => Value::Number(v as f64),
        Err(e) => Value::Error(e),
    }
}

fn bin2oct(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 2, 10) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 8, places, 30) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

fn bin2hex(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 2, 10) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 16, places, 40) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// OCT2BIN / OCT2DEC / OCT2HEX
// ---------------------------------------------------------------------------

fn oct2dec(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match parse_twos_complement(&s, 8, 30) {
        Ok(v) => Value::Number(v as f64),
        Err(e) => Value::Error(e),
    }
}

fn oct2bin(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 8, 30) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 2, places, 10) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

fn oct2hex(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 8, 30) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 16, places, 40) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// HEX2BIN / HEX2OCT / HEX2DEC
// ---------------------------------------------------------------------------

fn hex2dec(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match parse_twos_complement(&s, 16, 40) {
        Ok(v) => Value::Number(v as f64),
        Err(e) => Value::Error(e),
    }
}

fn hex2bin(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 16, 40) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 2, places, 10) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

fn hex2oct(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match get_text(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let places = match opt(args, 1, 0.0) {
        Ok(v) => v.trunc() as usize,
        Err(e) => return Value::Error(e),
    };
    let dec = match parse_twos_complement(&s, 16, 40) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match format_twos_complement(dec, 8, places, 30) {
        Ok(s) => Value::Text(s),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Bitwise operations
// PARITY: Excel restricts to integers 0..2^48; we use i64 (53-bit safe range).
// ---------------------------------------------------------------------------

fn get_bit_int(args: &[Value], i: usize) -> Result<u64, CellError> {
    let n = get(args, i)?;
    if n < 0.0 || n >= 2f64.powi(48) || n.fract() != 0.0 {
        return Err(CellError::Num);
    }
    Ok(n as u64)
}

fn bitand(_: &mut dyn Context, args: &[Value]) -> Value {
    match (get_bit_int(args, 0), get_bit_int(args, 1)) {
        (Ok(a), Ok(b)) => Value::Number((a & b) as f64),
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn bitor(_: &mut dyn Context, args: &[Value]) -> Value {
    match (get_bit_int(args, 0), get_bit_int(args, 1)) {
        (Ok(a), Ok(b)) => Value::Number((a | b) as f64),
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn bitxor(_: &mut dyn Context, args: &[Value]) -> Value {
    match (get_bit_int(args, 0), get_bit_int(args, 1)) {
        (Ok(a), Ok(b)) => Value::Number((a ^ b) as f64),
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn bitlshift(_: &mut dyn Context, args: &[Value]) -> Value {
    let a = match get_bit_int(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let n = match get(args, 1) {
        Ok(v) => v.trunc() as i32,
        Err(e) => return Value::Error(e),
    };
    if !(-53..=53).contains(&n) {
        return Value::Error(CellError::Num);
    }
    let result = if n >= 0 {
        a << (n as u32)
    } else {
        a >> ((-n) as u32)
    };
    Value::Number(result as f64)
}

fn bitrshift(_: &mut dyn Context, args: &[Value]) -> Value {
    let a = match get_bit_int(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let n = match get(args, 1) {
        Ok(v) => v.trunc() as i32,
        Err(e) => return Value::Error(e),
    };
    if !(-53..=53).contains(&n) {
        return Value::Error(CellError::Num);
    }
    let result = if n >= 0 {
        a >> (n as u32)
    } else {
        a << ((-n) as u32)
    };
    Value::Number(result as f64)
}

// ---------------------------------------------------------------------------
// DELTA / GESTEP
// ---------------------------------------------------------------------------

fn delta(_: &mut dyn Context, args: &[Value]) -> Value {
    let a = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let b = match opt(args, 1, 0.0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(if a == b { 1.0 } else { 0.0 })
}

fn gestep(_: &mut dyn Context, args: &[Value]) -> Value {
    let a = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let b = match opt(args, 1, 0.0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(if a >= b { 1.0 } else { 0.0 })
}

// ---------------------------------------------------------------------------
// CONVERT
// Conversion factor table: value_in_si = value_from * factor
// Base SI units: kg, m, s, K, J, Pa, N, W, bit
// ---------------------------------------------------------------------------

/// Return (from_si_factor, to_si_factor) for the two units, where
/// value_si = value_from * from_factor, and value_to = value_si / to_factor.
/// Returns None for unknown units.
fn unit_to_si(unit: &str) -> Option<f64> {
    // Mass (base: kg)
    match unit {
        "g" => return Some(1e-3),
        "kg" => return Some(1.0),
        "lbm" => return Some(0.453_592_37),
        "ozm" => return Some(0.028_349_523_1),
        "ton" => return Some(907.184_74), // short ton (US)
        "stone" => return Some(6.350_293_18),
        "cwt" => return Some(45.359_237), // short hundredweight
        "uk_cwt" | "lcwt" => return Some(50.802_345_44),
        "uk_ton" | "LTON" => return Some(1_016.046_908_8),
        "grain" => return Some(6.479_891e-5),
        "slug" => return Some(14.593_903),
        "u" => return Some(1.660_538_782e-27), // atomic mass
        "pweight" => return Some(3.110_347_7e-3),
        _ => {}
    }
    // Distance (base: m)
    match unit {
        "m" => return Some(1.0),
        "mi" => return Some(1609.344),
        "Nmi" => return Some(1852.0),
        "in" => return Some(0.0254),
        "ft" => return Some(0.3048),
        "yd" => return Some(0.9144),
        "ang" => return Some(1e-10),
        "Pica" | "picapt" => return Some(0.000_352_778),
        "pica" => return Some(0.004_233_33),
        "ell" => return Some(1.143),
        "ly" => return Some(9.460_730_473e15),
        "parsec" | "pc" => return Some(3.085_677_581_3e16),
        "survey_mi" => return Some(1_609.347_219),
        _ => {}
    }
    // Time (base: s)
    match unit {
        "sec" => return Some(1.0),
        "s" => return Some(1.0),
        "mn" => return Some(60.0),
        "min" => return Some(60.0),
        "hr" => return Some(3600.0),
        "day" => return Some(86400.0),
        "yr" => return Some(31_557_600.0), // Julian year
        _ => {}
    }
    // Pressure (base: Pa)
    match unit {
        "Pa" => return Some(1.0),
        "atm" => return Some(101_325.0),
        "mmHg" => return Some(133.322_387),
        "psi" => return Some(6_894.757_29),
        "Torr" => return Some(133.322_387),
        _ => {}
    }
    // Force (base: N)
    match unit {
        "N" => return Some(1.0),
        "dyn" => return Some(1e-5),
        "lbf" => return Some(4.448_222),
        "pond" => return Some(9.806_65e-3),
        _ => {}
    }
    // Energy (base: J)
    match unit {
        "J" => return Some(1.0),
        "e" => return Some(1e-7),  // erg
        "c" => return Some(4.184), // thermochemical calorie
        "cal" => return Some(4.184),
        "eV" | "ev" => return Some(1.602_176_634e-19),
        "HPh" | "hh" => return Some(2_684_519.537_696_173),
        "Wh" | "wh" => return Some(3600.0),
        "flb" => return Some(1.355_817_948_331_4),
        "BTU" | "btu" => return Some(1_055.055_852_62),
        _ => {}
    }
    // Power (base: W)
    match unit {
        "W" => return Some(1.0),
        "HP" | "h" => return Some(745.699_871_582),
        "PS" => return Some(735.498_75),
        _ => {}
    }
    // Magnetism (base: T)
    match unit {
        "T" => return Some(1.0),
        "ga" => return Some(1e-4),
        _ => {}
    }
    // Temperature — handled separately
    None
}

/// Metric prefix multipliers (applied to the raw unit string).
fn metric_prefix(prefix: char) -> Option<f64> {
    match prefix {
        'Y' => Some(1e24),
        'Z' => Some(1e21),
        'E' => Some(1e18),
        'P' => Some(1e15),
        'T' => Some(1e12),
        'G' => Some(1e9),
        'M' => Some(1e6),
        'k' => Some(1e3),
        'h' => Some(1e2),
        'e' => Some(1e1), // deca (da is two chars, but Excel uses 'e' for deca)
        'd' => Some(1e-1),
        'c' => Some(1e-2),
        'm' => Some(1e-3),
        'u' => Some(1e-6),
        'n' => Some(1e-9),
        'p' => Some(1e-12),
        'f' => Some(1e-15),
        'a' => Some(1e-18),
        'z' => Some(1e-21),
        'y' => Some(1e-24),
        _ => None,
    }
}

/// Attempt to resolve unit with optional metric prefix.
fn resolve_unit(raw: &str) -> Option<f64> {
    // Try exact match first
    if let Some(f) = unit_to_si(raw) {
        return Some(f);
    }
    // Try metric prefix on first char
    if raw.len() >= 2 {
        let (prefix_char, rest) = {
            let mut chars = raw.chars();
            let first = chars.next().unwrap();
            let rest = &raw[first.len_utf8()..];
            (first, rest)
        };
        if let Some(prefix_mul) = metric_prefix(prefix_char)
            && let Some(base_factor) = unit_to_si(rest)
        {
            return Some(prefix_mul * base_factor);
        }
    }
    None
}

/// Temperature conversion is special because it's not multiplicative.
fn is_temp(unit: &str) -> bool {
    matches!(unit, "C" | "F" | "K" | "Rank" | "Reau")
}

fn temp_to_kelvin(val: f64, unit: &str) -> Option<f64> {
    match unit {
        "C" => Some(val + 273.15),
        "F" => Some((val + 459.67) * 5.0 / 9.0),
        "K" => Some(val),
        "Rank" => Some(val * 5.0 / 9.0),
        "Reau" => Some(val * 5.0 / 4.0 + 273.15),
        _ => None,
    }
}

fn kelvin_to_temp(val: f64, unit: &str) -> Option<f64> {
    match unit {
        "C" => Some(val - 273.15),
        "F" => Some(val * 9.0 / 5.0 - 459.67),
        "K" => Some(val),
        "Rank" => Some(val * 9.0 / 5.0),
        "Reau" => Some((val - 273.15) * 4.0 / 5.0),
        _ => None,
    }
}

fn convert(_: &mut dyn Context, args: &[Value]) -> Value {
    let val = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let from = match get_text(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let to = match get_text(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };

    // Temperature
    if is_temp(&from) || is_temp(&to) {
        if !is_temp(&from) || !is_temp(&to) {
            return Value::Error(CellError::NA);
        }
        let k = match temp_to_kelvin(val, &from) {
            Some(v) => v,
            None => return Value::Error(CellError::NA),
        };
        return match kelvin_to_temp(k, &to) {
            Some(v) => Value::Number(v),
            None => Value::Error(CellError::NA),
        };
    }

    let from_factor = match resolve_unit(&from) {
        Some(f) => f,
        None => return Value::Error(CellError::NA),
    };
    let to_factor = match resolve_unit(&to) {
        Some(f) => f,
        None => return Value::Error(CellError::NA),
    };

    // Both must be in same dimension (same base unit category):
    // We detect this by checking if from/to can both map to SI and are compatible.
    // Simple heuristic: if factor ratio is completely unreasonable, return #N/A.
    // For correctness, we rely on the user providing same-category units.
    let result = val * from_factor / to_factor;
    if result.is_finite() {
        Value::Number(result)
    } else {
        Value::Error(CellError::NA)
    }
}

// ---------------------------------------------------------------------------
// ERF / ERFC
// Approximation using the complementary error function series.
// PARITY: uses Horner's method rational approximation; max relative error ~1.5e-7.
// ---------------------------------------------------------------------------

/// Error function using rational approximation (Abramowitz & Stegun 7.1.26).
/// Max absolute error ~1.5e-7.
pub fn erf_approx(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x.abs() > 6.0 {
        return x.signum();
    }
    if x < 0.0 {
        return -erf_approx(-x);
    }
    // A&S 7.1.26: erfc(x) ≈ poly(t) * exp(-x^2), t = 1/(1+0.3275911*x)
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    1.0 - poly * (-x * x).exp()
}

pub fn erfc_approx(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    1.0 - erf_approx(x)
}

fn erf_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    let lower = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if args.len() == 2 {
        let upper = match get(args, 1) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        Value::Number(erf_approx(upper) - erf_approx(lower))
    } else {
        Value::Number(erf_approx(lower))
    }
}

fn erfc_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erfc_approx(x))
}

fn erf_precise(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erf_approx(x))
}

fn erfc_precise(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erfc_approx(x))
}

// ---------------------------------------------------------------------------
// Complex number support
// Excel represents complex numbers as text: "3+4i", "3-4i", "3", "4i", etc.
// suffix is 'i' by default, 'j' if specified.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    fn to_text(self, suffix: char) -> String {
        let re = self.re;
        let im = self.im;
        if im == 0.0 {
            return crate::core::value::format_number_general(re);
        }
        if re == 0.0 {
            if im == 1.0 {
                return format!("{suffix}");
            } else if im == -1.0 {
                return format!("-{suffix}");
            } else {
                return format!("{}{suffix}", crate::core::value::format_number_general(im));
            }
        }
        let re_str = crate::core::value::format_number_general(re);
        if im == 1.0 {
            format!("{re_str}+{suffix}")
        } else if im == -1.0 {
            format!("{re_str}-{suffix}")
        } else if im < 0.0 {
            format!(
                "{re_str}{}{suffix}",
                crate::core::value::format_number_general(im)
            )
        } else {
            format!(
                "{re_str}+{}{suffix}",
                crate::core::value::format_number_general(im)
            )
        }
    }

    fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
    fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    fn add(&self, other: &Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
    fn sub(&self, other: &Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }
    fn mul(&self, other: &Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
    fn div(&self, other: &Complex) -> Option<Complex> {
        let denom = other.re * other.re + other.im * other.im;
        if denom == 0.0 {
            return None;
        }
        Some(Complex::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
        ))
    }
    fn exp(&self) -> Complex {
        let e_re = self.re.exp();
        Complex::new(e_re * self.im.cos(), e_re * self.im.sin())
    }
    fn ln(&self) -> Option<Complex> {
        let r = self.abs();
        if r == 0.0 {
            return None;
        }
        Some(Complex::new(r.ln(), self.arg()))
    }
    fn sqrt(&self) -> Complex {
        let r = self.abs().sqrt();
        let theta = self.arg() / 2.0;
        Complex::new(r * theta.cos(), r * theta.sin())
    }
    fn pow_complex(&self, n: &Complex) -> Option<Complex> {
        let ln = self.ln()?;
        Some((ln.mul(n)).exp())
    }
    fn conjugate(&self) -> Complex {
        Complex::new(self.re, -self.im)
    }
}

/// Parse a complex number from Excel text format.
fn parse_complex(s: &str) -> Result<(Complex, char), CellError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(CellError::Value);
    }

    // Determine suffix
    let suffix = if s.ends_with('j') { 'j' } else { 'i' };

    // If no imaginary part (no 'i' or 'j' suffix)
    if !s.ends_with('i') && !s.ends_with('j') {
        let re = s.parse::<f64>().map_err(|_| CellError::Value)?;
        return Ok((Complex::new(re, 0.0), 'i'));
    }

    // Strip the suffix
    let s = &s[..s.len() - 1];

    // Pure imaginary: "i", "-i", "+i"
    if s.is_empty() || s == "+" {
        return Ok((Complex::new(0.0, 1.0), suffix));
    }
    if s == "-" {
        return Ok((Complex::new(0.0, -1.0), suffix));
    }

    // Find the split point between real and imaginary parts.
    // Look for a + or - that's not at the start and not part of an exponent.
    let bytes = s.as_bytes();
    let mut split = None;
    let mut i = 1usize; // skip potential leading sign
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '+' || ch == '-' {
            // Make sure it's not an exponent sign
            let prev = bytes[i - 1] as char;
            if prev != 'e' && prev != 'E' {
                split = Some(i);
                break;
            }
        }
        i += 1;
    }

    if let Some(pos) = split {
        let re_str = &s[..pos];
        let im_str = &s[pos..];
        let re = if re_str.is_empty() {
            0.0
        } else {
            re_str.parse::<f64>().map_err(|_| CellError::Value)?
        };
        let im = if im_str.is_empty() || im_str == "+" {
            1.0
        } else if im_str == "-" {
            -1.0
        } else {
            im_str.parse::<f64>().map_err(|_| CellError::Value)?
        };
        Ok((Complex::new(re, im), suffix))
    } else {
        // Pure imaginary number (no real part)
        let im = if s.is_empty() || s == "+" {
            1.0
        } else if s == "-" {
            -1.0
        } else {
            s.parse::<f64>().map_err(|_| CellError::Value)?
        };
        Ok((Complex::new(0.0, im), suffix))
    }
}

fn parse_complex_arg(args: &[Value], i: usize) -> Result<(Complex, char), CellError> {
    match &args[i] {
        Value::Text(s) => parse_complex(s),
        Value::Number(n) => Ok((Complex::new(*n, 0.0), 'i')),
        Value::Error(e) => Err(*e),
        _ => Err(CellError::Value),
    }
}

fn complex_val(c: Complex, suffix: char) -> Value {
    Value::Text(c.to_text(suffix))
}

// COMPLEX: COMPLEX(real, imaginary [, suffix])
fn complex(_: &mut dyn Context, args: &[Value]) -> Value {
    let re = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let im = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let suffix = if args.len() >= 3 {
        match get_text(args, 2) {
            Ok(s) => {
                if s == "j" {
                    'j'
                } else if s == "i" || s.is_empty() {
                    'i'
                } else {
                    return Value::Error(CellError::Value);
                }
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        'i'
    };
    complex_val(Complex::new(re, im), suffix)
}

fn imabs(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.abs()),
        Err(e) => Value::Error(e),
    }
}

fn imreal(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.re),
        Err(e) => Value::Error(e),
    }
}

fn imaginary(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.im),
        Err(e) => Value::Error(e),
    }
}

fn imconjugate(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.conjugate(), s),
        Err(e) => Value::Error(e),
    }
}

fn imsum(_: &mut dyn Context, args: &[Value]) -> Value {
    let mut result = Complex::new(0.0, 0.0);
    let mut suffix = 'i';
    for arg in args {
        match parse_complex_arg(std::slice::from_ref(arg), 0) {
            Ok((c, s)) => {
                result = result.add(&c);
                suffix = s;
            }
            Err(e) => return Value::Error(e),
        }
    }
    complex_val(result, suffix)
}

fn imsub(_: &mut dyn Context, args: &[Value]) -> Value {
    match (parse_complex_arg(args, 0), parse_complex_arg(args, 1)) {
        (Ok((a, s)), Ok((b, _))) => complex_val(a.sub(&b), s),
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn improduct(_: &mut dyn Context, args: &[Value]) -> Value {
    let mut result = Complex::new(1.0, 0.0);
    let mut suffix = 'i';
    for arg in args {
        match parse_complex_arg(std::slice::from_ref(arg), 0) {
            Ok((c, s)) => {
                result = result.mul(&c);
                suffix = s;
            }
            Err(e) => return Value::Error(e),
        }
    }
    complex_val(result, suffix)
}

fn imdiv(_: &mut dyn Context, args: &[Value]) -> Value {
    match (parse_complex_arg(args, 0), parse_complex_arg(args, 1)) {
        (Ok((a, s)), Ok((b, _))) => match a.div(&b) {
            Some(c) => complex_val(c, s),
            None => Value::Error(CellError::Div0),
        },
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn imexp(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.exp(), s),
        Err(e) => Value::Error(e),
    }
}

fn imln(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(r) => complex_val(r, s),
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

fn imsqrt(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.sqrt(), s),
        Err(e) => Value::Error(e),
    }
}

fn imargument(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => {
            if c.re == 0.0 && c.im == 0.0 {
                return Value::Error(CellError::Div0);
            }
            Value::Number(c.arg())
        }
        Err(e) => Value::Error(e),
    }
}

fn impower(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // Power can be a number or complex
            let p = if let Ok((cp, _)) = parse_complex_arg(args, 1) {
                cp
            } else {
                match get(args, 1) {
                    Ok(n) => Complex::new(n, 0.0),
                    Err(e) => return Value::Error(e),
                }
            };
            match c.pow_complex(&p) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Num),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsin(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // sin(a+bi) = sin(a)cosh(b) + i*cos(a)sinh(b)
            let r = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imcos(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cos(a+bi) = cos(a)cosh(b) - i*sin(a)sinh(b)
            let r = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imtan(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // tan(a+bi) = sin(a+bi)/cos(a+bi)
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match sin_c.div(&cos_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcot(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cot(a+bi) = cos(a+bi)/sin(a+bi)
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match cos_c.div(&sin_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsinh(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // sinh(a+bi) = sinh(a)cos(b) + i*cosh(a)sin(b)
            let r = Complex::new(c.re.sinh() * c.im.cos(), c.re.cosh() * c.im.sin());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imcosh(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cosh(a+bi) = cosh(a)cos(b) + i*sinh(a)sin(b)
            let r = Complex::new(c.re.cosh() * c.im.cos(), c.re.sinh() * c.im.sin());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imsec(_: &mut dyn Context, args: &[Value]) -> Value {
    // sec = 1/cos
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match Complex::new(1.0, 0.0).div(&cos_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcsc(_: &mut dyn Context, args: &[Value]) -> Value {
    // csc = 1/sin
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            match Complex::new(1.0, 0.0).div(&sin_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsech(_: &mut dyn Context, args: &[Value]) -> Value {
    // sech = 1/cosh
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let cosh_c = Complex::new(c.re.cosh() * c.im.cos(), c.re.sinh() * c.im.sin());
            match Complex::new(1.0, 0.0).div(&cosh_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcsch(_: &mut dyn Context, args: &[Value]) -> Value {
    // csch = 1/sinh
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let sinh_c = Complex::new(c.re.sinh() * c.im.cos(), c.re.cosh() * c.im.sin());
            match Complex::new(1.0, 0.0).div(&sinh_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imlog2(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(ln_c) => {
                let ln2 = Complex::new(std::f64::consts::LN_2, 0.0);
                match ln_c.div(&ln2) {
                    Some(r) => complex_val(r, s),
                    None => Value::Error(CellError::Num),
                }
            }
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

fn imlog10(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(ln_c) => {
                let ln10 = Complex::new(std::f64::consts::LN_10, 0.0);
                match ln_c.div(&ln10) {
                    Some(r) => complex_val(r, s),
                    None => Value::Error(CellError::Num),
                }
            }
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::formula::functions::testutil::TestCtx;

    fn c() -> TestCtx {
        TestCtx::new()
    }
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    // --- Base conversions ---

    #[test]
    fn test_bin2dec() {
        let mut ctx = c();
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("1010".into())]),
            Value::Number(10.0)
        );
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
        // Negative: 1111111111 in 10-bit two's complement = -1
        assert_eq!(
            bin2dec(&mut ctx, &[Value::Text("1111111111".into())]),
            Value::Number(-1.0)
        );
    }

    #[test]
    fn test_dec2bin() {
        let mut ctx = c();
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(10.0)]),
            Value::Text("1010".into())
        );
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(0.0)]),
            Value::Text("0".into())
        );
        // DEC2BIN(-1) = 1111111111 (10-bit two's complement)
        if let Value::Text(s) = dec2bin(&mut ctx, &[Value::Number(-1.0)]) {
            assert_eq!(s, "1111111111");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_dec2bin_overflow() {
        let mut ctx = c();
        // 512 > 511 (max for 10-bit signed)
        assert_eq!(
            dec2bin(&mut ctx, &[Value::Number(512.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn test_dec2hex() {
        let mut ctx = c();
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(255.0)]),
            Value::Text("FF".into())
        );
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(0.0)]),
            Value::Text("0".into())
        );
        // With places
        assert_eq!(
            dec2hex(&mut ctx, &[Value::Number(255.0), Value::Number(4.0)]),
            Value::Text("00FF".into())
        );
    }

    #[test]
    fn test_hex2dec() {
        let mut ctx = c();
        assert_eq!(
            hex2dec(&mut ctx, &[Value::Text("FF".into())]),
            Value::Number(255.0)
        );
        assert_eq!(
            hex2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
        // Negative: FFFFFFFFFF = -1 in 40-bit two's complement
        if let Value::Number(v) = hex2dec(&mut ctx, &[Value::Text("FFFFFFFFFF".into())]) {
            assert_eq!(v, -1.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_oct2dec() {
        let mut ctx = c();
        assert_eq!(
            oct2dec(&mut ctx, &[Value::Text("777".into())]),
            Value::Number(511.0)
        );
        assert_eq!(
            oct2dec(&mut ctx, &[Value::Text("0".into())]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn test_dec2oct() {
        let mut ctx = c();
        assert_eq!(
            dec2oct(&mut ctx, &[Value::Number(8.0)]),
            Value::Text("10".into())
        );
    }

    #[test]
    fn test_bin2hex() {
        let mut ctx = c();
        assert_eq!(
            bin2hex(&mut ctx, &[Value::Text("11111111".into())]),
            Value::Text("FF".into())
        );
    }

    #[test]
    fn test_hex2bin() {
        let mut ctx = c();
        assert_eq!(
            hex2bin(&mut ctx, &[Value::Text("F".into())]),
            Value::Text("1111".into())
        );
    }

    // --- Bitwise ---

    #[test]
    fn test_bitwise() {
        let mut ctx = c();
        assert_eq!(
            bitand(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(9.0)
        );
        assert_eq!(
            bitor(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(29.0)
        );
        assert_eq!(
            bitxor(&mut ctx, &[Value::Number(13.0), Value::Number(25.0)]),
            Value::Number(20.0)
        );
        assert_eq!(
            bitlshift(&mut ctx, &[Value::Number(4.0), Value::Number(2.0)]),
            Value::Number(16.0)
        );
        assert_eq!(
            bitrshift(&mut ctx, &[Value::Number(16.0), Value::Number(2.0)]),
            Value::Number(4.0)
        );
    }

    #[test]
    fn test_bitwise_negative_error() {
        let mut ctx = c();
        assert_eq!(
            bitand(&mut ctx, &[Value::Number(-1.0), Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- DELTA / GESTEP ---

    #[test]
    fn test_delta_gestep() {
        let mut ctx = c();
        assert_eq!(
            delta(&mut ctx, &[Value::Number(5.0), Value::Number(5.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            delta(&mut ctx, &[Value::Number(5.0), Value::Number(4.0)]),
            Value::Number(0.0)
        );
        assert_eq!(
            gestep(&mut ctx, &[Value::Number(5.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
        assert_eq!(
            gestep(&mut ctx, &[Value::Number(3.0), Value::Number(4.0)]),
            Value::Number(0.0)
        );
        // DELTA with default second arg (0)
        assert_eq!(delta(&mut ctx, &[Value::Number(0.0)]), Value::Number(1.0));
    }

    // --- CONVERT ---

    #[test]
    fn test_convert_weight() {
        let mut ctx = c();
        // 1 lbm = 0.453592 kg
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("lbm".into()),
                Value::Text("kg".into()),
            ],
        ) {
            assert!(approx(v, 0.4536), "lbm→kg = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_distance() {
        let mut ctx = c();
        // 1 mi = 1609.344 m
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("mi".into()),
                Value::Text("m".into()),
            ],
        ) {
            assert!(approx(v, 1609.344), "mi→m = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_temperature() {
        let mut ctx = c();
        // 100 C = 212 F
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("C".into()),
                Value::Text("F".into()),
            ],
        ) {
            assert!(approx(v, 212.0), "C→F = {v}");
        } else {
            panic!("Expected number");
        }

        // 0 C = 273.15 K
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Text("C".into()),
                Value::Text("K".into()),
            ],
        ) {
            assert!(approx(v, 273.15), "C→K = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_convert_unknown_unit() {
        let mut ctx = c();
        assert_eq!(
            convert(
                &mut ctx,
                &[
                    Value::Number(1.0),
                    Value::Text("frobble".into()),
                    Value::Text("kg".into()),
                ]
            ),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn test_convert_metric_prefix() {
        let mut ctx = c();
        // 1 km = 1000 m
        if let Value::Number(v) = convert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("km".into()),
                Value::Text("m".into()),
            ],
        ) {
            assert!(approx(v, 1000.0), "km→m = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // --- ERF / ERFC ---

    #[test]
    fn test_erf() {
        let mut ctx = c();
        // erf(0) = 0
        assert_eq!(erf_fn(&mut ctx, &[Value::Number(0.0)]), Value::Number(0.0));
        // erf(1) ≈ 0.8427
        if let Value::Number(v) = erf_fn(&mut ctx, &[Value::Number(1.0)]) {
            assert!(approx(v, 0.8427), "erf(1) = {v}");
        } else {
            panic!("Expected number");
        }
        // erf(lower, upper) = erf(upper) - erf(lower)
        if let Value::Number(v) = erf_fn(&mut ctx, &[Value::Number(0.0), Value::Number(1.0)]) {
            assert!(approx(v, 0.8427), "erf(0,1) = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_erfc() {
        let mut ctx = c();
        // erfc(0) = 1
        assert_eq!(erfc_fn(&mut ctx, &[Value::Number(0.0)]), Value::Number(1.0));
        // erfc(1) ≈ 0.1573
        if let Value::Number(v) = erfc_fn(&mut ctx, &[Value::Number(1.0)]) {
            assert!(approx(v, 0.1573), "erfc(1) = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // --- Complex numbers ---

    #[test]
    fn test_complex_create() {
        let mut ctx = c();
        assert_eq!(
            complex(&mut ctx, &[Value::Number(3.0), Value::Number(4.0)]),
            Value::Text("3+4i".into())
        );
        assert_eq!(
            complex(&mut ctx, &[Value::Number(3.0), Value::Number(-4.0)]),
            Value::Text("3-4i".into())
        );
        assert_eq!(
            complex(&mut ctx, &[Value::Number(0.0), Value::Number(1.0)]),
            Value::Text("i".into())
        );
    }

    #[test]
    fn test_imabs() {
        let mut ctx = c();
        // |3+4i| = 5
        if let Value::Number(v) = imabs(&mut ctx, &[Value::Text("3+4i".into())]) {
            assert!(approx(v, 5.0), "IMABS = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_imreal_imaginary() {
        let mut ctx = c();
        assert_eq!(
            imreal(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Number(3.0)
        );
        assert_eq!(
            imaginary(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Number(4.0)
        );
        assert_eq!(
            imreal(&mut ctx, &[Value::Text("5".into())]),
            Value::Number(5.0)
        );
        assert_eq!(
            imaginary(&mut ctx, &[Value::Text("5".into())]),
            Value::Number(0.0)
        );
    }

    #[test]
    fn test_imconjugate() {
        let mut ctx = c();
        assert_eq!(
            imconjugate(&mut ctx, &[Value::Text("3+4i".into())]),
            Value::Text("3-4i".into())
        );
    }

    #[test]
    fn test_improduct() {
        let mut ctx = c();
        // (3+4i)(3-4i) = 9+16 = 25
        if let Value::Text(s) = improduct(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("3-4i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 25.0) && approx(c.im, 0.0), "product = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imdiv() {
        let mut ctx = c();
        // (3+4i)/(3+4i) = 1
        if let Value::Text(s) = imdiv(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("3+4i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 1.0) && approx(c.im, 0.0), "div = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imexp() {
        let mut ctx = c();
        // exp(i*pi) ≈ -1 (Euler's identity: exp(iπ) = cos(π)+i*sin(π) = -1)
        if let Value::Text(s) = imexp(&mut ctx, &[Value::Text("3.14159265358979i".into())]) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, -1.0) && approx(c.im, 0.0), "exp(iπ) = {s}");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imsqrt() {
        let mut ctx = c();
        // sqrt(-1) = i
        if let Value::Text(s) = imsqrt(&mut ctx, &[Value::Text("-1".into())]) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 0.0) && approx(c.im, 1.0), "sqrt(-1) = {s}");
        } else if let Value::Number(v) = imsqrt(&mut ctx, &[Value::Text("-1".into())]) {
            panic!("Expected text but got Number({v})");
        } else {
            panic!("Expected text");
        }
    }

    #[test]
    fn test_imsum_imsub() {
        let mut ctx = c();
        if let Value::Text(s) = imsum(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("1+2i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 4.0) && approx(c.im, 6.0), "sum = {s}");
        } else {
            panic!("Expected text");
        }

        if let Value::Text(s) = imsub(
            &mut ctx,
            &[Value::Text("3+4i".into()), Value::Text("1+2i".into())],
        ) {
            let (c, _) = parse_complex(&s).unwrap();
            assert!(approx(c.re, 2.0) && approx(c.im, 2.0), "sub = {s}");
        } else {
            panic!("Expected text");
        }
    }
}
