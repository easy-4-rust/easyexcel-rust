/// BIFF8 模板中的 VBA 项目处理策略。
///
/// 对应 Java：POI `HSSFWorkbook` 对 `_VBA_PROJECT_CUR` CFB storage 的保留行为。
/// Rust 永不执行宏；`Replace` 的字节必须是包含该 storage 的完整 OLE/CFB 文件。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Biff8MacroPolicy {
    /// 原样保留模板中的 VBA storage、stream、CLSID 与 state bits。
    #[default]
    Preserve,
    /// 删除模板中的完整 VBA 项目 storage。
    Strip,
    /// 从另一个 OLE/CFB 文件复制完整 VBA 项目 storage。
    Replace(Vec<u8>),
}

