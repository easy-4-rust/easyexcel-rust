//! 单元格样式类型。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/style/WriteCellStyle.java

use crate::write::horizontal_alignment::HorizontalAlignment;
use crate::write::vertical_alignment::VerticalAlignment;

/// 后端中立的写入样式（用于表头或内容行）。
///
/// 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle`。
/// 通过策略合并（`AbstractCellStyleStrategy`）应用到每个单元格。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CellStyle {
    /// 粗体字体。
    pub bold: bool,
    /// 斜体字体。
    pub italic: bool,
    /// RGB 字体颜色，例如 `0xFF0000`。
    pub font_color: Option<u32>,
    /// 实心 RGB 背景颜色。
    pub background_color: Option<u32>,
    /// 水平对齐。
    pub horizontal_alignment: Option<HorizontalAlignment>,
    /// 垂直对齐。
    pub vertical_alignment: Option<VerticalAlignment>,
    /// 是否自动换行。
    pub wrap_text: bool,
    /// Excel 数字格式字符串。
    pub number_format: Option<String>,
}

impl CellStyle {
    /// 创建空样式。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            font_color: None,
            background_color: None,
            horizontal_alignment: None,
            vertical_alignment: None,
            wrap_text: false,
            number_format: None,
        }
    }

    /// 设置粗体字体。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn bold(mut self, enabled: bool) -> Self {
        self.bold = enabled;
        self
    }

    /// 设置斜体字体。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn italic(mut self, enabled: bool) -> Self {
        self.italic = enabled;
        self
    }

    /// 设置 RGB 字体颜色。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn font_color(mut self, color: u32) -> Self {
        self.font_color = Some(color);
        self
    }

    /// 设置实心 RGB 背景颜色。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// 设置水平对齐。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = Some(alignment);
        self
    }

    /// 设置垂直对齐。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = Some(alignment);
        self
    }

    /// 启用或禁用文本换行。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn wrap_text(mut self, enabled: bool) -> Self {
        self.wrap_text = enabled;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。 设置 Excel 数字格式字符串。
    #[must_use]
    pub fn number_format(mut self, format: impl Into<String>) -> Self {
        self.number_format = Some(format.into());
        self
    }
}
