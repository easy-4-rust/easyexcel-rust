//! 对应 Java：`com.alibaba.excel.util.StyleUtil`。
//!
//! 格式注册、OOXML/BIFF 样式序列化由基础引擎负责；这里仅组合 Java
//! `EasyExcel` 风格的写入元数据，并把通用坐标算法委托给 `easyexcel-utils`。

use crate::metadata::data::DataFormatData;
use crate::metadata::data::hyperlink_data::HyperlinkType;
use crate::metadata::data::rich_text_string_data::RichTextStringData;
use crate::write::metadata::style::write_cell_style::{WriteCellStyle, merge_write_cell_style};
use crate::write::metadata::style::write_font::{WriteFont, merge_write_font};

/// 合并来源样式与新写入样式。
///
/// 对应 Java：`StyleUtil#buildCellStyle`。`write_cell_style` 的非空字段覆盖
/// `origin_cell_style`，具体 XLS/XLSX 格式对象由相应引擎在写入时创建。
#[must_use]
pub fn build_cell_style(
    origin_cell_style: Option<&WriteCellStyle>,
    write_cell_style: Option<&WriteCellStyle>,
) -> WriteCellStyle {
    let origin = origin_cell_style.cloned().unwrap_or_default();
    write_cell_style.map_or(origin.clone(), |style| merge_write_cell_style(style, origin))
}

/// 解析数据格式元数据。
///
/// 对应 Java：`StyleUtil#buildDataFormat`。有效的非负索引优先，其次保留
/// 非空自定义格式；两者都没有时回退到 General（索引 0）。实际自定义索引
/// 分配由 XLS/XLSX/CSV 引擎各自的格式表完成。
#[must_use]
pub fn build_data_format(data_format_data: Option<&DataFormatData>) -> DataFormatData {
    DataFormatData::resolve(data_format_data)
}

/// 合并来源字体与新写入字体。
///
/// 对应 Java：`StyleUtil#buildFont`。两个参数都为空时返回 `None`；否则
/// `write_font` 的已设置字段覆盖来源字体。
#[must_use]
pub fn build_font(
    origin_font: Option<&WriteFont>,
    write_font: Option<&WriteFont>,
) -> Option<WriteFont> {
    match (origin_font, write_font) {
        (None, None) => None,
        (Some(origin), None) => Some(origin.clone()),
        (None, Some(write)) => Some(write.clone()),
        (Some(origin), Some(write)) => Some(merge_write_font(write, origin.clone())),
    }
}

/// 构建门面富文本元数据。
///
/// 对应 Java：`StyleUtil#buildRichTextString`；具体 HSSF/XSSF 富文本对象在
/// 格式引擎写入阶段生成。
#[must_use]
pub fn build_rich_text_string(
    rich_text_string_data: Option<&RichTextStringData>,
) -> Option<RichTextStringData> {
    rich_text_string_data.cloned()
}

/// 对应 Java：com.alibaba.excel.util.StyleUtil。 返回有效的超链接类型，缺失时回退为 `None`。
#[must_use]
pub fn get_hyperlink_type(hyperlink_type: Option<HyperlinkType>) -> HyperlinkType {
    hyperlink_type.unwrap_or_default()
}

/// 对应 Java：com.alibaba.excel.util.StyleUtil。 将可选点坐标转换为 EMU。
#[must_use]
pub fn get_coordinate(coordinate: Option<i32>) -> i32 {
    easyexcel_utils::coordinate_utils::points_to_emu(coordinate)
}

/// 对应 Java：com.alibaba.excel.util.StyleUtil。 按绝对坐标、相对坐标、当前坐标的优先级解析单元格坐标。
#[must_use]
pub fn get_cell_coordinate(
    current_coordinate: i32,
    absolute_coordinate: Option<i32>,
    relative_coordinate: Option<i32>,
) -> i32 {
    easyexcel_utils::coordinate_utils::resolve_cell_coordinate(
        current_coordinate,
        absolute_coordinate,
        relative_coordinate,
    )
}

#[cfg(test)]
mod tests {
    use crate::metadata::excel_horizontal_alignment::ExcelHorizontalAlignment;

    use super::{
        build_cell_style, build_data_format, build_font, build_rich_text_string,
        get_cell_coordinate, get_coordinate, get_hyperlink_type,
    };
    use crate::metadata::data::DataFormatData;
    use crate::metadata::data::hyperlink_data::HyperlinkType;
    use crate::metadata::data::rich_text_string_data::RichTextStringData;
    use crate::write::metadata::style::write_cell_style::WriteCellStyle;
    use crate::write::metadata::style::write_font::WriteFont;

    #[test]
    fn builds_real_metadata_instead_of_placeholder_values() {
        let origin = WriteCellStyle {
            wrapped: Some(true),
            ..WriteCellStyle::new()
        };
        let update = WriteCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..WriteCellStyle::new()
        };
        let style = build_cell_style(Some(&origin), Some(&update));
        assert_eq!(style.wrapped, Some(true));
        assert_eq!(
            style.horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );

        assert_eq!(build_data_format(None).index, Some(0));
        let custom = DataFormatData {
            index: None,
            format: Some("0.000".to_owned()),
        };
        assert_eq!(build_data_format(Some(&custom)), custom);

        let origin_font = WriteFont::new().italic(true);
        let update_font = WriteFont::new().bold(true);
        let font = build_font(Some(&origin_font), Some(&update_font)).expect("font");
        assert_eq!(font.get_italic(), Some(true));
        assert_eq!(font.get_bold(), Some(true));

        let rich = RichTextStringData::new("rich");
        assert_eq!(build_rich_text_string(Some(&rich)), Some(rich));
        assert_eq!(get_hyperlink_type(None), HyperlinkType::None);
        assert_eq!(
            get_hyperlink_type(Some(HyperlinkType::Url)),
            HyperlinkType::Url
        );
    }

    #[test]
    fn delegates_coordinate_algorithms_to_easyexcel_utils() {
        assert_eq!(get_coordinate(None), 0);
        assert_eq!(get_coordinate(Some(1)), 12_700);
        assert_eq!(get_cell_coordinate(10, Some(20), Some(3)), 20);
        assert_eq!(get_cell_coordinate(10, None, Some(3)), 13);
    }
}
