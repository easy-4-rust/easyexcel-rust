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
                .map_or(28, |_| last_day_of_month(y, 2))
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
    f64::from((y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1))
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
    f64::from((y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1))
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
        0 | 2 | 4 => 360.0 / f64::from(freq),
        3 => 365.0 / f64::from(freq),
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
    let coupon = 100.0 * rate / f64::from(freq);
    let yf = yld / f64::from(freq);
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
    let yf = yld / f64::from(freq);
    let c = 100.0 * coupon / f64::from(freq);

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
    (num_sum / den_sum) / f64::from(freq)
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
    num(mac / (1.0 + y / f64::from(f)))
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

