/// 一次 BIFF8 宏策略应用结果。
///
/// Rust 扩展：让上层 facade 在不解析或执行 VBA 的前提下，公开项目是否存在、
/// 实际采取的结构操作以及是否需要重新签名。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Biff8MacroPolicyReport {
    project_present_before: bool,
    project_present_after: bool,
    action: Biff8MacroProjectAction,
    signature_status: Biff8MacroSignatureStatus,
}

impl Biff8MacroPolicyReport {
    pub(super) const fn new(
        project_present_before: bool,
        project_present_after: bool,
        action: Biff8MacroProjectAction,
        signature_status: Biff8MacroSignatureStatus,
    ) -> Self {
        Self {
            project_present_before,
            project_present_after,
            action,
            signature_status,
        }
    }

    /// 返回输入模板是否包含 `/_VBA_PROJECT_CUR` storage。
    #[must_use]
    pub const fn project_present_before(self) -> bool {
        self.project_present_before
    }

    /// 返回输出模板是否包含 `/_VBA_PROJECT_CUR` storage。
    #[must_use]
    pub const fn project_present_after(self) -> bool {
        self.project_present_after
    }

    /// 返回实际执行的 VBA storage 操作。
    #[must_use]
    pub const fn action(self) -> Biff8MacroProjectAction {
        self.action
    }

    /// 返回签名的可观察状态。
    #[must_use]
    pub const fn signature_status(self) -> Biff8MacroSignatureStatus {
        self.signature_status
    }

    /// 返回调用方是否必须重新签名才能声称 VBA 签名有效。
    #[must_use]
    pub const fn requires_resigning(self) -> bool {
        matches!(self.signature_status, Biff8MacroSignatureStatus::RequiresResign)
    }
}
