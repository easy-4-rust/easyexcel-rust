//! 对应 Java：`com.alibaba.excel.util.BooleanUtils` 的可复用实现。

#![allow(dead_code)]

/// 对应 Java：com.alibaba.excel.util.BooleanUtils。 Mirrors `org.apache.commons.lang3.BooleanUtils#valueOf`.
#[must_use]
pub fn value_of(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "t" | "yes" | "y" | "on" | "1"
    )
}

/// 对应 Java：com.alibaba.excel.util.BooleanUtils。 Mirrors `org.apache.commons.lang3.BooleanUtils#isTrue`.
#[must_use]
pub fn is_true(value: Option<bool>) -> bool {
    matches!(value, Some(true))
}

/// 对应 Java：com.alibaba.excel.util.BooleanUtils。 Mirrors `org.apache.commons.lang3.BooleanUtils#isNotTrue`.
#[must_use]
pub fn is_not_true(value: Option<bool>) -> bool {
    !is_true(value)
}

/// 对应 Java：com.alibaba.excel.util.BooleanUtils。 Mirrors `org.apache.commons.lang3.BooleanUtils#isFalse`.
#[must_use]
pub fn is_false(value: Option<bool>) -> bool {
    matches!(value, Some(false))
}

/// 对应 Java：com.alibaba.excel.util.BooleanUtils。 Mirrors `org.apache.commons.lang3.BooleanUtils#isNotFalse`.
#[must_use]
pub fn is_not_false(value: Option<bool>) -> bool {
    !is_false(value)
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn value_of_accepts_true_forms() {
        // 对应 Java：BooleanUtils.valueOf 各种 true 写法
        for text in [
            "true", "TRUE", "t", "T", "yes", "YES", "y", "on", "1", " true ",
        ] {
            assert!(value_of(text), "{text} should be true");
        }
    }

    #[test]
    fn value_of_rejects_false_and_unknown_forms() {
        // 对应 Java：其余输入视为 false
        for text in ["false", "no", "off", "0", "2", "", "maybe", "yesx"] {
            assert!(!value_of(text), "{text} should be false");
        }
    }

    #[test]
    fn is_true_and_false_families() {
        // 对应 Java：BooleanUtils isTrue/isNotTrue/isFalse/isNotFalse
        assert!(is_true(Some(true)));
        assert!(!is_true(Some(false)));
        assert!(!is_true(None));

        assert!(!is_not_true(Some(true)));
        assert!(is_not_true(Some(false)));
        assert!(is_not_true(None));

        assert!(is_false(Some(false)));
        assert!(!is_false(Some(true)));
        assert!(!is_false(None));

        assert!(!is_not_false(Some(false)));
        assert!(is_not_false(Some(true)));
        assert!(is_not_false(None));
    }
}
