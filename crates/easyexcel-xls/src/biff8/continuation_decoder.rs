//! BIFF8 可续接逻辑记录的增量解码状态机。

use easyexcel_io::Result;

use super::continuation_chain::Biff8ContinuationChain;

/// 可跨 `CONTINUE` 记录保存的 BIFF8 逻辑记录类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8ContinuableRecordKind {
    /// 共享字符串表（SST）。
    SharedStringTable,
    /// 公式缓存结果后的 Unicode STRING 记录。
    UnicodeString,
}

/// 已完整解码的 BIFF8 可续接逻辑记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8DecodedContinuableRecord {
    /// 按 BIFF 索引顺序排列的共享字符串。
    SharedStrings(Vec<String>),
    /// 一个完整的 Unicode 字符串。
    UnicodeString(String),
}

/// 增量解码结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8ContinuationStatus {
    /// 当前没有待解码记录。
    Idle,
    /// 当前数据尚不足，需要后续 `CONTINUE` 记录。
    Pending,
    /// 逻辑记录已经完整解码。
    Complete(Biff8DecodedContinuableRecord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingContinuation {
    kind: Biff8ContinuableRecordKind,
    chain: Biff8ContinuationChain,
}

/// 管理 BIFF8 逻辑记录和后续 `CONTINUE` 物理记录的生命周期。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Biff8ContinuableRecordDecoder {
    pending: Option<PendingContinuation>,
}

impl Biff8ContinuableRecordDecoder {
    /// 使用逻辑记录的首个物理记录体开始解码。
    pub fn begin(&mut self, kind: Biff8ContinuableRecordKind, first_segment: &[u8]) {
        self.pending = Some(PendingContinuation {
            kind,
            chain: Biff8ContinuationChain::new(first_segment),
        });
    }

    /// 向当前逻辑记录追加一个 `CONTINUE` 记录体。
    ///
    /// 没有待处理记录时返回 `false`，调用方可将该 `CONTINUE` 视为游离记录。
    pub fn push(&mut self, continuation: &[u8]) -> bool {
        let Some(pending) = self.pending.as_mut() else {
            return false;
        };
        pending.chain.push(continuation);
        true
    }

    /// 返回当前是否存在尚未完成的逻辑记录。
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// 尝试解码当前逻辑记录。
    ///
    /// `require_complete=false` 时，当前解码错误被解释为还需要后续
    /// `CONTINUE`；`require_complete=true` 时，同一错误作为损坏记录返回。
    /// 成功解码后自动清空内部状态。
    ///
    /// # Errors
    ///
    /// 要求完整记录但 SST 或 Unicode 字符串仍损坏/截断时返回错误。
    pub fn try_finish(&mut self, require_complete: bool) -> Result<Biff8ContinuationStatus> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(Biff8ContinuationStatus::Idle);
        };
        let decoded = match pending.kind {
            Biff8ContinuableRecordKind::SharedStringTable => pending
                .chain
                .decode_sst()
                .map(Biff8DecodedContinuableRecord::SharedStrings),
            Biff8ContinuableRecordKind::UnicodeString => pending
                .chain
                .decode_unicode_string()
                .map(Biff8DecodedContinuableRecord::UnicodeString),
        };
        match decoded {
            Ok(record) => {
                self.pending = None;
                Ok(Biff8ContinuationStatus::Complete(record))
            }
            Err(_) if !require_complete => Ok(Biff8ContinuationStatus::Pending),
            Err(error) => Err(error),
        }
    }
}
