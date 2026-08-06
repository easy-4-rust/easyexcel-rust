//! 对应 Java：`com.alibaba.excel.util.ListUtils` 的可复用实现。

#![allow(dead_code)]

use std::vec::Vec;

/// 对应 Java：com.alibaba.excel.util.ListUtils。 Mirrors `org.apache.commons.collections4.ListUtils#newArrayList` /
/// the `EasyExcel` helper that wraps `new ArrayList<>()`.
#[must_use]
pub fn new_array_list<T>() -> Vec<T> {
    Vec::new()
}

/// 对应 Java：com.alibaba.excel.util.ListUtils。 Mirrors `com.alibaba.excel.util.ListUtils#newArrayListWithCapacity`.
#[must_use]
pub fn new_array_list_with_capacity<T>(capacity: usize) -> Vec<T> {
    Vec::with_capacity(capacity)
}

/// 对应 Java：com.alibaba.excel.util.ListUtils。 Mirrors `com.google.common.collect.Lists#newArrayListWithExpectedSize`.
#[must_use]
pub fn new_array_list_with_expected_size<T>(expected_size: usize) -> Vec<T> {
    // Guava's sizing: 1.5 * expected + 1, capped by isize::MAX.
    let cap = expected_size + (expected_size >> 1) + 1;
    Vec::with_capacity(cap)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_array_list_creates_empty() {
        // 对应 Java：ListUtils.newArrayList
        let list: Vec<i32> = new_array_list();
        assert!(list.is_empty());
    }

    #[test]
    fn new_array_list_with_capacity_reserves() {
        // 对应 Java：ListUtils.newArrayListWithCapacity
        let list: Vec<i32> = new_array_list_with_capacity(8);
        assert!(list.is_empty());
        assert!(list.capacity() >= 8);
    }

    #[test]
    fn new_array_list_with_expected_size_uses_guava_sizing() {
        // 对应 Java：Guava 1.5*expected+1 容量估算
        let list: Vec<i32> = new_array_list_with_expected_size(4);
        assert!(list.capacity() >= 7);
        let zero: Vec<i32> = new_array_list_with_expected_size(0);
        assert!(zero.capacity() >= 1);
    }
}
