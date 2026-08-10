/// BIFF8 模板保存后 VBA 项目的结构变化。
///
/// Rust 扩展：用于报告 [`super::Biff8MacroPolicy`] 的实际结果；宏内容始终按
/// opaque CFB 数据处理，绝不解析或执行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8MacroProjectAction {
    /// 输入和输出均没有 VBA project storage。
    Absent,
    /// 原样保留输入中的 VBA project storage。
    Preserved,
    /// 删除输入中的 VBA project storage。
    Stripped,
    /// 使用调用方提供的 OLE/CFB 文件替换 VBA project storage。
    Replaced,
}
