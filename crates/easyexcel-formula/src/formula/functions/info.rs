//! Information functions (IS*, TYPE, N, NA, …).

use super::Registry;
use crate::formula::coerce::{to_number, to_text};
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

pub fn register(r: &mut Registry) {
    r.add("ISBLANK", 1, 1, false, isblank_fn);
    r.add("ISERROR", 1, 1, false, iserror_fn);
    r.add("ISERR", 1, 1, false, iserr_fn);
    r.add("ISNA", 1, 1, false, isna_fn);
    r.add("ISNUMBER", 1, 1, false, isnumber_fn);
    r.add("ISTEXT", 1, 1, false, istext_fn);
    r.add("ISNONTEXT", 1, 1, false, isnontext_fn);
    r.add("ISLOGICAL", 1, 1, false, islogical_fn);
    r.add("ISREF", 1, 1, false, isref_fn);
    r.add("ISFORMULA", 1, 1, false, isformula_fn);
    r.add("ISEVEN", 1, 1, false, iseven_fn);
    r.add("ISODD", 1, 1, false, isodd_fn);
    r.add("N", 1, 1, false, n_fn);
    r.add("NA", 0, 0, false, |_, _| Value::Error(CellError::NA));
    r.add("TYPE", 1, 1, false, type_fn);
    r.add("ERROR.TYPE", 1, 1, false, error_type_fn);
    r.add("SHEET", 0, 1, false, sheet_fn);
    r.add("SHEETS", 0, 1, false, sheets_fn);
    r.add("CELL", 2, 2, false, cell_fn);
    r.add("INFO", 1, 1, false, info_fn);
}

// ---------------------------------------------------------------------------
// IS* functions — all take a single arg and return Bool.
// ---------------------------------------------------------------------------

fn isblank_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Empty))
}

fn iserror_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Error(_)))
}

fn iserr_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    // ISERR is like ISERROR but returns FALSE for #N/A
    Value::Bool(matches!(args[0], Value::Error(e) if e != CellError::NA))
}

fn isna_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Error(CellError::NA)))
}

fn isnumber_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Number(_)))
}

fn istext_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Text(_)))
}

fn isnontext_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(!matches!(args[0], Value::Text(_)))
}

fn islogical_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Bool(_)))
}

fn isref_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    Value::Bool(matches!(args[0], Value::Ref(_)))
}

fn isformula_fn(_ctx: &mut dyn Context, _args: &[Value]) -> Value {
    // PARITY: cannot determine if a cell contains a formula from the Value alone.
    // Return FALSE as a safe fallback.
    Value::Bool(false)
}

fn iseven_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match to_number(&args[0]) {
        Ok(n) => Value::Bool((n.trunc() as i64) % 2 == 0),
        Err(e) => Value::Error(e),
    }
}

fn isodd_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match to_number(&args[0]) {
        Ok(n) => Value::Bool((n.trunc() as i64) % 2 != 0),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// N — convert to number
// ---------------------------------------------------------------------------

fn n_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Number(n) => Value::Number(*n),
        Value::Bool(b) => Value::Number(if *b { 1.0 } else { 0.0 }),
        Value::Error(e) => Value::Error(*e),
        _ => Value::Number(0.0),
    }
}

// ---------------------------------------------------------------------------
// TYPE
// ---------------------------------------------------------------------------

fn type_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    // Excel TYPE codes: 1=number, 2=text, 4=bool, 8=formula/array, 16=error, 64=array
    let code = match &args[0] {
        Value::Number(_) => 1,
        Value::Text(_) => 2,
        Value::Bool(_) => 4,
        Value::Array(_) => 64,
        Value::Error(_) => 16,
        Value::Empty => 1,  // empty treated as 0 (number)
        Value::Ref(_) => 8, // PARITY: ref → treat as 8 (formula result)
        Value::Lambda(_) => 128,
    };
    Value::Number(code as f64)
}

// ---------------------------------------------------------------------------
// ERROR.TYPE
// ---------------------------------------------------------------------------

fn error_type_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    match &args[0] {
        Value::Error(e) => {
            let code = match e {
                CellError::Null => 1,
                CellError::Div0 => 2,
                CellError::Value => 3,
                CellError::Ref => 4,
                CellError::Name => 5,
                CellError::Num => 6,
                CellError::NA => 7,
                CellError::GettingData => 8,
                CellError::Spill => 9,
                CellError::Calc => 14,
            };
            Value::Number(code as f64)
        }
        _ => Value::Error(CellError::NA),
    }
}

// ---------------------------------------------------------------------------
// SHEET / SHEETS
// ---------------------------------------------------------------------------

fn sheet_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.is_empty() {
        // No arg → current sheet (1-based)
        return Value::Number((ctx.current().sheet + 1) as f64);
    }
    match &args[0] {
        Value::Ref(r) => Value::Number((r.sheet + 1) as f64),
        Value::Text(s) => match ctx.sheet_index(s) {
            Some(i) => Value::Number((i + 1) as f64),
            None => Value::Error(CellError::NA),
        },
        _ => Value::Error(CellError::Value),
    }
}

fn sheets_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Number(ctx.sheet_count() as f64);
    }
    // With a ref arg: return 1 (we only model single-area refs which span 1 sheet).
    // PARITY: multi-sheet 3D ranges not modelled.
    match &args[0] {
        Value::Ref(_) => Value::Number(1.0),
        _ => Value::Error(CellError::Value),
    }
}

// ---------------------------------------------------------------------------
// CELL — best-effort for common info_type values
// ---------------------------------------------------------------------------

fn cell_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let info_type = match to_text(&args[0]) {
        Ok(s) => s.to_lowercase(),
        Err(e) => return Value::Error(e),
    };

    let ref_val = &args[1];
    let (sheet, row, col) = match ref_val {
        Value::Ref(r) => (r.sheet, r.start_row, r.start_col),
        _ => return Value::Error(CellError::Value),
    };

    match info_type.as_str() {
        "row" => Value::Number((row + 1) as f64),
        "col" | "column" => Value::Number((col + 1) as f64),
        "address" => {
            let col_letters = easyexcel_model::addr::col_index_to_letters(col);
            Value::Text(format!("${}${}", col_letters, row + 1))
        }
        "contents" => ctx.cell(sheet, row, col),
        "type" => {
            let v = ctx.cell(sheet, row, col);
            let t = match v {
                Value::Empty => "b",
                Value::Text(_) => "l",
                _ => "v",
            };
            Value::Text(t.into())
        }
        "sheet" => Value::Number((sheet + 1) as f64),
        // PARITY: other info_type values not implemented.
        _ => Value::Error(CellError::NA),
    }
}

// ---------------------------------------------------------------------------
// INFO — best-effort constants
// ---------------------------------------------------------------------------

fn info_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let info_type = match to_text(&args[0]) {
        Ok(s) => s.to_lowercase(),
        Err(e) => return Value::Error(e),
    };

    match info_type.as_str() {
        "numfile" => Value::Number(1.0), // PARITY: always 1
        "osversion" => Value::Text("Windows (32-bit) NT 10.00".into()), // PARITY: constant
        "release" => Value::Text("16.0".into()), // PARITY: Excel 2016 compat string
        "system" => Value::Text("pcdos".into()), // PARITY: Windows system
        "directory" | "origin" | "recalc" | "memavail" | "memused" | "totmem" => {
            // PARITY: these require OS/workbook state we don't have.
            Value::Error(CellError::NA)
        }
        _ => Value::Error(CellError::NA),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    #[test]
    fn is_functions() {
        let mut c = TestCtx::new();
        assert_eq!(isblank_fn(&mut c, &[Value::Empty]), Value::Bool(true));
        assert_eq!(
            isblank_fn(&mut c, &[Value::Number(0.0)]),
            Value::Bool(false)
        );
        assert_eq!(
            iserror_fn(&mut c, &[Value::Error(CellError::Value)]),
            Value::Bool(true)
        );
        assert_eq!(
            iserror_fn(&mut c, &[Value::Number(1.0)]),
            Value::Bool(false)
        );
        assert_eq!(
            iserr_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Bool(false)
        );
        assert_eq!(
            iserr_fn(&mut c, &[Value::Error(CellError::Value)]),
            Value::Bool(true)
        );
        assert_eq!(
            isna_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Bool(true)
        );
        assert_eq!(
            isnumber_fn(&mut c, &[Value::Number(3.0)]),
            Value::Bool(true)
        );
        assert_eq!(
            isnumber_fn(&mut c, &[Value::Text("3".into())]),
            Value::Bool(false)
        );
        assert_eq!(
            istext_fn(&mut c, &[Value::Text("hi".into())]),
            Value::Bool(true)
        );
        assert_eq!(
            isnontext_fn(&mut c, &[Value::Number(1.0)]),
            Value::Bool(true)
        );
        assert_eq!(
            islogical_fn(&mut c, &[Value::Bool(false)]),
            Value::Bool(true)
        );
        assert_eq!(isref_fn(&mut c, &[rng(0, 0, 0, 0)]), Value::Bool(true));
        assert_eq!(isref_fn(&mut c, &[Value::Number(1.0)]), Value::Bool(false));
    }

    #[test]
    fn iseven_isodd() {
        let mut c = TestCtx::new();
        assert_eq!(iseven_fn(&mut c, &[Value::Number(4.0)]), Value::Bool(true));
        assert_eq!(iseven_fn(&mut c, &[Value::Number(3.0)]), Value::Bool(false));
        assert_eq!(isodd_fn(&mut c, &[Value::Number(3.0)]), Value::Bool(true));
        assert_eq!(isodd_fn(&mut c, &[Value::Number(4.0)]), Value::Bool(false));
    }

    #[test]
    fn n_fn_conversions() {
        let mut c = TestCtx::new();
        assert_eq!(n_fn(&mut c, &[Value::Number(5.0)]), Value::Number(5.0));
        assert_eq!(n_fn(&mut c, &[Value::Bool(true)]), Value::Number(1.0));
        assert_eq!(
            n_fn(&mut c, &[Value::Text("hi".into())]),
            Value::Number(0.0)
        );
        assert_eq!(n_fn(&mut c, &[Value::Empty]), Value::Number(0.0));
    }

    #[test]
    fn type_fn_codes() {
        let mut c = TestCtx::new();
        assert_eq!(type_fn(&mut c, &[Value::Number(1.0)]), Value::Number(1.0));
        assert_eq!(
            type_fn(&mut c, &[Value::Text("x".into())]),
            Value::Number(2.0)
        );
        assert_eq!(type_fn(&mut c, &[Value::Bool(true)]), Value::Number(4.0));
        assert_eq!(
            type_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Number(16.0)
        );
    }

    #[test]
    fn error_type_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(
            error_type_fn(&mut c, &[Value::Error(CellError::Null)]),
            Value::Number(1.0)
        );
        assert_eq!(
            error_type_fn(&mut c, &[Value::Error(CellError::Div0)]),
            Value::Number(2.0)
        );
        assert_eq!(
            error_type_fn(&mut c, &[Value::Error(CellError::NA)]),
            Value::Number(7.0)
        );
        assert_eq!(
            error_type_fn(&mut c, &[Value::Number(1.0)]),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn sheet_fn_basic() {
        let mut c = TestCtx::new();
        // no arg → current sheet 1-based
        assert_eq!(sheet_fn(&mut c, &[]), Value::Number(1.0));
        // with ref on sheet 0
        assert_eq!(sheet_fn(&mut c, &[rng(0, 0, 0, 0)]), Value::Number(1.0));
    }

    #[test]
    fn sheets_fn_basic() {
        let mut c = TestCtx::new();
        assert_eq!(sheets_fn(&mut c, &[]), Value::Number(1.0));
    }

    #[test]
    fn cell_fn_row_col() {
        let mut c = TestCtx::with_cells(&[(2, 3, Value::Number(42.0))]);
        let r = cell_fn(&mut c, &[Value::Text("row".into()), rng(2, 3, 2, 3)]);
        assert_eq!(r, Value::Number(3.0));
        let r = cell_fn(&mut c, &[Value::Text("col".into()), rng(2, 3, 2, 3)]);
        assert_eq!(r, Value::Number(4.0));
    }

    #[test]
    fn cell_fn_contents() {
        let mut c = TestCtx::with_cells(&[(1, 0, Value::Number(99.0))]);
        let r = cell_fn(&mut c, &[Value::Text("contents".into()), rng(1, 0, 1, 0)]);
        assert_eq!(r, Value::Number(99.0));
    }

    #[test]
    fn na_fn() {
        let r = Value::Error(CellError::NA);
        assert_eq!(r, Value::Error(CellError::NA));
    }

    #[test]
    fn info_fn_release() {
        let mut c = TestCtx::new();
        let r = info_fn(&mut c, &[Value::Text("release".into())]);
        assert!(matches!(r, Value::Text(_)));
    }

    #[test]
    fn isformula_returns_false() {
        let mut c = TestCtx::new();
        let r = isformula_fn(&mut c, &[rng(0, 0, 0, 0)]);
        assert_eq!(r, Value::Bool(false));
    }
}
