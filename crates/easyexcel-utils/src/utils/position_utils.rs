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
