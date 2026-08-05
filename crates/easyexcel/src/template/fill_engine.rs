//! Java EasyExcel 模板元数据到中立 XLSX 填充引擎的适配层。
//!
//! 集合游标、标量替换、XML 渲染、行追加和引用平移均由
//! `easyexcel-xlsx` 实现；本模块只保留门面值与错误契约转换。

use crate::core::{CellValue, ExcelError, Result};
use crate::template::sheet_fill_state::PendingCollectionFill;
use crate::template::template_entry::TemplateEntry;
use crate::TemplateData;

use easyexcel_xlsx::{
    TemplateCollectionFill, TemplateFillData, TemplateFillDirection,
};
use easyexcel_xlsx::xlsx::template_xml::TemplateCellValue;

// 保留既有内部路径，供模板生命周期与迁移中的特征测试引用；真实算法位于引擎 crate。
#[cfg(test)]
pub(crate) use easyexcel_xlsx::xlsx::template_xml::{
    all_cells, attribute_value, cell_references, column_name, contains_unescaped, element_value,
    escape_xml, last_worksheet_row, merge_collection_cells, parse_cell_reference,
    remove_attribute, replace_attribute, replace_tag_attribute, row_index, row_tag_with_reference,
    shared_string_values, shift_a1_reference, shift_cell_reference, shift_formula_elements,
    shift_formula_references, shift_reference_list, shift_row, shift_rows, shift_tag_references,
    shift_worksheet_metadata, shift_worksheet_rows_after, text_node_values,
    update_worksheet_dimension, upsert_collection_row, validate_collection_target,
    worksheet_max_row,
};

/// 执行指定工作表的集合模板填充。
pub(crate) fn replace_collection_fills_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    fills: &[PendingCollectionFill],
) -> Result<()> {
    let fills = fills.iter().map(template_collection_fill).collect::<Vec<_>>();
    easyexcel_xlsx::replace_collection_fills_in_sheet(entries, worksheet, &fills)
        .map_err(ExcelError::from)
}

/// 执行指定工作表的标量模板填充。
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
        rows: fill
            .wrapper
            .rows()
            .iter()
            .map(template_fill_data)
            .collect(),
        direction: match fill.config.get_direction() {
            crate::FillDirection::Vertical => TemplateFillDirection::Vertical,
            crate::FillDirection::Horizontal => TemplateFillDirection::Horizontal,
        },
        force_new_row: fill.config.get_force_new_row(),
        auto_style: fill.config.get_auto_style(),
        order: fill.order,
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
        CellValue::String(text) | CellValue::Hyperlink { text, .. } => {
            TemplateCellValue::Text(text.clone())
        }
        CellValue::RichText(value) => {
            TemplateCellValue::Text(value.text_string().to_owned())
        }
        CellValue::Bool(value) => TemplateCellValue::Bool(*value),
        CellValue::Int(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Float(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Decimal(value) => TemplateCellValue::Number(value.to_string()),
        CellValue::Date(value) => {
            TemplateCellValue::Date(value.format("%Y-%m-%d").to_string())
        }
        CellValue::DateTime(value) => {
            TemplateCellValue::Date(value.format("%Y-%m-%dT%H:%M:%S").to_string())
        }
        CellValue::Error(value) => TemplateCellValue::Error(value.clone()),
        CellValue::Formula(value) => TemplateCellValue::Formula(value.clone()),
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => {
            return template_cell_value(value);
        }
    }
}
