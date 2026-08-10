/// VBA 数字签名在一次 BIFF8 模板保存后的可观察状态。
///
/// Rust 扩展。根据 MS-OSHARED，XLS 的 VBA 数字签名位于 OLE Document
/// Summary Information 属性集，而不是 `/_VBA_PROJECT_CUR` storage。当前实现
/// 不解析或验证 PKCS#7，因此只报告能够由本次结构操作确定的边界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8MacroSignatureStatus {
    /// 没有 VBA 项目，签名状态不适用。
    NotApplicable,
    /// VBA storage 与其余 OLE streams 均按 opaque bytes 保留，但未验证签名密码学有效性。
    PreservedOpaque,
    /// VBA 项目被删除或替换；若输入曾签名，调用方必须重新签名。
    RequiresResign,
}
