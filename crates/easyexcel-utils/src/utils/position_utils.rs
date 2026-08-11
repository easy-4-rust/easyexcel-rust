//! Java `com.alibaba.excel.util.PositionUtils` 的可复用坐标算法。

/// 根据 OOXML `row` 标签的一基行号返回零基行号。
///
/// 标签缺失时，`before == None` 从 0 开始，否则返回 `before + 1`。
#[must_use]
pub fn get_row_by_row_tagt(row_tagt: Option<&str>, before: Option<i32>) -> i32 {
    row_tagt.map_or_else(
        || before.unwrap_or(-1).saturating_add(1),
        |value| {
            value
                .parse::<i32>()
                .expect("row tag must contain an unsigned decimal row number")
                .saturating_sub(1)
        },
    )
}

/// 从 A1 引用读取零基行号；引用缺失返回 -1。
#[must_use]
pub fn get_row(current_cell_index: Option<&str>) -> i32 {
    let Some(reference) = current_cell_index else {
        return -1;
    };
    let digit_start = reference
        .char_indices()
        .rev()
        .find_map(|(index, value)| (!value.is_ascii_digit()).then_some(index + value.len_utf8()))
        .unwrap_or(0);
    reference[digit_start..]
        .parse::<u32>()
        .expect("cell reference must end with an unsigned decimal row number")
        .saturating_sub(1) as i32
}

/// 从 A1 引用读取零基列号；引用缺失时按 `before` 递增。
#[must_use]
pub fn get_col(current_cell_index: Option<&str>, before: Option<i32>) -> i32 {
    let Some(reference) = current_cell_index else {
        return before.unwrap_or(-1).saturating_add(1);
    };
    let mut column = 0_i32;
    for value in reference.trim_start_matches('$').chars() {
        if value == '$' || value.is_ascii_digit() {
            break;
        }
        let upper = value.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            break;
        }
        column = column
            .saturating_mul(26)
            .saturating_add(i32::from(upper as u8 - b'A' + 1));
    }
    column - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── get_row_by_row_tagt ───────────────────────────────────────────

    #[test]
    fn row_tagt_none_before_none_returns_zero() {
        // tagt=None, before=None → (-1).saturating_add(1) = 0
        assert_eq!(get_row_by_row_tagt(None, None), 0);
    }

    #[test]
    fn row_tagt_none_before_some_increments() {
        // tagt=None, before=5 → 5+1 = 6
        assert_eq!(get_row_by_row_tagt(None, Some(5)), 6);
    }

    #[test]
    fn row_tagt_some_parses_and_subtracts_one() {
        // "1" → 1-1 = 0
        assert_eq!(get_row_by_row_tagt(Some("1"), None), 0);
        // "10" → 10-1 = 9
        assert_eq!(get_row_by_row_tagt(Some("10"), None), 9);
    }

    #[test]
    fn row_tagt_some_ignores_before() {
        // before 参数在 tagt 存在时不使用
        assert_eq!(get_row_by_row_tagt(Some("5"), Some(100)), 4);
    }

    // ── get_row ───────────────────────────────────────────────────────

    #[test]
    fn get_row_none_returns_minus_one() {
        assert_eq!(get_row(None), -1);
    }

    #[test]
    fn get_row_simple_a1() {
        // "A1" → 数字部分 "1" → 1-1 = 0
        assert_eq!(get_row(Some("A1")), 0);
    }

    #[test]
    fn get_row_with_dollar_sign() {
        // "$A$10" → 数字部分 "10" → 10-1 = 9
        assert_eq!(get_row(Some("$A$10")), 9);
    }

    #[test]
    fn get_row_large_number() {
        // "B100" → 100-1 = 99
        assert_eq!(get_row(Some("B100")), 99);
    }

    #[test]
    fn get_row_mixed_ref() {
        // "A$5" → 数字部分 "5" → 5-1 = 4
        assert_eq!(get_row(Some("A$5")), 4);
    }

    // ── get_col ───────────────────────────────────────────────────────

    #[test]
    fn get_col_none_before_none_returns_zero() {
        // None, None → (-1).saturating_add(1) = 0
        assert_eq!(get_col(None, None), 0);
    }

    #[test]
    fn get_col_none_before_some_increments() {
        // None, Some(3) → 3+1 = 4
        assert_eq!(get_col(None, Some(3)), 4);
    }

    #[test]
    fn get_col_simple_a() {
        // "A1" → A=1, 1-1=0
        assert_eq!(get_col(Some("A1"), None), 0);
    }

    #[test]
    fn get_col_b() {
        // "B1" → B=2, 2-1=1
        assert_eq!(get_col(Some("B1"), None), 1);
    }

    #[test]
    fn get_col_z() {
        // "Z1" → Z=26, 26-1=25
        assert_eq!(get_col(Some("Z1"), None), 25);
    }

    #[test]
    fn get_col_aa() {
        // "AA1" → A*26+A = 1*26+1=27, 27-1=26
        assert_eq!(get_col(Some("AA1"), None), 26);
    }

    #[test]
    fn get_col_az() {
        // "AZ1" → A*26+Z = 1*26+26=52, 52-1=51
        assert_eq!(get_col(Some("AZ1"), None), 51);
    }

    #[test]
    fn get_col_with_dollar_prefix() {
        // "$A$1" → trim_start_matches('$') → "A$1", break at '$'
        assert_eq!(get_col(Some("$A$1"), None), 0);
    }

    #[test]
    fn get_col_lowercase_input() {
        // "b1" → to_ascii_uppercase → B=2, 2-1=1
        assert_eq!(get_col(Some("b1"), None), 1);
    }

    #[test]
    fn get_col_before_ignored_when_ref_present() {
        // ref 存在时 before 不使用
        assert_eq!(get_col(Some("A1"), Some(100)), 0);
    }
}
