/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板图片移动语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateImageMovement {
    /// 随单元格移动并调整尺寸。
    MoveAndResize,
    /// 随单元格移动但不调整尺寸。
    MoveDontResize,
    /// 不随单元格移动或调整尺寸。
    DontMoveOrResize,
}

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。待写入模板 package 的图片。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateImage {
    /// 编码后的 PNG/JPEG/GIF/BMP 图片。
    pub bytes: Vec<u8>,
    /// 起始行坐标。
    pub first_row: super::AnchorCoordinate,
    /// 起始列坐标。
    pub first_column: super::AnchorCoordinate,
    /// 结束行坐标。
    pub last_row: super::AnchorCoordinate,
    /// 结束列坐标。
    pub last_column: super::AnchorCoordinate,
    /// 左侧像素边距。
    pub left: u32,
    /// 右侧像素边距。
    pub right: u32,
    /// 顶部像素边距。
    pub top: u32,
    /// 底部像素边距。
    pub bottom: u32,
    /// 图片移动语义。
    pub movement: TemplateImageMovement,
}

impl TemplateImage {
    /// 创建仅覆盖当前单元格的默认图片。
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            first_row: super::AnchorCoordinate::default(),
            first_column: super::AnchorCoordinate::default(),
            last_row: super::AnchorCoordinate::default(),
            last_column: super::AnchorCoordinate::default(),
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            movement: TemplateImageMovement::MoveAndResize,
        }
    }
}
