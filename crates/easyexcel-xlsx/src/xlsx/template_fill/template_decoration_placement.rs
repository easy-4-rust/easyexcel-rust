/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。装饰值最终物理坐标。
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateDecorationPlacement {
    /// 零基行号。
    pub row: u32,
    /// 零基列号。
    pub column: u16,
    /// 需要由 package 层写入的装饰。
    pub decoration: TemplateDecoration,
}
