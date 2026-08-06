/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
        Value::Number(n) => Ok(easyexcel_model::value::format_number_general(*n)),
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
        Ok(format!("{s:0>digits$}"))
    }
}

fn format_radix(mut n: u64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let digits = b"0123456789ABCDEF";
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[(n % u64::from(radix)) as usize]);
        n /= u64::from(radix);
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

/// Return (`from_si_factor`, `to_si_factor`) for the two units, where
/// `value_si` = `value_from` * `from_factor`, and `value_to` = `value_si` / `to_factor`.
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

