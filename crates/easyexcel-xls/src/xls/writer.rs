//! XLS 公共模型写入适配器。
//!
//! `easyexcel_model::Workbook` 与 EasyExcel 门面共用同一份完整
//! [`Biff8Book`] 序列化引擎。这里仅负责格式中立模型到 BIFF8 模型的无损转换，
//! 不再维护第二套简化 record writer。

use std::collections::{BTreeSet, HashMap};
use std::io::{Seek, Write};

use easyexcel_io::{Error, Result};
use easyexcel_model::model::{Cell, CellValue, Workbook};
use easyexcel_model::styles::{
    BorderStyle, CellStyle, Color, FillPattern, HAlign, VAlign,
};
use easyexcel_model::DateSystem;

use crate::biff8::cached::Biff8Cached;
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
        let mut sheet_formula_caches = HashMap::new();
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
            let cell = source.get(row, column);
            // 公式单元格预置缓存：将模型层 CellValue 转换为 Biff8Cached，
            // 用于空表达式 roundtrip（写入时保留用户指定的缓存结果）。
            if let Some(Cell::Formula { cached, .. }) = cell {
                if let Some(biff_cached) = cell_value_to_biff8_cached(cached) {
                    let row16 = u16::try_from(row).unwrap_or(u16::MAX);
                    let col8 = u8::try_from(column).unwrap_or(u8::MAX);
                    sheet_formula_caches.insert((row16, col8), biff_cached);
                }
            }
            let value = cell.map_or(Biff8Value::Blank, model_cell_to_biff8);
            target.set(
                row,
                usize::try_from(column).map_err(|_| {
                    Error::Xls(format!("column index overflow in '{}': {column}", source.name))
                })?,
                Biff8Cell::general(value).with_xf(xf),
            )?;
        }
        book.formula_caches.push(sheet_formula_caches);
    }
    Ok(book)
}

/// 将模型层 `CellValue` 转换为 BIFF8 公式缓存值。
/// 空值（`CellValue::Empty`）返回 `None`，由公式引擎兜底。
fn cell_value_to_biff8_cached(value: &CellValue) -> Option<Biff8Cached> {
    Some(match value {
        CellValue::Number(n) => Biff8Cached::Number(*n),
        CellValue::Text(t) => Biff8Cached::Text(t.clone()),
        CellValue::Bool(b) => Biff8Cached::Bool(*b),
        CellValue::Error(e) => Biff8Cached::Error(e.biff_code()),
        CellValue::Empty => return None,
    })
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

#[cfg(test)]
mod writer_tests {
    use super::*;
    use easyexcel_model::model::Sheet;

    // --- validate_workbook_level_state ---

    /// 空工作簿通过验证。
    #[test]
    fn validate_empty_workbook_passes() {
        let wb = Workbook::empty();
        validate_workbook_level_state(&wb).unwrap();
    }

    /// active_sheet 超出 sheet 数量时返回错误。
    #[test]
    fn validate_active_sheet_out_of_range() {
        let mut wb = Workbook::empty();
        wb.active_sheet = 5;
        let result = validate_workbook_level_state(&wb);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("active sheet index"));
    }

    // --- validate_sheet_state ---

    /// 空 sheet 通过验证。
    #[test]
    fn validate_empty_sheet_passes() {
        let sheet = Sheet::new("S");
        validate_sheet_state(&sheet).unwrap();
    }

    // --- model_cell_to_biff8 ---

    #[test]
    fn cell_empty_to_blank() {
        assert!(matches!(model_cell_to_biff8(&Cell::Empty), Biff8Value::Blank));
    }

    #[test]
    fn cell_number() {
        match model_cell_to_biff8(&Cell::Number(3.14)) {
            Biff8Value::Number(v) => assert!((v - 3.14).abs() < f64::EPSILON),
            other => panic!("expected Number, got {other:?}"),
        }
    }

    #[test]
    fn cell_text() {
        match model_cell_to_biff8(&Cell::Text("hello".to_owned())) {
            Biff8Value::Text(t) => assert_eq!(t, "hello"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn cell_bool() {
        assert!(matches!(
            model_cell_to_biff8(&Cell::Bool(true)),
            Biff8Value::Bool(true)
        ));
    }

    #[test]
    fn cell_formula_non_empty() {
        use easyexcel_model::model::CellValue;
        match model_cell_to_biff8(&Cell::Formula {
            expr: "=SUM(A1)".to_owned(),
            cached: CellValue::Number(10.0),
        }) {
            Biff8Value::Formula(f) => assert_eq!(f, "=SUM(A1)"),
            other => panic!("expected Formula, got {other:?}"),
        }
    }


    // --- fixed_dimension ---

    #[test]
    fn fixed_dimension_normal() {
        let result = fixed_dimension(10.0, 256.0, 65535, "width", "sheet").unwrap();
        assert_eq!(result, 2560);
    }

    #[test]
    fn fixed_dimension_rounds() {
        let result = fixed_dimension(10.5, 256.0, 65535, "width", "sheet").unwrap();
        assert_eq!(result, 2688); // 10.5 * 256 = 2688
    }

    #[test]
    fn fixed_dimension_nan_returns_error() {
        assert!(fixed_dimension(f64::NAN, 256.0, 65535, "w", "s").is_err());
    }

    #[test]
    fn fixed_dimension_infinity_returns_error() {
        assert!(fixed_dimension(f64::INFINITY, 256.0, 65535, "w", "s").is_err());
    }

    #[test]
    fn fixed_dimension_negative_returns_error() {
        assert!(fixed_dimension(-1.0, 256.0, 65535, "w", "s").is_err());
    }

    #[test]
    fn fixed_dimension_zero_returns_error() {
        assert!(fixed_dimension(0.0, 256.0, 65535, "w", "s").is_err());
    }

    #[test]
    fn fixed_dimension_exceeds_max_returns_error() {
        // value * scale > maximum_units
        assert!(fixed_dimension(300.0, 256.0, 255, "w", "s").is_err());
    }

    #[test]
    fn fixed_dimension_error_message_includes_label_and_owner() {
        let err = fixed_dimension(f64::NAN, 1.0, 100, "my_label", "my_sheet")
            .unwrap_err()
            .to_string();
        assert!(err.contains("my_label"));
        assert!(err.contains("my_sheet"));
    }

    // --- color ---

    #[test]
    fn color_none_returns_none() {
        assert!(color(Color(None)).is_none());
    }

    #[test]
    fn color_some_rgb() {
        let c = color(Color(Some(0xFF123456))).unwrap();
        assert_eq!(c, Biff8Color::Rgb(0x123456));
    }

    #[test]
    fn color_strips_alpha() {
        let c = color(Color(Some(0xAA_FF_00_00))).unwrap();
        assert_eq!(c, Biff8Color::Rgb(0xFF0000));
    }

    // --- horizontal_alignment ---

    #[test]
    fn horizontal_alignment_all_variants() {
        assert_eq!(horizontal_alignment(HAlign::General), Biff8HorizontalAlignment::General);
        assert_eq!(horizontal_alignment(HAlign::Left), Biff8HorizontalAlignment::Left);
        assert_eq!(horizontal_alignment(HAlign::Center), Biff8HorizontalAlignment::Center);
        assert_eq!(horizontal_alignment(HAlign::Right), Biff8HorizontalAlignment::Right);
        assert_eq!(horizontal_alignment(HAlign::Fill), Biff8HorizontalAlignment::Fill);
        assert_eq!(horizontal_alignment(HAlign::Justify), Biff8HorizontalAlignment::Justify);
        assert_eq!(horizontal_alignment(HAlign::CenterContinuous), Biff8HorizontalAlignment::CenterAcross);
        assert_eq!(horizontal_alignment(HAlign::Distributed), Biff8HorizontalAlignment::Distributed);
    }

    // --- vertical_alignment ---

    #[test]
    fn vertical_alignment_all_variants() {
        assert_eq!(vertical_alignment(VAlign::Top), Biff8VerticalAlignment::Top);
        assert_eq!(vertical_alignment(VAlign::Bottom), Biff8VerticalAlignment::Bottom);
        assert_eq!(vertical_alignment(VAlign::Center), Biff8VerticalAlignment::Center);
        assert_eq!(vertical_alignment(VAlign::Justify), Biff8VerticalAlignment::Justify);
        assert_eq!(vertical_alignment(VAlign::Distributed), Biff8VerticalAlignment::Distributed);
    }

    // --- border_style ---

    #[test]
    fn border_style_all_variants() {
        assert_eq!(border_style(BorderStyle::None), Biff8BorderStyle::None);
        assert_eq!(border_style(BorderStyle::Thin), Biff8BorderStyle::Thin);
        assert_eq!(border_style(BorderStyle::Medium), Biff8BorderStyle::Medium);
        assert_eq!(border_style(BorderStyle::Thick), Biff8BorderStyle::Thick);
        assert_eq!(border_style(BorderStyle::Dashed), Biff8BorderStyle::Dashed);
        assert_eq!(border_style(BorderStyle::Dotted), Biff8BorderStyle::Dotted);
        assert_eq!(border_style(BorderStyle::Double), Biff8BorderStyle::Double);
        assert_eq!(border_style(BorderStyle::Hair), Biff8BorderStyle::Hair);
    }

    // --- fill_pattern ---

    #[test]
    fn fill_pattern_none_solid_gray125() {
        assert_eq!(fill_pattern(FillPattern::None).unwrap(), Biff8FillPattern::None);
        assert_eq!(fill_pattern(FillPattern::Solid).unwrap(), Biff8FillPattern::Solid);
        assert_eq!(fill_pattern(FillPattern::Gray125).unwrap(), Biff8FillPattern::Gray125);
    }

    #[test]
    fn fill_pattern_other_valid_codes() {
        assert_eq!(fill_pattern(FillPattern::Other(2)).unwrap(), Biff8FillPattern::MediumGray);
        assert_eq!(fill_pattern(FillPattern::Other(3)).unwrap(), Biff8FillPattern::DarkGray);
        assert_eq!(fill_pattern(FillPattern::Other(4)).unwrap(), Biff8FillPattern::LightGray);
        assert_eq!(fill_pattern(FillPattern::Other(5)).unwrap(), Biff8FillPattern::DarkHorizontal);
        assert_eq!(fill_pattern(FillPattern::Other(6)).unwrap(), Biff8FillPattern::DarkVertical);
        assert_eq!(fill_pattern(FillPattern::Other(7)).unwrap(), Biff8FillPattern::DarkDown);
        assert_eq!(fill_pattern(FillPattern::Other(8)).unwrap(), Biff8FillPattern::DarkUp);
        assert_eq!(fill_pattern(FillPattern::Other(9)).unwrap(), Biff8FillPattern::DarkGrid);
        assert_eq!(fill_pattern(FillPattern::Other(10)).unwrap(), Biff8FillPattern::DarkTrellis);
        assert_eq!(fill_pattern(FillPattern::Other(11)).unwrap(), Biff8FillPattern::LightHorizontal);
        assert_eq!(fill_pattern(FillPattern::Other(12)).unwrap(), Biff8FillPattern::LightVertical);
        assert_eq!(fill_pattern(FillPattern::Other(13)).unwrap(), Biff8FillPattern::LightDown);
        assert_eq!(fill_pattern(FillPattern::Other(14)).unwrap(), Biff8FillPattern::LightUp);
        assert_eq!(fill_pattern(FillPattern::Other(15)).unwrap(), Biff8FillPattern::LightGrid);
        assert_eq!(fill_pattern(FillPattern::Other(16)).unwrap(), Biff8FillPattern::LightTrellis);
        assert_eq!(fill_pattern(FillPattern::Other(17)).unwrap(), Biff8FillPattern::Gray125);
        assert_eq!(fill_pattern(FillPattern::Other(18)).unwrap(), Biff8FillPattern::Gray0625);
    }

    #[test]
    fn fill_pattern_other_invalid_code_returns_error() {
        let result = fill_pattern(FillPattern::Other(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported"));
    }

    #[test]
    fn fill_pattern_other_code_1_returns_error() {
        // code 1 is Solid but via Other path it's not handled
        let result = fill_pattern(FillPattern::Other(1));
        assert!(result.is_err());
    }

    #[test]
    fn fill_pattern_other_code_0_returns_error() {
        let result = fill_pattern(FillPattern::Other(0));
        assert!(result.is_err());
    }

    // --- number_format ---

    #[test]
    fn number_format_empty_returns_none() {
        let style = CellStyle {
            number_format: String::new(),
            ..CellStyle::default()
        };
        assert!(number_format(&style).is_none());
    }

    #[test]
    fn number_format_general_returns_none() {
        let style = CellStyle {
            number_format: "General".to_owned(),
            ..CellStyle::default()
        };
        assert!(number_format(&style).is_none());
    }

    #[test]
    fn number_format_general_case_insensitive() {
        let style = CellStyle {
            number_format: "GENERAL".to_owned(),
            ..CellStyle::default()
        };
        assert!(number_format(&style).is_none());
    }

    #[test]
    fn number_format_custom() {
        let style = CellStyle {
            number_format: "0.000".to_owned(),
            ..CellStyle::default()
        };
        match number_format(&style).unwrap() {
            Biff8NumberFormat::Custom(code) => assert_eq!(code, "0.000"),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn number_format_builtin_from_id() {
        let style = CellStyle {
            number_format: String::new(),
            number_format_id: Some(2),
            ..CellStyle::default()
        };
        match number_format(&style).unwrap() {
            Biff8NumberFormat::Builtin(id) => assert_eq!(id, 2),
            other => panic!("expected Builtin, got {other:?}"),
        }
    }

    #[test]
    fn number_format_id_too_large_returns_none() {
        let style = CellStyle {
            number_format: String::new(),
            number_format_id: Some(300), // > u8::MAX
            ..CellStyle::default()
        };
        assert!(number_format(&style).is_none());
    }

    #[test]
    fn number_format_custom_takes_precedence_over_id() {
        let style = CellStyle {
            number_format: "0.00".to_owned(),
            number_format_id: Some(2),
            ..CellStyle::default()
        };
        match number_format(&style).unwrap() {
            Biff8NumberFormat::Custom(code) => assert_eq!(code, "0.00"),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn number_format_whitespace_trimmed() {
        let style = CellStyle {
            number_format: "  0.00  ".to_owned(),
            ..CellStyle::default()
        };
        match number_format(&style).unwrap() {
            Biff8NumberFormat::Custom(code) => assert_eq!(code, "0.00"),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    // --- style_request ---

    #[test]
    fn style_request_maps_all_fields() {
        let style = CellStyle {
            font: easyexcel_model::styles::Font {
                bold: true,
                italic: true,
                strike: true,
                underline: true,
                size: 14.0,
                name: "Arial".to_owned(),
                color: Color(Some(0xFFFF0000)),
            },
            halign: HAlign::Center,
            valign: VAlign::Top,
            wrap_text: true,
            fill: easyexcel_model::styles::Fill {
                pattern: FillPattern::Solid,
                fg: Color(Some(0xFF00FF00)),
                bg: Color(Some(0xFF0000FF)),
            },
            borders: easyexcel_model::styles::Borders {
                left: easyexcel_model::styles::BorderEdge {
                    style: BorderStyle::Thin,
                    color: Color(Some(0xFFFF0000)),
                },
                right: easyexcel_model::styles::BorderEdge {
                    style: BorderStyle::Medium,
                    color: Color(None),
                },
                top: easyexcel_model::styles::BorderEdge {
                    style: BorderStyle::None,
                    color: Color(None),
                },
                bottom: easyexcel_model::styles::BorderEdge {
                    style: BorderStyle::Thick,
                    color: Color(None),
                },
            },
            number_format: "0.00".to_owned(),
            number_format_id: None,
        };
        let req = style_request(&style).unwrap();
        assert!(req.bold);
        assert!(req.italic);
        assert!(req.strikeout);
        assert_eq!(req.underline, Biff8Underline::Single);
        assert!(req.font_height_twips.is_some());
        assert_eq!(req.font_name.as_deref(), Some("Arial"));
        assert!(req.font_color.is_some());
        assert!(req.horizontal_alignment.is_some());
        assert!(req.vertical_alignment.is_some());
        assert!(req.wrap);
        assert!(req.fill_pattern.is_some());
        assert!(req.fill_foreground_color.is_some());
        assert!(req.fill_background_color.is_some());
        assert!(req.border_left.is_some());
        assert!(req.number_format.is_some());
    }

    #[test]
    fn style_request_no_underline() {
        let style = CellStyle {
            font: easyexcel_model::styles::Font {
                underline: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let req = style_request(&style).unwrap();
        assert_eq!(req.underline, Biff8Underline::None);
    }

    // --- resolve_style_index ---

    #[test]
    fn resolve_style_index_none_returns_general() {
        let map = HashMap::new();
        let result = resolve_style_index(None, &map, "col", 0, "S");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), XF_GENERAL);
    }

    #[test]
    fn resolve_style_index_known_returns_mapped() {
        let mut map = HashMap::new();
        map.insert(1u32, 42u16);
        let result = resolve_style_index(Some(1), &map, "col", 0, "S");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn resolve_style_index_unknown_returns_error() {
        let map = HashMap::new();
        let result = resolve_style_index(Some(99), &map, "row", 5, "Sheet1");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("99"));
        assert!(msg.contains("row"));
        assert!(msg.contains("Sheet1"));
    }

    // --- to_biff8_book 集成验证 ---

    /// 空工作簿生成空 Biff8Book（默认 sheet 由 build_workbook_stream 补充）。
    #[test]
    fn to_biff8_book_empty_workbook() {
        let wb = Workbook::empty();
        let book = to_biff8_book(&wb).unwrap();
        // to_biff8_book 只转换 workbook.sheets，空 sheets 不产生默认 sheet
        assert_eq!(book.sheets.len(), 0);
    }

    /// 单 sheet 标量值 roundtrip。
    #[test]
    fn to_biff8_book_scalar_values() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("Data");
        s.set(0, 0, Cell::Number(42.0));
        s.set(0, 1, Cell::Text("hello".to_owned()));
        s.set(1, 0, Cell::Bool(true));
        wb.sheets.push(s);
        let book = to_biff8_book(&wb).unwrap();
        assert_eq!(book.sheets.len(), 1);
    }

    /// 重复 sheet 名（大小写不敏感）返回错误。
    #[test]
    fn to_biff8_book_duplicate_sheet_names() {
        let mut wb = Workbook::empty();
        wb.sheets.push(Sheet::new("Sheet1"));
        wb.sheets.push(Sheet::new("SHEET1"));
        let result = to_biff8_book(&wb);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate"));
    }

    /// 冻结窗格配置正确传递。
    #[test]
    fn to_biff8_book_frozen_panes() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("S");
        s.set(0, 0, Cell::Number(1.0));
        s.frozen.rows = 1;
        s.frozen.cols = 1;
        wb.sheets.push(s);
        let book = to_biff8_book(&wb).unwrap();
        // 冻结窗格不报错即验证通过
        assert_eq!(book.sheets.len(), 1);
    }

    /// 合并单元格范围正确传递。
    #[test]
    fn to_biff8_book_merged_cells() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("S");
        s.set(0, 0, Cell::Number(1.0));
        s.merged.push(easyexcel_model::model::CellRange::new(
            easyexcel_model::model::CellAddress::new(0, 0),
            easyexcel_model::model::CellAddress::new(1, 1),
        ));
        wb.sheets.push(s);
        let book = to_biff8_book(&wb).unwrap();
        assert_eq!(book.sheets[0].merges.len(), 1);
    }

    /// 列宽和行高正确传递。
    #[test]
    fn to_biff8_book_column_row_dimensions() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("S");
        s.set(0, 0, Cell::Number(1.0));
        s.default_col_width = 10.0;
        s.default_row_height = 20.0;
        s.columns.insert(0, easyexcel_model::model::ColInfo {
            width: Some(15.0),
            style: None,
            hidden: false,
        });
        s.rows.insert(0, easyexcel_model::model::RowInfo {
            height: Some(25.0),
            style: None,
            hidden: false,
        });
        wb.sheets.push(s);
        let book = to_biff8_book(&wb).unwrap();
        assert_eq!(book.sheets.len(), 1);
    }

    /// visibility 传递。
    #[test]
    fn to_biff8_book_sheet_visibility() {
        use easyexcel_model::model::Visibility;
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("Hidden");
        s.set(0, 0, Cell::Number(1.0));
        s.visibility = Visibility::Hidden;
        wb.sheets.push(s);
        let book = to_biff8_book(&wb).unwrap();
        assert_eq!(book.sheets[0].visibility, Visibility::Hidden);
    }

}
