//! Holder 接口镜像。
//!
//! 对应 Java：`com.alibaba.excel.metadata.Holder`
//! Java 枚举 `HolderEnum` 在 `enums/holder_enum.rs` 中实现。

use crate::enums::holder_enum::HolderEnum;

/// Java `Holder` 接口的 Rust trait。
///
/// # Java 对应
/// - 接口：`com.alibaba.excel.metadata.Holder`
/// - 方法：`HolderEnum holderType()` → [`Self::holder_type`]
pub trait Holder {
    /// 返回 holder 类型。对应 Java `holderType()`。
    fn holder_type(&self) -> HolderEnum;
}
