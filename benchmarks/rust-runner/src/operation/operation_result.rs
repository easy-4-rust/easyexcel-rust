//! 被测操作的可观察输出。

/// 被测操作的可观察输出，用于基准测试正确性校验和性能统计。
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct OperationResult {
    /// 观测到的行数
    pub(crate) observed_rows: u64,
    /// 行数据校验和
    pub(crate) checksum: String,
    /// 输出文件大小（字节）
    pub(crate) file_size_bytes: u64,
}
