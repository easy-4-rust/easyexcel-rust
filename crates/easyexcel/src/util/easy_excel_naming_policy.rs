//! 对应 Java：`com.alibaba.excel.util.BeanMapUtils.EasyExcelNamingPolicy`。

/// CGLIB 命名策略兼容对象。
///
/// Rust 不生成 CGLIB 类，但保留确定的名称标签和全局实例语义。
/// 对应 Java：`BeanMapUtils.EasyExcelNamingPolicy`。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct EasyExcelNamingPolicy;

impl EasyExcelNamingPolicy {
    /// Java `INSTANCE` 单例。
    pub const INSTANCE: Self = Self;

    /// 创建命名策略，对应公共无参构造器。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 返回 Java 覆盖的 CGLIB 名称标签。
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        "ByEasyExcelCGLIB"
    }
}
