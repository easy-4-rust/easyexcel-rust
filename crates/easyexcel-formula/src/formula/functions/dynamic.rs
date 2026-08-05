//! Dynamic-array / spill functions: SORT, FILTER, UNIQUE, SEQUENCE, the
//! stacking/shaping helpers (VSTACK, TAKE, …), and RANDARRAY.
//!
//! These return [`Value::Array`]. In the `eval` CLI they render as a grid; in a
//! cell formula the recalc engine *spills* the array into neighboring cells.

use std::cmp::Ordering;

use super::super::context::Context;
use super::super::value::{Array, Value};
use super::Registry;
use crate::formula::coerce::{self, to_number};
use easyexcel_model::error::CellError;

pub fn register(r: &mut Registry) {
    r.add("SORT", 1, 4, false, sort);
    r.add("SORTBY", 2, super::VARIADIC, false, sortby);
    r.add("UNIQUE", 1, 3, false, unique);
    r.add("FILTER", 2, 3, false, filter);
    r.add("SEQUENCE", 1, 4, false, sequence);
    r.add("RANDARRAY", 0, 5, true, randarray); // volatile
    r.add("VSTACK", 1, super::VARIADIC, false, vstack);
    r.add("HSTACK", 1, super::VARIADIC, false, hstack);
    r.add("TOROW", 1, 3, false, torow);
    r.add("TOCOL", 1, 3, false, tocol);
    r.add("WRAPROWS", 2, 3, false, wraprows);
    r.add("WRAPCOLS", 2, 3, false, wrapcols);
    r.add("TAKE", 2, 3, false, take);
    r.add("DROP", 2, 3, false, drop_fn);
    r.add("EXPAND", 2, 4, false, expand);
    r.add("CHOOSEROWS", 2, super::VARIADIC, false, chooserows);
    r.add("CHOOSECOLS", 2, super::VARIADIC, false, choosecols);
    r.add("TRIMRANGE", 1, 2, false, trimrange);
}

// ─── helpers ───────────────────────────────────────────────────────────────

/// Materialize any argument to a dense 2-D array (scalar → 1×1).
fn to_array(ctx: &mut dyn Context, v: &Value) -> Array {
    match v {
        Value::Array(a) => a.clone(),
        Value::Ref(r) => ctx.ref_to_array(*r),
        other => Array::scalar(other.clone()),
    }
}

/// Row `r` of `a` as a Vec.
fn row_of(a: &Array, r: usize) -> Vec<Value> {
    (0..a.cols)
        .map(|c| a.data[r * a.cols + c].clone())
        .collect()
}

/// Column `c` of `a` as a Vec.
fn col_of(a: &Array, c: usize) -> Vec<Value> {
    (0..a.rows)
        .map(|r| a.data[r * a.cols + c].clone())
        .collect()
}

/// Build an array from a list of rows (each the same length).
fn from_rows(rows: Vec<Vec<Value>>) -> Value {
    let nrows = rows.len();
    if nrows == 0 {
        return Value::Error(CellError::Calc);
    }
    let ncols = rows[0].len();
    let mut data = Vec::with_capacity(nrows * ncols);
    for row in rows {
        data.extend(row);
    }
    Value::Array(Array::new(nrows, ncols, data))
}

/// Get an integer arg, defaulting when absent.
fn int_arg(args: &[Value], i: usize, default: i64) -> Result<i64, CellError> {
    match args.get(i) {
        None | Some(Value::Empty) => Ok(default),
        Some(v) => Ok(to_number(v)?.trunc() as i64),
    }
}

fn bool_arg(args: &[Value], i: usize, default: bool) -> Result<bool, CellError> {
    match args.get(i) {
        None | Some(Value::Empty) => Ok(default),
        Some(v) => coerce::to_bool(v),
    }
}

/// Excel sort order between two cells for SORT/SORTBY (numbers < text < bool).
fn sort_cmp(a: &Value, b: &Value) -> Ordering {
    coerce::compare(a, b)
}

// ─── SORT / SORTBY ───────────────────────────────────────────────────────────

fn sort(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let by_col = match bool_arg(args, 3, false) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    let sort_index = match int_arg(args, 1, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let order = match int_arg(args, 2, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    if sort_index < 1 {
        return Value::Error(CellError::Value);
    }
    let descending = order < 0;

    if by_col {
        // Sort columns by the key row `sort_index`.
        let key = (sort_index - 1) as usize;
        if key >= arr.rows {
            return Value::Error(CellError::Value);
        }
        let mut cols: Vec<usize> = (0..arr.cols).collect();
        cols.sort_by(|&x, &y| {
            let o = sort_cmp(&arr.data[key * arr.cols + x], &arr.data[key * arr.cols + y]);
            if descending { o.reverse() } else { o }
        });
        let mut data = Vec::with_capacity(arr.rows * arr.cols);
        for r in 0..arr.rows {
            for &c in &cols {
                data.push(arr.data[r * arr.cols + c].clone());
            }
        }
        Value::Array(Array::new(arr.rows, arr.cols, data))
    } else {
        // Sort rows by the key column `sort_index`.
        let key = (sort_index - 1) as usize;
        if key >= arr.cols {
            return Value::Error(CellError::Value);
        }
        let mut idx: Vec<usize> = (0..arr.rows).collect();
        idx.sort_by(|&x, &y| {
            let o = sort_cmp(&arr.data[x * arr.cols + key], &arr.data[y * arr.cols + key]);
            if descending { o.reverse() } else { o }
        });
        let rows: Vec<Vec<Value>> = idx.into_iter().map(|r| row_of(&arr, r)).collect();
        from_rows(rows)
    }
}

fn sortby(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    // Collect (by_array, order) pairs.
    let mut keys: Vec<(Array, bool)> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let by = to_array(ctx, &args[i]);
        if by.rows != arr.rows {
            return Value::Error(CellError::Value);
        }
        let order = match int_arg(args, i + 1, 1) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        keys.push((by, order < 0));
        i += 2;
    }
    if keys.is_empty() {
        return Value::Error(CellError::Value);
    }
    let mut idx: Vec<usize> = (0..arr.rows).collect();
    idx.sort_by(|&x, &y| {
        for (by, desc) in &keys {
            // by-array is treated as a column vector keyed by row.
            let a = &by.data[x * by.cols];
            let b = &by.data[y * by.cols];
            let o = sort_cmp(a, b);
            let o = if *desc { o.reverse() } else { o };
            if o != Ordering::Equal {
                return o;
            }
        }
        Ordering::Equal
    });
    let rows: Vec<Vec<Value>> = idx.into_iter().map(|r| row_of(&arr, r)).collect();
    from_rows(rows)
}

// ─── UNIQUE ────────────────────────────────────────────────────────────────

fn unique(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let by_col = match bool_arg(args, 1, false) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };
    let exactly_once = match bool_arg(args, 2, false) {
        Ok(b) => b,
        Err(e) => return Value::Error(e),
    };

    // Group identical rows (or columns) preserving first-seen order.
    let items: Vec<Vec<Value>> = if by_col {
        (0..arr.cols).map(|c| col_of(&arr, c)).collect()
    } else {
        (0..arr.rows).map(|r| row_of(&arr, r)).collect()
    };
    let key =
        |item: &[Value]| -> String { item.iter().map(value_key).collect::<Vec<_>>().join("\u{1}") };

    let mut order: Vec<usize> = Vec::new();
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut first: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, item) in items.iter().enumerate() {
        let k = key(item);
        *counts.entry(k.clone()).or_insert(0) += 1;
        if !first.contains_key(&k) {
            first.insert(k.clone(), i);
            order.push(i);
        }
    }
    let kept: Vec<usize> = order
        .into_iter()
        .filter(|&i| !exactly_once || counts[&key(&items[i])] == 1)
        .collect();
    if kept.is_empty() {
        return Value::Error(CellError::Calc);
    }

    if by_col {
        let mut data = Vec::with_capacity(arr.rows * kept.len());
        #[allow(clippy::needless_range_loop)]
        for r in 0..arr.rows {
            for &c in &kept {
                data.push(items[c][r].clone());
            }
        }
        Value::Array(Array::new(arr.rows, kept.len(), data))
    } else {
        from_rows(kept.into_iter().map(|i| items[i].clone()).collect())
    }
}

/// A stable string key for value equality/grouping.
fn value_key(v: &Value) -> String {
    match v {
        Value::Number(n) => format!("n:{n}"),
        Value::Text(s) => format!("t:{}", s.to_lowercase()),
        Value::Bool(b) => format!("b:{b}"),
        Value::Empty => "e:".to_string(),
        Value::Error(e) => format!("x:{}", e.as_str()),
        Value::Array(_) | Value::Ref(_) | Value::Lambda(_) => "?".to_string(),
    }
}

// ─── FILTER ────────────────────────────────────────────────────────────────

fn filter(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let inc = to_array(ctx, &args[1]);
    // `include` is a column vector (filter rows) or row vector (filter cols).
    let by_col = inc.rows == 1 && inc.cols == arr.cols && arr.rows != 1;
    let keep_mask: Vec<bool> = inc
        .data
        .iter()
        .map(|v| matches!(coerce::to_bool(v), Ok(true)))
        .collect();

    if by_col {
        if keep_mask.len() != arr.cols {
            return Value::Error(CellError::Value);
        }
        let cols: Vec<usize> = (0..arr.cols).filter(|&c| keep_mask[c]).collect();
        if cols.is_empty() {
            return if_empty(args, 2);
        }
        let mut data = Vec::with_capacity(arr.rows * cols.len());
        for r in 0..arr.rows {
            for &c in &cols {
                data.push(arr.data[r * arr.cols + c].clone());
            }
        }
        Value::Array(Array::new(arr.rows, cols.len(), data))
    } else {
        if keep_mask.len() != arr.rows {
            return Value::Error(CellError::Value);
        }
        let rows: Vec<Vec<Value>> = (0..arr.rows)
            .filter(|&r| keep_mask[r])
            .map(|r| row_of(&arr, r))
            .collect();
        if rows.is_empty() {
            return if_empty(args, 2);
        }
        from_rows(rows)
    }
}

fn if_empty(args: &[Value], i: usize) -> Value {
    match args.get(i) {
        Some(v) if !matches!(v, Value::Empty) => v.clone(),
        _ => Value::Error(CellError::Calc),
    }
}

// ─── SEQUENCE / RANDARRAY ────────────────────────────────────────────────────

fn sequence(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let rows = match int_arg(args, 0, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let cols = match int_arg(args, 1, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let start = match args.get(2) {
        Some(v) if !matches!(v, Value::Empty) => match to_number(v) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        },
        _ => 1.0,
    };
    let step = match args.get(3) {
        Some(v) if !matches!(v, Value::Empty) => match to_number(v) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        },
        _ => 1.0,
    };
    if rows < 1 || cols < 1 {
        return Value::Error(CellError::Value);
    }
    let (rows, cols) = (rows as usize, cols as usize);
    let mut data = Vec::with_capacity(rows * cols);
    for i in 0..(rows * cols) {
        data.push(Value::Number(start + step * i as f64));
    }
    Value::Array(Array::new(rows, cols, data))
}

fn randarray(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let rows = match int_arg(args, 0, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let cols = match int_arg(args, 1, 1) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let min = match args.get(2) {
        Some(v) if !matches!(v, Value::Empty) => to_number(v).unwrap_or(0.0),
        _ => 0.0,
    };
    let max = match args.get(3) {
        Some(v) if !matches!(v, Value::Empty) => to_number(v).unwrap_or(1.0),
        _ => 1.0,
    };
    let whole = matches!(bool_arg(args, 4, false), Ok(true));
    if rows < 1 || cols < 1 || min > max {
        return Value::Error(CellError::Value);
    }
    let (rows, cols) = (rows as usize, cols as usize);
    // Deterministic-but-varying PRNG seeded from the cell + clock (no rand dep).
    let cur = ctx.current();
    let mut state = (ctx.now_serial().to_bits())
        ^ ((cur.row as u64) << 20)
        ^ ((cur.col as u64) << 5)
        ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        // xorshift64*
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let r = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (r >> 11) as f64 / (1u64 << 53) as f64 // [0,1)
    };
    let mut data = Vec::with_capacity(rows * cols);
    for _ in 0..(rows * cols) {
        let u = next();
        let v = if whole {
            (min + (u * (max - min + 1.0)).floor()).min(max)
        } else {
            min + u * (max - min)
        };
        data.push(Value::Number(v));
    }
    Value::Array(Array::new(rows, cols, data))
}

// ─── VSTACK / HSTACK ─────────────────────────────────────────────────────────

fn vstack(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arrays: Vec<Array> = args.iter().map(|a| to_array(ctx, a)).collect();
    let ncols = arrays.iter().map(|a| a.cols).max().unwrap_or(0);
    let mut data = Vec::new();
    let mut nrows = 0;
    for a in &arrays {
        for r in 0..a.rows {
            for c in 0..ncols {
                data.push(if c < a.cols {
                    a.data[r * a.cols + c].clone()
                } else {
                    Value::Error(CellError::NA)
                });
            }
            nrows += 1;
        }
    }
    if nrows == 0 {
        return Value::Error(CellError::Calc);
    }
    Value::Array(Array::new(nrows, ncols, data))
}

fn hstack(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arrays: Vec<Array> = args.iter().map(|a| to_array(ctx, a)).collect();
    let nrows = arrays.iter().map(|a| a.rows).max().unwrap_or(0);
    let ncols: usize = arrays.iter().map(|a| a.cols).sum();
    if nrows == 0 || ncols == 0 {
        return Value::Error(CellError::Calc);
    }
    let mut data = Vec::with_capacity(nrows * ncols);
    for r in 0..nrows {
        for a in &arrays {
            for c in 0..a.cols {
                data.push(if r < a.rows {
                    a.data[r * a.cols + c].clone()
                } else {
                    Value::Error(CellError::NA)
                });
            }
        }
    }
    Value::Array(Array::new(nrows, ncols, data))
}

// ─── TOROW / TOCOL ───────────────────────────────────────────────────────────

/// Flatten respecting `scan_by_col`; `ignore` 1/3 skips blanks, 2/3 skips errors.
fn flatten_scan(a: &Array, ignore: i64, by_col: bool) -> Vec<Value> {
    let mut out = Vec::new();
    let push = |v: &Value, out: &mut Vec<Value>| {
        let skip = match v {
            Value::Empty => ignore == 1 || ignore == 3,
            Value::Error(_) => ignore == 2 || ignore == 3,
            _ => false,
        };
        if !skip {
            out.push(v.clone());
        }
    };
    if by_col {
        for c in 0..a.cols {
            for r in 0..a.rows {
                push(&a.data[r * a.cols + c], &mut out);
            }
        }
    } else {
        for v in &a.data {
            push(v, &mut out);
        }
    }
    out
}

fn torow(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let ignore = int_arg(args, 1, 0).unwrap_or(0);
    let by_col = matches!(bool_arg(args, 2, false), Ok(true));
    let data = flatten_scan(&arr, ignore, by_col);
    if data.is_empty() {
        return Value::Error(CellError::Calc);
    }
    let n = data.len();
    Value::Array(Array::new(1, n, data))
}

fn tocol(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let ignore = int_arg(args, 1, 0).unwrap_or(0);
    let by_col = matches!(bool_arg(args, 2, false), Ok(true));
    let data = flatten_scan(&arr, ignore, by_col);
    if data.is_empty() {
        return Value::Error(CellError::Calc);
    }
    let n = data.len();
    Value::Array(Array::new(n, 1, data))
}

// ─── WRAPROWS / WRAPCOLS ─────────────────────────────────────────────────────

fn wrap(ctx: &mut dyn Context, args: &[Value], by_col: bool) -> Value {
    let arr = to_array(ctx, &args[0]);
    let wrap_count = match int_arg(args, 1, 0) {
        Ok(n) if n >= 1 => n as usize,
        Ok(_) => return Value::Error(CellError::Value),
        Err(e) => return Value::Error(e),
    };
    let pad = args.get(2).cloned().filter(|v| !matches!(v, Value::Empty));
    let flat: Vec<Value> = arr.data.clone();
    if flat.is_empty() {
        return Value::Error(CellError::Calc);
    }
    let lines = flat.len().div_ceil(wrap_count);
    let pad_v = || pad.clone().unwrap_or(Value::Error(CellError::NA));
    if by_col {
        // wrap_count is the number of rows per column.
        let (rows, cols) = (wrap_count, lines);
        let mut data = vec![Value::Empty; rows * cols];
        for (k, v) in flat.into_iter().enumerate() {
            let (c, r) = (k / wrap_count, k % wrap_count);
            data[r * cols + c] = v;
        }
        for slot in data.iter_mut() {
            if matches!(slot, Value::Empty) {
                *slot = pad_v();
            }
        }
        Value::Array(Array::new(rows, cols, data))
    } else {
        let (rows, cols) = (lines, wrap_count);
        let mut data = Vec::with_capacity(rows * cols);
        for i in 0..(rows * cols) {
            data.push(flat.get(i).cloned().unwrap_or_else(pad_v));
        }
        Value::Array(Array::new(rows, cols, data))
    }
}

fn wraprows(ctx: &mut dyn Context, args: &[Value]) -> Value {
    wrap(ctx, args, false)
}
fn wrapcols(ctx: &mut dyn Context, args: &[Value]) -> Value {
    wrap(ctx, args, true)
}

// ─── TAKE / DROP ─────────────────────────────────────────────────────────────

/// Resolve a take/drop count to a kept index range `[lo, hi)` over `len`.
fn take_range(n: i64, len: usize) -> (usize, usize) {
    let len = len as i64;
    if n >= 0 {
        (0, n.min(len) as usize)
    } else {
        ((len + n).max(0) as usize, len as usize)
    }
}
fn drop_range(n: i64, len: usize) -> (usize, usize) {
    let len_i = len as i64;
    if n >= 0 {
        (n.min(len_i) as usize, len)
    } else {
        (0, (len_i + n).max(0) as usize)
    }
}

fn slice(arr: &Array, rr: (usize, usize), cr: (usize, usize)) -> Value {
    if rr.0 >= rr.1 || cr.0 >= cr.1 {
        return Value::Error(CellError::Calc);
    }
    let mut data = Vec::with_capacity((rr.1 - rr.0) * (cr.1 - cr.0));
    for r in rr.0..rr.1 {
        for c in cr.0..cr.1 {
            data.push(arr.data[r * arr.cols + c].clone());
        }
    }
    Value::Array(Array::new(rr.1 - rr.0, cr.1 - cr.0, data))
}

fn take(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let rows = match int_arg(args, 1, arr.rows as i64) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let cr = match args.get(2) {
        Some(v) if !matches!(v, Value::Empty) => match to_number(v) {
            Ok(n) => take_range(n.trunc() as i64, arr.cols),
            Err(e) => return Value::Error(e),
        },
        _ => (0, arr.cols),
    };
    slice(&arr, take_range(rows, arr.rows), cr)
}

fn drop_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let rows = match int_arg(args, 1, 0) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let cr = match args.get(2) {
        Some(v) if !matches!(v, Value::Empty) => match to_number(v) {
            Ok(n) => drop_range(n.trunc() as i64, arr.cols),
            Err(e) => return Value::Error(e),
        },
        _ => (0, arr.cols),
    };
    slice(&arr, drop_range(rows, arr.rows), cr)
}

// ─── EXPAND ────────────────────────────────────────────────────────────────

fn expand(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let rows = match int_arg(args, 1, arr.rows as i64) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let cols = match args.get(2) {
        Some(v) if !matches!(v, Value::Empty) => match to_number(v) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        },
        _ => arr.cols as i64,
    };
    if rows < arr.rows as i64 || cols < arr.cols as i64 {
        return Value::Error(CellError::Value);
    }
    let pad = args
        .get(3)
        .cloned()
        .filter(|v| !matches!(v, Value::Empty))
        .unwrap_or(Value::Error(CellError::NA));
    let (rows, cols) = (rows as usize, cols as usize);
    let mut data = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            data.push(if r < arr.rows && c < arr.cols {
                arr.data[r * arr.cols + c].clone()
            } else {
                pad.clone()
            });
        }
    }
    Value::Array(Array::new(rows, cols, data))
}

// ─── CHOOSEROWS / CHOOSECOLS ─────────────────────────────────────────────────

fn chooserows(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let mut picked = Vec::new();
    for a in &args[1..] {
        for v in flatten_one(ctx, a) {
            let n = match to_number(&v) {
                Ok(n) => n.trunc() as i64,
                Err(e) => return Value::Error(e),
            };
            let r = norm_index(n, arr.rows);
            match r {
                Some(r) => picked.push(row_of(&arr, r)),
                None => return Value::Error(CellError::Value),
            }
        }
    }
    if picked.is_empty() {
        return Value::Error(CellError::Calc);
    }
    from_rows(picked)
}

fn choosecols(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    let mut cols = Vec::new();
    for a in &args[1..] {
        for v in flatten_one(ctx, a) {
            let n = match to_number(&v) {
                Ok(n) => n.trunc() as i64,
                Err(e) => return Value::Error(e),
            };
            match norm_index(n, arr.cols) {
                Some(c) => cols.push(c),
                None => return Value::Error(CellError::Value),
            }
        }
    }
    if cols.is_empty() {
        return Value::Error(CellError::Calc);
    }
    let mut data = Vec::with_capacity(arr.rows * cols.len());
    for r in 0..arr.rows {
        for &c in &cols {
            data.push(arr.data[r * arr.cols + c].clone());
        }
    }
    Value::Array(Array::new(arr.rows, cols.len(), data))
}

/// 1-based index (negative = from end) → 0-based, bounds-checked.
fn norm_index(n: i64, len: usize) -> Option<usize> {
    let len = len as i64;
    let idx = if n < 0 { len + n } else { n - 1 };
    if idx >= 0 && idx < len {
        Some(idx as usize)
    } else {
        None
    }
}

fn flatten_one(ctx: &mut dyn Context, v: &Value) -> Vec<Value> {
    ctx.flatten(v)
}

// ─── TRIMRANGE ─────────────────────────────────────────────────────────────

fn trimrange(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arr = to_array(ctx, &args[0]);
    // trim: 0 none, 1 trailing (default), 2 leading, 3 both.
    let trim = int_arg(args, 1, 3).unwrap_or(3);
    let trim_lead = trim == 2 || trim == 3;
    let trim_trail = trim == 1 || trim == 3;

    let row_blank =
        |r: usize| (0..arr.cols).all(|c| matches!(arr.data[r * arr.cols + c], Value::Empty));
    let col_blank =
        |c: usize| (0..arr.rows).all(|r| matches!(arr.data[r * arr.cols + c], Value::Empty));

    let mut r0 = 0;
    let mut r1 = arr.rows;
    let mut c0 = 0;
    let mut c1 = arr.cols;
    if trim_lead {
        while r0 < r1 && row_blank(r0) {
            r0 += 1;
        }
        while c0 < c1 && col_blank(c0) {
            c0 += 1;
        }
    }
    if trim_trail {
        while r1 > r0 && row_blank(r1 - 1) {
            r1 -= 1;
        }
        while c1 > c0 && col_blank(c1 - 1) {
            c1 -= 1;
        }
    }
    slice(&arr, (r0, r1), (c0, c1))
}
