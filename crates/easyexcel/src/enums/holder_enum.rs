//! 对应 Java：`com.alibaba.excel.enums.HolderEnum`.

/// The types of holder.
///
/// Rust port of Java `HolderEnum`. Used to tag workbook / sheet / table / row
/// containers, although Rust collapses most of these into `ReadOptions` /
/// `WriteOptions` plus private state inside the writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.HolderEnum。
pub enum HolderEnum {
    /// Workbook-scoped holder.
    Workbook,
    /// Sheet-scoped holder.
    Sheet,
    /// Table-scoped holder.
    Table,
    /// Row-scoped holder.
    Row,
}

impl HolderEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 4] = [Self::Workbook, Self::Sheet, Self::Table, Self::Row];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::Workbook => "WORKBOOK", Self::Sheet => "SHEET", Self::Table => "TABLE", Self::Row => "ROW" }
    }
}

impl std::str::FromStr for HolderEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown HolderEnum value: {value}"))
    }
}
