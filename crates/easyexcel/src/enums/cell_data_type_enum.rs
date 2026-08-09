//! 对应 Java：`com.alibaba.excel.enums.CellDataTypeEnum`.
//!
//! Java 定义了 8 个变体；Rust 额外补齐了 `Formula` 和 `Image`，与 `CellValue`
//! 中 `Formula(String) / Image(Vec<u8>)` 变体对齐。
//!
//! 原 Java `buildFromCellType(String)` 通过类型码 `"s" / "str" / "inlineStr" / "e" / "b" / "n"`
//! 路由到枚举；见 [`CellDataTypeEnum::build_from_cell_type`]。

/// Logical Excel cell type used as the read-converter dispatch key.
///
/// This is the Rust port of Java `CellDataTypeEnum`. Two additional variants
/// (`Formula`, `Image`) keep it aligned with the `CellValue` enum so writers
/// can carry Java-equivalent rich metadata without an extra wrapper class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// 对应 Java：com.alibaba.excel.enums.CellDataTypeEnum。
pub enum CellDataTypeEnum {
    /// Shared or inline string.       (Java `STRING`)
    String,
    /// Direct inline string.          (Java `DIRECT_STRING`)
    DirectString,
    /// Numeric value.                  (Java `NUMBER`)
    Number,
    /// Boolean value.                  (Java `BOOLEAN`)
    Boolean,
    /// Empty or physically absent cell. (Java `EMPTY`)
    #[default]
    Empty,
    /// Excel error value.              (Java `ERROR`)
    Error,
    /// Date or date-time value.        (Java `DATE`)
    Date,
    /// Rich text string.              (Java `RICH_TEXT_STRING`)
    RichTextString,
    /// Formula expression supplied as a write value. (Rust extension)
    Formula,
    /// Encoded image bytes.            (Rust extension)
    Image,
}

impl CellDataTypeEnum {
    /// Java `values()` 的声明顺序；Rust 扩展 `Formula/Image` 不混入 Java 结果。
    pub const JAVA_VALUES: [Self; 8] = [
        Self::String, Self::DirectString, Self::Number, Self::Boolean,
        Self::Empty, Self::Error, Self::Date, Self::RichTextString,
    ];
    /// Java `values()` 的兼容名称。
    pub const ALL: [Self; 8] = Self::JAVA_VALUES;
    /// Java 枚举常量名；Rust 扩展使用显式扩展名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self {
            Self::String => "STRING", Self::DirectString => "DIRECT_STRING", Self::Number => "NUMBER",
            Self::Boolean => "BOOLEAN", Self::Empty => "EMPTY", Self::Error => "ERROR",
            Self::Date => "DATE", Self::RichTextString => "RICH_TEXT_STRING",
            Self::Formula => "FORMULA", Self::Image => "IMAGE",
        }
    }
    /// 对应 Java：com.alibaba.excel.enums.CellDataTypeEnum。 Java `CellDataTypeEnum.buildFromCellType(String)`.
    ///
    /// Maps OOXML `c@t` codes onto the enum used by `CellTagHandler.startElement`.
    /// Unknown codes return [`None`] (Java would return `null` and later NPE —
    /// Rust callers treat that as a format error).
    #[must_use]
    pub fn build_from_cell_type(cell_type: Option<&str>) -> Option<Self> {
        match cell_type {
            None | Some("") => Some(Self::Empty),
            Some("s") => Some(Self::String),
            // Rust path also accepts date serials marked `d` (OOXML extension).
            Some("str" | "inlineStr" | "d") => Some(Self::DirectString),
            Some("e") => Some(Self::Error),
            Some("b") => Some(Self::Boolean),
            Some("n") => Some(Self::Number),
            Some(_) => None,
        }
    }
}

impl std::str::FromStr for CellDataTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::JAVA_VALUES.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown CellDataTypeEnum value: {value}"))
    }
}
