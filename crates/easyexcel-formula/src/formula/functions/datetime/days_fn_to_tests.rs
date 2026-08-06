fn days_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let end_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let start_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    Value::Number(end_serial.trunc() - start_serial.trunc())
}

// ---------------------------------------------------------------------------
// DAYS360
// ---------------------------------------------------------------------------

fn days360_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let european = if args.len() >= 3 {
        match crate::formula::coerce::to_bool(&args[2]) {
            Ok(b) => b,
            Err(e) => return Value::Error(e),
        }
    } else {
        false
    };

    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let end = match serial_to_date(ctx.date_system(), end_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let (sm, sd, sy) = (start.month(), start.day(), start.year());
    let (em, ed, ey) = (end.month(), end.day(), end.year());

    let (sd2, ed2) = if european {
        // European: clamp both to 30
        (sd.min(30), ed.min(30))
    } else {
        // US: last day of Feb → 30; end-of-month if start is 30/31
        let sd2 = if sd == days_in_month(sy, sm) { 30 } else { sd };
        let ed2 = if ed == 31 && sd2 >= 30 { 30 } else { ed };
        (sd2, ed2)
    };

    let days = (ey - sy) * 360 + (em as i32 - sm as i32) * 30 + ed2 as i32 - sd2 as i32;
    Value::Number(f64::from(days))
}

// ---------------------------------------------------------------------------
// YEARFRAC
// ---------------------------------------------------------------------------

fn yearfrac_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let basis = if args.len() >= 3 {
        match to_number(&args[2]) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let end = match serial_to_date(ctx.date_system(), end_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let result = match basis {
        0 => {
            // US 30/360
            let d360 = days360_us(start, end);
            f64::from(d360) / 360.0
        }
        1 => {
            // Actual/actual
            let days = (end - start).num_days() as f64;
            let avg_year = {
                let y1 = start.year();
                let y2 = end.year();
                let years_span = f64::from(y2 - y1) + 1.0;
                let leap_count = (y1..=y2).filter(|&y| is_leap_year(y)).count() as f64;
                (leap_count * 366.0 + (years_span - leap_count) * 365.0) / years_span
            };
            days / avg_year
        }
        2 => {
            // Actual/360
            let days = (end - start).num_days() as f64;
            days / 360.0
        }
        3 => {
            // Actual/365
            let days = (end - start).num_days() as f64;
            days / 365.0
        }
        4 => {
            // European 30/360
            let d360 = days360_eu(start, end);
            f64::from(d360) / 360.0
        }
        _ => return Value::Error(CellError::Num),
    };

    Value::Number(result)
}

fn days360_us(start: NaiveDate, end: NaiveDate) -> i32 {
    let (sm, sd, sy) = (start.month(), start.day(), start.year());
    let (em, ed, ey) = (end.month(), end.day(), end.year());
    let sd2 = if sd == days_in_month(sy, sm) { 30 } else { sd };
    let ed2 = if ed == 31 && sd2 >= 30 { 30 } else { ed };
    (ey - sy) * 360 + (em as i32 - sm as i32) * 30 + ed2 as i32 - sd2 as i32
}

fn days360_eu(start: NaiveDate, end: NaiveDate) -> i32 {
    let (sm, sd, sy) = (start.month(), start.day(), start.year());
    let (em, ed, ey) = (end.month(), end.day(), end.year());
    let sd2 = sd.min(30);
    let ed2 = ed.min(30);
    (ey - sy) * 360 + (em as i32 - sm as i32) * 30 + ed2 as i32 - sd2 as i32
}

// ---------------------------------------------------------------------------
// NETWORKDAYS
// ---------------------------------------------------------------------------

fn networkdays_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let end = match serial_to_date(ctx.date_system(), end_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let system = ctx.date_system();
    let holidays = if args.len() >= 3 {
        collect_holidays(ctx, &args[2].clone(), system)
    } else {
        vec![]
    };

    let weekend = weekend_code(1).unwrap(); // Sat+Sun
    let count = count_workdays(start, end, &weekend, &holidays);
    Value::Number(count as f64)
}

fn count_workdays(
    start: NaiveDate,
    end: NaiveDate,
    weekend: &[bool; 7],
    holidays: &[NaiveDate],
) -> i64 {
    if start > end {
        return -count_workdays(end, start, weekend, holidays);
    }
    let mut d = start;
    let mut count = 0i64;
    while d <= end {
        if is_workday(d, weekend, holidays) {
            count += 1;
        }
        d += Duration::days(1);
    }
    count
}

// ---------------------------------------------------------------------------
// NETWORKDAYS.INTL
// ---------------------------------------------------------------------------

fn networkdays_intl_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let weekend_v = if args.len() >= 3 {
        args[2].clone()
    } else {
        Value::Number(1.0)
    };
    let weekend = match parse_weekend(&weekend_v) {
        Ok(w) => w,
        Err(e) => return Value::Error(e),
    };

    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let end = match serial_to_date(ctx.date_system(), end_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let system = ctx.date_system();
    let holidays = if args.len() >= 4 {
        collect_holidays(ctx, &args[3].clone(), system)
    } else {
        vec![]
    };

    let count = count_workdays(start, end, &weekend, &holidays);
    Value::Number(count as f64)
}

// ---------------------------------------------------------------------------
// WORKDAY
// ---------------------------------------------------------------------------

fn workday_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let days = match to_number(&args[1]) {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };

    let system = ctx.date_system();
    let holidays = if args.len() >= 3 {
        collect_holidays(ctx, &args[2].clone(), system)
    } else {
        vec![]
    };

    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let weekend = weekend_code(1).unwrap();
    let result = advance_workdays(start, days, &weekend, &holidays);
    Value::Number(ctx.date_system().date_to_serial(result) as f64)
}

fn advance_workdays(
    start: NaiveDate,
    days: i64,
    weekend: &[bool; 7],
    holidays: &[NaiveDate],
) -> NaiveDate {
    let direction = if days >= 0 { 1i64 } else { -1i64 };
    let mut remaining = days.abs();
    let mut d = start;
    while remaining > 0 {
        d += Duration::days(direction);
        if is_workday(d, weekend, holidays) {
            remaining -= 1;
        }
    }
    d
}

// ---------------------------------------------------------------------------
// WORKDAY.INTL
// ---------------------------------------------------------------------------

fn workday_intl_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let days = match to_number(&args[1]) {
        Ok(n) => n as i64,
        Err(e) => return Value::Error(e),
    };
    let weekend_v = if args.len() >= 3 {
        args[2].clone()
    } else {
        Value::Number(1.0)
    };
    let weekend = match parse_weekend(&weekend_v) {
        Ok(w) => w,
        Err(e) => return Value::Error(e),
    };

    let system = ctx.date_system();
    let holidays = if args.len() >= 4 {
        collect_holidays(ctx, &args[3].clone(), system)
    } else {
        vec![]
    };

    let start = match serial_to_date(ctx.date_system(), start_serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    let result = advance_workdays(start, days, &weekend, &holidays);
    Value::Number(ctx.date_system().date_to_serial(result) as f64)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../datetime_tests/tests.rs"]
mod tests;
