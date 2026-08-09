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
