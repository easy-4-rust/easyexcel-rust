//! 事件驱动读取的内部状态。

use crate::checksum::RowChecksum;

/// 事件驱动读取的内部状态，跟踪已读取行数和校验和。
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
#[derive(Default)]
pub(super) struct EventState {
    /// 已读取行数
    pub(super) rows: u64,
    /// 行数据校验和计算器
    pub(super) checksum: RowChecksum,
}
