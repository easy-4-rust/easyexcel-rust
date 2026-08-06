/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
            Value::Array(crate::formula::value::Array::new(n, 1, matches))
        }
        2 => match re.captures(&text) {
            Some(caps) => {
                let groups: Vec<Value> = caps
                    .iter()
                    .skip(1)
                    .map(|g| g.map_or(Value::Empty, |m| Value::Text(m.as_str().to_string())))
                    .collect();
                if groups.is_empty() {
                    Value::Text(
                        caps.get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    )
                } else {
                    let n = groups.len();
                    Value::Array(crate::formula::value::Array::new(1, n, groups))
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
    Value::Text(format!("{before}{new_text}{after}"))
}

// ---------------------------------------------------------------------------
// REPT
// ---------------------------------------------------------------------------

