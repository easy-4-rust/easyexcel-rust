//! 对应 Java：`com.alibaba.excel.enums.CacheLocationEnum`.
//!
//! Used by Java `BasicParameter.filedCacheLocation`. Rust has collapsed this
//! concept into `easyexcel_reader::ReadCacheMode`, but the enum is kept for
//! API completeness when reading Java `ReadWorkbookHolder` payloads.

/// Cache location strategy.
///
/// Rust port of Java `CacheLocationEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 对应 Java：com.alibaba.excel.enums.CacheLocationEnum。
pub enum CacheLocationEnum {
    /// Stored in `ThreadLocal`; cleared when the read or write completes.
    ThreadLocal,
    /// Never cleared unless the application exits.
    Memory,
    /// Caching disabled.
    None,
}

impl CacheLocationEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::ThreadLocal, Self::Memory, Self::None];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::ThreadLocal => "THREAD_LOCAL", Self::Memory => "MEMORY", Self::None => "NONE" }
    }
}

impl std::str::FromStr for CacheLocationEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown CacheLocationEnum value: {value}"))
    }
}
