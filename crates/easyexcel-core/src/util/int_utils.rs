//! Mirrors Java com.alibaba.excel.util.IntUtils.

#![allow(dead_code)]

/// Mirrors `com.google.common.primitives.Ints#saturatedCast`.
///
/// Clamps a wider integer (`i64`) into `i32` instead of panicking on
/// overflow: values outside `i32::MIN..=i32::MAX` are clipped to the
/// nearest bound.
#[must_use]
pub fn saturated_cast(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < i32::MIN as i64 {
        i32::MIN
    } else {
        value as i32
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn saturated_cast_clamps_in_range() {
        // 对应 Java：Ints.saturatedCast 直接转换
        assert_eq!(saturated_cast(0), 0);
        assert_eq!(saturated_cast(42), 42);
        assert_eq!(saturated_cast(i32::MAX as i64), i32::MAX);
        assert_eq!(saturated_cast(i32::MIN as i64), i32::MIN);
    }

    #[test]
    fn saturated_cast_clamps_out_of_range() {
        // 对应 Java：超出范围时收敛到边界而非溢出
        assert_eq!(saturated_cast(i32::MAX as i64 + 1), i32::MAX);
        assert_eq!(saturated_cast(i64::MAX), i32::MAX);
        assert_eq!(saturated_cast(i32::MIN as i64 - 1), i32::MIN);
        assert_eq!(saturated_cast(i64::MIN), i32::MIN);
    }
}
