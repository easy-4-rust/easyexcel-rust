//! Java-compatible number conversion helpers.
//!
//! Mirrors `com.alibaba.excel.util.NumberUtils` and the `DecimalFormat`
//! subset used by `EasyExcel`'s built-in numeric string converters.

use std::str::FromStr;

use bigdecimal::{BigDecimal, RoundingMode};
use num_bigint::BigInt;

/// 数字格式解析和渲染错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct NumberFormatError {
    message: String,
}

impl NumberFormatError {
    /// 创建带诊断信息的格式错误。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 返回稳定的人类可读诊断信息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Java `java.math.RoundingMode` 对应的中立舍入模式。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NumberRoundingMode {
    /// 远离零。
    Up,
    /// 趋近零。
    Down,
    /// 趋向正无穷。
    Ceiling,
    /// 趋向负无穷。
    Floor,
    /// 最近值，中点远离零。
    #[default]
    HalfUp,
    /// 最近值，中点趋近零。
    HalfDown,
    /// 最近值，中点取偶数邻居。
    HalfEven,
    /// 需要舍入时返回错误。
    Unnecessary,
}

impl NumberRoundingMode {
    /// 返回 `bigdecimal` 舍入模式；`Unnecessary` 由调用方显式校验。
    #[must_use]
    pub const fn bigdecimal(self) -> Option<RoundingMode> {
        match self {
            Self::Up => Some(RoundingMode::Up),
            Self::Down => Some(RoundingMode::Down),
            Self::Ceiling => Some(RoundingMode::Ceiling),
            Self::Floor => Some(RoundingMode::Floor),
            Self::HalfUp => Some(RoundingMode::HalfUp),
            Self::HalfDown => Some(RoundingMode::HalfDown),
            Self::HalfEven => Some(RoundingMode::HalfEven),
            Self::Unnecessary => None,
        }
    }
}

impl From<RoundingMode> for NumberRoundingMode {
    fn from(value: RoundingMode) -> Self {
        match value {
            RoundingMode::Up => Self::Up,
            RoundingMode::Down => Self::Down,
            RoundingMode::Ceiling => Self::Ceiling,
            RoundingMode::Floor => Self::Floor,
            RoundingMode::HalfUp => Self::HalfUp,
            RoundingMode::HalfDown => Self::HalfDown,
            RoundingMode::HalfEven => Self::HalfEven,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonFiniteNumber {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
}

#[derive(Debug, Clone)]
struct DecimalSubpattern {
    prefix: String,
    suffix: String,
    min_integer_digits: usize,
    min_fraction_digits: usize,
    max_fraction_digits: usize,
    grouping_size: Option<usize>,
    exponent_digits: Option<usize>,
    exponent_integer_digits: usize,
    multiplier: i32,
}

#[derive(Debug, Clone)]
struct DecimalPattern {
    positive: DecimalSubpattern,
    negative: DecimalSubpattern,
}

#[derive(Debug, Clone, Copy)]
struct PatternToken {
    value: char,
    literal: bool,
}

/// Formats a finite number like Java `NumberUtils.format`.
pub fn format_decimal(
    value: &BigDecimal,
    negative: bool,
    pattern: Option<&str>,
    rounding_mode: NumberRoundingMode,
) -> Result<String, NumberFormatError> {
    let Some(pattern) = pattern.filter(|pattern| !pattern.is_empty()) else {
        return Ok(value.to_plain_string());
    };
    let pattern = DecimalPattern::parse(pattern)?;
    pattern.format(value, negative, rounding_mode)
}

/// Formats Java `NaN` / infinity through `DecimalFormat` affixes.
pub fn format_non_finite(
    value: NonFiniteNumber,
    pattern: Option<&str>,
) -> Result<String, NumberFormatError> {
    if value == NonFiniteNumber::Nan {
        return Ok("NaN".to_owned());
    }
    let Some(pattern) = pattern.filter(|pattern| !pattern.is_empty()) else {
        // NaN 已在上面的 if 中提前返回，此处只可能为 ±Infinity；
        // 用 if/else 替代含 unreachable! 的三臂 match（Nan 臂数学不可达，删除以消除死分支）。
        return Ok(if value == NonFiniteNumber::PositiveInfinity {
            "Infinity"
        } else {
            "-Infinity"
        }
        .to_owned());
    };
    let pattern = DecimalPattern::parse(pattern)?;
    let part = if value == NonFiniteNumber::NegativeInfinity {
        &pattern.negative
    } else {
        &pattern.positive
    };
    Ok(format!("{}∞{}", part.prefix, part.suffix))
}

/// Parses a string like Java `NumberUtils.parseBigDecimal`.
pub fn parse_decimal(
    value: &str,
    pattern: Option<&str>,
) -> Result<BigDecimal, NumberFormatError> {
    let Some(pattern) = pattern.filter(|pattern| !pattern.is_empty()) else {
        // Java `new BigDecimal(string)` does not trim leading/trailing spaces
        // and requires the complete input to be numeric.
        return BigDecimal::from_str(value)
            .map_err(|_| NumberFormatError::new(format!("parseBigDecimal failed for {value:?}")));
    };
    DecimalPattern::parse(pattern)?.parse_number(value)
}

impl DecimalPattern {
    fn parse(pattern: &str) -> Result<Self, NumberFormatError> {
        let subpatterns = tokenize_pattern(pattern)?;
        let positive = DecimalSubpattern::parse(
            subpatterns
                .first()
                .ok_or_else(|| invalid_pattern(pattern, "missing positive subpattern"))?,
            pattern,
        )?;
        let negative = if let Some(tokens) = subpatterns.get(1) {
            DecimalSubpattern::parse(tokens, pattern)?
        } else {
            DecimalSubpattern {
                prefix: format!("-{}", positive.prefix),
                ..positive.clone()
            }
        };
        if subpatterns.len() > 2 {
            return Err(invalid_pattern(pattern, "more than two subpatterns"));
        }
        Ok(Self { positive, negative })
    }

    fn format(
        &self,
        value: &BigDecimal,
        negative: bool,
        rounding_mode: NumberRoundingMode,
    ) -> Result<String, NumberFormatError> {
        let part = if negative {
            &self.negative
        } else {
            &self.positive
        };
        let absolute = if value < &BigDecimal::from(0) {
            -value.clone()
        } else {
            value.clone()
        };
        let rounding_mode = if negative {
            match rounding_mode {
                NumberRoundingMode::Ceiling => NumberRoundingMode::Down,
                NumberRoundingMode::Floor => NumberRoundingMode::Up,
                other => other,
            }
        } else {
            rounding_mode
        };
        part.format_absolute(&absolute, rounding_mode)
    }

    fn parse_number(&self, input: &str) -> Result<BigDecimal, NumberFormatError> {
        // Java DecimalFormat tests the explicit/default negative prefix before
        // the positive one when the input starts with a minus sign.
        if let Some(result) = self.negative.parse_number(input, true)? {
            return Ok(result);
        }
        if let Some(result) = self.positive.parse_number(input, false)? {
            return Ok(result);
        }
        Err(NumberFormatError::new(format!(
            "DecimalFormat could not parse {input:?}"
        )))
    }
}

impl DecimalSubpattern {
    fn parse(tokens: &[PatternToken], source: &str) -> Result<Self, NumberFormatError> {
        let first = tokens
            .iter()
            .position(|token| is_numeric_pattern_token(*token))
            .ok_or_else(|| invalid_pattern(source, "missing digit pattern"))?;
        let last = tokens
            .iter()
            .rposition(|token| is_numeric_pattern_token(*token))
            .expect("first numeric token exists");
        let prefix_tokens = &tokens[..first];
        let number_tokens = &tokens[first..=last];
        let suffix_tokens = &tokens[last + 1..];
        let prefix = render_affix(prefix_tokens);
        let suffix = render_affix(suffix_tokens);
        let multiplier = affix_multiplier(prefix_tokens, suffix_tokens)?;

        let exponent_index = number_tokens
            .iter()
            .position(|token| !token.literal && token.value == 'E');
        let (mantissa, exponent) = exponent_index.map_or((number_tokens, None), |index| {
            (&number_tokens[..index], Some(&number_tokens[index + 1..]))
        });
        let exponent_digits = exponent
            .map(|tokens| {
                if tokens.is_empty()
                    || tokens
                        .iter()
                        .any(|token| token.literal || token.value != '0')
                {
                    Err(invalid_pattern(source, "invalid exponent"))
                } else {
                    Ok(tokens.len())
                }
            })
            .transpose()?;

        let decimal_index = mantissa
            .iter()
            .position(|token| !token.literal && token.value == '.');
        let (integer, fraction) = decimal_index.map_or((mantissa, &[][..]), |index| {
            (&mantissa[..index], &mantissa[index + 1..])
        });
        if integer
            .iter()
            .any(|token| token.literal || !matches!(token.value, '#' | '0' | ','))
            || fraction
                .iter()
                .any(|token| token.literal || !matches!(token.value, '#' | '0'))
        {
            return Err(invalid_pattern(source, "invalid mantissa"));
        }
        let integer_digits = integer
            .iter()
            .filter(|token| matches!(token.value, '#' | '0'))
            .count();
        if integer_digits == 0 && fraction.is_empty() {
            return Err(invalid_pattern(source, "missing digit"));
        }
        let min_integer_digits = integer.iter().filter(|token| token.value == '0').count();
        let min_fraction_digits = fraction.iter().filter(|token| token.value == '0').count();
        let max_fraction_digits = fraction.len();
        let grouping_size = integer
            .iter()
            .rposition(|token| token.value == ',')
            .map(|index| {
                integer[index + 1..]
                    .iter()
                    .filter(|token| matches!(token.value, '#' | '0'))
                    .count()
            })
            .filter(|size| *size > 0);
        Ok(Self {
            prefix,
            suffix,
            min_integer_digits,
            min_fraction_digits,
            max_fraction_digits,
            grouping_size,
            exponent_digits,
            exponent_integer_digits: integer_digits.max(1),
            multiplier,
        })
    }

    fn format_absolute(
        &self,
        value: &BigDecimal,
        rounding_mode: NumberRoundingMode,
    ) -> Result<String, NumberFormatError> {
        let scaled = value * BigDecimal::from(self.multiplier);
        let number = if self.exponent_digits.is_some() {
            self.format_scientific(&scaled, rounding_mode)?
        } else {
            self.format_plain(&scaled, rounding_mode)?
        };
        Ok(format!("{}{}{}", self.prefix, number, self.suffix))
    }

    fn format_plain(
        &self,
        value: &BigDecimal,
        rounding_mode: NumberRoundingMode,
    ) -> Result<String, NumberFormatError> {
        let rounded = round_decimal(value, self.max_fraction_digits, rounding_mode)?;
        let mut text = rounded.to_plain_string();
        if let Some((integer, fraction)) = text.split_once('.') {
            // rounded 的 scale == max_fraction_digits >= min_fraction_digits，
            // 故小数位长度不小于最小值，只需收缩末尾零（原 push('0') 补位循环恒不可达，已删除）。
            let mut fraction = fraction.to_owned();
            while fraction.len() > self.min_fraction_digits && fraction.ends_with('0') {
                fraction.pop();
            }
            text = if fraction.is_empty() {
                integer.to_owned()
            } else {
                format!("{integer}.{fraction}")
            };
        }
        let (integer, fraction) = text
            .split_once('.')
            .map_or((text.as_str(), None), |parts| (parts.0, Some(parts.1)));
        let mut integer = integer.to_owned();
        while integer.len() < self.min_integer_digits.max(1) {
            integer.insert(0, '0');
        }
        if let Some(grouping_size) = self.grouping_size {
            integer = group_integer(&integer, grouping_size);
        }
        Ok(fraction.map_or(integer.clone(), |fraction| format!("{integer}.{fraction}")))
    }

    fn format_scientific(
        &self,
        value: &BigDecimal,
        rounding_mode: NumberRoundingMode,
    ) -> Result<String, NumberFormatError> {
        let exponent_digits = self.exponent_digits.expect("scientific pattern");
        let (coefficient, scale) = value.as_bigint_and_exponent();
        let mut exponent = if coefficient == BigInt::from(0) {
            0
        } else {
            // 系数位数与指数整数位数均远小于 i64 上限，try_from 恒成功
            let digits = i64::try_from(coefficient.to_str_radix(10).trim_start_matches('-').len())
                .expect("BigDecimal 系数位数不可能超过 i64::MAX");
            let scientific = digits - scale - 1;
            let width = i64::try_from(self.exponent_integer_digits)
                .expect("指数整数位数不可能超过 i64::MAX");
            scientific.div_euclid(width) * width
        };
        let mut mantissa = BigDecimal::new(coefficient, scale + exponent);
        let mut formatted = self.format_plain(&mantissa, rounding_mode)?;
        let integer_digits = formatted.split('.').next().unwrap_or("").len();
        if integer_digits > self.exponent_integer_digits {
            exponent += i64::try_from(self.exponent_integer_digits)
                .expect("指数整数位数不可能超过 i64::MAX");
            let (coefficient, scale) = value.as_bigint_and_exponent();
            mantissa = BigDecimal::new(coefficient, scale + exponent);
            formatted = self.format_plain(&mantissa, rounding_mode)?;
        }
        let sign = if exponent < 0 { "-" } else { "" };
        Ok(format!(
            "{formatted}E{sign}{:0width$}",
            exponent.unsigned_abs(),
            width = exponent_digits
        ))
    }

    fn parse_number(&self, input: &str, negative: bool) -> Result<Option<BigDecimal>, NumberFormatError> {
        let Some(mut remaining) = input.strip_prefix(&self.prefix) else {
            return Ok(None);
        };
        let mut byte_end = 0;
        let mut saw_digit = false;
        let mut saw_decimal = false;
        let mut saw_exponent = false;
        for (index, ch) in remaining.char_indices() {
            let accepted = if ch.is_ascii_digit() {
                saw_digit = true;
                true
            } else if ch == '.' && !saw_decimal && !saw_exponent {
                saw_decimal = true;
                true
            } else if ch == ',' && self.grouping_size.is_some() && !saw_decimal && !saw_exponent {
                true
            } else if matches!(ch, 'E' | 'e')
                && self.exponent_digits.is_some()
                && saw_digit
                && !saw_exponent
            {
                saw_exponent = true;
                true
            } else {
                matches!(ch, '+' | '-') && saw_exponent && remaining[..index].ends_with(['E', 'e'])
            };
            if !accepted {
                break;
            }
            byte_end = index + ch.len_utf8();
        }
        if !saw_digit {
            return Ok(None);
        }
        let numeric = &remaining[..byte_end];
        remaining = &remaining[byte_end..];
        if !self.suffix.is_empty() {
            let Some(after_suffix) = remaining.strip_prefix(&self.suffix) else {
                return Ok(None);
            };
            let _ = after_suffix;
        }
        let normalized = numeric.replace(',', "");
        let mut value = BigDecimal::from_str(&normalized)
            .map_err(|_| NumberFormatError::new(format!("DecimalFormat could not parse {input:?}")))?;
        if self.multiplier != 1 {
            value /= self.multiplier;
        }
        if negative {
            value = -value;
        }
        Ok(Some(value))
    }
}

fn tokenize_pattern(pattern: &str) -> Result<Vec<Vec<PatternToken>>, NumberFormatError> {
    let mut subpatterns = vec![Vec::new()];
    let mut quoted = false;
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                subpatterns
                    .last_mut()
                    .expect("one subpattern")
                    .push(PatternToken {
                        value: '\'',
                        literal: true,
                    });
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if ch == ';' && !quoted {
            subpatterns.push(Vec::new());
            continue;
        }
        subpatterns
            .last_mut()
            .expect("one subpattern")
            .push(PatternToken {
                value: ch,
                literal: quoted,
            });
    }
    if quoted {
        return Err(invalid_pattern(pattern, "unterminated quote"));
    }
    Ok(subpatterns)
}

fn is_numeric_pattern_token(token: PatternToken) -> bool {
    !token.literal && matches!(token.value, '#' | '0' | '.' | ',' | 'E')
}

fn render_affix(tokens: &[PatternToken]) -> String {
    tokens.iter().map(|token| token.value).collect()
}

fn affix_multiplier(prefix: &[PatternToken], suffix: &[PatternToken]) -> Result<i32, NumberFormatError> {
    let percent = prefix
        .iter()
        .chain(suffix)
        .filter(|token| !token.literal && token.value == '%')
        .count();
    let per_mille = prefix
        .iter()
        .chain(suffix)
        .filter(|token| !token.literal && token.value == '‰')
        .count();
    if percent + per_mille > 1 {
        return Err(NumberFormatError::new(
            "DecimalFormat pattern contains multiple multipliers".to_owned(),
        ));
    }
    Ok(if percent == 1 {
        100
    } else if per_mille == 1 {
        1_000
    } else {
        1
    })
}

fn round_decimal(
    value: &BigDecimal,
    scale: usize,
    mode: NumberRoundingMode,
) -> Result<BigDecimal, NumberFormatError> {
    let scale = i64::try_from(scale)
        .map_err(|_| NumberFormatError::new("DecimalFormat scale exceeds i64".to_owned()))?;
    if mode == NumberRoundingMode::Unnecessary {
        let truncated = value.with_scale_round(scale, bigdecimal::RoundingMode::Down);
        if &truncated != value {
            return Err(NumberFormatError::new(
                "rounding necessary for RoundingMode.UNNECESSARY".to_owned(),
            ));
        }
        return Ok(truncated);
    }
    Ok(value.with_scale_round(scale, mode.bigdecimal().expect("non-UNNECESSARY mode")))
}

fn group_integer(value: &str, size: usize) -> String {
    let mut output = String::with_capacity(value.len() + value.len() / size);
    for (index, ch) in value.chars().enumerate() {
        if index > 0 && (value.len() - index).is_multiple_of(size) {
            output.push(',');
        }
        output.push(ch);
    }
    output
}

fn invalid_pattern(pattern: &str, reason: &str) -> NumberFormatError {
    NumberFormatError::new(format!(
        "invalid DecimalFormat pattern {pattern:?}: {reason}"
    ))
}

/// 对应 Java：`NumberUtils.parseShort` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_short(value: &str) -> Result<i16, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_java_i16(&value))
}

/// 对应 Java：`NumberUtils.parseLong` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_long(value: &str) -> Result<i64, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_java_i64(&value))
}

/// 对应 Java：`NumberUtils.parseInteger` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_integer(value: &str) -> Result<i32, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_java_i32(&value))
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
    parse_decimal(value, None)
        .map(|value| i8::from_le_bytes(java_signed_low_bytes::<1>(&decimal_to_big_int(&value))))
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

/// Mirrors Apache Commons `NumberUtils.createBigInteger`.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制整数时返回 [`NumberFormatError::new`]。
pub fn parse_big_int(value: &str) -> Result<BigInt, NumberFormatError> {
    BigInt::from_str(value)
        .map_err(|_| NumberFormatError::new(format!("parseBigInteger failed for {value:?}")))
}

fn decimal_to_big_int(value: &BigDecimal) -> BigInt {
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
fn decimal_java_i16(value: &BigDecimal) -> i16 {
    i16::from_le_bytes(java_signed_low_bytes::<2>(&decimal_to_big_int(value)))
}

fn decimal_java_i32(value: &BigDecimal) -> i32 {
    i32::from_le_bytes(java_signed_low_bytes::<4>(&decimal_to_big_int(value)))
}

fn decimal_java_i64(value: &BigDecimal) -> i64 {
    i64::from_le_bytes(java_signed_low_bytes::<8>(&decimal_to_big_int(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> BigDecimal {
        value.parse().unwrap()
    }

    #[test]
    fn decimal_format_matches_java_golden_patterns() {
        for (pattern, value, expected) in [
            ("#.##%", "1.235", "123.5%"),
            ("#", "1.235", "1"),
            ("0.00", "1.235", "1.24"),
            ("#,##0.00", "1234.5", "1,234.50"),
            ("0.00;[neg]0.00", "-1.235", "[neg]1.24"),
            ("0.00E00", "1235", "1.24E03"),
        ] {
            let value = decimal(value);
            assert_eq!(
                format_decimal(&value, value < 0, Some(pattern), NumberRoundingMode::HalfUp,)
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn decimal_parse_matches_java_parse_position_behavior() {
        assert_eq!(
            parse_decimal("12.34%", Some("#.##%")).unwrap(),
            decimal("0.1234")
        );
        assert!(parse_decimal("12.34", Some("#.##%")).is_err());
        assert_eq!(
            parse_decimal("1,234.50", Some("#,##0.00")).unwrap(),
            decimal("1234.50")
        );
        assert_eq!(
            parse_decimal("1.00abc", Some("0.00")).unwrap(),
            decimal("1.00")
        );
        assert!(parse_decimal(" 1.00", Some("0.00")).is_err());
        assert!(parse_decimal("abc1.00", Some("0.00")).is_err());
    }

    #[test]
    fn no_format_is_full_input_big_decimal_and_unnecessary_rejects_rounding() {
        assert_eq!(parse_integer("1.00").unwrap(), 1);
        assert_eq!(parse_byte("255.9").unwrap(), -1);
        assert!(parse_big_decimal(" 1.00").is_err());
        assert!(parse_big_decimal("1.00 ").is_err());
        assert!(
            format_decimal(
                &decimal("1.001"),
                false,
                Some("0.00"),
                NumberRoundingMode::Unnecessary,
            )
            .is_err()
        );
    }

    #[test]
    fn all_java_rounding_modes_match_direction_and_tie_rules() {
        for (mode, positive, negative) in [
            (NumberRoundingMode::Up, "1.3", "-1.3"),
            (NumberRoundingMode::Down, "1.2", "-1.2"),
            (NumberRoundingMode::Ceiling, "1.3", "-1.2"),
            (NumberRoundingMode::Floor, "1.2", "-1.3"),
        ] {
            assert_eq!(
                format_decimal(&decimal("1.21"), false, Some("0.0"), mode).unwrap(),
                positive
            );
            assert_eq!(
                format_decimal(&decimal("-1.21"), true, Some("0.0"), mode).unwrap(),
                negative
            );
        }
        for (mode, expected) in [
            (NumberRoundingMode::HalfUp, "1.3"),
            (NumberRoundingMode::HalfDown, "1.2"),
            (NumberRoundingMode::HalfEven, "1.2"),
        ] {
            assert_eq!(
                format_decimal(&decimal("1.25"), false, Some("0.0"), mode).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn quoted_affixes_per_mille_and_scientific_parse_are_supported() {
        assert_eq!(
            format_decimal(
                &decimal("12.5"),
                false,
                Some("'USD '0.00"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "USD 12.50"
        );
        assert_eq!(
            format_decimal(
                &decimal("0.01234"),
                false,
                Some("#.##‰"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "12.34‰"
        );
        assert_eq!(
            parse_decimal("1.24E03", Some("0.00E00")).unwrap(),
            decimal("1240")
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    fn decimal(value: &str) -> BigDecimal {
        value.parse().unwrap()
    }

    #[test]
    fn format_without_pattern_returns_java_plain_string() {
        // 对应 Java：`NumberUtils.format` 在无格式时返回 `toPlainString()`
        assert_eq!(
            format_decimal(&decimal("1.50"), false, None, NumberRoundingMode::HalfUp).unwrap(),
            "1.50"
        );
        assert_eq!(
            format_decimal(
                &decimal("-1.50"),
                true,
                Some(""),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "-1.50"
        );
    }

    #[test]
    fn format_non_finite_negative_infinity_without_pattern() {
        // 对应 Java：`DecimalFormat` 对 `-Infinity` 使用负子模式
        assert_eq!(
            format_non_finite(NonFiniteNumber::NegativeInfinity, None).unwrap(),
            "-Infinity"
        );
        assert_eq!(
            format_non_finite(NonFiniteNumber::NegativeInfinity, Some("#")).unwrap(),
            "-∞"
        );
    }

    #[test]
    fn invalid_patterns_are_rejected_with_java_messages() {
        // 对应 Java：`DecimalFormat` 非法模式抛 `IllegalArgumentException`
        for pattern in ["abc", "0a0", "0.00E#", ",", "0;0;0", "'0.00", "0%‰"] {
            assert!(
                format_decimal(
                    &decimal("1.5"),
                    false,
                    Some(pattern),
                    NumberRoundingMode::HalfUp
                )
                .is_err(),
                "pattern {pattern:?} should be rejected"
            );
            assert!(parse_decimal("1.5", Some(pattern)).is_err());
        }
    }

    #[test]
    fn plain_format_pads_integer_and_strips_trailing_fraction_zeros() {
        // 对应 Java：`DecimalFormat` 整数位补零、小数位去除末尾零
        assert_eq!(
            format_decimal(
                &decimal("5"),
                false,
                Some("00.0"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "05.0"
        );
        assert_eq!(
            format_decimal(
                &decimal("2"),
                false,
                Some("#.##"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "2"
        );
        assert_eq!(
            format_decimal(
                &decimal("1.2"),
                false,
                Some("0.00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.20"
        );
    }

    #[test]
    fn scientific_format_zero_and_exponent_carry() {
        // 对应 Java：`DecimalFormat` 科学计数法，零值指数为 0，舍入进位后修正指数
        assert_eq!(
            format_decimal(
                &decimal("0"),
                false,
                Some("0.00E00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "0.00E00"
        );
        assert_eq!(
            format_decimal(
                &decimal("12.5"),
                false,
                Some("0.00E00"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.25E01"
        );
        assert_eq!(
            format_decimal(
                &decimal("9.95"),
                false,
                Some("0.0E0"),
                NumberRoundingMode::HalfUp
            )
            .unwrap(),
            "1.0E1"
        );
    }

    #[test]
    fn parse_with_exponent_signs_and_quoted_apostrophes() {
        // 对应 Java：`DecimalFormat.parse` 支持指数正负号与转义单引号
        assert_eq!(
            parse_decimal("1.24E-03", Some("0.00E00")).unwrap(),
            decimal("0.00124")
        );
        assert_eq!(
            parse_decimal("1.24E+03", Some("0.00E00")).unwrap(),
            decimal("1240")
        );
        assert_eq!(
            format_decimal(
                &decimal("1.5"),
                false,
                Some("'it''s'0.00"),
                NumberRoundingMode::HalfUp,
            )
            .unwrap(),
            "it's1.50"
        );
    }

    #[test]
    fn unnecessary_rounding_succeeds_when_value_fits_scale() {
        // 对应 Java：`RoundingMode.UNNECESSARY` 在无需舍入时直接返回
        assert_eq!(
            format_decimal(
                &decimal("1.00"),
                false,
                Some("0.00"),
                NumberRoundingMode::Unnecessary,
            )
            .unwrap(),
            "1.00"
        );
    }

    #[test]
    fn parse_short_and_long_match_java_wrapping_and_errors() {
        // 对应 Java：`NumberUtils.parseShort` / `parseLong` 低位截断回绕
        assert_eq!(parse_short("127").unwrap(), 127);
        assert_eq!(parse_short("-128.9").unwrap(), -128);
        assert_eq!(parse_short("65535.9").unwrap(), -1);
        assert!(parse_short("abc").is_err());
        assert_eq!(parse_long("123").unwrap(), 123);
        assert_eq!(parse_long("18446744073709551615.9").unwrap(), -1);
        assert!(parse_long("abc").is_err());
        assert!(parse_integer("abc").is_err());
    }

    #[test]
    // 1.5 / 2.5 均可被 f32/f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn parse_float_double_and_big_int_match_java() {
        // 对应 Java：`NumberUtils.parseFloat` / `parseDouble` / Apache Commons `createBigInteger`
        assert_eq!(parse_float("1.5").unwrap(), 1.5);
        assert!(parse_float("1e100").unwrap().is_infinite());
        assert_eq!(parse_double("2.5").unwrap(), 2.5);
        assert!(parse_double("1e100000").unwrap().is_infinite());
        assert_eq!(parse_big_int("123").unwrap(), BigInt::from(123));
        assert_eq!(parse_big_int("-123").unwrap(), BigInt::from(-123));
        assert!(parse_big_int("abc").is_err());
    }

    #[test]
    fn parse_byte_negative_values_sign_extend_low_byte() {
        // 对应 Java：`NumberUtils.parseByte` 使用二进制补码低位字节（符号扩展）
        assert_eq!(parse_byte("-1.0").unwrap(), -1);
        assert_eq!(parse_byte("127.9").unwrap(), 127);
    }
}
