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
                    easyexcel_model::numfmt::format_value(0.0, &format_code, ctx.date_system());
                Value::Text(formatted)
            }
        }
        other => match to_number(other) {
            Ok(n) => {
                let formatted =
                    easyexcel_model::numfmt::format_value(n, &format_code, ctx.date_system());
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
        Value::Text(s) => match crate::formula::coerce::parse_number_text(s) {
            Some(n) => Value::Number(n),
            None => Value::Error(CellError::Value),
        },
        _ => Value::Error(CellError::Value),
    }
}

/// NUMBERVALUE(text, `decimal_separator`, [`group_separator`])
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

/// FIXED(number, [decimals], [`no_commas`])
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
    let int_with_commas = if no_commas {
        int_str
    } else {
        add_thousands_commas(&int_str)
    };

    let sign = if is_neg { "-" } else { "" };
    format!("{sign}{int_with_commas}{dec_str}")
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
        Value::Text(format!("${formatted}"))
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
            Some(c) => Value::Number(f64::from(c as u32)),
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
            Some(c) => Value::Number(f64::from(c as u32)),
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

/// TEXTBEFORE(text, delimiter, [`instance_num`], [`match_mode`], [`match_end`], [`if_not_found`])
///
/// Returns the substring before the Nth occurrence of `delimiter`.
/// `instance_num` defaults to 1; negative values count from the end.
/// `match_mode` 1 = case-insensitive (default 0 = case-sensitive).
/// PARITY: `match_end` and `if_not_found` beyond a plain scalar are simplified here.
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

/// TEXTAFTER(text, delimiter, [`instance_num`], [`match_mode`], [`match_end`], [`if_not_found`])
///
/// Returns the substring after the Nth occurrence of `delimiter`.
/// PARITY: `match_end` and `if_not_found` beyond a plain scalar are simplified here.
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
        if let Some((ms, ml)) = best {
            out.push(s[start..ms].to_string());
            pos = ms + ml;
            start = pos;
        } else {
            out.push(s[start..].to_string());
            break;
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
    let ignore_empty =
        matches!(args.get(3), Some(v) if matches!(crate::formula::coerce::to_bool(v), Ok(true)));
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
    let ncols = rows.iter().map(std::vec::Vec::len).max().unwrap_or(0);
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
        Value::Number(n) => easyexcel_model::value::format_number_general(*n),
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

