/// 对应 Java：`NumberUtils.parseInteger` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_integer(value: &str) -> Result<i32, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_to_java_i32(&value))
}

/// 对应 Java：`NumberUtils.parseFloat` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_float(value: &str) -> Result<f32, NumberFormatError> {
    parse_decimal(value, None).and_then(|value| {
        value
            .to_string()
            .parse()
            .map_err(|_| NumberFormatError::new(format!("parseFloat failed for {value}")))
    })
}

/// 对应 Java：`NumberUtils.parseBigDecimal` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_big_decimal(value: &str) -> Result<BigDecimal, NumberFormatError> {
    parse_decimal(value, None)
}

/// 对应 Java：`NumberUtils.parseByte` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_byte(value: &str) -> Result<i8, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_to_java_i8(&value))
}

/// 对应 Java：`NumberUtils.parseDouble` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_double(value: &str) -> Result<f64, NumberFormatError> {
    parse_decimal(value, None).and_then(|value| {
        value
            .to_string()
            .parse()
            .map_err(|_| NumberFormatError::new(format!("parseDouble failed for {value}")))
    })
}

/// 对应 Java：NumberUtils.parseInteger。 Mirrors Apache Commons `NumberUtils.createBigInteger`.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制整数时返回 [`NumberFormatError::new`]。
pub fn parse_big_int(value: &str) -> Result<BigInt, NumberFormatError> {
    BigInt::from_str(value)
        .map_err(|_| NumberFormatError::new(format!("parseBigInteger failed for {value:?}")))
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java `BigDecimal#toBigInteger` 语义截断小数部分。
#[must_use]
pub fn decimal_to_big_int(value: &BigDecimal) -> BigInt {
    value.with_scale(0).into_bigint_and_exponent().0
}

fn java_signed_low_bytes<const N: usize>(value: &BigInt) -> [u8; N] {
    let extension = if value.sign() == num_bigint::Sign::Minus {
        u8::MAX
    } else {
        0
    };
    let mut output = [extension; N];
    let source = value.to_signed_bytes_le();
    let count = source.len().min(N);
    output[..count].copy_from_slice(&source[..count]);
    output
}

// 对应 Java：按 2 的补码取低 N 字节解释为有符号整数（截断语义与 Java 一致）
/// 对应 Java：NumberUtils.parseInteger。 按 Java 二补码低位截断语义转换为 `byte`。
#[must_use]
pub fn decimal_to_java_i8(value: &BigDecimal) -> i8 {
    i8::from_le_bytes(java_signed_low_bytes::<1>(&decimal_to_big_int(value)))
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java 二补码低位截断语义转换为 `short`。
#[must_use]
pub fn decimal_to_java_i16(value: &BigDecimal) -> i16 {
    i16::from_le_bytes(java_signed_low_bytes::<2>(&decimal_to_big_int(value)))
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java 二补码低位截断语义转换为 `int`。
#[must_use]
pub fn decimal_to_java_i32(value: &BigDecimal) -> i32 {
    i32::from_le_bytes(java_signed_low_bytes::<4>(&decimal_to_big_int(value)))
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java 二补码低位截断语义转换为 `long`。
#[must_use]
pub fn decimal_to_java_i64(value: &BigDecimal) -> i64 {
    i64::from_le_bytes(java_signed_low_bytes::<8>(&decimal_to_big_int(value)))
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java `Float#toString` 规则渲染有限值、零、无穷与 NaN。
#[must_use]
pub fn java_f32_string(value: f32) -> String {
    java_float_string(
        f64::from(value),
        value.to_string(),
        &format!("{value:e}"),
        value.is_sign_negative(),
    )
}

/// 对应 Java：NumberUtils.parseInteger。 按 Java `Double#toString` 规则渲染有限值、零、无穷与 NaN。
#[must_use]
pub fn java_f64_string(value: f64) -> String {
    java_float_string(
        value,
        value.to_string(),
        &format!("{value:e}"),
        value.is_sign_negative(),
    )
}

fn java_float_string(value: f64, plain: String, scientific: &str, negative: bool) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    if value == 0.0 {
        return if negative { "-0.0" } else { "0.0" }.to_owned();
    }
    let absolute = value.abs();
    if !(1.0e-3..1.0e7).contains(&absolute) {
        let (mantissa, exponent) = scientific
            .split_once(['e', 'E'])
            .expect("Rust scientific formatting contains an exponent");
        let mantissa = if mantissa.contains('.') {
            mantissa.to_owned()
        } else {
            format!("{mantissa}.0")
        };
        return format!("{mantissa}E{}", exponent.trim_start_matches('+'));
    }
    if plain.contains('.') {
        plain
    } else {
        format!("{plain}.0")
    }
}

#[cfg(test)]
#[path = "../number_utils_tests/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../number_utils_tests/tests_extra.rs"]
mod tests_extra;
