//! Java `EasyExcel` 模板元数据到中立 XLSX 填充引擎的适配层。
//!
//! 集合游标、标量替换、XML 渲染、行追加和引用平移均由
//! `easyexcel-xlsx` 实现；本模块只保留门面值与错误契约转换。

use std::collections::BTreeMap;

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
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::replace_collection_fills_in_sheet(entries, worksheet, &fills)
        .map_err(ExcelError::from)
}

pub(crate) fn replace_collection_fills_in_sheet_with_comments(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    fills: &[PendingCollectionFill],
) -> Result<Vec<easyexcel_xlsx::TemplateCommentPlacement>> {
    let fills = fills
        .iter()
        .map(template_collection_fill)
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::replace_collection_fills_in_sheet_with_comments(
        entries,
        worksheet,
        &fills,
    )
    .map_err(ExcelError::from)
}

pub(crate) fn replace_collection_fills_in_sheet_with_decorations(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    fills: &[PendingCollectionFill],
) -> Result<Vec<easyexcel_xlsx::TemplateDecorationPlacement>> {
    let fills = fills
        .iter()
        .map(template_collection_fill)
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::replace_collection_fills_in_sheet_with_decorations(
        entries,
        worksheet,
        &fills,
    )
    .map_err(ExcelError::from)
}

/// 执行指定工作表的标量模板填充。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn replace_scalar_cells_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    data: &TemplateData,
) -> Result<()> {
    let data = template_fill_data(data)?;
    easyexcel_xlsx::replace_scalar_cells_in_sheet(entries, worksheet, &data)
        .map_err(ExcelError::from)
}

pub(crate) fn replace_scalar_cells_in_sheet_with_comments(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    data: &TemplateData,
) -> Result<Vec<easyexcel_xlsx::TemplateCommentPlacement>> {
    let data = template_fill_data(data)?;
    easyexcel_xlsx::replace_scalar_cells_in_sheet_with_comments(entries, worksheet, &data)
        .map_err(ExcelError::from)
}

pub(crate) fn replace_scalar_cells_in_sheet_with_decorations(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    data: &TemplateData,
) -> Result<Vec<easyexcel_xlsx::TemplateDecorationPlacement>> {
    let data = template_fill_data(data)?;
    easyexcel_xlsx::replace_scalar_cells_in_sheet_with_decorations(entries, worksheet, &data)
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
        .map(|row| row.iter().map(template_cell_value).collect::<Result<Vec<_>>>())
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::append_rows_to_sheet(entries, worksheet, &rows).map_err(ExcelError::from)
}

pub(crate) fn append_rows_to_sheet_with_comments(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    rows: &[Vec<CellValue>],
) -> Result<Vec<easyexcel_xlsx::TemplateCommentPlacement>> {
    let rows = rows
        .iter()
        .map(|row| row.iter().map(template_cell_value).collect::<Result<Vec<_>>>())
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::append_rows_to_sheet_with_comments(entries, worksheet, &rows)
        .map_err(ExcelError::from)
}

pub(crate) fn append_rows_to_sheet_with_decorations(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    rows: &[Vec<CellValue>],
) -> Result<Vec<easyexcel_xlsx::TemplateDecorationPlacement>> {
    let rows = rows
        .iter()
        .map(|row| row.iter().map(template_cell_value).collect::<Result<Vec<_>>>())
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::append_rows_to_sheet_with_decorations(entries, worksheet, &rows)
        .map_err(ExcelError::from)
}

fn template_collection_fill(fill: &PendingCollectionFill) -> Result<TemplateCollectionFill> {
    Ok(TemplateCollectionFill {
        name: fill.wrapper.name().map(str::to_owned),
        rows: fill
            .wrapper
            .rows()
            .iter()
            .map(template_fill_data)
            .collect::<Result<Vec<_>>>()?,
        direction: match fill.config.effective_direction() {
            crate::FillDirection::Vertical => TemplateFillDirection::Vertical,
            crate::FillDirection::Horizontal => TemplateFillDirection::Horizontal,
        },
        force_new_row: fill.config.effective_force_new_row(),
        auto_style: fill.config.effective_auto_style(),
        order: fill.order,
        column_styles: fill.column_styles.clone(),
    })
}

fn template_fill_data(data: &TemplateData) -> Result<TemplateFillData> {
    let mut values = BTreeMap::new();
    for (key, value) in data.values() {
        values.insert(key.clone(), template_cell_value(value)?);
    }
    Ok(TemplateFillData { values })
}

fn template_cell_value(value: &CellValue) -> Result<TemplateCellValue> {
    Ok(match value {
        CellValue::Empty => TemplateCellValue::Empty,
        CellValue::Image(bytes) => TemplateCellValue::Images {
            value: Box::new(TemplateCellValue::Empty),
            images: vec![easyexcel_xlsx::TemplateImage::new(bytes.clone())],
        },
        CellValue::String(text) => TemplateCellValue::Text(text.clone()),
        CellValue::Hyperlink { url, text } => crate::write::template_write::template_hyperlink_value(
            url,
            text,
            crate::HyperlinkType::Url,
            crate::CoordinateData::new(),
        ),
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            coordinates,
        } => {
            if *hyperlink_type == crate::HyperlinkType::None {
                TemplateCellValue::Text(text.clone())
            } else {
                crate::write::template_write::template_hyperlink_value(
                    address,
                    text,
                    *hyperlink_type,
                    *coordinates,
                )
            }
        }
        CellValue::RichText(value) => {
            return crate::write::excel_writer_core::template_rich_text_cell_value(value);
        }
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
        CellValue::Comment { value, text } => TemplateCellValue::Comment {
            value: Box::new(template_cell_value(value)?),
            comment: easyexcel_xlsx::TemplateComment {
                text: text.clone(),
                ..easyexcel_xlsx::TemplateComment::default()
            },
        },
        CellValue::CommentWithMetadata { value, comment } => TemplateCellValue::Comment {
            value: Box::new(template_cell_value(value)?),
            comment: crate::write::template_write::template_comment_data(comment),
        },
        CellValue::Images { value, images } => {
            return crate::write::template_write::template_images_value(value, images);
        }
    })
}
