/// Excel's decimal math context retains at most 15 significant digits.
/// 对应 Java：NumberUtils.parseShort。
pub const EXCEL_MATH_CONTEXT_PRECISION: u32 = 15;

include!("excel_math_context_precision_to_parse_long/number_format_error.rs");



/// 对应 Java：NumberUtils.parseShort。 将任意精度十进制数转换为有限 `f64`。
///
/// # Errors
///
/// 超出目标电子表格数值范围时返回格式错误。
pub fn finite_decimal_f64(value: &BigDecimal, format: &str) -> Result<f64, NumberFormatError> {
    value
        .to_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            NumberFormatError::new(format!("decimal value exceeds {format} numeric range"))
        })
}

/// 对应 Java：NumberUtils.parseShort。 判断整数十进制数是否超出 Excel 可精确表示的 53 位范围。
///
/// # Errors
///
/// 数值无法转换为有限 Excel 数字时返回格式错误。
pub fn decimal_integer_requires_text(value: &BigDecimal) -> Result<bool, NumberFormatError> {
    const MAX_EXACT_EXCEL_INTEGER: i64 = 9_007_199_254_740_991;
    let _ = finite_decimal_f64(value, "Excel")?;
    if value != &value.with_scale(0) {
        return Ok(false);
    }
    let maximum = BigDecimal::from(MAX_EXACT_EXCEL_INTEGER);
    let minimum = -maximum.clone();
    Ok(value > &maximum || value < &minimum)
}

/// 对应 Java：NumberUtils.parseShort。 将 chrono/strftime 日期格式占位符转换为 Excel 数字格式代码。
#[must_use]
pub fn excel_date_format_code(format: Option<&str>, default: &str) -> String {
    format
        .unwrap_or(default)
        .replace("%Y", "yyyy")
        .replace("%m", "mm")
        .replace("%d", "dd")
        .replace("%H", "hh")
        .replace("%M", "mm")
        .replace("%S", "ss")
}

include!("excel_math_context_precision_to_parse_long/number_rounding_mode.rs");





include!("excel_math_context_precision_to_parse_long/non_finite_number.rs");

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

/// 对应 Java：NumberUtils.parseShort。 Formats a finite number like Java `NumberUtils.format`.
///
/// # Errors
///
/// Returns [`NumberFormatError`] when the supplied decimal pattern is invalid.
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

/// 对应 Java：NumberUtils.parseShort。 Formats Java `NaN` / infinity through `DecimalFormat` affixes.
///
/// # Errors
///
/// Returns [`NumberFormatError`] when the supplied decimal pattern is invalid.
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

/// 对应 Java：NumberUtils.parseShort。 Parses a string like Java `NumberUtils.parseBigDecimal`.
///
/// # Errors
///
/// Returns [`NumberFormatError`] when the value or supplied decimal pattern is invalid.
pub fn parse_decimal(value: &str, pattern: Option<&str>) -> Result<BigDecimal, NumberFormatError> {
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

    fn parse_number(
        &self,
        input: &str,
        negative: bool,
    ) -> Result<Option<BigDecimal>, NumberFormatError> {
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
        let mut value = BigDecimal::from_str(&normalized).map_err(|_| {
            NumberFormatError::new(format!("DecimalFormat could not parse {input:?}"))
        })?;
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

fn affix_multiplier(
    prefix: &[PatternToken],
    suffix: &[PatternToken],
) -> Result<i32, NumberFormatError> {
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
    parse_decimal(value, None).map(|value| decimal_to_java_i16(&value))
}

/// 对应 Java：`NumberUtils.parseLong` without a format.
///
/// # Errors
///
/// 当 `value` 无法解析为合法的十进制数时返回 [`NumberFormatError::new`]。
pub fn parse_long(value: &str) -> Result<i64, NumberFormatError> {
    parse_decimal(value, None).map(|value| decimal_to_java_i64(&value))
}

