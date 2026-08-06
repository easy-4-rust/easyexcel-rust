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
        "ATS" => 13.7603,   // Austrian schilling
        "BEF" => 40.3399,   // Belgian franc
        "DEM" => 1.95583,   // German mark
        "ESP" => 166.386,   // Spanish peseta
        "FIM" => 5.94573,   // Finnish markka
        "FRF" => 6.55957,   // French franc
        "GRD" => 340.75,    // Greek drachma
        "IEP" => 0.787_564, // Irish pound
        "ITL" => 1936.27,   // Italian lira
        "LUF" => 40.3399,   // Luxembourg franc
        "NLG" => 2.20371,   // Dutch guilder
        "PTE" => 200.482,   // Portuguese escudo
        "SIT" => 239.640,   // Slovenian tolar
        "CYP" => 0.585_274, // Cypriot pound
        "MTL" => 0.4293,    // Maltese lira
        "SKK" => 30.1260,   // Slovak koruna
        "EEK" => 15.6466,   // Estonian kroon
        "LVL" => 0.702_804, // Latvian lats
        "LTL" => 3.45280,   // Lithuanian litas
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
#[path = "../financial_tests/tests.rs"]
mod tests;
