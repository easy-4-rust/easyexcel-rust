//! Java `EasyExcel` 模板元数据到中立 XLSX 填充引擎的适配层。
//!
//! 集合游标、标量替换、XML 渲染、行追加和引用平移均由
//! `easyexcel-xlsx` 实现；本模块只保留门面值与错误契约转换。

use crate::TemplateData;
use crate::core::{CellValue, ExcelError, Result};
use crate::template::sheet_fill_state::PendingCollectionFill;
use crate::template::template_entry::TemplateEntry;

use easyexcel_xlsx::xlsx::template_xml::TemplateCellValue;
use easyexcel_xlsx::{TemplateCollectionFill, TemplateFillData, TemplateFillDirection};

// 保留既有内部路径，供模板生命周期与迁移中的特征测试引用；真实算法位于引擎 crate。
#[cfg(test)]
pub(crate) use easyexcel_xlsx::xlsx::template_xml::{
    all_cells, attribute_value, contains_unescaped, escape_xml, parse_cell_reference,
    replace_tag_attribute, shift_a1_reference, shift_formula_elements, shift_formula_references,
    shift_reference_list, shift_tag_references, shift_worksheet_metadata,
    update_worksheet_dimension,
};

/// 执行指定工作表的集合模板填充。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn replace_collection_fills_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    fills: &[PendingCollectionFill],
) -> Result<()> {
    let fills = fills
        .iter()
        .map(template_collection_fill)
        .collect::<Vec<_>>();
    easyexcel_xlsx::replace_collection_fills_in_sheet(entries, worksheet, &fills)
        .map_err(ExcelError::from)
}

/// 执行指定工作表的标量模板填充。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn replace_scalar_cells_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    data: &TemplateData,
) -> Result<()> {
    let data = template_fill_data(data);
    easyexcel_xlsx::replace_scalar_cells_in_sheet(entries, worksheet, &data)
        .map_err(ExcelError::from)
}

/// 向指定模板工作表追加普通行。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn append_rows_to_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    rows: &[Vec<CellValue>],
) -> Result<()> {
    let rows = rows
        .iter()
        .map(|row| row.iter().map(template_cell_value).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    easyexcel_xlsx::append_rows_to_sheet(entries, worksheet, &rows).map_err(ExcelError::from)
}

fn template_collection_fill(fill: &PendingCollectionFill) -> TemplateCollectionFill {
    TemplateCollectionFill {
        name: fill.wrapper.name().map(str::to_owned),
        rows: fill.wrapper.rows().iter().map(template_fill_data).collect(),
        direction: match fill.config.get_direction() {
            crate::FillDirection::Vertical => TemplateFillDirection::Vertical,
            crate::FillDirection::Horizontal => TemplateFillDirection::Horizontal,
        },
        force_new_row: fill.config.get_force_new_row(),
        auto_style: fill.config.get_auto_style(),
        order: fill.order,
        column_styles: fill.column_styles.clone(),
    }
}

fn template_fill_data(data: &TemplateData) -> TemplateFillData {
    TemplateFillData {
        values: data
            .values()
            .iter()
            .map(|(key, value)| (key.clone(), template_cell_value(value)))
            .collect(),
    }
}

fn template_cell_value(value: &CellValue) -> TemplateCellValue {
    match value {
        CellValue::Empty | CellValue::Image(_) => TemplateCellValue::Empty,
        CellValue::String(text)
        | CellValue::Hyperlink { text, .. }
        | CellValue::HyperlinkWithMetadata { text, .. } => TemplateCellValue::Text(text.clone()),
        CellValue::RichText(value) => TemplateCellValue::Text(value.text_string().to_owned()),
        CellValue::Bool(value) => TemplateCellValue::Bool(*value),
        CellValue::Int(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Float(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Decimal(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Date(value) => TemplateCellValue::Date(value.format("%Y-%m-%d").to_string()),
        CellValue::DateTime(value) => {
            TemplateCellValue::Date(value.format("%Y-%m-%dT%H:%M:%S").to_string())
        }
        CellValue::Error(value) => TemplateCellValue::Error(value.clone()),
        CellValue::Formula(value) => TemplateCellValue::Formula(value.clone()),
        CellValue::Comment { value, .. }
        | CellValue::CommentWithMetadata { value, .. }
        | CellValue::Images { value, .. } => {
            template_cell_value(value)
        }
    }
}
