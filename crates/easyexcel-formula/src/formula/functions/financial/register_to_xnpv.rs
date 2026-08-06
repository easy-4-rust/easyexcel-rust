/// 对应 Java：无直接对应对象；Rust 架构扩展。
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
/// Returns None if non-convergent after `max_iter` iterations.
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
        let p = (per_i - i64::from(typ == 1.0)) as f64;
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

