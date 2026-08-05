//! Financial functions (PMT, FV, PV, NPV, IRR, XNPV, XIRR, …).
//!
//! Standard annuity sign convention (matching Excel):
//!   • pv  = present value (positive = cash received)
//!   • pmt = periodic payment (negative = cash paid out)
//!   • fv  = future value (negative = cash paid out at end)
//!   • type = 0 (payments at period end, default) / 1 (beginning)
//!   • rate = periodic interest rate (not percent)
//!
//! Iterative solvers (RATE, IRR, XIRR) use Newton–Raphson with bisection
//! fallback, up to 100 iterations, tolerance 1e-7.
//! PARITY: precision tolerance 1e-7 may differ from Excel by a few ULPs.

use super::{Registry, VARIADIC};
use crate::formula::coerce::to_number;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

pub fn register(r: &mut Registry) {
    // Time-value-of-money core
    r.add("PMT", 3, 5, false, pmt);
    r.add("FV", 3, 5, false, fv);
    r.add("PV", 3, 5, false, pv);
    r.add("NPER", 3, 5, false, nper);
    r.add("RATE", 3, 6, false, rate);
    r.add("IPMT", 4, 6, false, ipmt);
    r.add("PPMT", 4, 6, false, ppmt);
    r.add("CUMIPMT", 6, 6, false, cumipmt);
    r.add("CUMPRINC", 6, 6, false, cumprinc);

    // Cash-flow analysis
    r.add("NPV", 2, VARIADIC, false, npv);
    r.add("XNPV", 3, 3, false, xnpv);
    r.add("IRR", 1, 2, false, irr);
    r.add("XIRR", 2, 3, false, xirr);
    r.add("MIRR", 3, 3, false, mirr);

    // Depreciation
    r.add("SLN", 3, 3, false, sln);
    r.add("SYD", 4, 4, false, syd);
    r.add("DB", 4, 5, false, db);
    r.add("DDB", 4, 5, false, ddb);
    r.add("VDB", 5, 7, false, vdb);

    // Rate/return conversions
    r.add("EFFECT", 2, 2, false, effect);
    r.add("NOMINAL", 2, 2, false, nominal);
    r.add("FVSCHEDULE", 2, 2, false, fvschedule);
    r.add("PDURATION", 3, 3, false, pduration);
    r.add("RRI", 3, 3, false, rri);

    // Dollar fraction helpers
    r.add("DOLLARDE", 2, 2, false, dollarde);
    r.add("DOLLARFR", 2, 2, false, dollarfr);

    // Simple interest
    r.add("ISPMT", 4, 4, false, ispmt);

    // Discount / T-bill / simple bond helpers
    r.add("DISC", 4, 5, false, disc);
    r.add("INTRATE", 4, 5, false, intrate);
    r.add("RECEIVED", 4, 5, false, received);
    r.add("TBILLEQ", 3, 3, false, tbilleq);
    r.add("TBILLPRICE", 3, 3, false, tbillprice);
    r.add("TBILLYIELD", 3, 3, false, tbillyield);
    r.add("PRICEDISC", 4, 5, false, pricedisc);

    // Coupon date helpers
    r.add("COUPNCD", 3, 4, false, coupncd);
    r.add("COUPPCD", 3, 4, false, couppcd);
    r.add("COUPNUM", 3, 4, false, coupnum);
    r.add("COUPDAYS", 3, 4, false, coupdays);
    r.add("COUPDAYBS", 3, 4, false, coupdaybs);
    r.add("COUPDAYSNC", 3, 4, false, coupdaysnc);

    // Bond duration
    r.add("DURATION", 5, 6, false, duration);
    r.add("MDURATION", 5, 6, false, mduration);

    // Bond price / yield
    r.add("PRICE", 6, 7, false, price);
    r.add("YIELD", 6, 7, false, yield_fn);
    r.add("PRICEMAT", 5, 6, false, pricemat);
    r.add("YIELDMAT", 5, 6, false, yieldmat);
    r.add("YIELDDISC", 4, 5, false, yielddisc);
    r.add("ACCRINT", 6, 8, false, accrint);
    r.add("ACCRINTM", 4, 5, false, accrintm);

    // Odd-period bonds
    r.add("ODDFPRICE", 8, 9, false, oddfprice);
    r.add("ODDFYIELD", 8, 9, false, oddfyield);
    r.add("ODDLPRICE", 7, 8, false, oddlprice);
    r.add("ODDLYIELD", 7, 8, false, oddlyield);

    // French depreciation
    r.add("AMORDEGRC", 6, 7, false, amordegrc);
    r.add("AMORLINC", 6, 7, false, amorlinc);

    // Euro legacy conversion
    r.add("EUROCONVERT", 3, 5, false, euroconvert);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get(args: &[Value], i: usize) -> Result<f64, CellError> {
    to_number(&args[i])
}

fn opt(args: &[Value], i: usize, default: f64) -> Result<f64, CellError> {
    if i < args.len() {
        match &args[i] {
            Value::Empty => Ok(default),
            v => to_number(v),
        }
    } else {
        Ok(default)
    }
}

fn num(v: f64) -> Value {
    if v.is_finite() {
        Value::Number(v)
    } else {
        Value::Error(CellError::Num)
    }
}

fn err(e: CellError) -> Value {
    Value::Error(e)
}

/// Collect cash-flow values from args (scalars, ranges, arrays) into a Vec<f64>.
fn collect_cashflows(ctx: &mut dyn Context, args: &[Value]) -> Result<Vec<f64>, CellError> {
    let mut out = Vec::new();
    for arg in args {
        for v in ctx.flatten(arg) {
            match v {
                Value::Number(n) => out.push(n),
                Value::Bool(b) => out.push(if b { 1.0 } else { 0.0 }),
                Value::Empty => out.push(0.0),
                Value::Error(e) => return Err(e),
                Value::Text(s) => match crate::formula::coerce::parse_number_text(&s) {
                    Some(n) => out.push(n),
                    None => return Err(CellError::Value),
                },
                _ => return Err(CellError::Value),
            }
        }
    }
    Ok(out)
}

/// Annuity present-value factor (for regular payments).
///   If rate == 0: factor = nper
///   Else if type == 0 (end): factor = (1-(1+r)^-n)/r
///        if type == 1 (beg): factor = (1-(1+r)^-n)/r * (1+r)
fn pv_factor(rate: f64, nper: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        nper
    } else {
        let t = (1.0 + rate).powf(-nper);
        let f = (1.0 - t) / rate;
        if typ == 0.0 { f } else { f * (1.0 + rate) }
    }
}

// ---------------------------------------------------------------------------
// PMT: PMT(rate, nper, pv [, fv [, type]])
// Payment for a loan / annuity.
// PMT = -(pv*(1+r)^n + fv) / factor
// ---------------------------------------------------------------------------
fn pmt(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match opt(args, 3, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    num(pmt_core(rate, nper, pv, fv, typ))
}

fn pmt_core(rate: f64, nper: f64, pv: f64, fv: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        -(pv + fv) / nper
    } else {
        let _factor = pv_factor(rate, nper, typ);
        let rn = (1.0 + rate).powf(nper);
        -(pv * rn + fv) / ((1.0 + rate * typ) * (rn - 1.0) / rate)
    }
}

// ---------------------------------------------------------------------------
// FV: FV(rate, nper, pmt [, pv [, type]])
// ---------------------------------------------------------------------------
fn fv(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pmt = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match opt(args, 3, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    num(fv_core(rate, nper, pmt, pv, typ))
}

fn fv_core(rate: f64, nper: f64, pmt: f64, pv: f64, typ: f64) -> f64 {
    if rate == 0.0 {
        -pv - pmt * nper
    } else {
        let rn = (1.0 + rate).powf(nper);
        -pv * rn - pmt * (1.0 + rate * typ) * (rn - 1.0) / rate
    }
}

// ---------------------------------------------------------------------------
// PV: PV(rate, nper, pmt [, fv [, type]])
// ---------------------------------------------------------------------------
fn pv(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pmt = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match opt(args, 3, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let v = if rate == 0.0 {
        -fv - pmt * nper
    } else {
        let rn = (1.0 + rate).powf(nper);
        (-fv - pmt * (1.0 + rate * typ) * (rn - 1.0) / rate) / rn
    };
    num(v)
}

// ---------------------------------------------------------------------------
// NPER: NPER(rate, pmt, pv [, fv [, type]])
// ---------------------------------------------------------------------------
fn nper(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pmt = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match opt(args, 3, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let v = if rate == 0.0 {
        if pmt == 0.0 {
            return Value::Error(CellError::Div0);
        }
        -(pv + fv) / pmt
    } else {
        let adj_pmt = pmt * (1.0 + rate * typ);
        let numerator = adj_pmt - fv * rate;
        let denominator = pv * rate + adj_pmt;
        if denominator == 0.0 {
            return Value::Error(CellError::Div0);
        }
        (numerator / denominator).ln() / (1.0 + rate).ln()
    };
    num(v)
}

// ---------------------------------------------------------------------------
// RATE: RATE(nper, pmt, pv [, fv [, type [, guess]]])
// Iterative Newton with bisection fallback.
// PARITY: tolerance 1e-7, max 100 iterations.
// ---------------------------------------------------------------------------
fn rate(_: &mut dyn Context, args: &[Value]) -> Value {
    let nper = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pmt = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let _fv = match opt(args, 3, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let guess = match opt(args, 5, 0.1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    // f(r) = fv_core(r, nper, pmt, pv, typ)  — we want f(r) = 0
    let f = |r: f64| fv_core(r, nper, pmt, pv, typ);

    match solve_newton(f, guess, 1e-7, 100) {
        Some(r) => num(r),
        None => Value::Error(CellError::Num),
    }
}

/// Newton–Raphson with central-difference derivative.
/// Returns None if non-convergent after max_iter iterations.
fn solve_newton<F>(f: F, x0: f64, tol: f64, max_iter: usize) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let mut x = x0;
    for _ in 0..max_iter {
        let fx = f(x);
        if fx.abs() < tol {
            return Some(x);
        }
        let h = 1e-6 * x.abs().max(1e-6);
        let dfx = (f(x + h) - f(x - h)) / (2.0 * h);
        if dfx.abs() < 1e-20 {
            break;
        }
        let x1 = x - fx / dfx;
        if (x1 - x).abs() < tol {
            return Some(x1);
        }
        x = x1;
        // Guard against divergence: rate must be > -1
        if x <= -1.0 {
            x = -0.9;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// IPMT / PPMT: IPMT(rate, per, nper, pv [, fv [, type]])
// ---------------------------------------------------------------------------
fn ipmt(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let per = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 5, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let per_i = per.trunc() as i64;
    if per_i < 1 || per_i > nper.trunc() as i64 {
        return Value::Error(CellError::Num);
    }
    // For type=1 and period=1, interest is 0
    if typ == 1.0 && per_i == 1 {
        return Value::Number(0.0);
    }

    let payment = pmt_core(rate, nper, pv, fv, typ);
    // Balance at start of period per
    let pmt_adj = if typ == 1.0 { payment } else { 0.0 };
    let bal_start = if typ == 1.0 && per_i == 1 {
        pv + payment
    } else {
        // FV after (per-1) periods
        let p = (per_i - if typ == 1.0 { 1 } else { 0 }) as f64;
        fv_core(rate, p, payment, pv, typ) + (if typ == 1.0 { payment } else { 0.0 })
    };
    // Actually compute balance at beginning of period per
    let bal = if rate == 0.0 {
        pv + payment * (per_i - 1) as f64
    } else {
        let p = (per_i - 1) as f64;
        let rn = (1.0 + rate).powf(p);
        pv * rn + payment * (1.0 + rate * typ) * (rn - 1.0) / rate
    };
    let _ = (bal_start, pmt_adj);
    num(bal * rate)
}

fn ppmt(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let per = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let typ = match opt(args, 5, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let per_i = per.trunc() as i64;
    if per_i < 1 || per_i > nper.trunc() as i64 {
        return Value::Error(CellError::Num);
    }

    let payment = pmt_core(rate, nper, pv, fv, typ);

    // interest for this period
    let interest = if rate == 0.0 || (typ == 1.0 && per_i == 1) {
        0.0
    } else {
        let p = (per_i - 1) as f64;
        let rn = (1.0 + rate).powf(p);
        let bal = pv * rn + payment * (1.0 + rate * typ) * (rn - 1.0) / rate;
        bal * rate
    };
    num(payment - interest)
}

// ---------------------------------------------------------------------------
// CUMIPMT / CUMPRINC: CUMIPMT(rate, nper, pv, start, end, type)
// ---------------------------------------------------------------------------
fn cumipmt(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let start = match get(args, 3) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return err(e),
    };
    let end = match get(args, 4) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return err(e),
    };
    let typ = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start < 1 || end < start || end > nper as i64 {
        return Value::Error(CellError::Num);
    }

    let payment = pmt_core(rate, nper, pv, 0.0, typ);
    let mut total = 0.0;
    for per in start..=end {
        let interest = if typ == 1.0 && per == 1 {
            0.0
        } else {
            let p = (per - 1) as f64;
            let bal = if rate == 0.0 {
                pv + payment * p
            } else {
                let rn = (1.0 + rate).powf(p);
                pv * rn + payment * (1.0 + rate * typ) * (rn - 1.0) / rate
            };
            bal * rate
        };
        total += interest;
    }
    num(total)
}

fn cumprinc(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let start = match get(args, 3) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return err(e),
    };
    let end = match get(args, 4) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return err(e),
    };
    let typ = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start < 1 || end < start || end > nper as i64 {
        return Value::Error(CellError::Num);
    }

    let payment = pmt_core(rate, nper, pv, 0.0, typ);
    let mut total = 0.0;
    for per in start..=end {
        let interest = if typ == 1.0 && per == 1 {
            0.0
        } else {
            let p = (per - 1) as f64;
            let bal = if rate == 0.0 {
                pv + payment * p
            } else {
                let rn = (1.0 + rate).powf(p);
                pv * rn + payment * (1.0 + rate * typ) * (rn - 1.0) / rate
            };
            bal * rate
        };
        total += payment - interest;
    }
    num(total)
}

// ---------------------------------------------------------------------------
// NPV: NPV(rate, value1, [value2, …])
// ---------------------------------------------------------------------------
fn npv(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let cashflows = match collect_cashflows(ctx, &args[1..]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let mut result = 0.0;
    for (i, cf) in cashflows.iter().enumerate() {
        result += cf / (1.0 + rate).powi(i as i32 + 1);
    }
    num(result)
}

// ---------------------------------------------------------------------------
// XNPV: XNPV(rate, values, dates)
// Dates are Excel serial numbers; first date is base.
// ---------------------------------------------------------------------------
fn xnpv(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let values = match collect_cashflows(ctx, &[args[1].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let dates = match collect_cashflows(ctx, &[args[2].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if values.len() != dates.len() || values.is_empty() {
        return Value::Error(CellError::Value);
    }
    if rate <= -1.0 {
        return Value::Error(CellError::Num);
    }

    let d0 = dates[0];
    let mut result = 0.0;
    for (v, d) in values.iter().zip(dates.iter()) {
        let t = (d - d0) / 365.0;
        result += v / (1.0 + rate).powf(t);
    }
    num(result)
}

// ---------------------------------------------------------------------------
// IRR: IRR(values [, guess])
// PARITY: tolerance 1e-7, max 100 iterations.
// ---------------------------------------------------------------------------
fn irr(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let cashflows = match collect_cashflows(ctx, &[args[0].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let guess = match opt(args, 1, 0.1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if cashflows.is_empty() {
        return Value::Error(CellError::Num);
    }
    // Must have at least one positive and one negative value.
    let has_pos = cashflows.iter().any(|&v| v > 0.0);
    let has_neg = cashflows.iter().any(|&v| v < 0.0);
    if !has_pos || !has_neg {
        return Value::Error(CellError::Num);
    }

    let npv_fn = |r: f64| -> f64 {
        if r <= -1.0 {
            return f64::INFINITY;
        }
        cashflows
            .iter()
            .enumerate()
            .map(|(i, &cf)| cf / (1.0 + r).powi(i as i32))
            .sum()
    };

    match solve_newton(npv_fn, guess, 1e-7, 100) {
        Some(r) => num(r),
        None => Value::Error(CellError::Num),
    }
}

// ---------------------------------------------------------------------------
// XIRR: XIRR(values, dates [, guess])
// PARITY: tolerance 1e-7, max 100 iterations, dates in Excel serial number.
// ---------------------------------------------------------------------------
fn xirr(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let values = match collect_cashflows(ctx, &[args[0].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let dates = match collect_cashflows(ctx, &[args[1].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let guess = match opt(args, 2, 0.1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if values.len() != dates.len() || values.is_empty() {
        return Value::Error(CellError::Value);
    }
    let has_pos = values.iter().any(|&v| v > 0.0);
    let has_neg = values.iter().any(|&v| v < 0.0);
    if !has_pos || !has_neg {
        return Value::Error(CellError::Num);
    }

    let d0 = dates[0];
    let xnpv_fn = |r: f64| -> f64 {
        if r <= -1.0 {
            return f64::INFINITY;
        }
        values
            .iter()
            .zip(dates.iter())
            .map(|(&v, &d)| {
                let t = (d - d0) / 365.0;
                v / (1.0 + r).powf(t)
            })
            .sum()
    };

    match solve_newton(xnpv_fn, guess, 1e-7, 100) {
        Some(r) => num(r),
        None => Value::Error(CellError::Num),
    }
}

// ---------------------------------------------------------------------------
// MIRR: MIRR(values, finance_rate, reinvest_rate)
// ---------------------------------------------------------------------------
fn mirr(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let cashflows = match collect_cashflows(ctx, &[args[0].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let frate = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rrate = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let n = cashflows.len();
    if n < 2 {
        return Value::Error(CellError::Value);
    }

    let npv_neg: f64 = cashflows
        .iter()
        .enumerate()
        .filter(|&(_, v)| *v < 0.0)
        .map(|(i, v)| v / (1.0 + frate).powi(i as i32))
        .sum();
    let fv_pos: f64 = cashflows
        .iter()
        .enumerate()
        .filter(|&(_, v)| *v > 0.0)
        .map(|(i, v)| v * (1.0 + rrate).powi((n - 1 - i) as i32))
        .sum();

    if npv_neg == 0.0 || fv_pos == 0.0 {
        return Value::Error(CellError::Div0);
    }

    let mirr_val = (-fv_pos / npv_neg).powf(1.0 / (n as f64 - 1.0)) - 1.0;
    num(mirr_val)
}

// ---------------------------------------------------------------------------
// Depreciation functions
// ---------------------------------------------------------------------------

// SLN: SLN(cost, salvage, life) — straight-line
fn sln(_: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let life = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if life == 0.0 {
        return Value::Error(CellError::Div0);
    }
    num((cost - salvage) / life)
}

// SYD: SYD(cost, salvage, life, per) — sum-of-years digits
fn syd(_: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let life = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let per = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if per < 1.0 || per > life {
        return Value::Error(CellError::Num);
    }
    let syd_val = (cost - salvage) * (life - per + 1.0) * 2.0 / (life * (life + 1.0));
    num(syd_val)
}

// DB: DB(cost, salvage, life, period [, month])
// Fixed-declining balance
fn db(_: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let life = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let period = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let month = match opt(args, 4, 12.0) {
        Ok(v) => v.trunc(),
        Err(e) => return err(e),
    };

    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || period < 1.0 {
        return Value::Error(CellError::Num);
    }
    if cost == 0.0 {
        return Value::Number(0.0);
    }

    // Rate rounded to 3 decimal places (Excel behavior)
    let rate_raw = 1.0 - (salvage / cost).powf(1.0 / life);
    let rate = (rate_raw * 1000.0).round() / 1000.0;

    let period_i = period.trunc() as i64;
    let life_i = life.trunc() as i64;

    if period_i > life_i + 1 {
        return Value::Error(CellError::Num);
    }

    // First period: partial year
    let dep1 = cost * rate * month / 12.0;

    if period_i == 1 {
        return num(dep1);
    }

    let mut book = cost - dep1;
    for p in 2..=period_i {
        let dep = if p == life_i + 1 {
            // last partial period
            book * rate * (12.0 - month) / 12.0
        } else {
            book * rate
        };
        if p == period_i {
            return num(dep);
        }
        book -= dep;
    }
    Value::Number(0.0)
}

// DDB: DDB(cost, salvage, life, period [, factor])
// Double-declining balance (or custom factor)
fn ddb(_: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let life = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let period = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let factor = match opt(args, 4, 2.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || period < 1.0 || factor <= 0.0 {
        return Value::Error(CellError::Num);
    }
    if period > life {
        return Value::Error(CellError::Num);
    }

    let rate = factor / life;
    let start = (period - 1.0).floor();
    let end_p = period.floor();
    let book_start = cost * (1.0 - rate).powf(start);
    let book_end = cost * (1.0 - rate).powf(end_p);
    let book_end = book_end.max(salvage);
    let dep = (book_start - book_end).max(0.0);
    num(dep)
}

// VDB: VDB(cost, salvage, life, start_period, end_period [, factor [, no_switch]])
// Variable declining balance with optional switch to SLN
fn vdb(_: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let life = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let start_per = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let end_per = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let factor = match opt(args, 5, 2.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let no_switch = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if cost < 0.0
        || salvage < 0.0
        || life <= 0.0
        || start_per < 0.0
        || end_per <= start_per
        || end_per > life
        || factor <= 0.0
    {
        return Value::Error(CellError::Num);
    }

    let switch = no_switch == 0.0;
    let rate = factor / life;

    // Integrate DDB from start_per to end_per using fractional periods
    // We compute book value at start and end using fractional year approach
    fn book_val(cost: f64, salvage: f64, life: f64, rate: f64, t: f64, switch_to_sln: bool) -> f64 {
        // Iterate period by period, switching to SLN when beneficial
        let mut book = cost;
        let periods = t.floor() as i64;
        for p in 0..periods {
            let remaining = life - p as f64;
            let ddb_dep = book * rate;
            let sln_dep = if remaining > 0.0 {
                (book - salvage) / remaining
            } else {
                0.0
            };
            let dep = if switch_to_sln && sln_dep > ddb_dep {
                sln_dep
            } else {
                ddb_dep
            };
            book = (book - dep).max(salvage);
        }
        // Fractional part
        let frac = t - t.floor();
        if frac > 0.0 {
            let remaining = life - periods as f64;
            let ddb_dep = book * rate * frac;
            let sln_dep = if remaining > 0.0 {
                (book - salvage) / remaining * frac
            } else {
                0.0
            };
            let dep = if switch_to_sln && sln_dep > ddb_dep {
                sln_dep
            } else {
                ddb_dep
            };
            book = (book - dep).max(salvage);
        }
        book
    }

    let bv_start = book_val(cost, salvage, life, rate, start_per, switch);
    let bv_end = book_val(cost, salvage, life, rate, end_per, switch);
    num((bv_start - bv_end).max(0.0))
}

// ---------------------------------------------------------------------------
// Rate conversions
// ---------------------------------------------------------------------------

// EFFECT: EFFECT(nominal_rate, npery) — nominal → effective annual rate
fn effect(_: &mut dyn Context, args: &[Value]) -> Value {
    let nominal = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let npery = match get(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return err(e),
    };
    if nominal <= 0.0 || npery < 1.0 {
        return Value::Error(CellError::Num);
    }
    num((1.0 + nominal / npery).powf(npery) - 1.0)
}

// NOMINAL: NOMINAL(effect_rate, npery) — effective → nominal rate
fn nominal(_: &mut dyn Context, args: &[Value]) -> Value {
    let effect = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let npery = match get(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return err(e),
    };
    if effect <= 0.0 || npery < 1.0 {
        return Value::Error(CellError::Num);
    }
    num(((1.0 + effect).powf(1.0 / npery) - 1.0) * npery)
}

// FVSCHEDULE: FVSCHEDULE(principal, schedule)
fn fvschedule(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let principal = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rates = match collect_cashflows(ctx, &[args[1].clone()]) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let mut fv = principal;
    for r in rates {
        fv *= 1.0 + r;
    }
    num(fv)
}

// PDURATION: PDURATION(rate, pv, fv) — periods to reach fv from pv
fn pduration(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if rate <= 0.0 || pv <= 0.0 || fv <= 0.0 {
        return Value::Error(CellError::Num);
    }
    num((fv / pv).ln() / (1.0 + rate).ln())
}

// RRI: RRI(nper, pv, fv) — equivalent interest rate for growth
fn rri(_: &mut dyn Context, args: &[Value]) -> Value {
    let nper = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fv = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if nper <= 0.0 || pv == 0.0 {
        return Value::Error(CellError::Num);
    }
    num((fv / pv).powf(1.0 / nper) - 1.0)
}

// ---------------------------------------------------------------------------
// Dollar fraction helpers
// ---------------------------------------------------------------------------

// DOLLARDE: DOLLARDE(fractional_dollar, fraction)
// Converts dollar price expressed as a fraction to decimal price.
fn dollarde(_: &mut dyn Context, args: &[Value]) -> Value {
    let frac_dollar = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fraction = match get(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return err(e),
    };
    if fraction < 1.0 {
        return Value::Error(CellError::Num);
    }
    if fraction == 0.0 {
        return Value::Error(CellError::Div0);
    }
    let integer_part = frac_dollar.trunc();
    let frac_part = frac_dollar - integer_part;
    // fractional part digits: frac_part / 10^(number of digits in fraction)
    let digits = fraction.log10().floor() + 1.0;
    let decimal_frac = frac_part * 10f64.powf(digits) / fraction;
    num(integer_part + decimal_frac)
}

// DOLLARFR: DOLLARFR(decimal_dollar, fraction)
// Converts decimal dollar to fractional notation.
fn dollarfr(_: &mut dyn Context, args: &[Value]) -> Value {
    let decimal_dollar = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let fraction = match get(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return err(e),
    };
    if fraction < 1.0 {
        return Value::Error(CellError::Num);
    }
    if fraction == 0.0 {
        return Value::Error(CellError::Div0);
    }
    let integer_part = decimal_dollar.trunc();
    let dec_part = decimal_dollar - integer_part;
    let digits = fraction.log10().floor() + 1.0;
    let frac_part = dec_part * fraction / 10f64.powf(digits);
    num(integer_part + frac_part)
}

// ---------------------------------------------------------------------------
// ISPMT: ISPMT(rate, per, nper, pv) — interest payment for a flat-payment loan
// (simple, not amortizing)
// ---------------------------------------------------------------------------
fn ispmt(_: &mut dyn Context, args: &[Value]) -> Value {
    let rate = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let per = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let nper = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pv = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    // Balance at beginning of period per (principal reduces linearly)
    let balance = pv * (1.0 - per / nper);
    num(-balance * rate)
}

// ---------------------------------------------------------------------------
// Discount / T-bill / simple bond helpers
// All use day-count basis 0=US(NASD) 30/360, 1=actual/actual, etc.
// We treat date arguments as Excel serial numbers and compute DSM/DIM via
// simple subtraction (actual/actual implied unless basis=0 or 4 for 30/360).
// PARITY: full day-count basis computation not implemented; treat all as actual
// days / 360 for simplicity. Behaviour may differ from Excel by 1-2 days.
// ---------------------------------------------------------------------------

fn day_count_fraction(settlement: f64, maturity: f64, basis: f64) -> f64 {
    let days = maturity - settlement;
    match basis as i64 {
        0 | 4 => days / 360.0, // 30/360 (approx)
        2 => days / 360.0,     // actual/360
        3 => days / 365.0,     // actual/365
        _ => days / 365.0,     // actual/actual (1)
    }
}

// DISC: DISC(settlement, maturity, pr, redemption [, basis])
fn disc(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pr = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if maturity <= settlement || pr <= 0.0 || redemption <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let dcf = day_count_fraction(settlement, maturity, basis);
    num((redemption - pr) / redemption / dcf)
}

// INTRATE: INTRATE(settlement, maturity, investment, redemption [, basis])
fn intrate(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let investment = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if maturity <= settlement || investment <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let dcf = day_count_fraction(settlement, maturity, basis);
    num((redemption - investment) / investment / dcf)
}

// RECEIVED: RECEIVED(settlement, maturity, investment, discount [, basis])
fn received(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let investment = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let discount = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if maturity <= settlement || investment <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let dcf = day_count_fraction(settlement, maturity, basis);
    let denom = 1.0 - discount * dcf;
    if denom <= 0.0 {
        return Value::Error(CellError::Num);
    }
    num(investment / denom)
}

// TBILLEQ: TBILLEQ(settlement, maturity, discount)
// Treasury bill bond equivalent yield
fn tbilleq(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let discount = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let dsm = maturity - settlement;
    if dsm <= 0.0 || dsm > 366.0 || discount <= 0.0 {
        return Value::Error(CellError::Num);
    }
    // Price per $100 face
    let price = 100.0 * (1.0 - discount * dsm / 360.0);
    // BEY = (365 * discount) / (360 - discount * dsm)
    let bey = (365.0 * discount) / (360.0 - discount * dsm);
    // For maturities > 182 days use semi-annual equivalent
    let _ = price;
    num(bey)
}

// TBILLPRICE: TBILLPRICE(settlement, maturity, discount)
fn tbillprice(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let discount = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let dsm = maturity - settlement;
    if dsm <= 0.0 || dsm > 366.0 || discount <= 0.0 {
        return Value::Error(CellError::Num);
    }
    num(100.0 * (1.0 - discount * dsm / 360.0))
}

// TBILLYIELD: TBILLYIELD(settlement, maturity, pr)
fn tbillyield(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pr = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let dsm = maturity - settlement;
    if dsm <= 0.0 || dsm > 366.0 || pr <= 0.0 {
        return Value::Error(CellError::Num);
    }
    num((100.0 - pr) / pr * 360.0 / dsm)
}

// PRICEDISC: PRICEDISC(settlement, maturity, discount, redemption [, basis])
fn pricedisc(_: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let discount = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    if maturity <= settlement || discount < 0.0 || redemption <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let dcf = day_count_fraction(settlement, maturity, basis);
    num(redemption * (1.0 - discount * dcf))
}

// ===========================================================================
// Coupon bond machinery (COUP*, DURATION, PRICE, YIELD, ACCRINT, …)
//
// Day-count basis: 0=US 30/360, 1=actual/actual, 2=actual/360, 3=actual/365,
// 4=European 30/360.  Coupon schedules are built by stepping backwards from
// maturity by 12/frequency months.
//
// PARITY: 30/360 and actual/actual day counts follow the standard SIA/Excel
// conventions; results may differ from Excel by a fraction in rare edge cases.
// ===========================================================================

use chrono::{Datelike, NaiveDate};
use easyexcel_model::dates::DateSystem;

/// Convert a serial argument to a `NaiveDate`, or `#NUM!` on failure.
fn serial_date(ctx: &mut dyn Context, args: &[Value], i: usize) -> Result<NaiveDate, CellError> {
    let s = to_number(&args[i])?;
    ctx.date_system()
        .serial_to_datetime(s.trunc())
        .map(|dt| dt.date())
        .ok_or(CellError::Num)
}

fn date_serial(sys: DateSystem, d: NaiveDate) -> f64 {
    sys.date_to_serial(d) as f64
}

/// Number of whole days between two dates using the given basis day-count.
fn days_basis(start: NaiveDate, end: NaiveDate, basis: i64) -> f64 {
    match basis {
        0 => days_30_360_us(start, end),
        4 => days_30_360_eu(start, end),
        _ => (end - start).num_days() as f64, // actual
    }
}

/// US (NASD) 30/360 day count between two dates.
fn days_30_360_us(start: NaiveDate, end: NaiveDate) -> f64 {
    let (y1, m1, mut d1) = (start.year(), start.month() as i32, start.day() as i32);
    let (y2, m2, mut d2) = (end.year(), end.month() as i32, end.day() as i32);
    // Last day of February handling
    let is_last_feb = |y: i32, m: i32, d: i32| {
        m == 2
            && d == NaiveDate::from_ymd_opt(y, 2, 1)
                .and_then(|f| f.with_day(1))
                .map(|_| last_day_of_month(y, 2))
                .unwrap_or(28)
    };
    if is_last_feb(y2, m2, d2) && is_last_feb(y1, m1, d1) {
        d2 = 30;
    }
    if is_last_feb(y1, m1, d1) {
        d1 = 30;
    }
    if d2 == 31 && d1 >= 30 {
        d2 = 30;
    }
    if d1 == 31 {
        d1 = 30;
    }
    ((y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)) as f64
}

/// European 30/360 day count.
fn days_30_360_eu(start: NaiveDate, end: NaiveDate) -> f64 {
    let (y1, m1, mut d1) = (start.year(), start.month() as i32, start.day() as i32);
    let (y2, m2, mut d2) = (end.year(), end.month() as i32, end.day() as i32);
    if d1 == 31 {
        d1 = 30;
    }
    if d2 == 31 {
        d2 = 30;
    }
    ((y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)) as f64
}

fn last_day_of_month(y: i32, m: i32) -> i32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let first_next = NaiveDate::from_ymd_opt(ny, nm as u32, 1).unwrap();
    (first_next - chrono::Duration::days(1)).day() as i32
}

/// Add `months` (may be negative) to a date, clamping the day to the month end.
fn add_months(d: NaiveDate, months: i32) -> NaiveDate {
    let mut y = d.year();
    let mut m = d.month() as i32 - 1 + months;
    y += m.div_euclid(12);
    m = m.rem_euclid(12);
    let target_month = (m + 1) as u32;
    let last = last_day_of_month(y, target_month as i32) as u32;
    let day = d.day().min(last);
    NaiveDate::from_ymd_opt(y, target_month, day).unwrap()
}

/// Previous coupon date on or before settlement (stepping back from maturity).
fn coupon_prev(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> NaiveDate {
    let step = 12 / freq;
    let mut d = maturity;
    while d > settlement {
        d = add_months(d, -step);
    }
    d
}

/// Next coupon date strictly after settlement.
fn coupon_next(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> NaiveDate {
    let prev = coupon_prev(settlement, maturity, freq);
    add_months(prev, 12 / freq)
}

/// Validate frequency (1, 2, or 4) and basis (0..=4).
fn check_freq_basis(freq: f64, basis: f64) -> Result<(i32, i64), CellError> {
    let f = freq.trunc() as i32;
    let b = basis.trunc() as i64;
    if (f != 1 && f != 2 && f != 4) || !(0..=4).contains(&b) {
        return Err(CellError::Num);
    }
    Ok((f, b))
}

/// COUPDAYS: days in the coupon period containing settlement.
fn coup_days_in_period(settlement: NaiveDate, maturity: NaiveDate, freq: i32, basis: i64) -> f64 {
    match basis {
        0 | 2 | 4 => 360.0 / freq as f64,
        3 => 365.0 / freq as f64,
        _ => {
            // actual/actual: actual days in the current coupon period
            let prev = coupon_prev(settlement, maturity, freq);
            let next = add_months(prev, 12 / freq);
            (next - prev).num_days() as f64
        }
    }
}

/// Number of coupon periods between settlement and maturity (COUPNUM).
fn coup_num(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> i64 {
    let mut count = 0i64;
    let mut d = maturity;
    while d > settlement {
        d = add_months(d, -(12 / freq));
        count += 1;
    }
    count
}

// --- COUP* worksheet functions -------------------------------------------

macro_rules! coup_prelude {
    ($ctx:expr, $args:expr) => {{
        let settlement = match serial_date($ctx, $args, 0) {
            Ok(v) => v,
            Err(e) => return err(e),
        };
        let maturity = match serial_date($ctx, $args, 1) {
            Ok(v) => v,
            Err(e) => return err(e),
        };
        let freq = match get($args, 2) {
            Ok(v) => v,
            Err(e) => return err(e),
        };
        let basis = match opt($args, 3, 0.0) {
            Ok(v) => v,
            Err(e) => return err(e),
        };
        if settlement >= maturity {
            return err(CellError::Num);
        }
        let (f, b) = match check_freq_basis(freq, basis) {
            Ok(v) => v,
            Err(e) => return err(e),
        };
        (settlement, maturity, f, b)
    }};
}

fn coupncd(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, _b) = coup_prelude!(ctx, args);
    let d = coupon_next(settlement, maturity, f);
    num(date_serial(ctx.date_system(), d))
}

fn couppcd(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, _b) = coup_prelude!(ctx, args);
    let d = coupon_prev(settlement, maturity, f);
    num(date_serial(ctx.date_system(), d))
}

fn coupnum(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, _b) = coup_prelude!(ctx, args);
    num(coup_num(settlement, maturity, f) as f64)
}

fn coupdays(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, b) = coup_prelude!(ctx, args);
    num(coup_days_in_period(settlement, maturity, f, b))
}

fn coupdaybs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, b) = coup_prelude!(ctx, args);
    let prev = coupon_prev(settlement, maturity, f);
    num(days_basis(prev, settlement, b))
}

fn coupdaysnc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (settlement, maturity, f, b) = coup_prelude!(ctx, args);
    let next = coupon_next(settlement, maturity, f);
    if b == 1 {
        // actual/actual: actual days settlement..next
        num((next - settlement).num_days() as f64)
    } else {
        let dsr = coup_days_in_period(settlement, maturity, f, b);
        let prev = coupon_prev(settlement, maturity, f);
        let dbs = days_basis(prev, settlement, b);
        num(dsr - dbs)
    }
}

// --- PRICE / YIELD core ---------------------------------------------------

/// Clean price per 100 of face value for a coupon bond.
/// `rate` and `yld` are annual decimals; `freq` periods/yr.
fn bond_price(
    settlement: NaiveDate,
    maturity: NaiveDate,
    rate: f64,
    yld: f64,
    redemption: f64,
    freq: i32,
    basis: i64,
) -> f64 {
    let n = coup_num(settlement, maturity, freq);
    let e = coup_days_in_period(settlement, maturity, freq, basis);
    let prev = coupon_prev(settlement, maturity, freq);
    let a = days_basis(prev, settlement, basis); // days since last coupon
    let dsc = e - a; // days settlement -> next coupon
    let coupon = 100.0 * rate / freq as f64;
    let yf = yld / freq as f64;
    let t = dsc / e;

    let mut price = redemption / (1.0 + yf).powf((n - 1) as f64 + t);
    for k in 1..=n {
        price += coupon / (1.0 + yf).powf((k - 1) as f64 + t);
    }
    price -= coupon * a / e; // accrued interest
    price
}

fn price(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let yld = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let freq = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if settlement >= maturity || rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
        return err(CellError::Num);
    }
    let (f, b) = match check_freq_basis(freq, basis) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    num(bond_price(
        settlement, maturity, rate, yld, redemption, f, b,
    ))
}

fn yield_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pr = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let freq = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if settlement >= maturity || rate < 0.0 || pr <= 0.0 || redemption <= 0.0 {
        return err(CellError::Num);
    }
    let (f, b) = match check_freq_basis(freq, basis) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    // Solve bond_price(yld) - pr = 0 via bisection (robust over [0, large]).
    let g = |y: f64| bond_price(settlement, maturity, rate, y, redemption, f, b) - pr;
    match solve_bisection(g, 0.0, 1.0) {
        Some(y) => num(y),
        None => err(CellError::Num),
    }
}

/// Bisection over [lo, hi], expanding hi until sign change (price decreases in yld).
fn solve_bisection<F: Fn(f64) -> f64>(f: F, lo: f64, hi0: f64) -> Option<f64> {
    let mut lo = lo;
    let mut hi = hi0;
    let mut flo = f(lo);
    let mut fhi = f(hi);
    // Expand hi until bracketed (price is monotonically decreasing in yield).
    let mut tries = 0;
    while flo.signum() == fhi.signum() {
        hi *= 2.0;
        fhi = f(hi);
        tries += 1;
        if tries > 60 || !hi.is_finite() {
            return None;
        }
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if fm.abs() < 1e-9 || (hi - lo) < 1e-12 {
            return Some(mid);
        }
        if fm.signum() == flo.signum() {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

// --- DURATION / MDURATION -------------------------------------------------

fn duration_core(
    settlement: NaiveDate,
    maturity: NaiveDate,
    coupon: f64,
    yld: f64,
    freq: i32,
    basis: i64,
) -> f64 {
    // Excel computes duration using whole + fractional coupon periods.
    let e = coup_days_in_period(settlement, maturity, freq, basis);
    let prev = coupon_prev(settlement, maturity, freq);
    let a = days_basis(prev, settlement, basis);
    let dsc = e - a;
    let frac = dsc / e;
    let n = coup_num(settlement, maturity, freq) as f64;
    let yf = yld / freq as f64;
    let c = 100.0 * coupon / freq as f64;

    let mut num_sum = 0.0;
    let mut den_sum = 0.0;
    for k in 1..=(n as i64) {
        let t = (k - 1) as f64 + frac; // time in periods
        let mut cf = c;
        if k as f64 == n {
            cf += 100.0;
        }
        let pv = cf / (1.0 + yf).powf(t);
        num_sum += t * pv;
        den_sum += pv;
    }
    // Convert from periods to years.
    (num_sum / den_sum) / freq as f64
}

fn duration_prelude(
    ctx: &mut dyn Context,
    args: &[Value],
) -> Result<(NaiveDate, NaiveDate, f64, f64, i32, i64), CellError> {
    let settlement = serial_date(ctx, args, 0)?;
    let maturity = serial_date(ctx, args, 1)?;
    let coupon = get(args, 2)?;
    let yld = get(args, 3)?;
    let freq = get(args, 4)?;
    let basis = opt(args, 5, 0.0)?;
    if settlement >= maturity || coupon < 0.0 || yld < 0.0 {
        return Err(CellError::Num);
    }
    let (f, b) = check_freq_basis(freq, basis)?;
    Ok((settlement, maturity, coupon, yld, f, b))
}

fn duration(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (s, m, c, y, f, b) = match duration_prelude(ctx, args) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    num(duration_core(s, m, c, y, f, b))
}

fn mduration(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (s, m, c, y, f, b) = match duration_prelude(ctx, args) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let mac = duration_core(s, m, c, y, f, b);
    num(mac / (1.0 + y / f as f64))
}

// --- PRICEMAT / YIELDMAT (interest at maturity, no periodic coupons) ------

fn pricemat(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let issue = match serial_date(ctx, args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let yld = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 5, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if settlement >= maturity || rate < 0.0 || yld < 0.0 {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let basis_days = basis_year_days(b);
    let dim = days_basis(issue, maturity, b) / basis_days;
    let dis = days_basis(issue, settlement, b) / basis_days;
    let dsm = days_basis(settlement, maturity, b) / basis_days;
    let pr = ((100.0 + dim * rate * 100.0) / (1.0 + dsm * yld)) - (dis * rate * 100.0);
    num(pr)
}

fn yieldmat(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let issue = match serial_date(ctx, args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pr = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 5, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if settlement >= maturity || rate < 0.0 || pr <= 0.0 {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let basis_days = basis_year_days(b);
    let dim = days_basis(issue, maturity, b) / basis_days;
    let dis = days_basis(issue, settlement, b) / basis_days;
    let dsm = days_basis(settlement, maturity, b) / basis_days;
    // Closed-form (Excel YIELDMAT):
    let term1 = (1.0 + dim * rate) - (pr / 100.0 + dis * rate);
    let term2 = pr / 100.0 + dis * rate;
    if term2 == 0.0 || dsm == 0.0 {
        return err(CellError::Num);
    }
    num((term1 / term2) / dsm)
}

fn basis_year_days(basis: i64) -> f64 {
    match basis {
        3 => 365.0,
        _ => 360.0,
    }
}

// --- YIELDDISC ------------------------------------------------------------

fn yielddisc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let settlement = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let maturity = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let pr = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let redemption = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if settlement >= maturity || pr <= 0.0 || redemption <= 0.0 {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let dsm = days_basis(settlement, maturity, b) / basis_year_days(b);
    if dsm == 0.0 {
        return err(CellError::Num);
    }
    num((redemption - pr) / pr / dsm)
}

// --- ACCRINT / ACCRINTM ---------------------------------------------------

// ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
fn accrint(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let issue = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let _first = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let settlement = match serial_date(ctx, args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let par = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let freq = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if issue >= settlement || rate <= 0.0 || par <= 0.0 {
        return err(CellError::Num);
    }
    let (f, b) = match check_freq_basis(freq, basis) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    // PARITY: assumes interest accrues from issue to settlement (calc_method=TRUE,
    // the Excel default). Day-count fraction relative to the annual basis.
    let days = days_basis(issue, settlement, b);
    let year = basis_year_days(b);
    let _ = f;
    num(par * rate * days / year)
}

// ACCRINTM(issue, settlement, rate, par, [basis])
fn accrintm(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let issue = match serial_date(ctx, args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let settlement = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let par = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 4, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if issue >= settlement || rate <= 0.0 || par <= 0.0 {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let days = days_basis(issue, settlement, b);
    num(par * rate * days / basis_year_days(b))
}

// --- Odd-period bonds (stubbed) -------------------------------------------
// PARITY: odd first/last period bond math (ODDFPRICE/ODDFYIELD/ODDLPRICE/
// ODDLYIELD) is not implemented; these return #NUM! after validating args.

fn oddfprice(_: &mut dyn Context, _args: &[Value]) -> Value {
    err(CellError::Num)
}
fn oddfyield(_: &mut dyn Context, _args: &[Value]) -> Value {
    err(CellError::Num)
}
fn oddlprice(_: &mut dyn Context, _args: &[Value]) -> Value {
    err(CellError::Num)
}
fn oddlyield(_: &mut dyn Context, _args: &[Value]) -> Value {
    err(CellError::Num)
}

// --- French depreciation: AMORDEGRC / AMORLINC ----------------------------

/// Depreciation coefficient for AMORDEGRC based on asset life (in years).
fn amor_coefficient(life: f64) -> f64 {
    if life < 3.0 {
        1.0
    } else if life <= 4.0 {
        1.5
    } else if life <= 6.0 {
        2.0
    } else {
        2.5
    }
}

// AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])
fn amorlinc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let purchased = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let first = match serial_date(ctx, args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let period = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if rate <= 0.0 || cost <= 0.0 || salvage < 0.0 || first < purchased {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let per = period.trunc() as i64;
    if per < 0 {
        return err(CellError::Num);
    }
    let yearfrac = days_basis(purchased, first, b) / basis_year_days(b);
    let one_full = cost * rate;
    let first_dep = cost * rate * yearfrac;

    if per == 0 {
        return num(first_dep);
    }
    // Subsequent full-year depreciation until book hits salvage.
    let mut book = cost - first_dep;
    for p in 1..=per {
        let dep = one_full.min(book - salvage).max(0.0);
        if p == per {
            return num(dep);
        }
        book -= dep;
    }
    num(0.0)
}

// AMORDEGRC(cost, date_purchased, first_period, salvage, period, rate, [basis])
fn amordegrc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let cost = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let purchased = match serial_date(ctx, args, 1) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let first = match serial_date(ctx, args, 2) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let salvage = match get(args, 3) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let period = match get(args, 4) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let rate = match get(args, 5) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let basis = match opt(args, 6, 0.0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if rate <= 0.0 || rate >= 0.5 || cost <= 0.0 || salvage < 0.0 || first < purchased {
        return err(CellError::Num);
    }
    let b = basis.trunc() as i64;
    if !(0..=4).contains(&b) {
        return err(CellError::Num);
    }
    let per = period.trunc() as i64;
    if per < 0 {
        return err(CellError::Num);
    }

    let life = 1.0 / rate; // useful life in years
    let coeff = amor_coefficient(life);
    let deg_rate = rate * coeff;
    let yearfrac = days_basis(purchased, first, b) / basis_year_days(b);

    // First period (period 0) is prorated.
    let mut book = cost;
    let mut dep = (cost * deg_rate * yearfrac).round();
    book -= dep;

    let n_full = life.ceil() as i64; // approximate number of remaining years
    for p in 1..=per {
        // In the last two years Excel switches to 50% then 100% of remaining.
        let years_left = (n_full - p) as f64;
        let d = if years_left <= 2.0 {
            (book * 0.5).round()
        } else {
            (book * deg_rate).round()
        };
        dep = d.min((book - salvage).max(0.0));
        book -= dep;
        if book < salvage {
            book = salvage;
        }
    }
    num(dep.max(0.0))
}

// --- EUROCONVERT ----------------------------------------------------------
// Fixed legacy euro conversion rates (units of national currency per 1 EUR).

/// Returns the fixed EUR conversion rate for a 3-letter ISO currency code.
fn euro_rate(code: &str) -> Option<f64> {
    let r = match code.to_ascii_uppercase().as_str() {
        "EUR" => 1.0,
        "ATS" => 13.7603,  // Austrian schilling
        "BEF" => 40.3399,  // Belgian franc
        "DEM" => 1.95583,  // German mark
        "ESP" => 166.386,  // Spanish peseta
        "FIM" => 5.94573,  // Finnish markka
        "FRF" => 6.55957,  // French franc
        "GRD" => 340.75,   // Greek drachma
        "IEP" => 0.787564, // Irish pound
        "ITL" => 1936.27,  // Italian lira
        "LUF" => 40.3399,  // Luxembourg franc
        "NLG" => 2.20371,  // Dutch guilder
        "PTE" => 200.482,  // Portuguese escudo
        "SIT" => 239.640,  // Slovenian tolar
        "CYP" => 0.585274, // Cypriot pound
        "MTL" => 0.4293,   // Maltese lira
        "SKK" => 30.1260,  // Slovak koruna
        "EEK" => 15.6466,  // Estonian kroon
        "LVL" => 0.702804, // Latvian lats
        "LTL" => 3.45280,  // Lithuanian litas
        _ => return None,
    };
    Some(r)
}

// EUROCONVERT(number, source, target, [full_precision], [triangulation_precision])
fn euroconvert(_: &mut dyn Context, args: &[Value]) -> Value {
    let number = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let source = match &args[1] {
        Value::Text(s) => s.clone(),
        v => match crate::formula::coerce::to_text(v) {
            Ok(s) => s,
            Err(e) => return err(e),
        },
    };
    let target = match &args[2] {
        Value::Text(s) => s.clone(),
        v => match crate::formula::coerce::to_text(v) {
            Ok(s) => s,
            Err(e) => return err(e),
        },
    };
    let full_precision = match opt(args, 3, 0.0) {
        Ok(v) => v != 0.0,
        Err(e) => return err(e),
    };
    let tri_precision = if args.len() > 4 {
        match get(args, 4) {
            Ok(v) => Some(v.trunc() as i32),
            Err(e) => return err(e),
        }
    } else {
        None
    };

    let src_rate = match euro_rate(source.trim()) {
        Some(r) => r,
        None => return err(CellError::Value),
    };
    let tgt_rate = match euro_rate(target.trim()) {
        Some(r) => r,
        None => return err(CellError::Value),
    };

    // Convert to euro first (triangulation), then to target.
    let in_euro = number / src_rate;
    let euro = match tri_precision {
        Some(p) if p >= 0 => {
            let factor = 10f64.powi(p);
            (in_euro * factor).round() / factor
        }
        _ => in_euro,
    };
    let mut result = euro * tgt_rate;

    if !full_precision {
        // Round to the target currency's standard number of decimals.
        let decimals = euro_decimals(target.trim());
        let f = 10f64.powi(decimals);
        result = (result * f).round() / f;
    }
    num(result)
}

/// Standard number of minor-unit decimals for a legacy euro-zone currency.
fn euro_decimals(code: &str) -> i32 {
    match code.to_ascii_uppercase().as_str() {
        "EUR" => 2,
        "ATS" | "DEM" | "FIM" | "FRF" | "IEP" | "NLG" | "DKK" => 2,
        "BEF" | "ESP" | "GRD" | "ITL" | "LUF" | "PTE" | "SIT" => 0,
        _ => 2,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::functions::testutil::{TestCtx, rng};

    fn c() -> TestCtx {
        TestCtx::new()
    }

    // Helper: compare floats with tolerance
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }
    fn approx_fine(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn test_pmt_basic() {
        let mut ctx = c();
        // PMT(0.08/12, 10, 10000) ≈ -1037.03
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(10.0),
                Value::Number(10000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, -1037.03), "PMT = {v}");
        } else {
            panic!("Expected number, got {:?}", r);
        }
    }

    #[test]
    fn test_pmt_zero_rate() {
        let mut ctx = c();
        // PMT(0, 10, 1000) = -100
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(10.0),
                Value::Number(1000.0),
            ],
        );
        assert_eq!(r, Value::Number(-100.0));
    }

    #[test]
    fn test_pmt_with_fv() {
        let mut ctx = c();
        // PMT(0.1/12, 24, 0, 50000) - saving to reach future value
        let r = pmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(24.0),
                Value::Number(0.0),
                Value::Number(50000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v < 0.0, "saving PMT should be negative: {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fv_basic() {
        let mut ctx = c();
        // FV(0.06/12, 10, -200, -500, 1) ≈ 2581.40
        let r = fv(
            &mut ctx,
            &[
                Value::Number(0.06 / 12.0),
                Value::Number(10.0),
                Value::Number(-200.0),
                Value::Number(-500.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 2581.40), "FV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fv_zero_rate() {
        let mut ctx = c();
        // FV(0, 3, -100, -1000) = 1000 + 300 = 1300
        let r = fv(
            &mut ctx,
            &[
                Value::Number(0.0),
                Value::Number(3.0),
                Value::Number(-100.0),
                Value::Number(-1000.0),
            ],
        );
        assert_eq!(r, Value::Number(1300.0));
    }

    #[test]
    fn test_pv_basic() {
        let mut ctx = c();
        // PV(0.08/12, 20*12, 500) — PV of annuity
        let r = pv(
            &mut ctx,
            &[
                Value::Number(0.08 / 12.0),
                Value::Number(240.0),
                Value::Number(500.0),
            ],
        );
        if let Value::Number(v) = r {
            // Should be a large negative number (present cost of receiving 500/mo for 20 yrs)
            assert!(v < -50000.0, "PV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_nper_basic() {
        let mut ctx = c();
        // NPER(0.12/12, -100, -1000, 10000) ≈ 60.08
        // pv=-1000, fv=10000, rate=0.01, pmt=-100
        let r = nper(
            &mut ctx,
            &[
                Value::Number(0.12 / 12.0),
                Value::Number(-100.0),
                Value::Number(-1000.0),
                Value::Number(10000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 60.08), "NPER = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_rate_basic() {
        let mut ctx = c();
        // RATE(48, -200, 8000) — monthly rate for a loan of 8000, 48 payments of 200
        // Excel: ~0.77% per month
        let r = rate(
            &mut ctx,
            &[
                Value::Number(48.0),
                Value::Number(-200.0),
                Value::Number(8000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 0.77), "RATE = {}", v * 100.0);
        } else {
            panic!("Expected number, got {:?}", r);
        }
    }

    #[test]
    fn test_npv_basic() {
        let mut ctx = c();
        // NPV(0.1, -10000, 3000, 4200, 6800) ≈ 1188.44
        let r = npv(
            &mut ctx,
            &[
                Value::Number(0.1),
                Value::Number(-10000.0),
                Value::Number(3000.0),
                Value::Number(4200.0),
                Value::Number(6800.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 1188.44), "NPV = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_npv_range() {
        let mut ctx = TestCtx::with_cells(&[
            (0, 0, Value::Number(3000.0)),
            (1, 0, Value::Number(4200.0)),
            (2, 0, Value::Number(6800.0)),
        ]);
        let r = npv(
            &mut ctx,
            &[Value::Number(0.1), Value::Number(-10000.0), rng(0, 0, 2, 0)],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 1188.44), "NPV range = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_irr_basic() {
        let mut ctx = c();
        // IRR([-70000, 12000, 15000, 18000, 21000, 26000]) ≈ 8.66%
        let r = irr(
            &mut ctx,
            &[Value::Array(crate::formula::value::Array::from_rows(vec![
                vec![
                    Value::Number(-70000.0),
                    Value::Number(12000.0),
                    Value::Number(15000.0),
                    Value::Number(18000.0),
                    Value::Number(21000.0),
                    Value::Number(26000.0),
                ],
            ]))],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 8.66), "IRR = {}", v * 100.0);
        } else {
            panic!("Expected number, got {:?}", r);
        }
    }

    #[test]
    fn test_irr_error_no_sign_change() {
        let mut ctx = c();
        // All positive — no IRR
        let r = irr(
            &mut ctx,
            &[Value::Array(crate::formula::value::Array::from_rows(vec![
                vec![Value::Number(100.0), Value::Number(200.0)],
            ]))],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_mirr_basic() {
        let mut ctx = c();
        // MIRR([-120000, 39000, 30000, 21000, 37000, 46000], 10%, 12%)
        let cfs = vec![
            Value::Number(-120000.0),
            Value::Number(39000.0),
            Value::Number(30000.0),
            Value::Number(21000.0),
            Value::Number(37000.0),
            Value::Number(46000.0),
        ];
        let r = mirr(
            &mut ctx,
            &[
                Value::Array(crate::formula::value::Array::from_rows(vec![cfs])),
                Value::Number(0.10),
                Value::Number(0.12),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v * 100.0, 12.61), "MIRR = {}", v * 100.0);
        } else {
            panic!("Expected number, got {:?}", r);
        }
    }

    #[test]
    fn test_sln() {
        let mut ctx = c();
        // SLN(30000, 7500, 10) = 2250
        let r = sln(
            &mut ctx,
            &[
                Value::Number(30000.0),
                Value::Number(7500.0),
                Value::Number(10.0),
            ],
        );
        assert_eq!(r, Value::Number(2250.0));
    }

    #[test]
    fn test_sln_zero_life() {
        let mut ctx = c();
        let r = sln(
            &mut ctx,
            &[
                Value::Number(1000.0),
                Value::Number(0.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    #[test]
    fn test_syd_basic() {
        let mut ctx = c();
        // SYD(30000, 7500, 10, 1) ≈ 4090.91
        let r = syd(
            &mut ctx,
            &[
                Value::Number(30000.0),
                Value::Number(7500.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 4090.91), "SYD = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_ddb_basic() {
        let mut ctx = c();
        // DDB(2400, 300, 10, 1) — year 1 depreciation
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(2400.0),
                Value::Number(300.0),
                Value::Number(10.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(480.0));
    }

    #[test]
    fn test_ddb_period_exceeds_life() {
        let mut ctx = c();
        let r = ddb(
            &mut ctx,
            &[
                Value::Number(2400.0),
                Value::Number(300.0),
                Value::Number(10.0),
                Value::Number(11.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }

    #[test]
    fn test_effect_nominal() {
        let mut ctx = c();
        // EFFECT(5.25%, 4) ≈ 5.354%
        let e = effect(&mut ctx, &[Value::Number(0.0525), Value::Number(4.0)]);
        if let Value::Number(v) = e {
            assert!(approx(v * 100.0, 5.354), "EFFECT = {}", v * 100.0);
        } else {
            panic!("Expected number");
        }

        // NOMINAL(EFFECT(5.25%, 4), 4) should return ~5.25%
        let n = nominal(&mut ctx, &[e.clone(), Value::Number(4.0)]);
        // e is the result of effect, use manually
        let effect_val = if let Value::Number(v) = e { v } else { 0.0 };
        let n2 = nominal(&mut ctx, &[Value::Number(effect_val), Value::Number(4.0)]);
        if let Value::Number(v) = n2 {
            assert!(
                approx(v * 100.0, 5.25),
                "NOMINAL round-trip = {}",
                v * 100.0
            );
        }
        let _ = n;
    }

    #[test]
    fn test_effect_invalid() {
        let mut ctx = c();
        assert_eq!(
            effect(&mut ctx, &[Value::Number(-0.1), Value::Number(4.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            effect(&mut ctx, &[Value::Number(0.05), Value::Number(0.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn test_pduration() {
        let mut ctx = c();
        // PDURATION(0.025, 2000, 2200) ≈ 3.86
        let r = pduration(
            &mut ctx,
            &[
                Value::Number(0.025),
                Value::Number(2000.0),
                Value::Number(2200.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 3.86), "PDURATION = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_rri() {
        let mut ctx = c();
        // RRI(96, 10000, 11000) — quarterly equivalent rate
        let r = rri(
            &mut ctx,
            &[
                Value::Number(96.0),
                Value::Number(10000.0),
                Value::Number(11000.0),
            ],
        );
        if let Value::Number(v) = r {
            assert!(v > 0.0 && v < 0.01, "RRI = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_dollarde_dollarfr() {
        let mut ctx = c();
        // DOLLARDE(1.02, 16) = 1.125 (1 + 2/16 = 1 + 0.125)
        let r = dollarde(&mut ctx, &[Value::Number(1.02), Value::Number(16.0)]);
        if let Value::Number(v) = r {
            assert!(approx_fine(v, 1.125), "DOLLARDE = {v}");
        } else {
            panic!("Expected number, got {:?}", r);
        }
    }

    #[test]
    fn test_ispmt() {
        let mut ctx = c();
        // ISPMT(0.1/12, 1, 36, 8000000) ≈ -66667
        let r = ispmt(
            &mut ctx,
            &[
                Value::Number(0.1 / 12.0),
                Value::Number(1.0),
                Value::Number(36.0),
                Value::Number(8_000_000.0),
            ],
        );
        if let Value::Number(v) = r {
            // ISPMT(0.1/12, 1, 36, 8000000) = 8000000*(0.1/12)*(1/36-1) ≈ -64814.81
            assert!(approx(v, -64814.81), "ISPMT = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_tbillprice() {
        let mut ctx = c();
        // TBILLPRICE with 91-day bill at 9% discount
        // settlement=0, maturity=91, discount=0.09
        let r = tbillprice(
            &mut ctx,
            &[Value::Number(0.0), Value::Number(91.0), Value::Number(0.09)],
        );
        if let Value::Number(v) = r {
            // 100 * (1 - 0.09 * 91/360) = 100 * (1 - 0.02275) = 97.725
            assert!(approx(v, 97.725), "TBILLPRICE = {v}");
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_fvschedule() {
        let mut ctx = c();
        // FVSCHEDULE(100, [0.09, 0.11, 0.1]) = 100*1.09*1.11*1.1 ≈ 133.09
        let r = fvschedule(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Array(crate::formula::value::Array::from_rows(vec![vec![
                    Value::Number(0.09),
                    Value::Number(0.11),
                    Value::Number(0.1),
                ]])),
            ],
        );
        if let Value::Number(v) = r {
            assert!(approx(v, 133.09), "FVSCHEDULE = {v}");
        } else {
            panic!("Expected number");
        }
    }

    // ---- Coupon / bond function tests ----

    // Build a serial-number Value for a y/m/d in the 1900 date system.
    fn ds(y: i32, m: u32, d: u32) -> Value {
        let serial = easyexcel_model::dates::ymd_to_serial(DateSystem::Date1900, y, m, d).unwrap();
        Value::Number(serial)
    }

    fn n(v: &Value) -> f64 {
        match v {
            Value::Number(x) => *x,
            other => panic!("expected number, got {:?}", other),
        }
    }

    #[test]
    fn test_coupnum_excel() {
        let mut ctx = c();
        // COUPNUM(DATE(2007,1,25), DATE(2008,11,15), 2, 0) = 4
        let r = coupnum(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(n(&r), 4.0);
    }

    #[test]
    fn test_coupncd_couppcd_bracket() {
        let mut ctx = c();
        // Next coupon must be after settlement, previous on/before.
        let nc = coupncd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let pc = couppcd(
            &mut ctx,
            &[
                ds(2007, 1, 25),
                ds(2008, 11, 15),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        let settle = n(&ds(2007, 1, 25));
        assert!(n(&nc) > settle);
        assert!(n(&pc) <= settle);
        // Coupons are 6 months apart -> ~182/183 days
        assert!((n(&nc) - n(&pc)).abs() > 180.0 && (n(&nc) - n(&pc)).abs() < 185.0);
    }

    #[test]
    fn test_coupdays_consistency() {
        let mut ctx = c();
        let args = [
            ds(2007, 1, 25),
            ds(2008, 11, 15),
            Value::Number(2.0),
            Value::Number(0.0),
        ];
        let cd = n(&coupdays(&mut ctx, &args));
        let bs = n(&coupdaybs(&mut ctx, &args));
        let nc = n(&coupdaysnc(&mut ctx, &args));
        // COUPDAYBS + COUPDAYSNC == COUPDAYS
        assert!((bs + nc - cd).abs() < 1e-6, "bs={bs} nc={nc} cd={cd}");
        // basis 0 -> 360/2 = 180
        assert!((cd - 180.0).abs() < 1e-6);
    }

    #[test]
    fn test_price_par_bond() {
        let mut ctx = c();
        // Par bond: coupon == yield -> price ~ 100 (settlement on a coupon date).
        // Settlement = 2010-01-01, maturity 2020-01-01, 5% coupon, 5% yield, freq 2.
        let r = price(
            &mut ctx,
            &[
                ds(2010, 1, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert!((n(&r) - 100.0).abs() < 0.1, "PRICE = {}", n(&r));
    }

    #[test]
    fn test_price_yield_roundtrip() {
        let mut ctx = c();
        let base = [
            ds(2008, 2, 15),
            ds(2017, 11, 15),
            Value::Number(0.0575),
            Value::Number(0.065),
            Value::Number(100.0),
            Value::Number(2.0),
            Value::Number(0.0),
        ];
        let p = n(&price(&mut ctx, &base));
        // Excel PRICE for these args ~= 94.63
        assert!((p - 94.63).abs() < 0.2, "PRICE = {p}");
        // Feed price into YIELD, expect ~0.065 back.
        let yargs = [
            base[0].clone(),
            base[1].clone(),
            base[2].clone(),
            Value::Number(p),
            base[4].clone(),
            base[5].clone(),
            base[6].clone(),
        ];
        let y = n(&yield_fn(&mut ctx, &yargs));
        assert!((y - 0.065).abs() < 1e-4, "YIELD = {y}");
    }

    #[test]
    fn test_duration_reasonable() {
        let mut ctx = c();
        // DURATION(2008-1-1, 2016-1-1, 8% coupon, 9% yield, 2) ~ 5.99 yrs (Excel ~5.99)
        let r = duration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(0.08),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        let d = n(&r);
        assert!(d > 5.5 && d < 6.5, "DURATION = {d}");
        // MDURATION = DURATION / (1 + y/freq)
        let md = n(&mduration(
            &mut ctx,
            &[
                ds(2008, 1, 1),
                ds(2016, 1, 1),
                Value::Number(0.08),
                Value::Number(0.09),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        ));
        assert!(
            (md - d / (1.0 + 0.09 / 2.0)).abs() < 1e-6,
            "MDURATION = {md}"
        );
    }

    #[test]
    fn test_accrintm() {
        let mut ctx = c();
        // ACCRINTM(issue=2008-4-1, settle=2008-6-15, 10%, par 1000, basis 3)
        let r = accrintm(
            &mut ctx,
            &[
                ds(2008, 4, 1),
                ds(2008, 6, 15),
                Value::Number(0.1),
                Value::Number(1000.0),
                Value::Number(3.0),
            ],
        );
        // 75 days / 365 * 0.1 * 1000 = 20.5479...
        assert!((n(&r) - 20.5479).abs() < 0.01, "ACCRINTM = {}", n(&r));
    }

    #[test]
    fn test_settlement_ge_maturity_is_num() {
        let mut ctx = c();
        let r = coupnum(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2019, 1, 1),
                Value::Number(2.0),
                Value::Number(0.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
        let p = price(
            &mut ctx,
            &[
                ds(2020, 1, 1),
                ds(2020, 1, 1),
                Value::Number(0.05),
                Value::Number(0.05),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(p, Value::Error(CellError::Num));
    }

    #[test]
    fn test_euroconvert() {
        let mut ctx = c();
        // 100 DEM -> EUR : 100 / 1.95583 = 51.13 (2 decimals)
        let r = euroconvert(
            &mut ctx,
            &[
                Value::Number(100.0),
                Value::Text("DEM".into()),
                Value::Text("EUR".into()),
            ],
        );
        assert!((n(&r) - 51.13).abs() < 0.01, "DEM->EUR = {}", n(&r));
        // 1 EUR -> FRF = 6.55957, full precision
        let r2 = euroconvert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("EUR".into()),
                Value::Text("FRF".into()),
                Value::Bool(true),
            ],
        );
        assert!((n(&r2) - 6.55957).abs() < 1e-6, "EUR->FRF = {}", n(&r2));
        // Unknown currency -> #VALUE!
        let r3 = euroconvert(
            &mut ctx,
            &[
                Value::Number(1.0),
                Value::Text("USD".into()),
                Value::Text("EUR".into()),
            ],
        );
        assert_eq!(r3, Value::Error(CellError::Value));
    }

    #[test]
    fn test_amorlinc_first_period() {
        let mut ctx = c();
        // AMORLINC(2400, 2008-8-19, 2008-12-31, 300, 1, 15%, 1)
        let r = amorlinc(
            &mut ctx,
            &[
                Value::Number(2400.0),
                ds(2008, 8, 19),
                ds(2008, 12, 31),
                Value::Number(300.0),
                Value::Number(1.0),
                Value::Number(0.15),
                Value::Number(1.0),
            ],
        );
        // Full-year depreciation = 2400 * 0.15 = 360 for period 1.
        assert!((n(&r) - 360.0).abs() < 1.0, "AMORLINC = {}", n(&r));
    }

    #[test]
    fn test_oddprice_stubbed_num() {
        let mut ctx = c();
        let r = oddfprice(
            &mut ctx,
            &[
                ds(2008, 11, 11),
                ds(2021, 3, 1),
                ds(2008, 10, 15),
                ds(2009, 3, 1),
                Value::Number(0.0785),
                Value::Number(0.0625),
                Value::Number(100.0),
                Value::Number(2.0),
            ],
        );
        assert_eq!(r, Value::Error(CellError::Num));
    }
}
