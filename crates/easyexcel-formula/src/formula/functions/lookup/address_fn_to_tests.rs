fn address_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let row = match to_number(&args[0]) {
        Ok(n) => n as u32,
        Err(e) => return Value::Error(e),
    };
    let col = match to_number(&args[1]) {
        Ok(n) => n as u32,
        Err(e) => return Value::Error(e),
    };
    let abs_num = if args.len() >= 3 {
        match to_number(&args[2]) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let _a1_style = if args.len() >= 4 {
        match crate::formula::coerce::to_bool(&args[3]) {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true
    };
    let sheet_text = if args.len() >= 5 {
        match to_text(&args[4]) {
            Ok(s) if !s.is_empty() => Some(s),
            Ok(_) => None,
            Err(e) => return Value::Error(e),
        }
    } else {
        None
    };

    if row == 0 || col == 0 {
        return Value::Error(CellError::Value);
    }

    let col_letters = col_index_to_letters(col - 1);
    let addr = match abs_num {
        1 => format!("${col_letters}${row}"), // $A$1
        2 => format!("{col_letters}${row}"),  // A$1  (col relative, row abs)
        3 => format!("${col_letters}${row}")
            .replace(&format!("${col_letters}$"), &format!("${col_letters}")), // $A1 — abs col, rel row
        4 => format!("{col_letters}{row}"), // A1
        _ => return Value::Error(CellError::Value),
    };
    // Fix abs_num=3: $col_rel_row
    let addr = match abs_num {
        3 => format!("${col_letters}{row}"),
        _ => addr,
    };

    let result = if let Some(sheet) = sheet_text {
        format!("{sheet}!{addr}")
    } else {
        addr
    };
    Value::Text(result)
}

// ---------------------------------------------------------------------------
// TRANSPOSE
// ---------------------------------------------------------------------------

fn transpose_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let rows = arr.rows;
    let cols = arr.cols;
    let mut data = vec![Value::Empty; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            data[c * rows + r] = arr.get(r, c).cloned().unwrap_or(Value::Empty);
        }
    }
    Value::Array(Array {
        rows: cols,
        cols: rows,
        data,
    })
}

// ---------------------------------------------------------------------------
// FORMULATEXT — PARITY: cannot retrieve formula text from Value; returns #N/A
// ---------------------------------------------------------------------------

fn formulatext_fn(_ctx: &mut dyn Context, _args: &[Value]) -> Value {
    // PARITY: formula text is not available through the Context trait.
    Value::Error(CellError::NA)
}

// ---------------------------------------------------------------------------
// HYPERLINK — return the friendly name (or the link if name omitted)
// ---------------------------------------------------------------------------

fn hyperlink_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.len() >= 2 {
        scalar(ctx, &args[1])
    } else {
        match to_text(&scalar(ctx, &args[0])) {
            Ok(s) => Value::Text(s),
            Err(e) => Value::Error(e),
        }
    }
}

// ---------------------------------------------------------------------------
// XLOOKUP
// ---------------------------------------------------------------------------

fn xlookup(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // XLOOKUP(lookup_value, lookup_array, return_array [, if_not_found [, match_mode [, search_mode]]])
    let lookup_val = scalar(ctx, &args[0]);
    let lookup_arr = to_array(ctx, &args[1]);
    let return_arr = to_array(ctx, &args[2]);
    let if_not_found = if args.len() >= 4 {
        Some(scalar(ctx, &args[3]))
    } else {
        None
    };
    let match_mode = if args.len() >= 5 {
        match to_number(&scalar(ctx, &args[4])) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };
    // search_mode: 1 (default first→last), -1 (last→first), 2 (binary asc), -2 (binary desc)
    let search_mode = if args.len() >= 6 {
        match to_number(&scalar(ctx, &args[5])) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };

    let items = &lookup_arr.data;

    let find_idx = |items: &[Value]| -> Option<usize> {
        let iter: Box<dyn Iterator<Item = (usize, &Value)>> = if search_mode == -1 {
            Box::new(items.iter().enumerate().rev())
        } else {
            Box::new(items.iter().enumerate())
        };

        match match_mode {
            0 => {
                // exact
                for (i, cell) in iter {
                    if exact_match_xlookup(&lookup_val, cell) {
                        return Some(i);
                    }
                }
                None
            }
            -1 => {
                // exact or next smaller
                let mut best: Option<usize> = None;
                for (i, cell) in items.iter().enumerate() {
                    if exact_match_xlookup(&lookup_val, cell) {
                        return Some(i);
                    }
                    if compare(cell, &lookup_val) == std::cmp::Ordering::Less {
                        best = Some(i);
                    }
                }
                best
            }
            1 => {
                // exact or next larger
                let mut best: Option<usize> = None;
                for (i, cell) in items.iter().enumerate().rev() {
                    if exact_match_xlookup(&lookup_val, cell) {
                        return Some(i);
                    }
                    if compare(cell, &lookup_val) == std::cmp::Ordering::Greater {
                        best = Some(i);
                    }
                }
                best
            }
            2 => {
                // wildcard match
                let pat = match &lookup_val {
                    Value::Text(s) => s.to_lowercase(),
                    _ => String::new(),
                };
                for (i, cell) in items.iter().enumerate() {
                    if let Value::Text(s) = cell
                        && super::wildcard_match(&pat, &s.to_lowercase())
                    {
                        return Some(i);
                    }
                }
                None
            }
            _ => None,
        }
    };

    match find_idx(items) {
        Some(i) => return_arr.data.get(i).cloned().unwrap_or(Value::Empty),
        None => match if_not_found {
            Some(v) => v,
            None => Value::Error(CellError::NA),
        },
    }
}

fn exact_match_xlookup(a: &Value, b: &Value) -> bool {
    exact_match(a, b)
}

// ---------------------------------------------------------------------------
// XMATCH
// ---------------------------------------------------------------------------

fn xmatch(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let lookup_val = scalar(ctx, &args[0]);
    let lookup_arr = to_array(ctx, &args[1]);
    let match_mode = if args.len() >= 3 {
        match to_number(&scalar(ctx, &args[2])) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };
    let search_mode = if args.len() >= 4 {
        match to_number(&scalar(ctx, &args[3])) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };

    let items = &lookup_arr.data;

    let iter_fwd: Vec<(usize, &Value)> = items.iter().enumerate().collect();
    let iter_rev: Vec<(usize, &Value)> = items.iter().enumerate().rev().collect();
    let iter: &[(usize, &Value)] = if search_mode == -1 {
        &iter_rev
    } else {
        &iter_fwd
    };

    let idx = match match_mode {
        0 => {
            let pat = match &lookup_val {
                Value::Text(s) => s.to_lowercase(),
                _ => String::new(),
            };
            let is_text = matches!(lookup_val, Value::Text(_));
            iter.iter()
                .find(|(_, cell)| {
                    if is_text {
                        if let Value::Text(s) = cell {
                            super::wildcard_match(&pat, &s.to_lowercase())
                        } else {
                            false
                        }
                    } else {
                        exact_match(&lookup_val, cell)
                    }
                })
                .map(|(i, _)| *i)
        }
        -1 => {
            let mut best: Option<usize> = None;
            for (i, cell) in items.iter().enumerate() {
                if exact_match(&lookup_val, cell) {
                    return Value::Number((i + 1) as f64);
                }
                if compare(cell, &lookup_val) == std::cmp::Ordering::Less {
                    best = Some(i);
                }
            }
            best
        }
        1 => {
            let mut best: Option<usize> = None;
            for (i, cell) in items.iter().enumerate().rev() {
                if exact_match(&lookup_val, cell) {
                    return Value::Number((i + 1) as f64);
                }
                if compare(cell, &lookup_val) == std::cmp::Ordering::Greater {
                    best = Some(i);
                }
            }
            best
        }
        2 => {
            let pat = match &lookup_val {
                Value::Text(s) => s.to_lowercase(),
                _ => String::new(),
            };
            iter.iter()
                .find(|(_, cell)| {
                    if let Value::Text(s) = cell {
                        super::wildcard_match(&pat, &s.to_lowercase())
                    } else {
                        false
                    }
                })
                .map(|(i, _)| *i)
        }
        _ => return Value::Error(CellError::Value),
    };

    match idx {
        Some(i) => Value::Number((i + 1) as f64),
        None => Value::Error(CellError::NA),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../lookup_tests/tests.rs"]
mod tests;
