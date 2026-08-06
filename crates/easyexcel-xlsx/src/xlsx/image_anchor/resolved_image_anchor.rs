/// 对应 Java：无直接对应对象；Rust 架构扩展。 已校验、可直接交给 XLSX 写入后端的图片锚点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedImageAnchor {
    /// 首行坐标。
    pub first_row: u32,
    /// 首列坐标。
    pub first_column: u16,
    /// 扣除左右边距后的图片宽度。
    pub width: u32,
    /// 扣除上下边距后的图片高度。
    pub height: u32,
    /// 左侧像素偏移。
    pub left: u32,
    /// 顶部像素偏移。
    pub top: u32,
}

