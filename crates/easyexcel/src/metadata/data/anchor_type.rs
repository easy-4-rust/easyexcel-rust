//! 对应 Java：`com.alibaba.excel.metadata.data.ClientAnchorData.AnchorType`.

/// Java `ClientAnchorData.AnchorType` equivalent.
///
/// Variant names are normalised to `PascalCase` while preserving the four POI
/// anchor modes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
/// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData.AnchorType。
pub enum AnchorType {
    /// Move and resize with the anchor cells.
    #[default]
    MoveAndResize,
    /// POI's completeness-only mode; XLSX serializes it as a one-cell anchor.
    DontMoveDoResize,
    /// Move with cells without resizing.
    MoveDontResize,
    /// Do not move or resize with cells.
    DontMoveAndResize,
}

impl AnchorType {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 4] = [
        Self::MoveAndResize, Self::DontMoveDoResize,
        Self::MoveDontResize, Self::DontMoveAndResize,
    ];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self {
            Self::MoveAndResize => "MOVE_AND_RESIZE",
            Self::DontMoveDoResize => "DONT_MOVE_DO_RESIZE",
            Self::MoveDontResize => "MOVE_DONT_RESIZE",
            Self::DontMoveAndResize => "DONT_MOVE_AND_RESIZE",
        }
    }
    /// Java `getValue()` 的后端中立值；格式引擎在边界转换为具体锚点类型。
    #[must_use] pub const fn get_value(self) -> Self { self }
    /// 返回 POI 锚点编号。对应 Java：`getId()`。
    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::MoveAndResize => 0,
            Self::DontMoveDoResize => 1,
            Self::MoveDontResize => 2,
            Self::DontMoveAndResize => 3,
        }
    }

    /// Java `getId()` 兼容别名。
    #[must_use]
    pub const fn get_id(self) -> i32 {
        self.id()
    }

    /// 按 POI 编号查找锚点类型。对应 Java：`AnchorType#byId`。
    #[must_use]
    pub const fn by_id(value: i32) -> Self {
        match value {
            0 => Self::MoveAndResize,
            1 => Self::DontMoveDoResize,
            2 => Self::MoveDontResize,
            3 => Self::DontMoveAndResize,
            _ => panic!("invalid ClientAnchorData.AnchorType id"),
        }
    }
}

impl std::str::FromStr for AnchorType {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown ClientAnchorData.AnchorType value: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_four_variants() {
        assert_eq!(AnchorType::ALL.len(), 4);
        assert_eq!(AnchorType::ALL[0], AnchorType::MoveAndResize);
        assert_eq!(AnchorType::ALL[1], AnchorType::DontMoveDoResize);
        assert_eq!(AnchorType::ALL[2], AnchorType::MoveDontResize);
        assert_eq!(AnchorType::ALL[3], AnchorType::DontMoveAndResize);
    }

    #[test]
    fn java_name_returns_correct_strings() {
        assert_eq!(AnchorType::MoveAndResize.java_name(), "MOVE_AND_RESIZE");
        assert_eq!(AnchorType::DontMoveDoResize.java_name(), "DONT_MOVE_DO_RESIZE");
        assert_eq!(AnchorType::MoveDontResize.java_name(), "MOVE_DONT_RESIZE");
        assert_eq!(AnchorType::DontMoveAndResize.java_name(), "DONT_MOVE_AND_RESIZE");
    }

    #[test]
    fn default_is_move_and_resize() {
        assert_eq!(AnchorType::default(), AnchorType::MoveAndResize);
    }

    #[test]
    fn id_returns_correct_values() {
        assert_eq!(AnchorType::MoveAndResize.id(), 0);
        assert_eq!(AnchorType::DontMoveDoResize.id(), 1);
        assert_eq!(AnchorType::MoveDontResize.id(), 2);
        assert_eq!(AnchorType::DontMoveAndResize.id(), 3);
    }

    #[test]
    fn get_id_matches_id() {
        for variant in AnchorType::ALL {
            assert_eq!(variant.get_id(), variant.id());
        }
    }

    #[test]
    fn get_value_returns_self() {
        for variant in AnchorType::ALL {
            assert_eq!(variant.get_value(), variant);
        }
    }

    #[test]
    fn by_id_roundtrips() {
        for variant in AnchorType::ALL {
            assert_eq!(AnchorType::by_id(variant.id()), variant);
        }
    }

    #[test]
    fn by_id_0_is_move_and_resize() {
        assert_eq!(AnchorType::by_id(0), AnchorType::MoveAndResize);
    }

    #[test]
    fn by_id_3_is_dont_move_and_resize() {
        assert_eq!(AnchorType::by_id(3), AnchorType::DontMoveAndResize);
    }

    #[test]
    #[should_panic(expected = "invalid ClientAnchorData.AnchorType id")]
    fn by_id_invalid_panics() {
        AnchorType::by_id(99);
    }

    #[test]
    fn from_str_parses_valid_names() {
        assert_eq!("MOVE_AND_RESIZE".parse::<AnchorType>().unwrap(), AnchorType::MoveAndResize);
        assert_eq!("DONT_MOVE_DO_RESIZE".parse::<AnchorType>().unwrap(), AnchorType::DontMoveDoResize);
        assert_eq!("MOVE_DONT_RESIZE".parse::<AnchorType>().unwrap(), AnchorType::MoveDontResize);
        assert_eq!("DONT_MOVE_AND_RESIZE".parse::<AnchorType>().unwrap(), AnchorType::DontMoveAndResize);
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!("INVALID".parse::<AnchorType>().is_err());
    }

    #[test]
    fn from_str_error_message_contains_input() {
        let err = "BOGUS".parse::<AnchorType>().unwrap_err();
        assert!(err.contains("BOGUS"), "error should contain input: {err}");
    }

    #[test]
    fn roundtrip_from_str_java_name() {
        for variant in AnchorType::ALL {
            let name = variant.java_name();
            let parsed: AnchorType = name.parse().unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn clone_copy_eq() {
        let a = AnchorType::MoveDontResize;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_format_contains_variant_name() {
        let text = format!("{:?}", AnchorType::DontMoveDoResize);
        assert!(text.contains("DontMoveDoResize"));
    }
}
