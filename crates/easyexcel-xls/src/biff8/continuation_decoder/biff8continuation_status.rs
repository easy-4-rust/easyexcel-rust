/// 对应 Java：无直接对应对象；Rust 架构扩展。 增量解码结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8ContinuationStatus {
    /// 当前没有待解码记录。
    Idle,
    /// 当前数据尚不足，需要后续 `CONTINUE` 记录。
    Pending,
    /// 逻辑记录已经完整解码。
    Complete(Biff8DecodedContinuableRecord),
}

