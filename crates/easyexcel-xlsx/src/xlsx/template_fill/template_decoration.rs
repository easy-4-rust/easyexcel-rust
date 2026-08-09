/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。需要 package 层落盘的单元格装饰。
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateDecoration {
    /// 传统批注及 VML 元数据。
    Comment(TemplateComment),
    /// 超链接及覆盖范围。
    Hyperlink(TemplateHyperlink),
    /// 图片及锚点元数据。
    Image(TemplateImage),
}
