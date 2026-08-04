//! 对应 Java：`com.alibaba.excel.enums.BooleanEnum`.
//!
//! Java uses `BooleanEnum { DEFAULT(null), TRUE, FALSE }` so annotations can
//! distinguish "unset" from "false". Rust uses `Option<bool>` for the same
//! effect, but we keep this enum for API compatibility with the Java
//! annotation model.

/// Tri-state boolean matching Java `BooleanEnum`.
///
/// 对应 Java：`BooleanEnum`. The `Default` variant carries `None` so that an
/// annotation that omits the field can be detected and distinguished from
/// `false`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BooleanEnum {
    /// Sentinel for "annotation did not specify this field".
    #[default]
    Default,
    /// Explicit `true`.
    True,
    /// Explicit `false`.
    False,
}

impl BooleanEnum {
    /// Resolves to a nullable `bool`.
    #[must_use]
    pub const fn value(self) -> Option<bool> {
        match self {
            Self::Default => None,
            Self::True => Some(true),
            Self::False => Some(false),
        }
    }
}

impl From<BooleanEnum> for Option<bool> {
    fn from(value: BooleanEnum) -> Self {
        value.value()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn into_option_bool_maps_all_variants() {
        // 对应 Java：BooleanEnum 转 Option<bool>
        let default: Option<bool> = BooleanEnum::Default.into();
        assert_eq!(default, None);
        let truthy: Option<bool> = BooleanEnum::True.into();
        assert_eq!(truthy, Some(true));
        let falsy: Option<bool> = BooleanEnum::False.into();
        assert_eq!(falsy, Some(false));
    }
}
