//! 对应 Java：`com.alibaba.excel.util.IntUtils` 的可复用实现。

#![allow(dead_code)]

/// 返回无符号 32 位整数的低 8 位。
///
/// 供 Java `short`/POI 调色板等兼容入口使用；位级截断语义集中在基础工具层。
#[must_use]
pub const fn low_u8(value: u32) -> u8 {
    value.to_le_bytes()[0]
}

/// Mirrors `com.google.common.primitives.Ints#saturatedCast`.
///
/// Clamps a wider integer (`i64`) into `i32` instead of panicking on
/// overflow: values outside `i32::MIN..=i32::MAX` are clipped to the
/// nearest bound.
///
/// # Panics
///
/// 不会 panic（范围检查保证转换恒成功，`expect` 仅为静态证明）。
#[must_use]
pub fn saturated_cast(value: i64) -> i32 {
    if value > i64::from(i32::MAX) {
        i32::MAX
    } else if value < i64::from(i32::MIN) {
        i32::MIN
    } else {
        // 已由上面的范围检查保证 value 落在 i32 范围内，try_from 恒成功
        i32::try_from(value).expect("saturated_cast 范围检查保证 value 在 i32 范围内")
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
        assert_eq!(saturated_cast(i64::from(i32::MAX)), i32::MAX);
        assert_eq!(saturated_cast(i64::from(i32::MIN)), i32::MIN);
    }

    #[test]
    fn saturated_cast_clamps_out_of_range() {
        // 对应 Java：超出范围时收敛到边界而非溢出
        assert_eq!(saturated_cast(i64::from(i32::MAX) + 1), i32::MAX);
        assert_eq!(saturated_cast(i64::MAX), i32::MAX);
        assert_eq!(saturated_cast(i64::from(i32::MIN) - 1), i32::MIN);
        assert_eq!(saturated_cast(i64::MIN), i32::MIN);
    }
}
