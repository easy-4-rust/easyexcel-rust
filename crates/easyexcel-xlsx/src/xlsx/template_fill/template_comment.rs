/// OOXML 模板填充使用的格式中立批注语义。
///
/// 这是 Rust 引擎扩展，不对应 Java 门面对象；门面 `CommentData` 只负责转换。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateComment {
    /// 批注正文。
    pub text: String,
    /// 可选作者。
    pub author: Option<String>,
    /// 对象移动语义：0=随单元格移动并缩放，1=移动但不缩放，2=不移动不缩放。
    pub movement: Option<u8>,
    /// 初始可见性；`None` 使用生成器默认值。
    pub visible: Option<bool>,
}
