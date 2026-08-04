//! Java `com.alibaba.excel.write` 包路径镜像（不删既有顶层实现）。
//!
//! 这些模块实际位于 `write/` 下的兄弟目录，通过 re-export 指向真实模块。

pub use crate::write::builder;
#[path = "excel_builder_impl.rs"]
pub mod excel_builder_impl;
pub use crate::write::handler;
pub use crate::write::metadata;
