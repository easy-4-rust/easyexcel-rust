//! Date & Time functions.

use super::Registry;
use crate::core::dates::{DateSystem, serial_time_parts};
use crate::core::error::CellError;
use crate::core::formula::coerce::{to_number, to_text};
use crate::core::formula::context::Context;
use crate::core::formula::value::Value;
use chrono::{Datelike, Duration, NaiveDate};

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

fn parse_date_text(s: &str, _system: DateSystem) -> Option<f64> {
    // Try common formats: YYYY-MM-DD, MM/DD/YYYY, DD-Mon-YYYY
    let s = s.trim();
    // ISO: YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(_system.date_to_serial(d) as f64);
    }
    // US: MM/DD/YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Some(_system.date_to_serial(d) as f64);
    }
    // DD-Mon-YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%b-%Y") {
        return Some(_system.date_to_serial(d) as f64);
    }
    // Mon DD, YYYY
    if let Ok(d) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
        return Some(_system.date_to_serial(d) as f64);
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
    NaiveDate::from_ymd_opt(date.year(), date.month(), dim).unwrap()
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
        Some(d) => d + Duration::days(day as i64 - 1),
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
        Some(d) => Value::Number(d.year() as f64),
        None => Value::Error(CellError::Value),
    }
}

fn month_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(ctx.date_system(), serial) {
        Some(d) => Value::Number(d.month() as f64),
        None => Value::Error(CellError::Value),
    }
}

fn day_fn(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match get_serial(ctx, &args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    match serial_to_date(ctx.date_system(), serial) {
        Some(d) => Value::Number(d.day() as f64),
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
    Value::Number(h as f64)
}

fn minute_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let (_, m, _) = serial_time_parts(serial);
    Value::Number(m as f64)
}

fn second_fn(_ctx: &mut dyn Context, args: &[Value]) -> Value {
    let serial = match to_number(&args[0]) {
        Ok(n) => n,
        Err(e) => return Value::Error(e),
    };
    let (_, _, s) = serial_time_parts(serial);
    Value::Number(s as f64)
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
    Value::Number(result as f64)
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

    let jan1 = NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap();
    let jan1_dow = jan1.weekday().num_days_from_sunday(); // Sun=0
    let day_of_year = date.ordinal() as i32 - 1; // 0-based
    // days from start of first week
    let adj = (jan1_dow as i32 - first_dow as i32).rem_euclid(7);
    let week = (day_of_year + adj) / 7 + 1;
    Value::Number(week as f64)
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
    Value::Number(date.iso_week().week() as f64)
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
            years as f64
        }
        "M" => {
            let mut months =
                (end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32;
            if end.day() < start.day() {
                months -= 1;
            }
            months as f64
        }
        "D" => (end - start).num_days() as f64,
        "MD" => {
            // days ignoring months and years
            let mut d = end.day() as i32 - start.day() as i32;
            if d < 0 {
                let prev_month = add_months(end, -1);
                d += days_in_month(prev_month.year(), prev_month.month()) as i32;
            }
            d as f64
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
            m as f64
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
                    .unwrap(),
                )
            } else {
                NaiveDate::from_ymd_opt(end.year() - 1, start.month(), start.day()).unwrap_or(
                    NaiveDate::from_ymd_opt(
                        end.year() - 1,
                        start.month(),
                        days_in_month(end.year() - 1, start.month()),
                    )
                    .unwrap(),
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
        match crate::core::formula::coerce::to_bool(&args[2]) {
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
    Value::Number(days as f64)
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
            d360 as f64 / 360.0
        }
        1 => {
            // Actual/actual
            let days = (end - start).num_days() as f64;
            let avg_year = {
                let y1 = start.year();
                let y2 = end.year();
                let years_span = (y2 - y1) as f64 + 1.0;
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
            d360 as f64 / 360.0
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
mod tests {
    use super::*;
    use crate::core::formula::functions::testutil::TestCtx;

    fn ctx() -> TestCtx {
        TestCtx::new()
    }

    // DATE(2023,1,1) in 1900 system → serial 44927
    #[test]
    fn date_basic() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[
                Value::Number(2023.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        );
        assert_eq!(r, Value::Number(44927.0));
    }

    #[test]
    fn date_two_digit_year() {
        let mut c = ctx();
        let r = date_fn(
            &mut c,
            &[Value::Number(25.0), Value::Number(6.0), Value::Number(15.0)],
        );
        // Year 2025
        let expected =
            DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
        assert_eq!(r, Value::Number(expected as f64));
    }

    #[test]
    fn time_basic() {
        let mut c = ctx();
        let r = time_fn(
            &mut c,
            &[Value::Number(12.0), Value::Number(0.0), Value::Number(0.0)],
        );
        // 12:00:00 = 0.5
        assert_eq!(r, Value::Number(0.5));
    }

    #[test]
    fn year_month_day() {
        let mut c = ctx();
        // 2008-12-31 = serial 39813
        let s = Value::Number(39813.0);
        assert_eq!(
            year_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(2008.0)
        );
        assert_eq!(
            month_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(12.0)
        );
        assert_eq!(
            day_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(31.0)
        );
    }

    #[test]
    fn hour_minute_second() {
        let mut c = ctx();
        // 0.5 = 12:00:00
        let s = Value::Number(0.5);
        assert_eq!(
            hour_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(12.0)
        );
        assert_eq!(
            minute_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(0.0)
        );
        assert_eq!(
            second_fn(&mut c, std::slice::from_ref(&s)),
            Value::Number(0.0)
        );
    }

    #[test]
    fn weekday_sunday_first() {
        let mut c = ctx();
        // 2023-01-01 is a Sunday. Serial 44927.
        let s = Value::Number(44927.0);
        // type 1: Sun=1
        assert_eq!(
            weekday_fn(&mut c, &[s.clone(), Value::Number(1.0)]),
            Value::Number(1.0)
        );
        // type 2: Sun=7
        assert_eq!(
            weekday_fn(&mut c, &[s.clone(), Value::Number(2.0)]),
            Value::Number(7.0)
        );
    }

    #[test]
    fn edate_basic() {
        let mut c = ctx();
        // 2023-01-31 + 1 month = 2023-02-28
        let serial = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = edate_fn(&mut c, &[Value::Number(serial), Value::Number(1.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 2, 28).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn eomonth_basic() {
        let mut c = ctx();
        // End of month for 2023-01-15 + 0 months = 2023-01-31
        let serial = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())
            as f64;
        let r = eomonth_fn(&mut c, &[Value::Number(serial), Value::Number(0.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn datedif_y() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("Y".into()),
            ],
        );
        assert_eq!(r, Value::Number(3.0));
    }

    #[test]
    fn datedif_d() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = datedif_fn(
            &mut c,
            &[
                Value::Number(s1),
                Value::Number(s2),
                Value::Text("D".into()),
            ],
        );
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn networkdays_basic() {
        let mut c = ctx();
        // 2023-01-02 (Mon) to 2023-01-06 (Fri) = 5 workdays
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 6).unwrap())
            as f64;
        let r = networkdays_fn(&mut c, &[Value::Number(s1), Value::Number(s2)]);
        assert_eq!(r, Value::Number(5.0));
    }

    #[test]
    fn workday_basic() {
        let mut c = ctx();
        // 2023-01-02 (Mon) + 5 workdays = 2023-01-09 (Mon)
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = workday_fn(&mut c, &[Value::Number(s1), Value::Number(5.0)]);
        let expected = DateSystem::Date1900
            .date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 9).unwrap())
            as f64;
        assert_eq!(r, Value::Number(expected));
    }

    #[test]
    fn days_fn_basic() {
        let mut c = ctx();
        let s1 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            as f64;
        let s2 = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 31).unwrap())
            as f64;
        let r = days_fn(&mut c, &[Value::Number(s2), Value::Number(s1)]);
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn isoweeknum_basic() {
        let mut c = ctx();
        // 2023-01-02 is ISO week 1
        let s = DateSystem::Date1900.date_to_serial(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap())
            as f64;
        let r = isoweeknum_fn(&mut c, &[Value::Number(s)]);
        assert_eq!(r, Value::Number(1.0));
    }
}
