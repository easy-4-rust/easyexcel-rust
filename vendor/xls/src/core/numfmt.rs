//! Number-format handling: detect date formats and apply format codes to values
//! for display. This is a pragmatic subset of Excel's format-code language —
//! enough for faithful TUI/CLI display and round-trip preservation of the codes.

use super::dates::DateSystem;
use chrono::{Datelike, Timelike};

/// Built-in number format IDs that always denote dates/times (no FORMAT record
/// needed). Per ECMA-376 / MS-XLS.
pub const BUILTIN_DATE_IDS: &[u16] = &[14, 15, 16, 17, 18, 19, 20, 21, 22, 45, 46, 47];

/// The built-in format code for a given numFmtId, if it is one of the reserved
/// IDs (0–49). Returns `None` for custom IDs (>= 164) and unknown ones.
pub fn builtin_format_code(id: u16) -> Option<&'static str> {
    Some(match id {
        0 => "General",
        1 => "0",
        2 => "0.00",
        3 => "#,##0",
        4 => "#,##0.00",
        9 => "0%",
        10 => "0.00%",
        11 => "0.00E+00",
        12 => "# ?/?",
        13 => "# ??/??",
        14 => "mm-dd-yy",
        15 => "d-mmm-yy",
        16 => "d-mmm",
        17 => "mmm-yy",
        18 => "h:mm AM/PM",
        19 => "h:mm:ss AM/PM",
        20 => "h:mm",
        21 => "h:mm:ss",
        22 => "m/d/yy h:mm",
        37 => "#,##0 ;(#,##0)",
        38 => "#,##0 ;[Red](#,##0)",
        39 => "#,##0.00;(#,##0.00)",
        40 => "#,##0.00;[Red](#,##0.00)",
        45 => "mm:ss",
        46 => "[h]:mm:ss",
        47 => "mmss.0",
        48 => "##0.0E+0",
        49 => "@",
        _ => return None,
    })
}

/// Heuristic: does this format code represent a date/time? Looks for unescaped
/// date/time tokens (`y`, `m`, `d`, `h`, `s`) outside of literal/quoted spans.
pub fn is_date_format(code: &str) -> bool {
    let lower = code.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    let mut in_quote = false;
    let mut in_bracket = false;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'"' => in_quote = !in_quote,
            b'[' if !in_quote => in_bracket = true,
            b']' if !in_quote => in_bracket = false,
            b'\\' => {
                i += 2;
                continue;
            }
            b'y' | b'd' | b'h' | b's' if !in_quote && !in_bracket => return true,
            b'm' if !in_quote && !in_bracket => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

/// True for IDs that are date formats (covers built-in IDs cheaply).
pub fn is_date_format_id(id: u16, custom_code: Option<&str>) -> bool {
    if BUILTIN_DATE_IDS.contains(&id) {
        return true;
    }
    match custom_code {
        Some(code) => is_date_format(code),
        None => false,
    }
}

/// Apply a number-format code to a numeric value, producing display text.
///
/// Supports General, the common date/time tokens, percent, thousands grouping,
/// fixed decimals, and a leading text section. Falls back to General rendering
/// for anything not understood — never panics.
pub fn format_value(value: f64, code: &str, system: DateSystem) -> String {
    let sections = split_sections(code);
    let has_neg_section = sections.len() > 1;
    // Select the applicable section and decide on sign handling. A dedicated
    // negative section formats the magnitude (its literal text carries the sign);
    // otherwise a single section prefixes `-` to negatives itself.
    let (section, render_value, add_sign): (&str, f64, bool) = if value < 0.0 {
        if has_neg_section {
            (sections[1], value.abs(), false)
        } else {
            (sections[0], value.abs(), true)
        }
    } else if value == 0.0 {
        (sections.get(2).copied().unwrap_or(sections[0]), 0.0, false)
    } else {
        (sections[0], value, false)
    };

    if section.eq_ignore_ascii_case("general") || section.is_empty() || section == "@" {
        return super::value::format_number_general(value);
    }
    if is_date_format(section) {
        return format_datetime(value, section, system);
    }
    format_numeric(render_value, section, add_sign)
}

fn split_sections(code: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = code.as_bytes();
    let mut start = 0;
    let mut in_quote = false;
    let mut in_bracket = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_quote = !in_quote,
            b'[' if !in_quote => in_bracket = true,
            b']' if !in_quote => in_bracket = false,
            b'\\' => {
                i += 2;
                continue;
            }
            b';' if !in_quote && !in_bracket => {
                out.push(&code[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&code[start..]);
    out
}

fn format_datetime(value: f64, code: &str, system: DateSystem) -> String {
    let Some(dt) = system.serial_to_datetime(value) else {
        return super::value::format_number_general(value);
    };
    let has_ampm =
        code.to_ascii_lowercase().contains("am/pm") || code.to_ascii_lowercase().contains("a/p");
    let mut out = String::new();
    let bytes = code.as_bytes();
    let lower = code.to_ascii_lowercase();
    let lb = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = lb[i];
        match c {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                i += 1;
            }
            b'\\' => {
                if i + 1 < bytes.len() {
                    out.push(bytes[i + 1] as char);
                }
                i += 2;
            }
            b'[' => {
                // elapsed-time token like [h] or [mm]
                let start = i;
                while i < bytes.len() && bytes[i] != b']' {
                    i += 1;
                }
                let tok = &lower[start + 1..i.min(bytes.len())];
                let total_secs = (value.fract() * 86400.0).round() as i64
                    + system.date_to_serial(dt.date()) * 86400;
                if tok.starts_with('h') {
                    out.push_str(&(total_secs / 3600).to_string());
                } else if tok.starts_with('m') {
                    out.push_str(&(total_secs / 60).to_string());
                } else if tok.starts_with('s') {
                    out.push_str(&total_secs.to_string());
                }
                i += 1;
            }
            b'y' => {
                let n = run_len(lb, i, b'y');
                if n >= 4 {
                    out.push_str(&format!("{:04}", dt.year()));
                } else {
                    out.push_str(&format!("{:02}", dt.year() % 100));
                }
                i += n;
            }
            b'm' => {
                let n = run_len(lb, i, b'm');
                // Disambiguate minutes vs months: if adjacent to h/s it's minutes.
                let minutes = near_time_context(lb, i);
                if minutes {
                    out.push_str(&fmt_pad(dt.minute() as i64, n));
                } else {
                    match n {
                        1 => out.push_str(&dt.month().to_string()),
                        2 => out.push_str(&format!("{:02}", dt.month())),
                        3 => out.push_str(MONTHS_ABBR[(dt.month() - 1) as usize]),
                        _ => out.push_str(MONTHS_FULL[(dt.month() - 1) as usize]),
                    }
                }
                i += n;
            }
            b'd' => {
                let n = run_len(lb, i, b'd');
                match n {
                    1 => out.push_str(&dt.day().to_string()),
                    2 => out.push_str(&format!("{:02}", dt.day())),
                    3 => out.push_str(WEEKDAYS_ABBR[dt.weekday().num_days_from_sunday() as usize]),
                    _ => out.push_str(WEEKDAYS_FULL[dt.weekday().num_days_from_sunday() as usize]),
                }
                i += n;
            }
            b'h' => {
                let n = run_len(lb, i, b'h');
                let mut h = dt.hour() as i64;
                if has_ampm {
                    h %= 12;
                    if h == 0 {
                        h = 12;
                    }
                }
                out.push_str(&fmt_pad(h, n));
                i += n;
            }
            b's' => {
                let n = run_len(lb, i, b's');
                out.push_str(&fmt_pad(dt.second() as i64, n));
                i += n;
            }
            b'a' if lower[i..].starts_with("am/pm") => {
                out.push_str(if dt.hour() < 12 { "AM" } else { "PM" });
                i += 5;
            }
            b'a' if lower[i..].starts_with("a/p") => {
                out.push_str(if dt.hour() < 12 { "A" } else { "P" });
                i += 3;
            }
            _ => {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
    }
    out
}

fn run_len(b: &[u8], start: usize, ch: u8) -> usize {
    let mut n = 0;
    while start + n < b.len() && b[start + n] == ch {
        n += 1;
    }
    n
}

fn fmt_pad(v: i64, width: usize) -> String {
    if width >= 2 {
        format!("{v:02}")
    } else {
        v.to_string()
    }
}

/// Decide whether an `m` run denotes minutes by looking for a neighbouring
/// hour/second token.
fn near_time_context(b: &[u8], pos: usize) -> bool {
    // look backwards skipping the m run and separators for h
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match b[i] {
            b'm' | b':' | b' ' => continue,
            b'h' => return true,
            _ => break,
        }
    }
    // look forward for s
    let mut j = pos;
    while j < b.len() && b[j] == b'm' {
        j += 1;
    }
    while j < b.len() {
        match b[j] {
            b':' | b' ' => j += 1,
            b's' => return true,
            _ => break,
        }
    }
    false
}

/// Format `value` (already made non-negative when a negative section applies)
/// against a numeric `code`, emitting the code's literal characters and
/// substituting the formatted number in place of its placeholder run.
fn format_numeric(value: f64, code: &str, add_sign: bool) -> String {
    let is_percent = code.contains('%');
    let v = if is_percent { value * 100.0 } else { value };
    let grouping = code.contains("#,##") || code.contains(",#");

    // Count fractional digit placeholders after the dot.
    let decimals = match code.split_once('.') {
        Some((_, frac)) => frac.chars().take_while(|c| *c == '0' || *c == '#').count(),
        None => 0,
    };

    let mut numstr = format!("{:.*}", decimals, v.abs());
    if grouping {
        numstr = group_thousands(&numstr);
    }
    if add_sign {
        numstr.insert(0, '-');
    }

    // Walk the code, emitting literals and replacing the first placeholder run.
    let bytes = code.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    let mut emitted = false;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                i += 1;
            }
            b'\\' => {
                if i + 1 < bytes.len() {
                    out.push(bytes[i + 1] as char);
                }
                i += 2;
            }
            b'0' | b'#' | b'?' | b'.' | b',' => {
                if !emitted {
                    out.push_str(&numstr);
                    emitted = true;
                }
                i += 1;
                while i < bytes.len() && matches!(bytes[i], b'0' | b'#' | b'?' | b'.' | b',') {
                    i += 1;
                }
            }
            b'%' => {
                out.push('%');
                i += 1;
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    if !emitted {
        out.push_str(&numstr);
    }
    out
}

fn group_thousands(s: &str) -> String {
    let (int_part, frac_part) = match s.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let mut grouped = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    let len = chars.len();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(*c);
    }
    match frac_part {
        Some(f) => format!("{grouped}.{f}"),
        None => grouped,
    }
}

const MONTHS_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS_ABBR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const WEEKDAYS_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dates() {
        assert!(is_date_format("yyyy-mm-dd"));
        assert!(is_date_format("h:mm:ss"));
        assert!(is_date_format("d-mmm-yy"));
        assert!(!is_date_format("#,##0.00"));
        assert!(!is_date_format("0%"));
        assert!(!is_date_format(r#""day "0"#));
    }

    #[test]
    fn builtin_date_ids() {
        assert!(is_date_format_id(14, None));
        assert!(!is_date_format_id(2, None));
        assert!(is_date_format_id(200, Some("yyyy")));
    }

    #[test]
    fn numeric_formats() {
        let sys = DateSystem::Date1900;
        assert_eq!(format_value(1234.5, "#,##0.00", sys), "1,234.50");
        assert_eq!(format_value(0.25, "0%", sys), "25%");
        assert_eq!(format_value(-5.0, "0.0", sys), "-5.0");
        assert_eq!(format_value(1234.0, "$#,##0", sys), "$1,234");
    }

    #[test]
    fn date_formats() {
        let sys = DateSystem::Date1900;
        // 2008-12-31 == serial 39813
        assert_eq!(format_value(39813.0, "yyyy-mm-dd", sys), "2008-12-31");
        assert_eq!(format_value(39813.0, "d-mmm-yy", sys), "31-Dec-08");
    }

    #[test]
    fn sections() {
        // negative numbers use the 2nd section
        assert_eq!(
            format_value(-5.0, "0.0;(0.0)", DateSystem::Date1900),
            "(5.0)"
        );
    }
}
