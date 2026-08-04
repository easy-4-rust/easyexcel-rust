//! Text worksheet functions.

use super::{Registry, VARIADIC, wildcard_match};
use crate::core::error::CellError;
use crate::core::formula::coerce::{to_number, to_text};
use crate::core::formula::context::Context;
use crate::core::formula::value::{Array, Value};

pub fn register(r: &mut Registry) {
    // concatenation
    r.add("CONCAT", 1, VARIADIC, false, concat);
    r.add("CONCATENATE", 1, VARIADIC, false, concat);
    r.add("TEXTJOIN", 3, VARIADIC, false, textjoin);

    // length / substrings
    r.add("LEN", 1, 1, false, len);
    r.add("LEFT", 1, 2, false, left);
    r.add("RIGHT", 1, 2, false, right);
    r.add("MID", 3, 3, false, mid);

    // case / trimming
    r.add("TRIM", 1, 1, false, trim);
    r.add("UPPER", 1, 1, false, upper);
    r.add("LOWER", 1, 1, false, lower);
    r.add("PROPER", 1, 1, false, proper);

    // find / search / replace / substitute
    r.add("FIND", 2, 3, false, find);
    r.add("SEARCH", 2, 3, false, search);
    r.add("SUBSTITUTE", 3, 4, false, substitute);
    r.add("REPLACE", 4, 4, false, replace);

    // repeat
    r.add("REPT", 2, 2, false, rept);

    // formatting / conversion
    r.add("TEXT", 2, 2, false, text_fn);
    r.add("VALUE", 1, 1, false, value_fn);
    r.add("NUMBERVALUE", 2, 3, false, numbervalue);
    r.add("FIXED", 1, 3, false, fixed);
    r.add("DOLLAR", 1, 2, false, dollar);

    // comparison / char codes
    r.add("EXACT", 2, 2, false, exact);
    r.add("CHAR", 1, 1, false, char_fn);
    r.add("CODE", 1, 1, false, code_fn);
    r.add("UNICHAR", 1, 1, false, unichar);
    r.add("UNICODE", 1, 1, false, unicode);

    // misc
    r.add("CLEAN", 1, 1, false, clean);
    r.add("T", 1, 1, false, t_fn);

    // ── DBCS byte-variants (we treat chars = bytes for all non-DBCS text)
    // PARITY: LENB/LEFTB/RIGHTB/MIDB/FINDB/SEARCHB/REPLACEB delegate to their
    // char-based equivalents because this engine does not handle DBCS encodings.
    r.add("LENB", 1, 1, false, len);
    r.add("LEFTB", 1, 2, false, left);
    r.add("RIGHTB", 1, 2, false, right);
    r.add("MIDB", 3, 3, false, mid);
    r.add("FINDB", 2, 3, false, find);
    r.add("SEARCHB", 2, 3, false, search);
    r.add("REPLACEB", 4, 4, false, replace);

    // ── Modern text functions
    r.add("TEXTBEFORE", 2, 6, false, textbefore);
    r.add("TEXTAFTER", 2, 6, false, textafter);
    r.add("TEXTSPLIT", 2, 6, false, textsplit);
    r.add("VALUETOTEXT", 1, 2, false, valuetotext);
    r.add("ARRAYTOTEXT", 1, 2, false, arraytotext);

    // ── Regular expressions (Excel 365). PARITY: backed by the `regex` crate
    // (RE2 syntax), so backreferences/lookaround in a pattern are unsupported.
    r.add("REGEXTEST", 2, 3, false, regextest);
    r.add("REGEXEXTRACT", 2, 4, false, regexextract);
    r.add("REGEXREPLACE", 3, 5, false, regexreplace);
}

/// Compile a pattern, honoring Excel's case-insensitivity flag (0 = sensitive,
/// non-zero = insensitive). Returns `#VALUE!` for an invalid/unsupported pattern.
fn build_regex(pattern: &str, case_insensitive: bool) -> Result<regex::Regex, CellError> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|_| CellError::Value)
}

fn ci_flag(args: &[Value], idx: usize) -> Result<bool, CellError> {
    match args.get(idx) {
        None | Some(Value::Empty) => Ok(false),
        Some(v) => Ok(to_number(v)? != 0.0),
    }
}

fn regextest(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let pat = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let ci = match ci_flag(args, 2) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    match build_regex(&pat, ci) {
        Ok(re) => Value::Bool(re.is_match(&text)),
        Err(e) => Value::Error(e),
    }
}

fn regexextract(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let pat = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // return_mode: 0 = first match (default), 1 = all matches, 2 = capture groups
    let mode = match args.get(2) {
        None | Some(Value::Empty) => 0,
        Some(v) => match to_number(v) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        },
    };
    let ci = match ci_flag(args, 3) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    let re = match build_regex(&pat, ci) {
        Ok(re) => re,
        Err(e) => return Value::Error(e),
    };
    match mode {
        0 => match re.find(&text) {
            Some(m) => Value::Text(m.as_str().to_string()),
            None => Value::Error(CellError::NA),
        },
        1 => {
            let matches: Vec<Value> = re
                .find_iter(&text)
                .map(|m| Value::Text(m.as_str().to_string()))
                .collect();
            if matches.is_empty() {
                return Value::Error(CellError::NA);
            }
            let n = matches.len();
            Value::Array(crate::core::formula::value::Array::new(n, 1, matches))
        }
        2 => match re.captures(&text) {
            Some(caps) => {
                let groups: Vec<Value> = caps
                    .iter()
                    .skip(1)
                    .map(|g| {
                        g.map(|m| Value::Text(m.as_str().to_string()))
                            .unwrap_or(Value::Empty)
                    })
                    .collect();
                if groups.is_empty() {
                    Value::Text(
                        caps.get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    )
                } else {
                    let n = groups.len();
                    Value::Array(crate::core::formula::value::Array::new(1, n, groups))
                }
            }
            None => Value::Error(CellError::NA),
        },
        _ => Value::Error(CellError::Value),
    }
}

fn regexreplace(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let pat = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let repl = match to_text(&args[2]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // occurrence: 0 (default) = replace all; N = replace only the Nth match.
    let occurrence = match args.get(3) {
        None | Some(Value::Empty) => 0,
        Some(v) => match to_number(v) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        },
    };
    let ci = match ci_flag(args, 4) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    if occurrence < 0 {
        return Value::Error(CellError::Value);
    }
    let re = match build_regex(&pat, ci) {
        Ok(re) => re,
        Err(e) => return Value::Error(e),
    };
    let out = if occurrence == 0 {
        re.replace_all(&text, repl.as_str()).into_owned()
    } else {
        // Replace only the Nth (1-based) occurrence.
        let mut count = 0;
        re.replace_all(&text, |caps: &regex::Captures| {
            count += 1;
            if count == occurrence {
                let mut s = String::new();
                caps.expand(&repl, &mut s);
                s
            } else {
                caps.get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            }
        })
        .into_owned()
    };
    Value::Text(out)
}

// ---------------------------------------------------------------------------
// Concatenation
// ---------------------------------------------------------------------------

fn concat(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut out = String::new();
    for arg in args {
        for v in ctx.flatten(arg) {
            match to_text(&v) {
                Ok(s) => out.push_str(&s),
                Err(e) => return Value::Error(e),
            }
        }
    }
    Value::Text(out)
}

fn textjoin(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // args: delimiter, ignore_empty, text1, [text2, ...]
    let delim = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let ignore_empty = match &args[1] {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Empty => true,
        Value::Text(s) => s.eq_ignore_ascii_case("true"),
        Value::Error(e) => return Value::Error(*e),
        _ => true,
    };
    let mut parts: Vec<String> = Vec::new();
    for arg in &args[2..] {
        for v in ctx.flatten(arg) {
            match to_text(&v) {
                Ok(s) => {
                    if !ignore_empty || !s.is_empty() {
                        parts.push(s);
                    }
                }
                Err(e) => return Value::Error(e),
            }
        }
    }
    Value::Text(parts.join(&delim))
}

// ---------------------------------------------------------------------------
// Length / substrings
// ---------------------------------------------------------------------------

fn len(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => Value::Number(s.chars().count() as f64),
        Err(e) => Value::Error(e),
    }
}

fn left(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let count: usize = if args.len() < 2 || matches!(args[1], Value::Empty) {
        1
    } else {
        match to_number(&args[1]) {
            Ok(n) => {
                if n < 0.0 {
                    return Value::Error(CellError::Value);
                }
                n.floor() as usize
            }
            Err(e) => return Value::Error(e),
        }
    };
    Value::Text(s.chars().take(count).collect())
}

fn right(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let count: usize = if args.len() < 2 || matches!(args[1], Value::Empty) {
        1
    } else {
        match to_number(&args[1]) {
            Ok(n) => {
                if n < 0.0 {
                    return Value::Error(CellError::Value);
                }
                n.floor() as usize
            }
            Err(e) => return Value::Error(e),
        }
    };
    let chars: Vec<char> = s.chars().collect();
    let start = chars.len().saturating_sub(count);
    Value::Text(chars[start..].iter().collect())
}

fn mid(_: &mut dyn Context, args: &[Value]) -> Value {
    let s = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let start = match to_number(&args[1]) {
        Ok(n) => {
            if n < 1.0 {
                return Value::Error(CellError::Value);
            }
            (n.floor() as usize) - 1 // convert 1-based to 0-based
        }
        Err(e) => return Value::Error(e),
    };
    let count = match to_number(&args[2]) {
        Ok(n) => {
            if n < 0.0 {
                return Value::Error(CellError::Value);
            }
            n.floor() as usize
        }
        Err(e) => return Value::Error(e),
    };
    let chars: Vec<char> = s.chars().collect();
    if start >= chars.len() {
        return Value::Text(String::new());
    }
    Value::Text(chars[start..].iter().take(count).collect())
}

// ---------------------------------------------------------------------------
// Case / whitespace
// ---------------------------------------------------------------------------

fn trim(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => {
            // Strip leading/trailing spaces and collapse internal runs.
            let mut result = String::new();
            let mut last_space = true; // start as true to eat leading spaces
            for ch in s.chars() {
                if ch == ' ' {
                    if !last_space {
                        result.push(' ');
                    }
                    last_space = true;
                } else {
                    result.push(ch);
                    last_space = false;
                }
            }
            // Strip any trailing space that was pushed
            if result.ends_with(' ') {
                result.pop();
            }
            Value::Text(result)
        }
        Err(e) => Value::Error(e),
    }
}

fn upper(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => Value::Text(s.to_uppercase()),
        Err(e) => Value::Error(e),
    }
}

fn lower(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => Value::Text(s.to_lowercase()),
        Err(e) => Value::Error(e),
    }
}

fn proper(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => {
            let mut result = String::with_capacity(s.len());
            let mut capitalize_next = true;
            for ch in s.chars() {
                if ch.is_alphabetic() {
                    if capitalize_next {
                        for c in ch.to_uppercase() {
                            result.push(c);
                        }
                        capitalize_next = false;
                    } else {
                        for c in ch.to_lowercase() {
                            result.push(c);
                        }
                    }
                } else {
                    result.push(ch);
                    // After any non-letter, capitalize next letter
                    capitalize_next = true;
                }
            }
            Value::Text(result)
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Find / Search
// ---------------------------------------------------------------------------

/// FIND: case-sensitive, no wildcards, 1-based result.
fn find(_: &mut dyn Context, args: &[Value]) -> Value {
    let needle = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let haystack = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let start_num: usize = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_number(&args[2]) {
            Ok(n) => {
                if n < 1.0 {
                    return Value::Error(CellError::Value);
                }
                (n.floor() as usize) - 1
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let h_chars: Vec<char> = haystack.chars().collect();
    let n_chars: Vec<char> = needle.chars().collect();

    if n_chars.is_empty() {
        // Excel: FIND("", text, n) returns n (the start position)
        return Value::Number((start_num + 1) as f64);
    }

    if start_num > h_chars.len() {
        return Value::Error(CellError::Value);
    }

    for i in start_num..=h_chars.len().saturating_sub(n_chars.len()) {
        if h_chars[i..].starts_with(&n_chars[..]) {
            return Value::Number((i + 1) as f64);
        }
    }
    Value::Error(CellError::Value)
}

/// SEARCH: case-insensitive, supports `*`/`?` wildcards, 1-based result.
fn search(_: &mut dyn Context, args: &[Value]) -> Value {
    let needle = match to_text(&args[0]) {
        Ok(s) => s.to_lowercase(),
        Err(e) => return Value::Error(e),
    };
    let haystack = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let start_num: usize = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_number(&args[2]) {
            Ok(n) => {
                if n < 1.0 {
                    return Value::Error(CellError::Value);
                }
                (n.floor() as usize) - 1
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let h_chars: Vec<char> = haystack.chars().collect();
    let n_chars: Vec<char> = needle.chars().collect();

    if n_chars.is_empty() {
        return Value::Number((start_num + 1) as f64);
    }

    if start_num > h_chars.len() {
        return Value::Error(CellError::Value);
    }

    // Check if pattern has wildcards
    let has_wildcards = needle.contains('*') || needle.contains('?');

    if has_wildcards {
        // Try matching at each position using wildcard_match
        for i in start_num..=h_chars.len() {
            for end in i..=h_chars.len() {
                let slice: String = h_chars[i..end].iter().collect::<String>().to_lowercase();
                if wildcard_match(&needle, &slice) {
                    return Value::Number((i + 1) as f64);
                }
            }
        }
    } else {
        // Plain case-insensitive search
        for i in start_num..=h_chars.len().saturating_sub(n_chars.len()) {
            let slice: String = h_chars[i..i + n_chars.len()]
                .iter()
                .collect::<String>()
                .to_lowercase();
            if slice == needle {
                return Value::Number((i + 1) as f64);
            }
        }
    }

    Value::Error(CellError::Value)
}

// ---------------------------------------------------------------------------
// SUBSTITUTE / REPLACE
// ---------------------------------------------------------------------------

fn substitute(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let old_text = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let new_text = match to_text(&args[2]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // Optional instance_num (4th arg)
    let instance_num: Option<usize> = if args.len() >= 4 && !matches!(args[3], Value::Empty) {
        match to_number(&args[3]) {
            Ok(n) => {
                if n < 1.0 {
                    return Value::Error(CellError::Value);
                }
                Some(n.floor() as usize)
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        None
    };

    if old_text.is_empty() {
        return Value::Text(text);
    }

    let old_chars: Vec<char> = old_text.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut occurrence = 0usize;
    let mut i = 0;

    while i < text_chars.len() {
        if text_chars[i..].starts_with(&old_chars[..]) {
            occurrence += 1;
            let should_replace = match instance_num {
                None => true,
                Some(n) => occurrence == n,
            };
            if should_replace {
                result.push_str(&new_text);
            } else {
                for ch in &old_chars {
                    result.push(*ch);
                }
            }
            i += old_chars.len();
        } else {
            result.push(text_chars[i]);
            i += 1;
        }
    }
    Value::Text(result)
}

fn replace(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let start = match to_number(&args[1]) {
        Ok(n) => {
            if n < 1.0 {
                return Value::Error(CellError::Value);
            }
            (n.floor() as usize) - 1
        }
        Err(e) => return Value::Error(e),
    };
    let num_chars = match to_number(&args[2]) {
        Ok(n) => {
            if n < 0.0 {
                return Value::Error(CellError::Value);
            }
            n.floor() as usize
        }
        Err(e) => return Value::Error(e),
    };
    let new_text = match to_text(&args[3]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };

    let chars: Vec<char> = text.chars().collect();
    let before: String = chars[..start.min(chars.len())].iter().collect();
    let after_start = (start + num_chars).min(chars.len());
    let after: String = chars[after_start..].iter().collect();
    Value::Text(format!("{}{}{}", before, new_text, after))
}

// ---------------------------------------------------------------------------
// REPT
// ---------------------------------------------------------------------------

fn rept(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let times = match to_number(&args[1]) {
        Ok(n) => {
            if n < 0.0 {
                return Value::Error(CellError::Value);
            }
            n.floor() as usize
        }
        Err(e) => return Value::Error(e),
    };
    Value::Text(text.repeat(times))
}

// ---------------------------------------------------------------------------
// TEXT / VALUE / NUMBERVALUE / FIXED / DOLLAR
// ---------------------------------------------------------------------------

fn text_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let format_code = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    match &args[0] {
        Value::Error(e) => Value::Error(*e),
        Value::Text(s) => {
            // TEXT on text: if format is "@" (text format) return as-is, else return as-is
            Value::Text(s.clone())
        }
        Value::Empty => {
            if format_code == "@" {
                Value::Text(String::new())
            } else {
                let formatted =
                    crate::core::numfmt::format_value(0.0, &format_code, ctx.date_system());
                Value::Text(formatted)
            }
        }
        other => match to_number(other) {
            Ok(n) => {
                let formatted =
                    crate::core::numfmt::format_value(n, &format_code, ctx.date_system());
                Value::Text(formatted)
            }
            Err(e) => Value::Error(e),
        },
    }
}

fn value_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Number(n) => Value::Number(*n),
        Value::Bool(b) => Value::Number(if *b { 1.0 } else { 0.0 }),
        Value::Empty => Value::Number(0.0),
        Value::Error(e) => Value::Error(*e),
        Value::Text(s) => match crate::core::formula::coerce::parse_number_text(s) {
            Some(n) => Value::Number(n),
            None => Value::Error(CellError::Value),
        },
        _ => Value::Error(CellError::Value),
    }
}

/// NUMBERVALUE(text, decimal_separator, [group_separator])
fn numbervalue(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let decimal_sep = match to_text(&args[1]) {
        Ok(s) => {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => c,
                None => return Value::Error(CellError::Value),
            }
        }
        Err(e) => return Value::Error(e),
    };
    let group_sep: Option<char> = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_text(&args[2]) {
            Ok(s) => s.chars().next(),
            Err(e) => return Value::Error(e),
        }
    } else {
        None
    };

    if decimal_sep == group_sep.unwrap_or('\0') {
        return Value::Error(CellError::Value);
    }

    let trimmed = text.trim();
    let is_percent = trimmed.ends_with('%');
    let core = if is_percent {
        &trimmed[..trimmed.len() - 1]
    } else {
        trimmed
    };

    // Remove group separators, then replace decimal sep with '.'
    let cleaned: String = core
        .chars()
        .filter(|&c| Some(c) != group_sep)
        .map(|c| if c == decimal_sep { '.' } else { c })
        .collect();

    match cleaned.parse::<f64>() {
        Ok(n) => {
            let result = if is_percent { n / 100.0 } else { n };
            Value::Number(result)
        }
        Err(_) => Value::Error(CellError::Value),
    }
}

/// FIXED(number, [decimals], [no_commas])
fn fixed(_: &mut dyn Context, args: &[Value]) -> Value {
    let n = match to_number(&args[0]) {
        Ok(x) => x,
        Err(e) => return Value::Error(e),
    };
    let decimals: i32 = if args.len() >= 2 && !matches!(args[1], Value::Empty) {
        match to_number(&args[1]) {
            Ok(d) => d.floor() as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        2
    };
    let no_commas: bool = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_number(&args[2]) {
            Ok(d) => d != 0.0,
            Err(e) => return Value::Error(e),
        }
    } else {
        false
    };

    Value::Text(format_fixed(n, decimals, no_commas))
}

fn format_fixed(n: f64, decimals: i32, no_commas: bool) -> String {
    let factor = 10f64.powi(decimals);
    let rounded = (n * factor).round() / factor;
    let abs_val = rounded.abs();
    let is_neg = rounded < 0.0;

    let int_part = abs_val.floor() as u64;
    let dec_str = if decimals > 0 {
        let frac = abs_val - int_part as f64;
        let scaled = (frac * 10f64.powi(decimals)).round() as u64;
        format!(".{:0>width$}", scaled, width = decimals as usize)
    } else {
        String::new()
    };

    let int_str = int_part.to_string();
    let int_with_commas = if !no_commas {
        add_thousands_commas(&int_str)
    } else {
        int_str
    };

    let sign = if is_neg { "-" } else { "" };
    format!("{}{}{}", sign, int_with_commas, dec_str)
}

fn add_thousands_commas(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// DOLLAR(number, [decimals])
fn dollar(_: &mut dyn Context, args: &[Value]) -> Value {
    let n = match to_number(&args[0]) {
        Ok(x) => x,
        Err(e) => return Value::Error(e),
    };
    let decimals: i32 = if args.len() >= 2 && !matches!(args[1], Value::Empty) {
        match to_number(&args[1]) {
            Ok(d) => d.floor() as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        2
    };

    let formatted = format_fixed(n, decimals, false);
    // Wrap negative numbers: Excel does -$1,234.56 (dollar sign before minus)
    if formatted.starts_with('-') {
        Value::Text(format!(
            "-${}",
            formatted.strip_prefix('-').unwrap_or(&formatted)
        ))
    } else {
        Value::Text(format!("${}", formatted))
    }
}

// ---------------------------------------------------------------------------
// EXACT
// ---------------------------------------------------------------------------

fn exact(_: &mut dyn Context, args: &[Value]) -> Value {
    let a = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let b = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    Value::Bool(a == b)
}

// ---------------------------------------------------------------------------
// CHAR / CODE / UNICHAR / UNICODE
// ---------------------------------------------------------------------------

fn char_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_number(&args[0]) {
        Ok(n) => {
            let code = n.floor() as u32;
            if !(1..=255).contains(&code) {
                return Value::Error(CellError::Value);
            }
            match char::from_u32(code) {
                Some(c) => Value::Text(c.to_string()),
                None => Value::Error(CellError::Value),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn code_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => match s.chars().next() {
            Some(c) => Value::Number(c as u32 as f64),
            None => Value::Error(CellError::Value),
        },
        Err(e) => Value::Error(e),
    }
}

fn unichar(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_number(&args[0]) {
        Ok(n) => {
            let code = n.floor() as u32;
            match char::from_u32(code) {
                Some(c) => Value::Text(c.to_string()),
                None => Value::Error(CellError::Value),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn unicode(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => match s.chars().next() {
            Some(c) => Value::Number(c as u32 as f64),
            None => Value::Error(CellError::Value),
        },
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// CLEAN / T
// ---------------------------------------------------------------------------

fn clean(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_text(&args[0]) {
        Ok(s) => {
            // Strip non-printable ASCII (chars 0–31)
            let cleaned: String = s.chars().filter(|&c| c as u32 >= 32).collect();
            Value::Text(cleaned)
        }
        Err(e) => Value::Error(e),
    }
}

fn t_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Text(s) => Value::Text(s.clone()),
        Value::Error(e) => Value::Error(*e),
        _ => Value::Text(String::new()),
    }
}

// ---------------------------------------------------------------------------
// TEXTBEFORE / TEXTAFTER
// ---------------------------------------------------------------------------

/// TEXTBEFORE(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])
///
/// Returns the substring before the Nth occurrence of `delimiter`.
/// instance_num defaults to 1; negative values count from the end.
/// match_mode 1 = case-insensitive (default 0 = case-sensitive).
/// PARITY: match_end and if_not_found beyond a plain scalar are simplified here.
fn textbefore(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let delim = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // instance_num (arg 2), default 1
    let instance_num: i64 = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_number(&args[2]) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    // match_mode (arg 3): 0 = case-sensitive (default), 1 = case-insensitive
    let case_insensitive = if args.len() >= 4 && !matches!(args[3], Value::Empty) {
        match to_number(&args[3]) {
            Ok(n) => n as i64 == 1,
            Err(e) => return Value::Error(e),
        }
    } else {
        false
    };
    // if_not_found (arg 5)
    let if_not_found: Option<Value> = if args.len() >= 6 && !matches!(args[5], Value::Empty) {
        Some(args[5].clone())
    } else {
        None
    };

    if delim.is_empty() {
        return if_not_found.unwrap_or(Value::Error(CellError::NA));
    }

    let (search_text, search_delim) = if case_insensitive {
        (text.to_lowercase(), delim.to_lowercase())
    } else {
        (text.clone(), delim.clone())
    };

    // Collect all occurrence positions (byte offsets in search_text)
    let mut positions: Vec<usize> = Vec::new();
    let mut start = 0;
    while let Some(pos) = search_text[start..].find(&search_delim) {
        positions.push(start + pos);
        start += pos + search_delim.len().max(1);
    }

    if positions.is_empty() {
        return if_not_found.unwrap_or(Value::Error(CellError::NA));
    }

    // Resolve instance_num
    let idx: usize = if instance_num > 0 {
        let n = instance_num as usize;
        if n > positions.len() {
            return if_not_found.unwrap_or(Value::Error(CellError::NA));
        }
        positions[n - 1]
    } else if instance_num < 0 {
        let n = (-instance_num) as usize;
        if n > positions.len() {
            return if_not_found.unwrap_or(Value::Error(CellError::NA));
        }
        positions[positions.len() - n]
    } else {
        return Value::Error(CellError::Value);
    };

    // The result is the text *before* the delimiter at `idx` (byte offset).
    // We return the substring of the *original* text up to that byte position.
    Value::Text(text[..idx].to_string())
}

/// TEXTAFTER(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])
///
/// Returns the substring after the Nth occurrence of `delimiter`.
/// PARITY: match_end and if_not_found beyond a plain scalar are simplified here.
/// Collect delimiter strings from an argument (an array yields several).
fn collect_delims(v: &Value) -> Vec<String> {
    let raw = match v {
        Value::Array(a) => a.data.iter().filter_map(|e| to_text(e).ok()).collect(),
        other => to_text(other).into_iter().collect::<Vec<_>>(),
    };
    raw.into_iter().filter(|d| !d.is_empty()).collect()
}

/// Split `s` on the earliest occurrence of any delimiter (case-insensitive when
/// `ci`). Delimiters are assumed ASCII (the common case) for byte alignment.
fn split_on_any(s: &str, delims: &[String], ci: bool) -> Vec<String> {
    if delims.is_empty() {
        return vec![s.to_string()];
    }
    let hay = if ci { s.to_lowercase() } else { s.to_string() };
    let dl: Vec<String> = if ci {
        delims.iter().map(|d| d.to_lowercase()).collect()
    } else {
        delims.to_vec()
    };
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut pos = 0usize;
    loop {
        let mut best: Option<(usize, usize)> = None;
        for d in &dl {
            if let Some(rel) = hay[pos..].find(d.as_str()) {
                let ms = pos + rel;
                if best.is_none_or(|(bs, _)| ms < bs) {
                    best = Some((ms, d.len()));
                }
            }
        }
        match best {
            Some((ms, ml)) => {
                out.push(s[start..ms].to_string());
                pos = ms + ml;
                start = pos;
            }
            None => {
                out.push(s[start..].to_string());
                break;
            }
        }
    }
    out
}

/// `TEXTSPLIT(text, col_delim, [row_delim], [ignore_empty], [match_mode], [pad])`
/// — split text into a 2-D array (a dynamic-array / spill function).
fn textsplit(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let col_delims = collect_delims(&args[1]);
    let row_delims = args.get(2).map(collect_delims).unwrap_or_default();
    let ignore_empty = matches!(args.get(3), Some(v) if matches!(crate::core::formula::coerce::to_bool(v), Ok(true)));
    let ci = matches!(args.get(4), Some(v) if matches!(to_number(v), Ok(n) if n != 0.0));
    let pad = args
        .get(5)
        .cloned()
        .filter(|v| !matches!(v, Value::Empty))
        .unwrap_or(Value::Error(CellError::NA));

    if col_delims.is_empty() && row_delims.is_empty() {
        return Value::Error(CellError::Value);
    }

    // Rows first (by row delimiters), then cells (by column delimiters).
    let row_strs: Vec<String> = if row_delims.is_empty() {
        vec![text]
    } else {
        split_on_any(&text, &row_delims, ci)
    };
    let mut rows: Vec<Vec<Value>> = Vec::new();
    for rs in row_strs {
        let mut cells: Vec<String> = split_on_any(&rs, &col_delims, ci);
        if ignore_empty {
            cells.retain(|c| !c.is_empty());
        }
        if ignore_empty && cells.is_empty() {
            continue;
        }
        rows.push(cells.into_iter().map(Value::Text).collect());
    }
    if rows.is_empty() {
        return Value::Error(CellError::Calc);
    }
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let nrows = rows.len();
    let mut data = Vec::with_capacity(nrows * ncols);
    for row in rows {
        let len = row.len();
        for v in row {
            data.push(v);
        }
        for _ in len..ncols {
            data.push(pad.clone());
        }
    }
    Value::Array(Array::new(nrows, ncols, data))
}

fn textafter(_: &mut dyn Context, args: &[Value]) -> Value {
    let text = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let delim = match to_text(&args[1]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let instance_num: i64 = if args.len() >= 3 && !matches!(args[2], Value::Empty) {
        match to_number(&args[2]) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let case_insensitive = if args.len() >= 4 && !matches!(args[3], Value::Empty) {
        match to_number(&args[3]) {
            Ok(n) => n as i64 == 1,
            Err(e) => return Value::Error(e),
        }
    } else {
        false
    };
    let if_not_found: Option<Value> = if args.len() >= 6 && !matches!(args[5], Value::Empty) {
        Some(args[5].clone())
    } else {
        None
    };

    if delim.is_empty() {
        return if_not_found.unwrap_or(Value::Error(CellError::NA));
    }

    let (search_text, search_delim) = if case_insensitive {
        (text.to_lowercase(), delim.to_lowercase())
    } else {
        (text.clone(), delim.clone())
    };

    let mut positions: Vec<usize> = Vec::new();
    let mut start = 0;
    while let Some(pos) = search_text[start..].find(&search_delim) {
        positions.push(start + pos);
        start += pos + search_delim.len().max(1);
    }

    if positions.is_empty() {
        return if_not_found.unwrap_or(Value::Error(CellError::NA));
    }

    let idx: usize = if instance_num > 0 {
        let n = instance_num as usize;
        if n > positions.len() {
            return if_not_found.unwrap_or(Value::Error(CellError::NA));
        }
        positions[n - 1] + search_delim.len()
    } else if instance_num < 0 {
        let n = (-instance_num) as usize;
        if n > positions.len() {
            return if_not_found.unwrap_or(Value::Error(CellError::NA));
        }
        positions[positions.len() - n] + search_delim.len()
    } else {
        return Value::Error(CellError::Value);
    };

    Value::Text(text[idx..].to_string())
}

// ---------------------------------------------------------------------------
// VALUETOTEXT / ARRAYTOTEXT
// ---------------------------------------------------------------------------

/// VALUETOTEXT(value, [format])
///
/// format 0 (default) = concise display form.
/// format 1 = strict: text values are double-quoted, errors as-is.
fn valuetotext(_: &mut dyn Context, args: &[Value]) -> Value {
    let format: i64 = if args.len() >= 2 && !matches!(args[1], Value::Empty) {
        match to_number(&args[1]) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let strict = format == 1;
    Value::Text(value_to_text_str(&args[0], strict))
}

fn value_to_text_str(v: &Value, strict: bool) -> String {
    match v {
        Value::Text(s) => {
            if strict {
                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        Value::Number(n) => crate::core::value::format_number_general(*n),
        Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.into(),
        Value::Empty => String::new(),
        Value::Error(e) => format!("{e:?}"), // e.g. "#VALUE!"
        Value::Array(a) => a
            .data
            .first()
            .map(|first| value_to_text_str(first, strict))
            .unwrap_or_default(),
        Value::Ref(_) | Value::Lambda(_) => String::new(),
    }
}

/// ARRAYTOTEXT(array, [format])
///
/// format 0 (default) = concise: values joined with ", ".
/// format 1 = strict: `{row1_col1,row1_col2;row2_col1,...}` with text quoted.
fn arraytotext(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let format: i64 = if args.len() >= 2 && !matches!(args[1], Value::Empty) {
        match to_number(&args[1]) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let strict = format == 1;

    // Materialise the array
    let arr = match &args[0] {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => {
            // Scalar: just format the single value
            return Value::Text(value_to_text_str(other, strict));
        }
    };

    if !strict {
        // format 0: flat list joined by ", "
        let parts: Vec<String> = arr
            .data
            .iter()
            .map(|v| value_to_text_str(v, false))
            .collect();
        return Value::Text(parts.join(", "));
    }

    // format 1: {col,col;row2...}
    let mut rows: Vec<String> = Vec::with_capacity(arr.rows);
    for r in 0..arr.rows {
        let mut cols: Vec<String> = Vec::with_capacity(arr.cols);
        for c in 0..arr.cols {
            let v = arr.get(r, c).unwrap_or(&Value::Empty);
            cols.push(value_to_text_str(v, true));
        }
        rows.push(cols.join(","));
    }
    Value::Text(format!("{{{}}}", rows.join(";")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::formula::functions::testutil::TestCtx;

    // --- regex --------------------------------------------------------------

    #[test]
    fn regex_functions() {
        let mut c = TestCtx::new();
        // REGEXTEST
        assert_eq!(
            regextest(
                &mut c,
                &[Value::Text("abc123".into()), Value::Text(r"\d+".into())]
            ),
            Value::Bool(true)
        );
        assert_eq!(
            regextest(
                &mut c,
                &[
                    Value::Text("ABC".into()),
                    Value::Text("abc".into()),
                    Value::Number(1.0)
                ]
            ),
            Value::Bool(true)
        );
        // invalid pattern → #VALUE!
        assert_eq!(
            regextest(&mut c, &[Value::Text("x".into()), Value::Text("(".into())]),
            Value::Error(CellError::Value)
        );
        // REGEXEXTRACT first match
        assert_eq!(
            regexextract(
                &mut c,
                &[Value::Text("id=42;".into()), Value::Text(r"\d+".into())]
            ),
            Value::Text("42".into())
        );
        // no match → #N/A
        assert_eq!(
            regexextract(
                &mut c,
                &[Value::Text("abc".into()), Value::Text(r"\d+".into())]
            ),
            Value::Error(CellError::NA)
        );
        // REGEXREPLACE all, with backref
        assert_eq!(
            regexreplace(
                &mut c,
                &[
                    Value::Text("a1b2".into()),
                    Value::Text(r"(\d)".into()),
                    Value::Text("[$1]".into())
                ]
            ),
            Value::Text("a[1]b[2]".into())
        );
        // REGEXREPLACE only 2nd occurrence
        assert_eq!(
            regexreplace(
                &mut c,
                &[
                    Value::Text("x-x-x".into()),
                    Value::Text("x".into()),
                    Value::Text("Y".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("x-Y-x".into())
        );
    }

    // --- concat / concatenate -----------------------------------------------

    #[test]
    fn concat_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(
                &mut c,
                &[Value::Text("Hello".into()), Value::Text(" World".into())]
            ),
            Value::Text("Hello World".into())
        );
    }

    #[test]
    fn concat_numbers() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(&mut c, &[Value::Number(1.0), Value::Text("x".into())]),
            Value::Text("1x".into())
        );
    }

    #[test]
    fn concat_error_propagates() {
        let mut c = TestCtx::new();
        assert_eq!(
            concat(&mut c, &[Value::Error(CellError::Value)]),
            Value::Error(CellError::Value)
        );
    }

    // --- textjoin -----------------------------------------------------------

    #[test]
    fn textjoin_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text(",".into()),
                    Value::Bool(true),
                    Value::Text("a".into()),
                    Value::Text("b".into()),
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a,b,c".into())
        );
    }

    #[test]
    fn textjoin_ignore_empty() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text("-".into()),
                    Value::Bool(true),
                    Value::Text("a".into()),
                    Value::Empty,
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a-c".into())
        );
    }

    #[test]
    fn textjoin_keep_empty() {
        let mut c = TestCtx::new();
        assert_eq!(
            textjoin(
                &mut c,
                &[
                    Value::Text("-".into()),
                    Value::Bool(false),
                    Value::Text("a".into()),
                    Value::Empty,
                    Value::Text("c".into()),
                ]
            ),
            Value::Text("a--c".into())
        );
    }

    // --- len ----------------------------------------------------------------

    #[test]
    fn len_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            len(&mut c, &[Value::Text("hello".into())]),
            Value::Number(5.0)
        );
        assert_eq!(
            len(&mut c, &[Value::Text(String::new())]),
            Value::Number(0.0)
        );
        assert_eq!(
            len(&mut c, &[Value::Error(CellError::Value)]),
            Value::Error(CellError::Value)
        );
    }

    // --- left ---------------------------------------------------------------

    #[test]
    fn left_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into()), Value::Number(2.0)]),
            Value::Text("he".into())
        );
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into()), Value::Number(0.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            left(&mut c, &[Value::Text("hello".into())]),
            Value::Text("h".into())
        );
    }

    #[test]
    fn left_over_length() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hi".into()), Value::Number(10.0)]),
            Value::Text("hi".into())
        );
    }

    #[test]
    fn left_negative_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            left(&mut c, &[Value::Text("hi".into()), Value::Number(-1.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- right --------------------------------------------------------------

    #[test]
    fn right_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            right(&mut c, &[Value::Text("hello".into()), Value::Number(3.0)]),
            Value::Text("llo".into())
        );
        assert_eq!(
            right(&mut c, &[Value::Text("hello".into())]),
            Value::Text("o".into())
        );
    }

    #[test]
    fn right_over_length() {
        let mut c = TestCtx::new();
        assert_eq!(
            right(&mut c, &[Value::Text("hi".into()), Value::Number(10.0)]),
            Value::Text("hi".into())
        );
    }

    // --- mid ----------------------------------------------------------------

    #[test]
    fn mid_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Text("ell".into())
        );
    }

    #[test]
    fn mid_past_end() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(4.0),
                    Value::Number(10.0)
                ]
            ),
            Value::Text("lo".into())
        );
    }

    #[test]
    fn mid_start_zero_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            mid(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(0.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- trim ---------------------------------------------------------------

    #[test]
    fn trim_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            trim(&mut c, &[Value::Text("  hello   world  ".into())]),
            Value::Text("hello world".into())
        );
        assert_eq!(
            trim(&mut c, &[Value::Text("  spaces  ".into())]),
            Value::Text("spaces".into())
        );
        assert_eq!(
            trim(&mut c, &[Value::Text("no spaces".into())]),
            Value::Text("no spaces".into())
        );
    }

    // --- upper / lower / proper ---------------------------------------------

    #[test]
    fn case_fns() {
        let mut c = TestCtx::new();
        assert_eq!(
            upper(&mut c, &[Value::Text("hello".into())]),
            Value::Text("HELLO".into())
        );
        assert_eq!(
            lower(&mut c, &[Value::Text("HELLO".into())]),
            Value::Text("hello".into())
        );
        assert_eq!(
            proper(&mut c, &[Value::Text("hello world".into())]),
            Value::Text("Hello World".into())
        );
    }

    #[test]
    fn proper_mixed() {
        let mut c = TestCtx::new();
        assert_eq!(
            proper(&mut c, &[Value::Text("HELLO WORLD".into())]),
            Value::Text("Hello World".into())
        );
        assert_eq!(
            proper(&mut c, &[Value::Text("it's a test".into())]),
            Value::Text("It'S A Test".into())
        );
    }

    // --- find ---------------------------------------------------------------

    #[test]
    fn find_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("e".into()), Value::Text("hello".into())]
            ),
            Value::Number(2.0)
        );
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("l".into()), Value::Text("hello".into())]
            ),
            Value::Number(3.0)
        );
    }

    #[test]
    fn find_case_sensitive() {
        let mut c = TestCtx::new();
        // FIND is case-sensitive — 'E' not in 'hello'
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("E".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn find_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            find(
                &mut c,
                &[Value::Text("z".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn find_with_start() {
        let mut c = TestCtx::new();
        // Find second 'l' starting from position 4
        assert_eq!(
            find(
                &mut c,
                &[
                    Value::Text("l".into()),
                    Value::Text("hello".into()),
                    Value::Number(4.0)
                ]
            ),
            Value::Number(4.0)
        );
    }

    // --- search -------------------------------------------------------------

    #[test]
    fn search_case_insensitive() {
        let mut c = TestCtx::new();
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("E".into()), Value::Text("hello".into())]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn search_wildcard() {
        let mut c = TestCtx::new();
        // "h*o" should match "hello" at position 1
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("h*o".into()), Value::Text("hello".into())]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn search_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            search(
                &mut c,
                &[Value::Text("z".into()), Value::Text("hello".into())]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- substitute ---------------------------------------------------------

    #[test]
    fn substitute_all() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Text("+".into()),
                ]
            ),
            Value::Text("a+b+c".into())
        );
    }

    #[test]
    fn substitute_instance() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Text("+".into()),
                    Value::Number(2.0),
                ]
            ),
            Value::Text("a-b+c".into())
        );
    }

    #[test]
    fn substitute_not_found() {
        let mut c = TestCtx::new();
        assert_eq!(
            substitute(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Text("z".into()),
                    Value::Text("x".into()),
                ]
            ),
            Value::Text("hello".into())
        );
    }

    // --- replace ------------------------------------------------------------

    #[test]
    fn replace_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("hello world".into()),
                    Value::Number(7.0),
                    Value::Number(5.0),
                    Value::Text("there".into()),
                ]
            ),
            Value::Text("hello there".into())
        );
    }

    #[test]
    fn replace_insert() {
        let mut c = TestCtx::new();
        // REPLACE with num_chars=0 inserts
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("abc".into()),
                    Value::Number(2.0),
                    Value::Number(0.0),
                    Value::Text("X".into()),
                ]
            ),
            Value::Text("aXbc".into())
        );
    }

    #[test]
    fn replace_start_1_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            replace(
                &mut c,
                &[
                    Value::Text("hello".into()),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    Value::Text("x".into()),
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- rept ---------------------------------------------------------------

    #[test]
    fn rept_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            rept(&mut c, &[Value::Text("ab".into()), Value::Number(3.0)]),
            Value::Text("ababab".into())
        );
        assert_eq!(
            rept(&mut c, &[Value::Text("x".into()), Value::Number(0.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            rept(&mut c, &[Value::Text("x".into()), Value::Number(-1.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- text_fn ------------------------------------------------------------

    #[test]
    fn text_fn_basic() {
        let mut c = TestCtx::new();
        // TEXT(1234.5, "#,##0.00") → "1,234.50"
        assert_eq!(
            text_fn(
                &mut c,
                &[Value::Number(1234.5), Value::Text("#,##0.00".into())]
            ),
            Value::Text("1,234.50".into())
        );
    }

    #[test]
    fn text_fn_text_passthrough() {
        let mut c = TestCtx::new();
        assert_eq!(
            text_fn(
                &mut c,
                &[Value::Text("hello".into()), Value::Text("@".into())]
            ),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn text_fn_zero() {
        let mut c = TestCtx::new();
        assert_eq!(
            text_fn(&mut c, &[Value::Number(0.0), Value::Text("0.00".into())]),
            Value::Text("0.00".into())
        );
    }

    // --- value_fn -----------------------------------------------------------

    #[test]
    fn value_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            value_fn(&mut c, &[Value::Text("9.88".into())]),
            Value::Number(9.88)
        );
        assert_eq!(
            value_fn(&mut c, &[Value::Text("1,234".into())]),
            Value::Number(1234.0)
        );
        assert_eq!(
            value_fn(&mut c, &[Value::Text("abc".into())]),
            Value::Error(CellError::Value)
        );
    }

    // --- numbervalue --------------------------------------------------------

    #[test]
    fn numbervalue_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[
                    Value::Text("1.234,56".into()),
                    Value::Text(",".into()),
                    Value::Text(".".into()),
                ]
            ),
            Value::Number(1234.56)
        );
    }

    #[test]
    fn numbervalue_percent() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[Value::Text("50%".into()), Value::Text(".".into()),]
            ),
            Value::Number(0.5)
        );
    }

    #[test]
    fn numbervalue_same_sep_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            numbervalue(
                &mut c,
                &[
                    Value::Text("1.234".into()),
                    Value::Text(".".into()),
                    Value::Text(".".into()),
                ]
            ),
            Value::Error(CellError::Value)
        );
    }

    // --- fixed --------------------------------------------------------------

    #[test]
    fn fixed_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            fixed(
                &mut c,
                &[
                    Value::Number(1234.5),
                    Value::Number(2.0),
                    Value::Bool(false)
                ]
            ),
            Value::Text("1,234.50".into())
        );
        assert_eq!(
            fixed(
                &mut c,
                &[Value::Number(1234.5), Value::Number(2.0), Value::Bool(true)]
            ),
            Value::Text("1234.50".into())
        );
        assert_eq!(
            fixed(&mut c, &[Value::Number(-1234.5), Value::Number(1.0)]),
            Value::Text("-1,234.5".into())
        );
    }

    // --- dollar -------------------------------------------------------------

    #[test]
    fn dollar_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            dollar(&mut c, &[Value::Number(1234.567)]),
            Value::Text("$1,234.57".into())
        );
        assert_eq!(
            dollar(&mut c, &[Value::Number(-1234.567)]),
            Value::Text("-$1,234.57".into())
        );
        assert_eq!(
            dollar(&mut c, &[Value::Number(1234.0), Value::Number(0.0)]),
            Value::Text("$1,234".into())
        );
    }

    // --- exact --------------------------------------------------------------

    #[test]
    fn exact_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("hello".into()), Value::Text("hello".into())]
            ),
            Value::Bool(true)
        );
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("Hello".into()), Value::Text("hello".into())]
            ),
            Value::Bool(false)
        );
        assert_eq!(
            exact(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("abc".into())]
            ),
            Value::Bool(true)
        );
    }

    // --- char_fn / code_fn --------------------------------------------------

    #[test]
    fn char_code_roundtrip() {
        let mut c = TestCtx::new();
        assert_eq!(
            char_fn(&mut c, &[Value::Number(65.0)]),
            Value::Text("A".into())
        );
        assert_eq!(
            code_fn(&mut c, &[Value::Text("A".into())]),
            Value::Number(65.0)
        );
        assert_eq!(
            char_fn(&mut c, &[Value::Number(0.0)]),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn char_fn_range() {
        let mut c = TestCtx::new();
        assert_eq!(
            char_fn(&mut c, &[Value::Number(32.0)]),
            Value::Text(" ".into())
        );
        assert_eq!(
            char_fn(&mut c, &[Value::Number(256.0)]),
            Value::Error(CellError::Value)
        );
    }

    // --- unichar / unicode --------------------------------------------------

    #[test]
    fn unichar_unicode_roundtrip() {
        let mut c = TestCtx::new();
        assert_eq!(
            unichar(&mut c, &[Value::Number(0x1F600 as f64)]),
            Value::Text("\u{1F600}".into())
        );
        assert_eq!(
            unicode(&mut c, &[Value::Text("\u{1F600}".into())]),
            Value::Number(0x1F600 as f64)
        );
    }

    #[test]
    fn unicode_empty_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            unicode(&mut c, &[Value::Text(String::new())]),
            Value::Error(CellError::Value)
        );
    }

    // --- clean --------------------------------------------------------------

    #[test]
    fn clean_basic() {
        let mut c = TestCtx::new();
        let s = "hel\x01lo\x1Fworld";
        assert_eq!(
            clean(&mut c, &[Value::Text(s.into())]),
            Value::Text("helloworld".into())
        );
        assert_eq!(
            clean(&mut c, &[Value::Text("normal".into())]),
            Value::Text("normal".into())
        );
        assert_eq!(
            clean(&mut c, &[Value::Text(String::new())]),
            Value::Text(String::new())
        );
    }

    // --- t_fn ---------------------------------------------------------------

    #[test]
    fn t_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            t_fn(&mut c, &[Value::Text("hello".into())]),
            Value::Text("hello".into())
        );
        assert_eq!(
            t_fn(&mut c, &[Value::Number(42.0)]),
            Value::Text(String::new())
        );
        assert_eq!(
            t_fn(&mut c, &[Value::Bool(true)]),
            Value::Text(String::new())
        );
    }

    // --- textbefore ---------------------------------------------------------

    #[test]
    fn textbefore_first_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[Value::Text("a-b-c".into()), Value::Text("-".into())]
            ),
            Value::Text("a".into())
        );
    }

    #[test]
    fn textbefore_second_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("a-b".into())
        );
    }

    #[test]
    fn textbefore_negative_instance() {
        let mut c = TestCtx::new();
        // instance -1 = last delimiter → before the last "-" → "a-b"
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(-1.0)
                ]
            ),
            Value::Text("a-b".into())
        );
    }

    #[test]
    fn textbefore_not_found_returns_na() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("x".into())]
            ),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn textbefore_not_found_if_not_found_arg() {
        let mut c = TestCtx::new();
        // 6 args: text, delim, instance, match_mode, match_end, if_not_found
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("abc".into()),
                    Value::Text("x".into()),
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Empty,
                    Value::Text("N/A".into()),
                ]
            ),
            Value::Text("N/A".into())
        );
    }

    #[test]
    fn textbefore_case_insensitive() {
        let mut c = TestCtx::new();
        assert_eq!(
            textbefore(
                &mut c,
                &[
                    Value::Text("helloXworld".into()),
                    Value::Text("x".into()),
                    Value::Number(1.0),
                    Value::Number(1.0), // case-insensitive
                ]
            ),
            Value::Text("hello".into())
        );
    }

    // --- textafter ----------------------------------------------------------

    #[test]
    fn textafter_first_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[Value::Text("a-b-c".into()), Value::Text("-".into())]
            ),
            Value::Text("b-c".into())
        );
    }

    #[test]
    fn textafter_second_occurrence() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(2.0)
                ]
            ),
            Value::Text("c".into())
        );
    }

    #[test]
    fn textafter_negative_instance() {
        let mut c = TestCtx::new();
        // instance -2 = second-to-last delimiter → after first "-" → "b-c"
        assert_eq!(
            textafter(
                &mut c,
                &[
                    Value::Text("a-b-c".into()),
                    Value::Text("-".into()),
                    Value::Number(-2.0)
                ]
            ),
            Value::Text("b-c".into())
        );
    }

    #[test]
    fn textafter_not_found_returns_na() {
        let mut c = TestCtx::new();
        assert_eq!(
            textafter(
                &mut c,
                &[Value::Text("abc".into()), Value::Text("x".into())]
            ),
            Value::Error(CellError::NA)
        );
    }

    // --- valuetotext --------------------------------------------------------

    #[test]
    fn valuetotext_number() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Number(42.0)]),
            Value::Text("42".into())
        );
    }

    #[test]
    fn valuetotext_text_concise() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Text("hello".into()), Value::Number(0.0)]),
            Value::Text("hello".into())
        );
    }

    #[test]
    fn valuetotext_text_strict() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Text("hello".into()), Value::Number(1.0)]),
            Value::Text("\"hello\"".into())
        );
    }

    #[test]
    fn valuetotext_bool() {
        let mut c = TestCtx::new();
        assert_eq!(
            valuetotext(&mut c, &[Value::Bool(true)]),
            Value::Text("TRUE".into())
        );
        assert_eq!(
            valuetotext(&mut c, &[Value::Bool(false)]),
            Value::Text("FALSE".into())
        );
    }

    // --- arraytotext --------------------------------------------------------

    #[test]
    fn arraytotext_concise() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
        ]));
        assert_eq!(
            arraytotext(&mut c, &[arr, Value::Number(0.0)]),
            Value::Text("1, 2, 3, 4".into())
        );
    }

    #[test]
    fn arraytotext_strict() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Text("a".into()), Value::Number(1.0)],
            vec![Value::Text("b".into()), Value::Number(2.0)],
        ]));
        assert_eq!(
            arraytotext(&mut c, &[arr, Value::Number(1.0)]),
            Value::Text("{\"a\",1;\"b\",2}".into())
        );
    }

    #[test]
    fn arraytotext_scalar_fallback() {
        let mut c = TestCtx::new();
        assert_eq!(
            arraytotext(&mut c, &[Value::Number(99.0)]),
            Value::Text("99".into())
        );
    }
}
