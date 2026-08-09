//! 写入引擎使用的可复制单元格样式。
//!
//! 该类型不是 Java 公共 `WriteCellStyle` 的替身，而是 Rust 注解派生与
//! XLS/XLSX 热路径共享的后端中立值对象。Java 运行期对象由
//! `write::metadata::style::WriteCellStyle` 承载，并在边界处显式转换。

use crate::core::{
    ExcelBorderStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern, ExcelFontStyle,
    ExcelHorizontalAlignment, ExcelVerticalAlignment,
};
use crate::write::metadata::style::write_font::merge_excel_font_style;

/// Rust 写入引擎的轻量单元格样式。
///
/// 对应 Java：无直接对应对象；这是对注解常量和格式引擎公共字段的 Rust
/// 惯用替代。动态字体名称等运行期状态保留在 `WriteCellStyle` / `WriteFont`。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExcelCellStyle {
    /// 工作表受保护时是否隐藏公式。
    pub hidden: Option<bool>,
    /// 工作表受保护时是否锁定单元格。
    pub locked: Option<bool>,
    /// 是否启用 quote-prefix。
    pub quote_prefix: Option<bool>,
    /// 水平对齐方式。
    pub horizontal_alignment: Option<ExcelHorizontalAlignment>,
    /// 是否自动换行。
    pub wrapped: Option<bool>,
    /// 垂直对齐方式。
    pub vertical_alignment: Option<ExcelVerticalAlignment>,
    /// 文本旋转角度。
    pub rotation: Option<i16>,
    /// 文本缩进级别。
    pub indent: Option<u8>,
    /// 左边框样式。
    pub border_left: Option<ExcelBorderStyle>,
    /// 右边框样式。
    pub border_right: Option<ExcelBorderStyle>,
    /// 上边框样式。
    pub border_top: Option<ExcelBorderStyle>,
    /// 下边框样式。
    pub border_bottom: Option<ExcelBorderStyle>,
    /// 左边框颜色。
    pub left_border_color: Option<ExcelColor>,
    /// 右边框颜色。
    pub right_border_color: Option<ExcelColor>,
    /// 上边框颜色。
    pub top_border_color: Option<ExcelColor>,
    /// 下边框颜色。
    pub bottom_border_color: Option<ExcelColor>,
    /// 填充图案。
    pub fill_pattern: Option<ExcelFillPattern>,
    /// 填充背景色。
    pub fill_background_color: Option<ExcelColor>,
    /// 填充前景色。
    pub fill_foreground_color: Option<ExcelColor>,
    /// 是否缩小字体以适应单元格。
    pub shrink_to_fit: Option<bool>,
    /// 内建或静态自定义数字格式。
    pub data_format: Option<ExcelDataFormat>,
    /// 注解期字体样式；字体名只接受派生宏产生的静态字符串。
    pub font: Option<ExcelFontStyle>,
}

impl ExcelCellStyle {
    /// 创建所有字段均未指定的引擎样式。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hidden: None,
            locked: None,
            quote_prefix: None,
            horizontal_alignment: None,
            wrapped: None,
            vertical_alignment: None,
            rotation: None,
            indent: None,
            border_left: None,
            border_right: None,
            border_top: None,
            border_bottom: None,
            left_border_color: None,
            right_border_color: None,
            top_border_color: None,
            bottom_border_color: None,
            fill_pattern: None,
            fill_background_color: None,
            fill_foreground_color: None,
            shrink_to_fit: None,
            data_format: None,
            font: None,
        }
    }

    /// 附加注解期字体样式。
    #[must_use]
    pub const fn with_font(mut self, font: ExcelFontStyle) -> Self {
        self.font = Some(font);
        self
    }
}

/// 把源样式中已设置的字段覆盖到目标引擎样式。
///
/// 对应 Java：`WriteCellStyle.merge` 在引擎轻量模型上的等价运算。
#[must_use]
pub fn merge_excel_cell_style(
    source: &ExcelCellStyle,
    mut target: ExcelCellStyle,
) -> ExcelCellStyle {
    macro_rules! overwrite {
        ($field:ident) => {
            if source.$field.is_some() {
                target.$field = source.$field;
            }
        };
    }
    overwrite!(hidden);
    overwrite!(locked);
    overwrite!(quote_prefix);
    overwrite!(horizontal_alignment);
    overwrite!(wrapped);
    overwrite!(vertical_alignment);
    overwrite!(rotation);
    overwrite!(indent);
    overwrite!(border_left);
    overwrite!(border_right);
    overwrite!(border_top);
    overwrite!(border_bottom);
    overwrite!(left_border_color);
    overwrite!(right_border_color);
    overwrite!(top_border_color);
    overwrite!(bottom_border_color);
    overwrite!(fill_pattern);
    overwrite!(fill_background_color);
    overwrite!(fill_foreground_color);
    overwrite!(shrink_to_fit);
    overwrite!(data_format);
    if let Some(source_font) = source.font {
        target.font = Some(match target.font {
            Some(existing) => merge_excel_font_style(&source_font, existing),
            None => source_font,
        });
    }
    target
}
