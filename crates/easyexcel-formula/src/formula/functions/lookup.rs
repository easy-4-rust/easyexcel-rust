//! Lookup & Reference functions.

use super::Registry;
use crate::formula::coerce::{compare, to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::{Array, RefRange, Value};
use easyexcel_model::addr::{CellAddress, col_index_to_letters};
use easyexcel_model::error::CellError;

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

/// Refrange geometry for a Value::Ref argument (returns None if not a Ref).
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
        base_ref.rows() as i64
    };
    let width = if args.len() >= 5 {
        match to_number(&scalar(ctx, &args[4])) {
            Ok(n) => n as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        base_ref.cols() as i64
    };

    if height <= 0 || width <= 0 {
        return Value::Error(CellError::Value);
    }

    let new_row = base_ref.start_row as i64 + row_off;
    let new_col = base_ref.start_col as i64 + col_off;
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
        return Value::Number((ctx.current().row + 1) as f64);
    }
    match &args[0] {
        Value::Ref(r) => {
            if r.is_single() {
                Value::Number((r.start_row + 1) as f64)
            } else {
                // return vertical array of row numbers
                let data: Vec<Value> = (r.start_row..=r.end_row)
                    .map(|row| Value::Number((row + 1) as f64))
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
        Value::Ref(r) => Value::Number(r.rows() as f64),
        Value::Array(a) => Value::Number(a.rows as f64),
        _ => Value::Number(1.0),
    }
}

fn column_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Number((ctx.current().col + 1) as f64);
    }
    match &args[0] {
        Value::Ref(r) => {
            if r.is_single() {
                Value::Number((r.start_col + 1) as f64)
            } else {
                // return horizontal array of column numbers
                let data: Vec<Value> = (r.start_col..=r.end_col)
                    .map(|col| Value::Number((col + 1) as f64))
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
        Value::Ref(r) => Value::Number(r.cols() as f64),
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
        1 => format!("${}${}", col_letters, row), // $A$1
        2 => format!("{}${}", col_letters, row),  // A$1  (col relative, row abs)
        3 => format!("${}${}", col_letters, row)
            .replace(&format!("${}$", col_letters), &format!("${}", col_letters)), // $A1 — abs col, rel row
        4 => format!("{}{}", col_letters, row), // A1
        _ => return Value::Error(CellError::Value),
    };
    // Fix abs_num=3: $col_rel_row
    let addr = match abs_num {
        3 => format!("${}{}", col_letters, row),
        _ => addr,
    };

    let result = if let Some(sheet) = sheet_text {
        format!("{}!{}", sheet, addr)
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
mod tests {
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn make_table() -> TestCtx {
        // A column (col 0): 1,2,3,4,5
        // B column (col 1): "apple","banana","cherry","date","elderberry"
        TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
            (0, 1, Value::Text("apple".into())),
            (1, 1, Value::Text("banana".into())),
            (2, 1, Value::Text("cherry".into())),
            (3, 1, Value::Text("date".into())),
            (4, 1, Value::Text("elderberry".into())),
        ])
    }

    #[test]
    fn vlookup_exact() {
        let mut ctx = make_table();
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(3.0),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Text("cherry".into()));
    }

    #[test]
    fn vlookup_not_found() {
        let mut ctx = make_table();
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Error(CellError::NA));
    }

    #[test]
    fn vlookup_approx() {
        let mut ctx = make_table();
        // approx match: lookup 2.5 → row with 2 (largest ≤ 2.5) → "banana"
        let result = vlookup(
            &mut ctx,
            &[
                Value::Number(2.5),
                rng(0, 0, 4, 1),
                Value::Number(2.0),
                Value::Bool(true),
            ],
        );
        assert_eq!(result, Value::Text("banana".into()));
    }

    #[test]
    fn match_exact() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Text("apple".into())),
            (1, 0, Value::Text("banana".into())),
            (2, 0, Value::Text("cherry".into())),
        ]);
        let result = match_fn(
            &mut ctx,
            &[
                Value::Text("banana".into()),
                rng(0, 0, 2, 0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn match_not_found() {
        let mut ctx =
            TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        let result = match_fn(
            &mut ctx,
            &[Value::Number(5.0), rng(0, 0, 1, 0), Value::Number(0.0)],
        );
        assert_eq!(result, Value::Error(CellError::NA));
    }

    #[test]
    fn match_approx() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        let result = match_fn(
            &mut ctx,
            &[Value::Number(4.0), rng(0, 0, 2, 0), Value::Number(1.0)],
        );
        assert_eq!(result, Value::Number(2.0)); // 3 is largest ≤ 4
    }

    #[test]
    fn index_single() {
        let mut ctx = make_table();
        // INDEX(A1:B5, 2, 2) → "banana"
        let result = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(2.0), Value::Number(2.0)],
        );
        // Returns a Ref; deref it
        let v = match result {
            Value::Ref(r) => ctx.cell(r.sheet, r.start_row, r.start_col),
            other => other,
        };
        assert_eq!(v, Value::Text("banana".into()));
    }

    #[test]
    fn index_out_of_bounds() {
        let mut ctx = make_table();
        let result = index_fn(
            &mut ctx,
            &[rng(0, 0, 4, 1), Value::Number(10.0), Value::Number(1.0)],
        );
        assert_eq!(result, Value::Error(CellError::Ref));
    }

    #[test]
    fn offset_basic() {
        let mut ctx = make_table();
        // OFFSET(A1, 2, 0) → single cell 3 rows down from A1 = row 2, col 0 = 3.0
        let result = offset(
            &mut ctx,
            &[rng(0, 0, 0, 0), Value::Number(2.0), Value::Number(0.0)],
        );
        match result {
            Value::Ref(r) => {
                assert_eq!(r.start_row, 2);
                assert_eq!(r.start_col, 0);
                assert_eq!(r.end_row, 2);
                assert_eq!(r.end_col, 0);
            }
            other => panic!("expected Ref, got {:?}", other),
        }
    }

    #[test]
    fn offset_with_size() {
        let mut ctx = make_table();
        // OFFSET(A1, 0, 0, 3, 2) → range A1:B3
        let result = offset(
            &mut ctx,
            &[
                rng(0, 0, 0, 0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(3.0),
                Value::Number(2.0),
            ],
        );
        match result {
            Value::Ref(r) => {
                assert_eq!(r.rows(), 3);
                assert_eq!(r.cols(), 2);
            }
            other => panic!("expected Ref, got {:?}", other),
        }
    }

    #[test]
    fn row_column_no_arg() {
        let mut ctx = TestCtx::new();
        ctx.set_current(4, 2);
        assert_eq!(row_fn(&mut ctx, &[]), Value::Number(5.0));
        assert_eq!(column_fn(&mut ctx, &[]), Value::Number(3.0));
    }

    #[test]
    fn row_column_with_ref() {
        let mut ctx = TestCtx::new();
        assert_eq!(row_fn(&mut ctx, &[rng(2, 1, 2, 1)]), Value::Number(3.0));
        assert_eq!(column_fn(&mut ctx, &[rng(2, 1, 2, 1)]), Value::Number(2.0));
    }

    #[test]
    fn rows_columns_range() {
        let mut ctx = TestCtx::new();
        assert_eq!(rows_fn(&mut ctx, &[rng(0, 0, 4, 2)]), Value::Number(5.0));
        assert_eq!(columns_fn(&mut ctx, &[rng(0, 0, 4, 2)]), Value::Number(3.0));
    }

    #[test]
    fn transpose_basic() {
        let mut ctx = TestCtx::new();
        let arr = Value::Array(Array::from_rows(vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
        ]));
        let result = transpose_fn(&mut ctx, &[arr]);
        if let Value::Array(a) = result {
            assert_eq!(a.rows, 2);
            assert_eq!(a.cols, 2);
            assert_eq!(a.get(0, 1), Some(&Value::Number(3.0)));
        } else {
            panic!("expected Array");
        }
    }

    #[test]
    fn address_abs() {
        let mut ctx = TestCtx::new();
        let result = address_fn(&mut ctx, &[Value::Number(3.0), Value::Number(2.0)]);
        assert_eq!(result, Value::Text("$B$3".into()));
    }

    #[test]
    fn address_relative() {
        let mut ctx = TestCtx::new();
        let result = address_fn(
            &mut ctx,
            &[Value::Number(3.0), Value::Number(2.0), Value::Number(4.0)],
        );
        assert_eq!(result, Value::Text("B3".into()));
    }

    #[test]
    fn hlookup_exact() {
        // Row 0: 1,2,3; Row 1: "a","b","c"
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (0, 1, Value::Number(2.0)),
            (0, 2, Value::Number(3.0)),
            (1, 0, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (1, 2, Value::Text("c".into())),
        ]);
        let result = hlookup(
            &mut ctx,
            &[
                Value::Number(2.0),
                rng(0, 0, 1, 2),
                Value::Number(2.0),
                Value::Bool(false),
            ],
        );
        assert_eq!(result, Value::Text("b".into()));
    }

    #[test]
    fn xlookup_basic() {
        let mut ctx = make_table();
        let result = xlookup(
            &mut ctx,
            &[Value::Number(3.0), rng(0, 0, 4, 0), rng(0, 1, 4, 1)],
        );
        assert_eq!(result, Value::Text("cherry".into()));
    }

    #[test]
    fn xlookup_not_found_fallback() {
        let mut ctx = make_table();
        let result = xlookup(
            &mut ctx,
            &[
                Value::Number(99.0),
                rng(0, 0, 4, 0),
                rng(0, 1, 4, 1),
                Value::Text("nope".into()),
            ],
        );
        assert_eq!(result, Value::Text("nope".into()));
    }
}
