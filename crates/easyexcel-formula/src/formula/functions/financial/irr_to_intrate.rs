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

