/// 模板填充完成后解析出的批注物理坐标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCommentPlacement {
    /// 零基行号。
    pub row: u32,
    /// 零基列号。
    pub column: u16,
    /// 需要在该坐标创建的批注。
    pub comment: TemplateComment,
}
