//! XLS 公共模型写入适配器。
//!
//! `easyexcel_model::Workbook` 与 EasyExcel 门面共用同一份完整
//! [`Biff8Book`] 序列化引擎。这里仅负责格式中立模型到 BIFF8 模型的无损转换，
//! 不再维护第二套简化 record writer。

use std::collections::{BTreeSet, HashMap};
use std::io::{Seek, Write};

use easyexcel_io::{Error, Result};
use easyexcel_model::model::{Cell, Workbook};
use easyexcel_model::styles::{
    BorderStyle, CellStyle, Color, FillPattern, HAlign, VAlign,
};
use easyexcel_model::DateSystem;

use crate::biff8::encode::XF_GENERAL;
use crate::biff8::{
    Biff8Book, Biff8BorderStyle, Biff8Cell, Biff8Color, Biff8FillPattern,
    Biff8HorizontalAlignment, Biff8Merge, Biff8NumberFormat, Biff8StyleRequest,
    Biff8Underline, Biff8Value, Biff8VerticalAlignment,
};

/// 将格式中立工作簿写为 XLS。
///
/// 该入口与 EasyExcel 高层 `.xls` 写入共用 `Biff8Book`，因此公式、完整 XF
/// 边框、SST、跨 Sheet LinkTable 和密码加密不会再因调用低层 API 而切换到旧后端。
///
/// # Errors
///
/// 模型包含无法由生成式 BIFF8 工作簿无损表达的 opaque/table/name 内容、坐标或
/// 样式越界，或者序列化和输出失败时返回错误。
pub fn write<W: Write + Seek>(workbook: &Workbook, writer: W) -> Result<()> {
    write_with_password(workbook, writer, None)
}

/// 使用调用级密码将格式中立工作簿写为 XLS。
///
/// # Errors
///
/// 模型转换、CryptoAPI 密钥材料生成、BIFF8/OLE2 序列化或输出失败时返回错误。
pub fn write_with_password<W: Write + Seek>(
    workbook: &Workbook,
    writer: W,
    password: Option<&str>,
) -> Result<()> {
    to_biff8_book(workbook)?.write_to_with_password(writer, password)
}

/// 将共享 [`Workbook`] 转换为完整 BIFF8 写入模型。
///
/// 生成式 BIFF8 尚无损不了解的模型部分会明确失败；模板中的 VBA、图表和未知
/// CFB stream 应继续使用 `Biff8TemplatePackage` 的 preserve/strip/replace 路径。
///
/// # Errors
///
/// 存在无法无损表示的对象、无效 Sheet/坐标、尺寸越界或不支持的样式时返回错误。
pub fn to_biff8_book(workbook: &Workbook) -> Result<Biff8Book> {
    validate_workbook_level_state(workbook)?;

    let mut book = Biff8Book {
        use_1904_windowing: workbook.date_system == DateSystem::Date1904,
        active_sheet: workbook.active_sheet,
        ..Biff8Book::default()
    };
    let style_map = resolve_used_styles(workbook, &mut book)?;
    let mut sheet_names = BTreeSet::new();

    for source in &workbook.sheets {
        let normalized_name = source.name.to_lowercase();
        if !sheet_names.insert(normalized_name) {
            return Err(Error::Xls(format!(
                "duplicate BIFF8 worksheet name (case-insensitive): {}",
                source.name
            )));
        }
        validate_sheet_state(source)?;
        let target = book.create_sheet(source.name.clone())?;
        target.visibility = source.visibility;
        let default_column_width = fixed_dimension(
            source.default_col_width,
            256.0,
            255_u16.saturating_mul(256),
            "default column width",
            &source.name,
        )?;
        let default_row_height = fixed_dimension(
            source.default_row_height,
            20.0,
            8_179,
            "default row height",
            &source.name,
        )?;
        target.set_default_column_width_units(default_column_width);
        target.set_default_row_height_twips(default_row_height)?;

        for (&column, info) in &source.columns {
            let column = usize::try_from(column).map_err(|_| {
                Error::Xls(format!("column index overflow in '{}': {column}", source.name))
            })?;
            let width = info.width.map_or(Ok(default_column_width), |width| {
                fixed_dimension(
                    width,
                    256.0,
                    255_u16.saturating_mul(256),
                    "column width",
                    &source.name,
                )
            })?;
            let xf = resolve_style_index(info.style, &style_map, "column", column, &source.name)?;
            target.set_column_metadata_at(
                column,
                width,
                xf,
                info.hidden,
                info.width.is_some(),
            )?;
        }
        for (&row, info) in &source.rows {
            let height = info.height.map_or(Ok(default_row_height), |height| {
                fixed_dimension(height, 20.0, 8_192, "row height", &source.name)
            })?;
            let row_index = usize::try_from(row).map_err(|_| {
                Error::Xls(format!("row index overflow in '{}': {row}", source.name))
            })?;
            let xf = if info.style.is_some() {
                Some(resolve_style_index(
                    info.style,
                    &style_map,
                    "row",
                    row_index,
                    &source.name,
                )?)
            } else {
                None
            };
            target.set_row_metadata_at(row, height, info.height.is_some(), xf, info.hidden)?;
        }

        if source.frozen.rows != 0 || source.frozen.cols != 0 {
            let columns = u16::try_from(source.frozen.cols).map_err(|_| {
                Error::Xls(format!(
                    "frozen column count exceeds BIFF8 limit in '{}': {}",
                    source.name, source.frozen.cols
                ))
            })?;
            target.set_freeze_panes(source.frozen.rows, columns)?;
        }

        for range in &source.merged {
            if range.end.row < range.start.row || range.end.col < range.start.col {
                return Err(Error::Xls(format!(
                    "invalid merged range in '{}': {:?}",
                    source.name, range
                )));
            }
            let first_column = u16::try_from(range.start.col).map_err(|_| {
                Error::Xls(format!("merge column overflow in '{}': {}", source.name, range.start.col))
            })?;
            let last_column = u16::try_from(range.end.col).map_err(|_| {
                Error::Xls(format!("merge column overflow in '{}': {}", source.name, range.end.col))
            })?;
            target.add_merge(Biff8Merge::try_from_bounds(
                range.start.row,
                range.end.row,
                first_column,
                last_column,
            )?);
        }

        let coordinates = source
            .cells
            .keys()
            .chain(source.styles.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        for (row, column) in coordinates {
            let xf = match source.style_at(row, column) {
                Some(style) => *style_map.get(&style).ok_or_else(|| {
                    Error::Xls(format!(
                        "unknown style index {style} at {}!R{}C{}",
                        source.name,
                        row.saturating_add(1),
                        column.saturating_add(1)
                    ))
                })?,
                None => XF_GENERAL,
            };
            let value = source
                .get(row, column)
                .map_or(Biff8Value::Blank, model_cell_to_biff8);
            target.set(
                row,
                usize::try_from(column).map_err(|_| {
                    Error::Xls(format!("column index overflow in '{}': {column}", source.name))
                })?,
                Biff8Cell::general(value).with_xf(xf),
            )?;
        }
    }
    Ok(book)
}

fn validate_workbook_level_state(workbook: &Workbook) -> Result<()> {
    if !workbook.defined_names.is_empty() {
        return Err(Error::Unsupported(
            "generated BIFF8 model write cannot preserve defined names; use the template package for record-preserving edits"
                .to_owned(),
        ));
    }
    if !workbook.opaque.is_empty() {
        return Err(Error::Unsupported(
            "generated BIFF8 model write cannot preserve opaque/VBA CFB parts; use Biff8TemplatePackage with an explicit macro policy"
                .to_owned(),
        ));
    }
    let sheet_count = workbook.sheets.len().max(1);
    if workbook.active_sheet >= sheet_count {
        return Err(Error::Xls(format!(
            "active sheet index {} exceeds sheet count {sheet_count}",
            workbook.active_sheet
        )));
    }
    if workbook.metadata.title.is_some()
        || workbook.metadata.author.is_some()
        || workbook.metadata.company.is_some()
        || workbook.metadata.created.is_some()
        || workbook.metadata.modified.is_some()
        || workbook.metadata.application.is_some()
    {
        return Err(Error::Unsupported(
            "generated BIFF8 model write does not yet preserve SummaryInformation metadata"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_sheet_state(sheet: &easyexcel_model::Sheet) -> Result<()> {
    if !sheet.opaque.is_empty() {
        return Err(Error::Unsupported(format!(
            "generated BIFF8 model write cannot preserve opaque parts in '{}'",
            sheet.name
        )));
    }
    if !sheet.tables.is_empty() {
        return Err(Error::Unsupported(format!(
            "BIFF8 does not provide an OOXML table carrier for '{}'",
            sheet.name
        )));
    }
    if !sheet.spills.is_empty() {
        return Err(Error::Unsupported(format!(
            "dynamic spill state must be materialized before BIFF8 write for '{}'",
            sheet.name
        )));
    }
    Ok(())
}

fn resolve_used_styles(workbook: &Workbook, book: &mut Biff8Book) -> Result<HashMap<u32, u16>> {
    let used = workbook
        .sheets
        .iter()
        .flat_map(|sheet| {
            sheet
                .styles
                .values()
                .copied()
                .chain(sheet.columns.values().filter_map(|info| info.style))
                .chain(sheet.rows.values().filter_map(|info| info.style))
        })
        .collect::<BTreeSet<_>>();
    let mut result = HashMap::with_capacity(used.len());
    for index in used {
        let style = workbook.styles.get(index).ok_or_else(|| {
            Error::Xls(format!("workbook references missing style index {index}"))
        })?;
        let request = style_request(style)?;
        result.insert(index, book.styles.resolve_xf(&request, XF_GENERAL));
    }
    Ok(result)
}

fn style_request(style: &CellStyle) -> Result<Biff8StyleRequest> {
    Ok(Biff8StyleRequest {
        bold: style.font.bold,
        italic: style.font.italic,
        strikeout: style.font.strike,
        underline: if style.font.underline {
            Biff8Underline::Single
        } else {
            Biff8Underline::None
        },
        font_height_twips: Some(fixed_dimension(
            style.font.size,
            20.0,
            u16::MAX,
            "font size",
            "style",
        )?),
        font_height_points: None,
        font_name: Some(style.font.name.clone()),
        font_color: color(style.font.color),
        horizontal_alignment: Some(horizontal_alignment(style.halign)),
        vertical_alignment: Some(vertical_alignment(style.valign)),
        wrap: style.wrap_text,
        fill_pattern: Some(fill_pattern(style.fill.pattern)?),
        fill_foreground_color: color(style.fill.fg),
        fill_background_color: color(style.fill.bg),
        border_left: Some(border_style(style.borders.left.style)),
        border_right: Some(border_style(style.borders.right.style)),
        border_top: Some(border_style(style.borders.top.style)),
        border_bottom: Some(border_style(style.borders.bottom.style)),
        border_left_color: color(style.borders.left.color),
        border_right_color: color(style.borders.right.color),
        border_top_color: color(style.borders.top.color),
        border_bottom_color: color(style.borders.bottom.color),
        number_format: number_format(style),
    })
}

fn resolve_style_index(
    style: Option<u32>,
    style_map: &HashMap<u32, u16>,
    owner_kind: &str,
    owner_index: usize,
    sheet_name: &str,
) -> Result<u16> {
    style.map_or(Ok(XF_GENERAL), |style| {
        style_map.get(&style).copied().ok_or_else(|| {
            Error::Xls(format!(
                "unknown style index {style} for {owner_kind} {owner_index} in '{sheet_name}'"
            ))
        })
    })
}

fn model_cell_to_biff8(cell: &Cell) -> Biff8Value {
    match cell {
        Cell::Empty => Biff8Value::Blank,
        Cell::Number(value) => Biff8Value::Number(*value),
        Cell::Text(value) => Biff8Value::Text(value.clone()),
        Cell::Bool(value) => Biff8Value::Bool(*value),
        Cell::Error(value) => Biff8Value::Error(value.biff_code()),
        Cell::Formula { expr, .. } => Biff8Value::Formula(expr.clone()),
    }
}

fn fixed_dimension(
    value: f64,
    scale: f64,
    maximum_units: u16,
    label: &str,
    owner: &str,
) -> Result<u16> {
    let units = value * scale;
    if !value.is_finite() || value <= 0.0 || units > f64::from(maximum_units) {
        return Err(Error::Unsupported(format!(
            "BIFF8 {label} is outside its representable range in '{owner}': {value}"
        )));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok(units.round() as u16)
}

fn color(value: Color) -> Option<Biff8Color> {
    value.0.map(|argb| Biff8Color::Rgb(argb & 0x00ff_ffff))
}

const fn horizontal_alignment(value: HAlign) -> Biff8HorizontalAlignment {
    match value {
        HAlign::General => Biff8HorizontalAlignment::General,
        HAlign::Left => Biff8HorizontalAlignment::Left,
        HAlign::Center => Biff8HorizontalAlignment::Center,
        HAlign::Right => Biff8HorizontalAlignment::Right,
        HAlign::Fill => Biff8HorizontalAlignment::Fill,
        HAlign::Justify => Biff8HorizontalAlignment::Justify,
        HAlign::CenterContinuous => Biff8HorizontalAlignment::CenterAcross,
        HAlign::Distributed => Biff8HorizontalAlignment::Distributed,
    }
}

const fn vertical_alignment(value: VAlign) -> Biff8VerticalAlignment {
    match value {
        VAlign::Top => Biff8VerticalAlignment::Top,
        VAlign::Bottom => Biff8VerticalAlignment::Bottom,
        VAlign::Center => Biff8VerticalAlignment::Center,
        VAlign::Justify => Biff8VerticalAlignment::Justify,
        VAlign::Distributed => Biff8VerticalAlignment::Distributed,
    }
}

const fn border_style(value: BorderStyle) -> Biff8BorderStyle {
    match value {
        BorderStyle::None => Biff8BorderStyle::None,
        BorderStyle::Thin => Biff8BorderStyle::Thin,
        BorderStyle::Medium => Biff8BorderStyle::Medium,
        BorderStyle::Thick => Biff8BorderStyle::Thick,
        BorderStyle::Dashed => Biff8BorderStyle::Dashed,
        BorderStyle::Dotted => Biff8BorderStyle::Dotted,
        BorderStyle::Double => Biff8BorderStyle::Double,
        BorderStyle::Hair => Biff8BorderStyle::Hair,
    }
}

fn fill_pattern(value: FillPattern) -> Result<Biff8FillPattern> {
    Ok(match value {
        FillPattern::None => Biff8FillPattern::None,
        FillPattern::Solid => Biff8FillPattern::Solid,
        FillPattern::Gray125 => Biff8FillPattern::Gray125,
        FillPattern::Other(2) => Biff8FillPattern::MediumGray,
        FillPattern::Other(3) => Biff8FillPattern::DarkGray,
        FillPattern::Other(4) => Biff8FillPattern::LightGray,
        FillPattern::Other(5) => Biff8FillPattern::DarkHorizontal,
        FillPattern::Other(6) => Biff8FillPattern::DarkVertical,
        FillPattern::Other(7) => Biff8FillPattern::DarkDown,
        FillPattern::Other(8) => Biff8FillPattern::DarkUp,
        FillPattern::Other(9) => Biff8FillPattern::DarkGrid,
        FillPattern::Other(10) => Biff8FillPattern::DarkTrellis,
        FillPattern::Other(11) => Biff8FillPattern::LightHorizontal,
        FillPattern::Other(12) => Biff8FillPattern::LightVertical,
        FillPattern::Other(13) => Biff8FillPattern::LightDown,
        FillPattern::Other(14) => Biff8FillPattern::LightUp,
        FillPattern::Other(15) => Biff8FillPattern::LightGrid,
        FillPattern::Other(16) => Biff8FillPattern::LightTrellis,
        FillPattern::Other(17) => Biff8FillPattern::Gray125,
        FillPattern::Other(18) => Biff8FillPattern::Gray0625,
        FillPattern::Other(code) => {
            return Err(Error::Unsupported(format!(
                "unsupported BIFF8 fill pattern code: {code}"
            )));
        }
    })
}

fn number_format(style: &CellStyle) -> Option<Biff8NumberFormat> {
    let code = style.number_format.trim();
    if !code.is_empty() && !code.eq_ignore_ascii_case("general") {
        return Some(Biff8NumberFormat::Custom(code.to_owned()));
    }
    style
        .number_format_id
        .and_then(|value| u8::try_from(value).ok())
        .map(Biff8NumberFormat::Builtin)
}
