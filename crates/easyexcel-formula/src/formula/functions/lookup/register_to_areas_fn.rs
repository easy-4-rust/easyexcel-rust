/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    r.add("VLOOKUP", 3, 4, false, vlookup);
    r.add("HLOOKUP", 3, 4, false, hlookup);
    r.add("LOOKUP", 2, 3, false, lookup);
    r.add("INDEX", 2, 4, false, index_fn);
    r.add("MATCH", 2, 3, false, match_fn);
    r.add("OFFSET", 3, 5, true, offset);
    r.add("INDIRECT", 1, 2, true, indirect);
    r.add("ROW", 0, 1, false, row_fn);
    r.add("ROWS", 1, 1, false, rows_fn);
    r.add("COLUMN", 0, 1, false, column_fn);
    r.add("COLUMNS", 1, 1, false, columns_fn);
    r.add("AREAS", 1, 1, false, areas_fn);
    r.add("ADDRESS", 2, 5, false, address_fn);
    r.add("TRANSPOSE", 1, 1, false, transpose_fn);
    // CHOOSE is a special form; do NOT register here.
    r.add("FORMULATEXT", 1, 1, false, formulatext_fn);
    r.add("HYPERLINK", 1, 2, false, hyperlink_fn);
    r.add("XLOOKUP", 3, 6, false, xlookup);
    r.add("XMATCH", 2, 4, false, xmatch);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get a single scalar from an arg, de-referencing a single-cell Ref.
fn scalar(ctx: &mut dyn Context, v: &Value) -> Value {
    match v {
        Value::Ref(r) if r.is_single() => ctx.cell(r.sheet, r.start_row, r.start_col),
        Value::Ref(r) => {
            // multi-cell ref → take top-left
            ctx.cell(r.sheet, r.start_row, r.start_col)
        }
        Value::Array(a) => a.data.first().cloned().unwrap_or(Value::Empty),
        other => other.clone(),
    }
}

/// Materialise a range argument as an Array.
fn to_array(ctx: &mut dyn Context, v: &Value) -> Array {
    match v {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => Array::scalar(other.clone()),
    }
}

/// Refrange geometry for a `Value::Ref` argument (returns None if not a Ref).
fn as_ref(v: &Value) -> Option<RefRange> {
    match v {
        Value::Ref(r) => Some(*r),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// VLOOKUP
// ---------------------------------------------------------------------------

fn vlookup(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let lookup_val = scalar(ctx, &args[0]);
    let table = to_array(ctx, &args[1]);
    let col_index = match to_number(&scalar(ctx, &args[2])) {
        Ok(n) => n as usize,
        Err(e) => return Value::Error(e),
    };
    let approx = if args.len() >= 4 {
        match crate::formula::coerce::to_bool(&scalar(ctx, &args[3])) {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true
    };

    if col_index < 1 || col_index > table.cols {
        return Value::Error(CellError::Value);
    }

    let rows = table.rows;

    if approx {
        // sorted ascending; find last row where first col <= lookup
        let mut result_row: Option<usize> = None;
        for r in 0..rows {
            let cell = table.get(r, 0).cloned().unwrap_or(Value::Empty);
            match compare(&cell, &lookup_val) {
                std::cmp::Ordering::Greater => break,
                _ => result_row = Some(r),
            }
        }
        match result_row {
            Some(r) => table.get(r, col_index - 1).cloned().unwrap_or(Value::Empty),
            None => Value::Error(CellError::NA),
        }
    } else {
        // exact match (with wildcard support for text lookups)
        for r in 0..rows {
            let cell = table.get(r, 0).cloned().unwrap_or(Value::Empty);
            let matched = vlookup_exact_match(&lookup_val, &cell);
            if matched {
                return table.get(r, col_index - 1).cloned().unwrap_or(Value::Empty);
            }
        }
        Value::Error(CellError::NA)
    }
}

fn vlookup_exact_match(lookup_val: &Value, cell: &Value) -> bool {
    match lookup_val {
        Value::Text(pat) => {
            if let Value::Text(cell_text) = cell {
                super::wildcard_match(&pat.to_lowercase(), &cell_text.to_lowercase())
            } else {
                false
            }
        }
        _ => exact_match(lookup_val, cell),
    }
}

fn exact_match(a: &Value, b: &Value) -> bool {
    // numbers: numeric equality; text: case-insensitive; bool/bool exact
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Text(x), Value::Text(y)) => x.eq_ignore_ascii_case(y),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// HLOOKUP
// ---------------------------------------------------------------------------

fn hlookup(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let lookup_val = scalar(ctx, &args[0]);
    let table = to_array(ctx, &args[1]);
    let row_index = match to_number(&scalar(ctx, &args[2])) {
        Ok(n) => n as usize,
        Err(e) => return Value::Error(e),
    };
    let approx = if args.len() >= 4 {
        match crate::formula::coerce::to_bool(&scalar(ctx, &args[3])) {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true
    };

    if row_index < 1 || row_index > table.rows {
        return Value::Error(CellError::Value);
    }

    if approx {
        let mut result_col: Option<usize> = None;
        for c in 0..table.cols {
            let cell = table.get(0, c).cloned().unwrap_or(Value::Empty);
            match compare(&cell, &lookup_val) {
                std::cmp::Ordering::Greater => break,
                _ => result_col = Some(c),
            }
        }
        match result_col {
            Some(c) => table.get(row_index - 1, c).cloned().unwrap_or(Value::Empty),
            None => Value::Error(CellError::NA),
        }
    } else {
        for c in 0..table.cols {
            let cell = table.get(0, c).cloned().unwrap_or(Value::Empty);
            if vlookup_exact_match(&lookup_val, &cell) {
                return table.get(row_index - 1, c).cloned().unwrap_or(Value::Empty);
            }
        }
        Value::Error(CellError::NA)
    }
}

// ---------------------------------------------------------------------------
// LOOKUP (vector form + array form)
// ---------------------------------------------------------------------------

fn lookup(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let lookup_val = scalar(ctx, &args[0]);
    let lookup_vec = to_array(ctx, &args[1]);

    let result_vec = if args.len() == 3 {
        Some(to_array(ctx, &args[2]))
    } else {
        None
    };

    // Array form: if lookup_vec has more rows than cols, search first col → return last col;
    // else search first row → return last row.
    let (search_items, result_idx_fn): (Vec<Value>, Box<dyn Fn(usize) -> Value>) =
        if let Some(rv) = result_vec {
            // vector form
            let items: Vec<Value> = lookup_vec.data.clone();
            (
                items,
                Box::new(move |i| rv.data.get(i).cloned().unwrap_or(Value::Empty)),
            )
        } else if lookup_vec.rows >= lookup_vec.cols {
            // array form: search first col → return last col
            let items: Vec<Value> = (0..lookup_vec.rows)
                .map(|r| lookup_vec.get(r, 0).cloned().unwrap_or(Value::Empty))
                .collect();
            let last_col = lookup_vec.cols.saturating_sub(1);
            let lv = lookup_vec.clone();
            (
                items,
                Box::new(move |r| lv.get(r, last_col).cloned().unwrap_or(Value::Empty)),
            )
        } else {
            // array form: search first row → return last row
            let items: Vec<Value> = (0..lookup_vec.cols)
                .map(|c| lookup_vec.get(0, c).cloned().unwrap_or(Value::Empty))
                .collect();
            let last_row = lookup_vec.rows.saturating_sub(1);
            let lv = lookup_vec.clone();
            (
                items,
                Box::new(move |c| lv.get(last_row, c).cloned().unwrap_or(Value::Empty)),
            )
        };

    // Binary / linear search (sorted ascending, return largest <= lookup)
    let mut result_idx: Option<usize> = None;
    for (i, cell) in search_items.iter().enumerate() {
        match compare(cell, &lookup_val) {
            std::cmp::Ordering::Greater => break,
            _ => result_idx = Some(i),
        }
    }
    match result_idx {
        Some(i) => result_idx_fn(i),
        None => Value::Error(CellError::NA),
    }
}

// ---------------------------------------------------------------------------
// INDEX
// ---------------------------------------------------------------------------

fn index_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // INDEX(array, row_num [, col_num [, area_num]])
    // PARITY: area_num for multi-area references is not supported; ignored.
    let arr = to_array(ctx, &args[0]);
    let ref_arg = as_ref(&args[0]);

    let row_num = match to_number(&scalar(ctx, &args[1])) {
        Ok(n) => n as usize,
        Err(e) => return Value::Error(e),
    };
    let col_num = if args.len() >= 3 {
        match to_number(&scalar(ctx, &args[2])) {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let rows = arr.rows;
    let cols = arr.cols;

    // row 0 means all rows; col 0 means all cols
    let r = if row_num == 0 {
        None
    } else if row_num <= rows {
        Some(row_num - 1)
    } else {
        return Value::Error(CellError::Ref);
    };
    let c = if col_num == 0 {
        None
    } else if col_num <= cols {
        Some(col_num - 1)
    } else {
        return Value::Error(CellError::Ref);
    };

    match (r, c) {
        (Some(ri), Some(ci)) => {
            // single cell — return a Ref if we have one, otherwise the value
            if let Some(rr) = ref_arg {
                Value::Ref(RefRange::single(
                    rr.sheet,
                    rr.start_row + ri as u32,
                    rr.start_col + ci as u32,
                ))
            } else {
                arr.get(ri, ci).cloned().unwrap_or(Value::Empty)
            }
        }
        (Some(ri), None) => {
            // whole row
            if let Some(rr) = ref_arg {
                Value::Ref(RefRange {
                    sheet: rr.sheet,
                    start_row: rr.start_row + ri as u32,
                    end_row: rr.start_row + ri as u32,
                    start_col: rr.start_col,
                    end_col: rr.end_col,
                })
            } else {
                let data: Vec<Value> = (0..cols)
                    .map(|c| arr.get(ri, c).cloned().unwrap_or(Value::Empty))
                    .collect();
                Value::Array(Array {
                    rows: 1,
                    cols,
                    data,
                })
            }
        }
        (None, Some(ci)) => {
            // whole column
            if let Some(rr) = ref_arg {
                Value::Ref(RefRange {
                    sheet: rr.sheet,
                    start_row: rr.start_row,
                    end_row: rr.end_row,
                    start_col: rr.start_col + ci as u32,
                    end_col: rr.start_col + ci as u32,
                })
            } else {
                let data: Vec<Value> = (0..rows)
                    .map(|r| arr.get(r, ci).cloned().unwrap_or(Value::Empty))
                    .collect();
                Value::Array(Array {
                    rows,
                    cols: 1,
                    data,
                })
            }
        }
        (None, None) => {
            // entire array/range
            if let Some(rr) = ref_arg {
                Value::Ref(rr)
            } else {
                Value::Array(arr)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MATCH
// ---------------------------------------------------------------------------

fn match_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let lookup_val = scalar(ctx, &args[0]);
    let lookup_arr = to_array(ctx, &args[1]);
    let match_type = if args.len() >= 3 {
        match to_number(&scalar(ctx, &args[2])) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };

    // Flatten to 1D list
    let items: Vec<Value> = lookup_arr.data.clone();
    match match_type {
        0 => {
            // Exact match (wildcards for text)
            let pat = match &lookup_val {
                Value::Text(s) => s.to_lowercase(),
                _ => String::new(),
            };
            let is_text_pattern = matches!(lookup_val, Value::Text(_));
            for (i, cell) in items.iter().enumerate() {
                let matched = if is_text_pattern {
                    if let Value::Text(s) = cell {
                        super::wildcard_match(&pat, &s.to_lowercase())
                    } else {
                        false
                    }
                } else {
                    exact_match(&lookup_val, cell)
                };
                if matched {
                    return Value::Number((i + 1) as f64);
                }
            }
            Value::Error(CellError::NA)
        }
        1 => {
            // sorted ascending; largest <= lookup
            let mut result: Option<usize> = None;
            for (i, cell) in items.iter().enumerate() {
                match compare(cell, &lookup_val) {
                    std::cmp::Ordering::Greater => break,
                    _ => result = Some(i),
                }
            }
            match result {
                Some(i) => Value::Number((i + 1) as f64),
                None => Value::Error(CellError::NA),
            }
        }
        -1 => {
            // sorted descending; smallest >= lookup
            let mut result: Option<usize> = None;
            for (i, cell) in items.iter().enumerate() {
                match compare(cell, &lookup_val) {
                    std::cmp::Ordering::Less => break,
                    _ => result = Some(i),
                }
            }
            match result {
                Some(i) => Value::Number((i + 1) as f64),
                None => Value::Error(CellError::NA),
            }
        }
        _ => Value::Error(CellError::Value),
    }
}

// ---------------------------------------------------------------------------
// OFFSET (volatile)
// ---------------------------------------------------------------------------

fn offset(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let base_ref = match as_ref(&args[0]) {
        Some(r) => r,
        None => return Value::Error(CellError::Value),
    };

    let row_off = match to_number(&scalar(ctx, &args[1])) {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };
    let col_off = match to_number(&scalar(ctx, &args[2])) {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };
    let height = if args.len() >= 4 {
        match to_number(&scalar(ctx, &args[3])) {
            Ok(n) => n as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        i64::from(base_ref.rows())
    };
    let width = if args.len() >= 5 {
        match to_number(&scalar(ctx, &args[4])) {
            Ok(n) => n as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        i64::from(base_ref.cols())
    };

    if height <= 0 || width <= 0 {
        return Value::Error(CellError::Value);
    }

    let new_row = i64::from(base_ref.start_row) + row_off;
    let new_col = i64::from(base_ref.start_col) + col_off;
    if new_row < 0 || new_col < 0 {
        return Value::Error(CellError::Ref);
    }

    let start_row = new_row as u32;
    let start_col = new_col as u32;
    let end_row = start_row + height as u32 - 1;
    let end_col = start_col + width as u32 - 1;

    Value::Ref(RefRange {
        sheet: base_ref.sheet,
        start_row,
        start_col,
        end_row,
        end_col,
    })
}

// ---------------------------------------------------------------------------
// INDIRECT (volatile)
// ---------------------------------------------------------------------------

fn indirect(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let ref_text = match to_text(&scalar(ctx, &args[0])) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    let a1_style = if args.len() >= 2 {
        match crate::formula::coerce::to_bool(&scalar(ctx, &args[1])) {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        true
    };

    // Split into optional sheet name and cell/range part.
    let (sheet_name, cell_part) = if let Some(pos) = ref_text.rfind('!') {
        let sn = ref_text[..pos].trim_matches('\'').to_string();
        let cp = ref_text[pos + 1..].to_string();
        (sn, cp)
    } else {
        let sn = ctx.sheet_name(ctx.current().sheet).unwrap_or_default();
        (sn, ref_text.clone())
    };

    let sheet_idx = match ctx.sheet_index(&sheet_name) {
        Some(i) => i,
        None => return Value::Error(CellError::Ref),
    };

    if a1_style {
        if let Some(cr) = easyexcel_model::addr::CellRange::parse_a1(&cell_part) {
            return Value::Ref(RefRange {
                sheet: sheet_idx,
                start_row: cr.start.row,
                start_col: cr.start.col,
                end_row: cr.end.row,
                end_col: cr.end.col,
            });
        }
    } else {
        let base = CellAddress::new(0, 0);
        if let Some(start) = CellAddress::parse_r1c1(&cell_part, base) {
            return Value::Ref(RefRange::single(sheet_idx, start.row, start.col));
        }
        // Try range R1C1:R2C2
        if let Some(colon_pos) = cell_part.find(':') {
            let left = &cell_part[..colon_pos];
            let right = &cell_part[colon_pos + 1..];
            if let (Some(s), Some(e)) = (
                CellAddress::parse_r1c1(left, base),
                CellAddress::parse_r1c1(right, base),
            ) {
                return Value::Ref(RefRange {
                    sheet: sheet_idx,
                    start_row: s.row.min(e.row),
                    start_col: s.col.min(e.col),
                    end_row: s.row.max(e.row),
                    end_col: s.col.max(e.col),
                });
            }
        }
    }

    Value::Error(CellError::Ref)
}

// ---------------------------------------------------------------------------
// ROW / ROWS / COLUMN / COLUMNS
// ---------------------------------------------------------------------------

fn row_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Number(f64::from(ctx.current().row + 1));
    }
    match &args[0] {
        Value::Ref(r) => {
            if r.is_single() {
                Value::Number(f64::from(r.start_row + 1))
            } else {
                // return vertical array of row numbers
                let data: Vec<Value> = (r.start_row..=r.end_row)
                    .map(|row| Value::Number(f64::from(row + 1)))
                    .collect();
                let rows = data.len();
                Value::Array(Array {
                    rows,
                    cols: 1,
                    data,
                })
            }
        }
        other => match to_number(other) {
            Ok(_) => Value::Error(CellError::Value),
            Err(e) => Value::Error(e),
        },
    }
}

fn rows_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Ref(r) => Value::Number(f64::from(r.rows())),
        Value::Array(a) => Value::Number(a.rows as f64),
        _ => Value::Number(1.0),
    }
}

fn column_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Number(f64::from(ctx.current().col + 1));
    }
    match &args[0] {
        Value::Ref(r) => {
            if r.is_single() {
                Value::Number(f64::from(r.start_col + 1))
            } else {
                // return horizontal array of column numbers
                let data: Vec<Value> = (r.start_col..=r.end_col)
                    .map(|col| Value::Number(f64::from(col + 1)))
                    .collect();
                let cols = data.len();
                Value::Array(Array {
                    rows: 1,
                    cols,
                    data,
                })
            }
        }
        other => match to_number(other) {
            Ok(_) => Value::Error(CellError::Value),
            Err(e) => Value::Error(e),
        },
    }
}

fn columns_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Ref(r) => Value::Number(f64::from(r.cols())),
        Value::Array(a) => Value::Number(a.cols as f64),
        _ => Value::Number(1.0),
    }
}

// ---------------------------------------------------------------------------
// AREAS — PARITY: single-area references only; always returns 1
// ---------------------------------------------------------------------------

fn areas_fn(_ctx: &mut dyn Context, _args: &[Value]) -> Value {
    // PARITY: multi-area references aren't modelled; always 1.
    Value::Number(1.0)
}

// ---------------------------------------------------------------------------
// ADDRESS
// ---------------------------------------------------------------------------

