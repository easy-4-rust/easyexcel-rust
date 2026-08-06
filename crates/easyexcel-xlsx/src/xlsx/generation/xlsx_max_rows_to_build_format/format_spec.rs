/// 对应 Java：无直接对应对象；Rust 架构扩展。 与 `EasyExcel` annotation/handler 类型解耦的 XLSX 单元格格式描述。
///
/// 门面负责合并 Java 风格元数据，本结构只表达最终后端意图。
#[derive(Debug, Clone, Default)]
pub struct FormatSpec {
    /// 是否隐藏公式。
    pub hidden: Option<bool>,
    /// 是否锁定单元格。
    pub locked: Option<bool>,
    /// 是否启用引用前缀。
    pub quote_prefix: Option<bool>,
    /// 水平对齐方式。
    pub horizontal_alignment: Option<FormatAlign>,
    /// 垂直对齐方式。
    pub vertical_alignment: Option<FormatAlign>,
    /// 是否自动换行。
    pub wrap_text: Option<bool>,
    /// 文本旋转角度。
    pub rotation: Option<i16>,
    /// 文本缩进级别。
    pub indent: Option<u8>,
    /// 左边框样式。
    pub border_left: Option<FormatBorder>,
    /// 右边框样式。
    pub border_right: Option<FormatBorder>,
    /// 上边框样式。
    pub border_top: Option<FormatBorder>,
    /// 下边框样式。
    pub border_bottom: Option<FormatBorder>,
    /// 左边框颜色。
    pub left_border_color: Option<Color>,
    /// 右边框颜色。
    pub right_border_color: Option<Color>,
    /// 上边框颜色。
    pub top_border_color: Option<Color>,
    /// 下边框颜色。
    pub bottom_border_color: Option<Color>,
    /// 填充图案。
    pub fill_pattern: Option<FormatPattern>,
    /// 填充背景色。
    pub fill_background_color: Option<Color>,
    /// 填充前景色。
    pub fill_foreground_color: Option<Color>,
    /// 是否缩小字体以适应单元格。
    pub shrink_to_fit: Option<bool>,
    /// 数字格式。
    pub number_format: Option<NumberFormatSpec>,
    /// 字体格式。
    pub font: FontFormatSpec,
}

