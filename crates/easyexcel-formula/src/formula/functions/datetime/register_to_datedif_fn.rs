/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    r.add("DATE", 3, 3, false, date_fn);
    r.add("TIME", 3, 3, false, time_fn);
    r.add("DATEVALUE", 1, 1, false, datevalue_fn);
    r.add("TIMEVALUE", 1, 1, false, timevalue_fn);
    r.add("YEAR", 1, 1, false, year_fn);
    r.add("MONTH", 1, 1, false, month_fn);
    r.add("DAY", 1, 1, false, day_fn);
    r.add("HOUR", 1, 1, false, hour_fn);
    r.add("MINUTE", 1, 1, false, minute_fn);
    r.add("SECOND", 1, 1, false, second_fn);
    r.add("WEEKDAY", 1, 2, false, weekday_fn);
    r.add("WEEKNUM", 1, 2, false, weeknum_fn);
    r.add("ISOWEEKNUM", 1, 1, false, isoweeknum_fn);
    r.add("NOW", 0, 0, true, |ctx, _| Value::Number(ctx.now_serial()));
    r.add("TODAY", 0, 0, true, |ctx, _| {
        Value::Number(ctx.today_serial())
    });
    r.add("EDATE", 2, 2, false, edate_fn);
    r.add("EOMONTH", 2, 2, false, eomonth_fn);
    r.add("DATEDIF", 3, 3, false, datedif_fn);
    r.add("DAYS", 2, 2, false, days_fn);
    r.add("DAYS360", 2, 3, false, days360_fn);
    r.add("YEARFRAC", 2, 3, false, yearfrac_fn);
    r.add("NETWORKDAYS", 2, 3, false, networkdays_fn);
    r.add("NETWORKDAYS.INTL", 2, 4, false, networkdays_intl_fn);
    r.add("WORKDAY", 2, 3, false, workday_fn);
    r.add("WORKDAY.INTL", 2, 4, false, workday_intl_fn);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_serial(ctx: &mut dyn Context, v: &Value) -> Result<f64, CellError> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::Text(s) => {
            // try parse as a date string
            parse_date_text(s, ctx.date_system()).ok_or(CellError::Value)
        }
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::Empty => Ok(0.0),
        Value::Error(e) => Err(*e),
        Value::Array(a) => match a.data.first() {
            Some(v) => get_serial(ctx, v),
            None => Err(CellError::Value),
        },
        Value::Ref(_) => Err(CellError::Value),
        Value::Lambda(_) => Err(CellError::Value),
    }
}

fn serial_to_date(system: DateSystem, serial: f64) -> Option<NaiveDate> {
    system.serial_to_datetime(serial).map(|dt| dt.date())
}

fn parse_date_text(s: &str, system: DateSystem) -> Option<f64> {
    // Try common formats: YYYY-MM-DD, MM/DD/YYYY, DD-Mon-YYYY
    let s = s.trim();
    // ISO: YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(system.date_to_serial(d) as f64);
    }
    // US: MM/DD/YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Some(system.date_to_serial(d) as f64);
    }
    // DD-Mon-YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%b-%Y") {
        return Some(system.date_to_serial(d) as f64);
    }
    // Mon DD, YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
        return Some(system.date_to_serial(d) as f64);
    }
    None
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let day = date.day().min(days_in_month(year, month as u32));
    NaiveDate::from_ymd_opt(year, month as u32, day).unwrap_or(date)
}

fn end_of_month(date: NaiveDate) -> NaiveDate {
    let dim = days_in_month(date.year(), date.month());
    // 修复: 日期超出范围时 unwrap 会 panic；回退原日期
    NaiveDate::from_ymd_opt(date.year(), date.month(), dim).unwrap_or(date)
}

/// Parse an INTL weekend string or integer. Returns an array of 7 bools (Mon..Sun) true = holiday.
fn parse_weekend(v: &Value) -> Result<[bool; 7], CellError> {
    match v {
        Value::Number(n) => weekend_code(*n as i32),
        Value::Text(s) => {
            if s.len() == 7 && s.chars().all(|c| c == '0' || c == '1') {
                let mut arr = [false; 7];
                for (i, c) in s.chars().enumerate() {
                    arr[i] = c == '1';
                }
                Ok(arr)
            } else {
                Err(CellError::Value)
            }
        }
        _ => Err(CellError::Value),
    }
}

fn weekend_code(code: i32) -> Result<[bool; 7], CellError> {
    // Mon=0 .. Sun=6
    let mut arr = [false; 7];
    match code {
        1 => {
            arr[5] = true;
            arr[6] = true;
        } // Sat+Sun
        2 => {
            arr[6] = true;
            arr[0] = true;
        } // Sun+Mon
        3 => {
            arr[0] = true;
            arr[1] = true;
        } // Mon+Tue
        4 => {
            arr[1] = true;
            arr[2] = true;
        } // Tue+Wed
        5 => {
            arr[2] = true;
            arr[3] = true;
        } // Wed+Thu
        6 => {
            arr[3] = true;
            arr[4] = true;
        } // Thu+Fri
        7 => {
            arr[4] = true;
            arr[5] = true;
        } // Fri+Sat
        11 => {
            arr[6] = true;
        } // Sun only
        12 => {
            arr[0] = true;
        } // Mon only
        13 => {
            arr[1] = true;
        } // Tue only
        14 => {
            arr[2] = true;
        } // Wed only
        15 => {
            arr[3] = true;
        } // Thu only
        16 => {
            arr[4] = true;
        } // Fri only
        17 => {
            arr[5] = true;
        } // Sat only
        _ => return Err(CellError::Value),
    }
    Ok(arr)
}

fn weekday_index_mon0(d: NaiveDate) -> usize {
    // Monday=0 .. Sunday=6
    d.weekday().num_days_from_monday() as usize
}

fn is_workday(d: NaiveDate, weekend: &[bool; 7], holidays: &[NaiveDate]) -> bool {
    !weekend[weekday_index_mon0(d)] && !holidays.contains(&d)
}

fn collect_holidays(ctx: &mut dyn Context, v: &Value, system: DateSystem) -> Vec<NaiveDate> {
    let serials = ctx.flatten(v);
    serials
        .into_iter()
        .filter_map(|s| match s {
            Value::Number(n) => serial_to_date(system, n),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// DATE
// ---------------------------------------------------------------------------

fn date_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let year = match to_number(&args[0]) {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };
    let month = match to_number(&args[1]) {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };
    let day = match to_number(&args[2]) {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };

    // Excel: two-digit years 0-29 → 2000+, 30-99 → 1900+
    let year = if year < 0 {
        return Value::Error(CellError::Num);
    } else if year < 100 {
        if year < 30 { year + 2000 } else { year + 1900 }
    } else {
        year
    };

    // Build the date with month/day overflow support.
    if NaiveDate::from_ymd_opt(year, 1, 1).is_none() {
        return Value::Error(CellError::Num);
    }
    let adjusted = add_months(NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), month - 1);
    let adjusted = match NaiveDate::from_ymd_opt(adjusted.year(), adjusted.month(), 1) {
        Some(d) => d + Duration::days(i64::from(day) - 1),
        None => return Value::Error(CellError::Num),
    };

    let serial = ctx.date_system().date_to_serial(adjusted);
    if serial < 0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(serial as f64)
}

// ---------------------------------------------------------------------------
// TIME
// ---------------------------------------------------------------------------

fn time_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let h = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let m = match to_number(&args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let s = match to_number(&args[2]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let total_secs = h * 3600.0 + m * 60.0 + s;
    let frac = (total_secs % 86400.0) / 86400.0;
    Value::Number(if frac < 0.0 { frac + 1.0 } else { frac })
}

// ---------------------------------------------------------------------------
// DATEVALUE / TIMEVALUE
// ---------------------------------------------------------------------------

fn datevalue_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let s = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    match parse_date_text(&s, ctx.date_system()) {
        Some(serial) => Value::Number(serial.trunc()),
        None => Value::Error(CellError::Value),
    }
}

fn timevalue_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let s = match to_text(&args[0]) {
        Ok(s) => s,
        Err(e) => return Value::Error(e),
    };
    // Try HH:MM:SS or HH:MM
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() < 2 {
        return Value::Error(CellError::Value);
    }
    let h: f64 = parts[0].trim().parse().unwrap_or(f64::NAN);
    let m: f64 = parts[1].trim().parse().unwrap_or(f64::NAN);
    let sec_str = parts.get(2).copied().unwrap_or("0");
    let s: f64 = sec_str.trim().parse().unwrap_or(f64::NAN);
    if h.is_nan() || m.is_nan() || s.is_nan() {
        return Value::Error(CellError::Value);
    }
    let frac = (h * 3600.0 + m * 60.0 + s) / 86400.0;
    Value::Number(frac)
}

// ---------------------------------------------------------------------------
// YEAR / MONTH / DAY
// ---------------------------------------------------------------------------

fn year_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(ctx.date_system(), serial) {
        Some(d) => Value::Number(f64::from(d.year())),
        None => Value::Error(CellError::Value),
    }
}

fn month_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(ctx.date_system(), serial) {
        Some(d) => Value::Number(f64::from(d.month())),
        None => Value::Error(CellError::Value),
    }
}

fn day_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(ctx.date_system(), serial) {
        Some(d) => Value::Number(f64::from(d.day())),
        None => Value::Error(CellError::Value),
    }
}

// ---------------------------------------------------------------------------
// HOUR / MINUTE / SECOND
// ---------------------------------------------------------------------------

fn hour_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let (h, _, _) = serial_time_parts(serial);
    Value::Number(f64::from(h))
}

fn minute_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let (_, m, _) = serial_time_parts(serial);
    Value::Number(f64::from(m))
}

fn second_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let (_, _, s) = serial_time_parts(serial);
    Value::Number(f64::from(s))
}

// ---------------------------------------------------------------------------
// WEEKDAY
// ---------------------------------------------------------------------------

fn weekday_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let return_type = if args.len() >= 2 {
        match to_number(&args[1]) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let date = match serial_to_date(ctx.date_system(), serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    // chrono: Mon=1..Sun=7 (num_days_from_monday: Mon=0..Sun=6)
    let dow_mon0 = date.weekday().num_days_from_monday(); // 0=Mon..6=Sun
    let result = match return_type {
        1 => (dow_mon0 + 1) % 7 + 1,  // Sun=1, Mon=2, ..., Sat=7
        2 => dow_mon0 + 1,            // Mon=1, ..., Sun=7
        3 => dow_mon0,                // Mon=0, ..., Sun=6
        11 => dow_mon0 + 1,           // Mon=1, ..., Sun=7
        12 => (dow_mon0 + 6) % 7 + 1, // Tue=1, ..., Mon=7
        13 => (dow_mon0 + 5) % 7 + 1, // Wed=1, ..., Tue=7
        14 => (dow_mon0 + 4) % 7 + 1, // Thu=1, ..., Wed=7
        15 => (dow_mon0 + 3) % 7 + 1, // Fri=1, ..., Thu=7
        16 => (dow_mon0 + 2) % 7 + 1, // Sat=1, ..., Fri=7
        17 => (dow_mon0 + 1) % 7 + 1, // Sun=1, ..., Sat=7 (same as type 1 but different base)
        _ => return Value::Error(CellError::Num),
    };
    Value::Number(f64::from(result))
}

// ---------------------------------------------------------------------------
// WEEKNUM
// ---------------------------------------------------------------------------

fn weeknum_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let return_type = if args.len() >= 2 {
        match to_number(&args[1]) {
            Ok(n) => n as i32,
            Err(e) => return Value::Error(e),
        }
    } else {
        1
    };
    let date = match serial_to_date(ctx.date_system(), serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };

    if return_type == 21 {
        // ISO week number
        return isoweeknum_fn(ctx, args);
    }

    // First day of week: 1=Sun, 2=Mon, 11=Mon, 12=Tue..17=Sat
    let first_dow = match return_type {
        1 => 0u32,   // Sun
        2 | 11 => 1, // Mon
        12 => 2,
        13 => 3,
        14 => 4,
        15 => 5,
        16 => 6,
        17 => 0,
        _ => return Value::Error(CellError::Num),
    };

    // 修复: 日期超出范围时 unwrap 会 panic；回退原日期
    let jan1 = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap_or(date);
    let jan1_dow = jan1.weekday().num_days_from_sunday(); // Sun=0
    let day_of_year = date.ordinal() as i32 - 1; // 0-based
    // days from start of first week
    let adj = (jan1_dow as i32 - first_dow as i32).rem_euclid(7);
    let week = (day_of_year + adj) / 7 + 1;
    Value::Number(f64::from(week))
}

// ---------------------------------------------------------------------------
// ISOWEEKNUM
// ---------------------------------------------------------------------------

fn isoweeknum_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let date = match serial_to_date(ctx.date_system(), serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    Value::Number(f64::from(date.iso_week().week()))
}

// ---------------------------------------------------------------------------
// EDATE
// ---------------------------------------------------------------------------

fn edate_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let months = match to_number(&args[1]) {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };
    let date = match serial_to_date(ctx.date_system(), serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let new_date = add_months(date, months);
    Value::Number(ctx.date_system().date_to_serial(new_date) as f64)
}

// ---------------------------------------------------------------------------
// EOMONTH
// ---------------------------------------------------------------------------

fn eomonth_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let months = match to_number(&args[1]) {
        Ok(n) => n as i32,
        Err(e) => return Value::Error(e),
    };
    let date = match serial_to_date(ctx.date_system(), serial) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    let new_date = end_of_month(add_months(date, months));
    Value::Number(ctx.date_system().date_to_serial(new_date) as f64)
}

// ---------------------------------------------------------------------------
// DATEDIF
// ---------------------------------------------------------------------------

fn datedif_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let start_serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let end_serial = match get_serial(ctx, &args[1]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let unit = match to_text(&args[2]) {
        Ok(s) => s.to_uppercase(),
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

    if start > end {
        return Value::Error(CellError::Num);
    }

    let result = match unit.as_str() {
        "Y" => {
            let mut years = end.year() - start.year();
            if end.month() < start.month()
                || (end.month() == start.month() && end.day() < start.day())
            {
                years -= 1;
            }
            f64::from(years)
        }
        "M" => {
            let mut months =
                (end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32;
            if end.day() < start.day() {
                months -= 1;
            }
            f64::from(months)
        }
        "D" => (end - start).num_days() as f64,
        "MD" => {
            // days ignoring months and years
            let mut d = end.day() as i32 - start.day() as i32;
            if d < 0 {
                let prev_month = add_months(end, -1);
                d += days_in_month(prev_month.year(), prev_month.month()) as i32;
            }
            f64::from(d)
        }
        "YM" => {
            // months ignoring years
            let mut m = end.month() as i32 - start.month() as i32;
            if m < 0 {
                m += 12;
            }
            if end.day() < start.day() {
                m -= 1;
                if m < 0 {
                    m += 12;
                }
            }
            f64::from(m)
        }
        "YD" => {
            // days ignoring years: use same year as start
            let start_same_year = if start.month() < end.month()
                || (start.month() == end.month() && start.day() <= end.day())
            {
                NaiveDate::from_ymd_opt(end.year(), start.month(), start.day()).unwrap_or(
                    NaiveDate::from_ymd_opt(
                        end.year(),
                        start.month(),
                        days_in_month(end.year(), start.month()),
                    )
                    // 修复: 日期超出范围时内层 unwrap 会 panic；回退原日期
                    .unwrap_or(start),
                )
            } else {
                NaiveDate::from_ymd_opt(end.year() - 1, start.month(), start.day()).unwrap_or(
                    NaiveDate::from_ymd_opt(
                        end.year() - 1,
                        start.month(),
                        days_in_month(end.year() - 1, start.month()),
                    )
                    // 修复: 日期超出范围时内层 unwrap 会 panic；回退原日期
                    .unwrap_or(start),
                )
            };
            (end - start_same_year).num_days() as f64
        }
        _ => return Value::Error(CellError::Value),
    };

    Value::Number(result)
}

// ---------------------------------------------------------------------------
// DAYS
// ---------------------------------------------------------------------------

