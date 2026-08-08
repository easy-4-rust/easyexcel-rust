//! 对应 Java：`com.alibaba.excel.util.MemberUtils`。

/// Java 反射成员访问辅助类。
///
/// Rust 不允许运行时绕过可见性；该对象保留 Java 公共构造形态，并公开纯
/// modifier 判定供 `FieldUtils` 等兼容代码使用。
#[derive(Debug, Clone, Copy, Default)]
pub struct MemberUtils;

impl MemberUtils {
    /// Java `Modifier.PUBLIC`。
    pub const PUBLIC: i32 = 0x0001;
    /// Java `Modifier.PRIVATE`。
    pub const PRIVATE: i32 = 0x0002;
    /// Java `Modifier.PROTECTED`。
    pub const PROTECTED: i32 = 0x0004;
    const ACCESS_TEST: i32 = Self::PUBLIC | Self::PROTECTED | Self::PRIVATE;

    /// 创建工具对象，对应 Java 隐式公共无参构造器。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 判断 Java modifier 位是否表示 package access。
    #[must_use]
    pub const fn is_package_access(modifiers: i32) -> bool {
        modifiers & Self::ACCESS_TEST == 0
    }

    /// Rust 可见性在编译期确定，因此不存在可变 `AccessibleObject`；返回
    /// `false` 与 Java 对 null/已可访问对象的行为一致。
    #[must_use]
    pub const fn set_accessible_workaround() -> bool {
        false
    }
}
