//! Conversions between Excel date serial numbers and `chrono` date/times.
//!
//! Excel stores dates as a floating point serial number: the integer part is a
//! day count, the fractional part is a fraction of a day. Two epochs exist:
//!
//! * **1900 system** (Windows default): serial `1` == 1900-01-01. Excel
//!   incorrectly treats 1900 as a leap year (the "Lotus 1-2-3 bug"), so serial
//!   `60` maps to the non-existent 1900-02-29. For serials `>= 61` the day count
//!   is effectively offset from `1899-12-30`.
//! * **1904 system** (legacy Mac): serial `0` == 1904-01-01.

use std::borrow::Cow;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};

use super::error::{Error, Result};

/// Which date epoch a workbook uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateSystem {
    /// 1900-based (Windows). Includes the fictional 1900-02-29 leap day.
    #[default]
    Date1900,
    /// 1904-based (legacy Macintosh).
    Date1904,
}

/// Days between the 1900 and 1904 epochs.
const EPOCH_DIFF_1904: f64 = 1462.0;

/// 按多个 Java `SimpleDateFormat` 模式依次解析日期时间。
pub fn parse_java_date<'a>(
    value: &str,
    patterns: impl IntoIterator<Item = &'a str>,
) -> Result<NaiveDateTime> {
    for pattern in patterns {
        let format = java_date_format_to_chrono(pattern);
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, &format) {
            return Ok(datetime);
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, &format) {
            return Ok(date.and_hms_opt(0, 0, 0).unwrap_or_default());
        }
    }
    Err(Error::Other(format!("date parse failed for {value:?}")))
}

/// 使用 Java `SimpleDateFormat` 模式格式化日期时间。
#[must_use]
pub fn format_java_date(value: NaiveDateTime, pattern: &str) -> String {
    value
        .format(&java_date_format_to_chrono(pattern))
        .to_string()
}

/// 将 Excel 1900 日期系统的整数天数转换为 UTC 时间。
#[must_use]
pub fn excel_days_to_utc(days: i64) -> DateTime<Utc> {
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)
        .unwrap_or_default()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default();
    DateTime::<Utc>::from_naive_utc_and_offset(base + Duration::days(days), Utc)
}

/// 判断代码中是否包含 Excel 日期或时间格式标记。
#[must_use]
pub fn is_internal_date_format(format: &str) -> bool {
    super::numfmt::is_date_format(format)
}

/// 将 Java `SimpleDateFormat` 的常用字母转换为 chrono 格式。
#[must_use]
pub fn java_date_format_to_chrono(pattern: &str) -> String {
    let mut output = String::with_capacity(pattern.len() * 2);
    let characters = pattern.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let current = characters[index];
        match current {
            '\'' => {
                index += 1;
                while index < characters.len() && characters[index] != '\'' {
                    output.push(characters[index]);
                    index += 1;
                }
            }
            'y' | 'M' | 'd' | 'H' | 'm' | 's' | 'S' => {
                while index < characters.len() && characters[index] == current {
                    index += 1;
                }
                output.push_str(match current {
                    'y' => "%Y",
                    'M' => "%m",
                    'd' => "%d",
                    'H' => "%H",
                    'm' => "%M",
                    's' => "%S",
                    _ => "%3f",
                });
                continue;
            }
            value => output.push(value),
        }
        index += 1;
    }
    output
}

/// 返回可直接交给 chrono 的日期格式。
///
/// 已包含 `%` 的 chrono 格式保持借用；否则按 Java `SimpleDateFormat`
/// 规则转换，供 EasyExcel annotation/converter 与格式引擎共同使用。
#[must_use]
pub fn chrono_date_format(pattern: &str) -> Cow<'_, str> {
    if pattern.contains('%') {
        Cow::Borrowed(pattern)
    } else {
        Cow::Owned(java_date_format_to_chrono(pattern))
    }
}

/// 将日期转换为 Excel 1900 或 1904 日期系统序列号。
#[must_use]
pub fn date_to_excel_serial(date: NaiveDate, use_1904_windowing: bool) -> f64 {
    let system = DateSystem::from_1904_windowing(use_1904_windowing);
    let days = system.date_to_serial(date);
    f64::from(i32::try_from(days).unwrap_or_else(|_| {
        if days.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        }
    }))
}

/// 将日期时间转换为 Excel 1900 或 1904 日期系统序列号。
#[must_use]
pub fn datetime_to_excel_serial(value: NaiveDateTime, use_1904_windowing: bool) -> f64 {
    let date_part = date_to_excel_serial(value.date(), use_1904_windowing);
    let time = value.time();
    let seconds = f64::from(time.num_seconds_from_midnight())
        + f64::from(time.nanosecond()) / 1_000_000_000.0;
    date_part + seconds / 86_400.0
}

impl DateSystem {
    /// 根据 EasyExcel/POI 的 `use1904windowing` 标志选择日期系统。
    #[must_use]
    pub const fn from_1904_windowing(use_1904_windowing: bool) -> Self {
        if use_1904_windowing {
            Self::Date1904
        } else {
            Self::Date1900
        }
    }

    /// Convert a serial number to a `NaiveDateTime`. Returns `None` for serials
    /// that cannot be represented (for example negative or non-finite values).
    ///
    /// Excel's fictional `1900-02-29` cannot be represented by chrono. For
    /// EasyExcel/POI compatibility serials `60` and `61` both resolve to
    /// `1900-03-01`; serial `59` remains `1900-02-28`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn serial_to_datetime(self, serial: f64) -> Option<NaiveDateTime> {
        if !serial.is_finite() || serial < 0.0 {
            return None;
        }
        let whole_days = serial.floor() as i64;
        let base = match self {
            // Before serial 61, POI uses 1899-12-31 so the unrepresentable
            // fictional leap day (60) resolves to the same real date as 61.
            DateSystem::Date1900 if whole_days < 61 => NaiveDate::from_ymd_opt(1899, 12, 31)?,
            // From serial 61 onward the phantom day is included in the epoch.
            DateSystem::Date1900 => NaiveDate::from_ymd_opt(1899, 12, 30)?,
            DateSystem::Date1904 => NaiveDate::from_ymd_opt(1904, 1, 1)?,
        };
        let milliseconds = ((serial - serial.floor()) * 86_400_000.0 + 0.5).floor() as i64;
        base.and_hms_opt(0, 0, 0)?
            .checked_add_signed(Duration::days(whole_days))?
            .checked_add_signed(Duration::milliseconds(milliseconds))
    }

    /// Convert a `NaiveDateTime` to an Excel serial number.
    pub fn datetime_to_serial(self, dt: NaiveDateTime) -> f64 {
        let date = dt.date();
        let time = dt.time();
        let seconds = f64::from(time.num_seconds_from_midnight())
            + f64::from(time.nanosecond()) / 1_000_000_000.0;
        let frac = seconds / 86_400.0;
        let day_serial = self.date_to_serial(date);
        f64::from(i32::try_from(day_serial).unwrap_or_else(|_| {
            if day_serial.is_negative() {
                i32::MIN
            } else {
                i32::MAX
            }
        })) + frac
    }

    /// Convert just a date to its integer serial.
    pub fn date_to_serial(self, date: NaiveDate) -> i64 {
        match self {
            DateSystem::Date1900 => {
                let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                let mut serial = (date - base).num_days();
                // Excel's phantom 1900-02-29 shifts everything from 1900-03-01 on.
                if date < NaiveDate::from_ymd_opt(1900, 3, 1).unwrap() {
                    serial -= 1;
                }
                serial
            }
            DateSystem::Date1904 => {
                let base = NaiveDate::from_ymd_opt(1904, 1, 1).unwrap();
                (date - base).num_days()
            }
        }
    }

    /// Translate a serial between this system and the other (used on XLS/XLSX
    /// round-trip when the date system changes).
    pub fn convert_serial_to(self, serial: f64, other: DateSystem) -> f64 {
        match (self, other) {
            (DateSystem::Date1900, DateSystem::Date1904) => serial - EPOCH_DIFF_1904,
            (DateSystem::Date1904, DateSystem::Date1900) => serial + EPOCH_DIFF_1904,
            _ => serial,
        }
    }
}

/// Build a serial from y/m/d, used by the DATE() worksheet function.
pub fn ymd_to_serial(system: DateSystem, year: i32, month: u32, day: u32) -> Option<f64> {
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(system.date_to_serial(date) as f64)
}

/// Extract the time-of-day fraction's H/M/S, used by HOUR/MINUTE/SECOND.
pub fn serial_time_parts(serial: f64) -> (u32, u32, u32) {
    let frac = serial.fract().abs();
    let total = (frac * 86400.0).round() as u32 % 86400;
    (total / 3600, (total % 3600) / 60, total % 60)
}

/// Parse `text` as a date/time using an **Excel-style** format code (e.g.
/// `dd/mm/yyyy`, `dd-mmm-yy`, `dd/mm/yyyy hh:mm:ss`, `hh:mm AM/PM`) and return
/// the Excel serial for `system`. Returns `None` if the text doesn't match.
///
/// This is the parser behind the `to-date` command — the date twin of
/// `parse_number_text`. The Excel `m`/`mm` token means **minute** when adjacent
/// to an hour or second token, else **month** (Excel's rule).
pub fn parse_text_date(text: &str, excel_fmt: &str, system: DateSystem) -> Option<f64> {
    use chrono::Timelike;
    let (chrono_fmt, has_date, has_time) = excel_format_to_chrono(excel_fmt)?;
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if has_date && has_time {
        let dt = NaiveDateTime::parse_from_str(text, &chrono_fmt).ok()?;
        Some(system.datetime_to_serial(dt))
    } else if has_date {
        let d = NaiveDate::parse_from_str(text, &chrono_fmt).ok()?;
        Some(system.date_to_serial(d) as f64)
    } else if has_time {
        let t = NaiveTime::parse_from_str(text, &chrono_fmt).ok()?;
        Some(t.num_seconds_from_midnight() as f64 / 86400.0)
    } else {
        None
    }
}

/// One token of an Excel date/time format code.
enum FmtTok {
    Letter(char, usize), // (y/m/d/h/s, run length); `m` resolved later
    AmPm,
    Lit(char),
}

/// Translate an Excel-style date format into a `chrono` parse format, returning
/// `(chrono_fmt, has_date, has_time)`. Returns `None` if no date/time tokens.
fn excel_format_to_chrono(fmt: &str) -> Option<(String, bool, bool)> {
    let chars: Vec<char> = fmt.chars().collect();
    let lower: Vec<char> = fmt.to_lowercase().chars().collect();

    // 1. Tokenize, treating AM/PM markers specially.
    let mut toks: Vec<FmtTok> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if lower[i..].starts_with(&['a', 'm', '/', 'p', 'm']) {
            toks.push(FmtTok::AmPm);
            i += 5;
            continue;
        }
        if lower[i..].starts_with(&['a', '/', 'p']) {
            toks.push(FmtTok::AmPm);
            i += 3;
            continue;
        }
        let c = lower[i];
        if matches!(c, 'y' | 'm' | 'd' | 'h' | 's') {
            let start = i;
            while i < chars.len() && lower[i] == c {
                i += 1;
            }
            toks.push(FmtTok::Letter(c, i - start));
        } else {
            toks.push(FmtTok::Lit(chars[i]));
            i += 1;
        }
    }

    let twelve_hour = toks.iter().any(|t| matches!(t, FmtTok::AmPm));

    // 2. Emit chrono specifiers, resolving `m` (month vs minute) by neighbours.
    let letter_at = |idx: usize| -> Option<char> {
        toks.get(idx).and_then(|t| match t {
            FmtTok::Letter(c, _) => Some(*c),
            _ => None,
        })
    };
    let prev_letter = |idx: usize| -> Option<char> { (0..idx).rev().find_map(letter_at) };
    let next_letter = |idx: usize| -> Option<char> { ((idx + 1)..toks.len()).find_map(letter_at) };

    let mut out = String::new();
    let mut has_date = false;
    let mut has_time = false;
    for (idx, tk) in toks.iter().enumerate() {
        match tk {
            FmtTok::AmPm => {
                out.push_str("%p");
                has_time = true;
            }
            FmtTok::Lit('%') => out.push_str("%%"),
            FmtTok::Lit(c) => out.push(*c),
            FmtTok::Letter('y', len) => {
                has_date = true;
                out.push_str(if *len <= 2 { "%y" } else { "%Y" });
            }
            FmtTok::Letter('d', len) => match len {
                n if *n >= 4 => out.push_str("%A"), // dddd weekday name
                3 => out.push_str("%a"),            // ddd weekday abbr
                _ => {
                    has_date = true;
                    out.push_str("%d");
                }
            },
            FmtTok::Letter('h', _) => {
                has_time = true;
                out.push_str(if twelve_hour { "%I" } else { "%H" });
            }
            FmtTok::Letter('s', _) => {
                has_time = true;
                out.push_str("%S");
            }
            FmtTok::Letter('m', len) => {
                // Minute when adjacent to an hour (before) or second (after).
                let is_minute = prev_letter(idx) == Some('h') || next_letter(idx) == Some('s');
                if is_minute {
                    has_time = true;
                    out.push_str("%M");
                } else {
                    has_date = true;
                    out.push_str(match len {
                        n if *n >= 4 => "%B", // full month name
                        3 => "%b",            // abbreviated month name
                        _ => "%m",
                    });
                }
            }
            FmtTok::Letter(_, _) => {}
        }
    }

    if !has_date && !has_time {
        return None;
    }
    Some((out, has_date, has_time))
}

/// Heuristic: does `s` *look like* a date stored as text? Used by `profile` to
/// flag text-stored dates (a soft hint to run `to-date`), not a hard claim.
pub fn looks_like_date(s: &str) -> bool {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r"(?ix)
            ^\s*(
                \d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}        # 04/04/2025, 2025-04-04, 4.4.25
              | \d{1,2}[-/\ ][A-Za-z]{3,9}[-/\ ]\d{2,4} # 04-Apr-2025
              | [A-Za-z]{3,9}\s+\d{1,2},?\s+\d{2,4}    # Apr 4, 2025
            )\s*$",
        )
        .expect("valid date heuristic regex")
    });
    re.is_match(s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_1900_dates() {
        let s = DateSystem::Date1900;
        // 1900-01-01 is serial 1.
        assert_eq!(
            s.date_to_serial(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()),
            1
        );
        // 1900-03-01 is serial 61 (phantom 02-29 took serial 60).
        assert_eq!(
            s.date_to_serial(NaiveDate::from_ymd_opt(1900, 3, 1).unwrap()),
            61
        );
        // 2008-12-31 is the classic serial 39813.
        assert_eq!(
            s.date_to_serial(NaiveDate::from_ymd_opt(2008, 12, 31).unwrap()),
            39813
        );
    }

    #[test]
    fn roundtrip_1900() {
        let s = DateSystem::Date1900;
        let dt = s.serial_to_datetime(39813.5).unwrap();
        assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2008, 12, 31).unwrap());
        assert_eq!(dt.time().hour(), 12);
        assert_eq!(s.datetime_to_serial(dt), 39813.5);
    }

    #[test]
    fn epoch_1904() {
        let s = DateSystem::Date1904;
        assert_eq!(
            s.date_to_serial(NaiveDate::from_ymd_opt(1904, 1, 1).unwrap()),
            0
        );
        // Same calendar date is 1462 less in 1904 system.
        let d = NaiveDate::from_ymd_opt(2008, 12, 31).unwrap();
        assert_eq!(
            DateSystem::Date1900.date_to_serial(d) - s.date_to_serial(d),
            1462
        );
    }

    #[test]
    fn convert_between() {
        assert_eq!(
            DateSystem::Date1900.convert_serial_to(1462.0, DateSystem::Date1904),
            0.0
        );
    }

    #[test]
    fn parse_text_date_day_first() {
        let s = DateSystem::Date1900;
        // Day-first format: 04/04/2025 must be April 4 (not mm/dd).
        let got = parse_text_date("04/04/2025", "dd/mm/yyyy", s).unwrap();
        assert_eq!(got, ymd_to_serial(s, 2025, 4, 4).unwrap());
        // A day > 12 only parses correctly as day-first.
        let got = parse_text_date("13/04/2025", "dd/mm/yyyy", s).unwrap();
        assert_eq!(got, ymd_to_serial(s, 2025, 4, 13).unwrap());
        // The same text under mm/dd is rejected (month 13 invalid).
        assert!(parse_text_date("13/04/2025", "mm/dd/yyyy", s).is_none());
    }

    #[test]
    fn parse_text_date_named_month_and_iso() {
        let s = DateSystem::Date1900;
        assert_eq!(
            parse_text_date("04-Apr-2025", "dd-mmm-yyyy", s).unwrap(),
            ymd_to_serial(s, 2025, 4, 4).unwrap()
        );
        assert_eq!(
            parse_text_date("2025-04-04", "yyyy-mm-dd", s).unwrap(),
            ymd_to_serial(s, 2025, 4, 4).unwrap()
        );
    }

    #[test]
    fn parse_text_date_with_time() {
        let s = DateSystem::Date1900;
        // m after h is a minute, not a month.
        let serial = parse_text_date("04/04/2025 16:05:50", "dd/mm/yyyy hh:mm:ss", s).unwrap();
        let day = ymd_to_serial(s, 2025, 4, 4).unwrap();
        assert_eq!(serial.trunc(), day);
        let (h, m, sec) = serial_time_parts(serial);
        assert_eq!((h, m, sec), (16, 5, 50));
    }

    #[test]
    fn parse_text_date_rejects_garbage() {
        let s = DateSystem::Date1900;
        assert!(parse_text_date("not a date", "dd/mm/yyyy", s).is_none());
        assert!(parse_text_date("6,000.00", "dd/mm/yyyy", s).is_none());
    }

    #[test]
    fn looks_like_date_heuristic() {
        assert!(looks_like_date("04/04/2025"));
        assert!(looks_like_date("2025-04-04"));
        assert!(looks_like_date("04-Apr-2025"));
        assert!(looks_like_date("Apr 4, 2025"));
        assert!(!looks_like_date("6,000.00")); // currency, not a date
        assert!(!looks_like_date("hello"));
        assert!(!looks_like_date(""));
    }
}
