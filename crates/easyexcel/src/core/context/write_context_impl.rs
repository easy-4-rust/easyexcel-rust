//! 写上下文实现。
//!
//! 对应 Java：`com.alibaba.excel.context.WriteContextImpl`
//! 既有实现：[`crate::write::write_context`]（不删减，此处 re-export）。

pub use crate::write::write_context::{
    WriteContext, WriteContextHolder, WriteContextHolderState, WriteContextImpl,
    WriteContextLifecycle, finish_write_context,
};
