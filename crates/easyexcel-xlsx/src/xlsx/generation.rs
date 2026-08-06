//! `rust_xlsxwriter` 生成后端与加密落盘边界。
//!
//! EasyExcel 门面通过本模块获得工作簿句柄；底层依赖、序列化、文件创建和
//! MS-OFFCRYPTO 包装由 `easyexcel-xlsx` 统一拥有。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{Error, Result};

pub use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript, FormatUnderline, Image,
    Note, ObjectMovement, Workbook, Worksheet,
};

use super::encrypt::{ReadWriteSeek, encrypt_package_to};

/// XLSX worksheet row count limit defined by ECMA-376.
pub const XLSX_MAX_ROWS: u32 = 1_048_576;

/// XLSX worksheet column count limit defined by ECMA-376.
pub const XLSX_MAX_COLUMNS: u16 = 16_384;

/// 创建 XLSX 生成工作簿。
#[must_use]
pub fn new_workbook() -> Workbook {
    Workbook::new()
}

/// 在工作簿末尾创建工作表。
pub fn add_worksheet(workbook: &mut Workbook) -> &mut Worksheet {
    workbook.add_worksheet()
}

/// 创建命名工作表，可选择常量内存写入模式。
///
/// # Errors
///
/// 名称违反 XLSX 约束时返回错误。
pub fn create_worksheet<'a>(
    workbook: &'a mut Workbook,
    name: &str,
    constant_memory: bool,
) -> Result<&'a mut Worksheet> {
    let worksheet = if constant_memory {
        workbook.add_worksheet_with_constant_memory()
    } else {
        workbook.add_worksheet()
    };
    set_worksheet_name(worksheet, name)
}

/// 按名称获取可变工作表句柄。
///
/// # Errors
///
/// 工作表不存在时返回错误。
pub fn worksheet_by_name<'a>(workbook: &'a mut Workbook, name: &str) -> Result<&'a mut Worksheet> {
    workbook.worksheet_from_name(name).map_err(xlsxwriter_error)
}

/// 设置工作表名称。
///
/// # Errors
///
/// 名称违反 XLSX 约束时返回错误。
pub fn set_worksheet_name<'a>(
    worksheet: &'a mut Worksheet,
    name: &str,
) -> Result<&'a mut Worksheet> {
    worksheet.set_name(name).map_err(xlsxwriter_error)
}

/// 按像素设置列宽。
///
/// # Errors
///
/// 行列坐标或宽度无效时返回错误。
pub fn set_column_width_pixels(worksheet: &mut Worksheet, column: u16, pixels: u32) -> Result<()> {
    worksheet
        .set_column_width_pixels(column, pixels)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 按 Excel 字符单位设置列宽，并保持 OOXML 中的精确字符宽度。
///
/// # Errors
///
/// 列坐标或宽度无效时返回错误。
pub fn set_column_width_chars(worksheet: &mut Worksheet, column: u16, chars: u16) -> Result<()> {
    set_column_width_pixels(worksheet, column, u32::from(chars).saturating_mul(7))
}

/// 将 Java/POI 字符列宽换算为图片布局像素宽度。
#[must_use]
pub const fn column_width_pixels(width: u16) -> u32 {
    if width == 0 { 0 } else { width as u32 * 7 + 5 }
}

/// 将 Java/POI 行高换算为图片布局像素高度。
#[must_use]
pub const fn row_height_pixels(height: Option<u16>) -> u32 {
    match height {
        Some(height) => (height as u32 * 4 + 1) / 3,
        None => 20,
    }
}

/// 将动态列索引收窄为 XLSX 列坐标。
///
/// # Errors
///
/// 索引超出 `u16` 范围时返回错误。
pub fn column_index(index: usize) -> Result<u16> {
    u16::try_from(index).map_err(|_| Error::Xlsx("column index exceeds XLSX limit".to_owned()))
}

/// Validates a zero-based XLSX row index.
///
/// # Errors
///
/// Returns [`Error::Xlsx`] when the index exceeds the worksheet row limit.
pub fn validate_row_index(row: u32) -> Result<()> {
    if row >= XLSX_MAX_ROWS {
        return Err(Error::Xlsx(format!(
            "XLSX row index {row} exceeds {}",
            XLSX_MAX_ROWS - 1
        )));
    }
    Ok(())
}

/// Validates a zero-based XLSX column index.
///
/// # Errors
///
/// Returns [`Error::Xlsx`] when the index exceeds the worksheet column limit.
pub fn validate_column_index(column: u16) -> Result<()> {
    if column >= XLSX_MAX_COLUMNS {
        return Err(Error::Xlsx(format!(
            "XLSX column index {column} exceeds {}",
            XLSX_MAX_COLUMNS - 1
        )));
    }
    Ok(())
}

/// 设置行高（磅）。
///
/// # Errors
///
/// 行坐标或高度无效时返回错误。
pub fn set_row_height(worksheet: &mut Worksheet, row: u32, height: u16) -> Result<()> {
    worksheet
        .set_row_height(row, height)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 冻结指定行列之前的窗格。
///
/// # Errors
///
/// 冻结坐标无效时返回错误。
pub fn freeze_panes(worksheet: &mut Worksheet, row: u32, column: u16) -> Result<()> {
    worksheet
        .set_freeze_panes(row, column)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 自动调整已写入工作表的行列尺寸。
pub fn autofit(worksheet: &mut Worksheet) {
    worksheet.autofit();
}

/// 合并单元格区域。
///
/// # Errors
///
/// 区域无效、重叠或超出 XLSX 限制时返回错误。
pub fn merge_range(
    worksheet: &mut Worksheet,
    first_row: u32,
    first_column: u16,
    last_row: u32,
    last_column: u16,
    value: &str,
    format: &Format,
) -> Result<()> {
    worksheet
        .merge_range(
            first_row,
            first_column,
            last_row,
            last_column,
            value,
            format,
        )
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入空白单元格及格式。
pub fn write_blank(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_blank(row, column, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入无格式字符串。
pub fn write_string(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: impl AsRef<str>,
) -> Result<()> {
    worksheet
        .write_string(row, column, value.as_ref())
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式字符串。
pub fn write_string_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: impl AsRef<str>,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_string_with_format(row, column, value.as_ref(), format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入无格式布尔值。
pub fn write_boolean(worksheet: &mut Worksheet, row: u32, column: u16, value: bool) -> Result<()> {
    worksheet
        .write_boolean(row, column, value)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式布尔值。
pub fn write_boolean_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: bool,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_boolean_with_format(row, column, value, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入无格式数字。
pub fn write_number(worksheet: &mut Worksheet, row: u32, column: u16, value: f64) -> Result<()> {
    worksheet
        .write_number(row, column, value)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式数字。
pub fn write_number_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: f64,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_number_with_format(row, column, value, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入整数；超出 Excel 可精确表示范围时保留为文本。
pub fn write_integer(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: i64,
    format: Option<&Format>,
) -> Result<()> {
    const MAX_EXACT_EXCEL_INTEGER: u64 = 9_007_199_254_740_991;
    if value.unsigned_abs() <= MAX_EXACT_EXCEL_INTEGER {
        #[allow(clippy::cast_precision_loss)]
        let number = value as f64;
        return match format {
            Some(format) => write_number_with_format(worksheet, row, column, number, format),
            None => write_number(worksheet, row, column, number),
        };
    }
    match format {
        Some(format) => write_string_with_format(worksheet, row, column, value.to_string(), format),
        None => write_string(worksheet, row, column, value.to_string()),
    }
}

/// 写入无格式公式。
pub fn write_formula(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    formula: &str,
) -> Result<()> {
    worksheet
        .write_formula(row, column, formula)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式公式。
pub fn write_formula_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    formula: &str,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_formula_with_format(row, column, formula, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带显示文本及格式的超链接。
pub fn write_url_with_options(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    url: &str,
    text: &str,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_url_with_options(row, column, url, text, "", Some(format))
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式日期。
pub fn write_date_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: NaiveDate,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_datetime_with_format(row, column, value, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入带格式日期时间。
pub fn write_datetime_with_format(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: NaiveDateTime,
    format: &Format,
) -> Result<()> {
    worksheet
        .write_datetime_with_format(row, column, value, format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 插入单元格批注。
pub fn insert_note(worksheet: &mut Worksheet, row: u32, column: u16, text: &str) -> Result<()> {
    worksheet
        .insert_note(row, column, &Note::new(text))
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 从内存图片创建 XLSX 图片对象。
///
/// # Errors
///
/// 图片头或编码不受支持时返回错误。
pub fn image_from_buffer(bytes: &[u8]) -> Result<Image> {
    if bytes.len() < 8 {
        return Err(Error::Xlsx(
            "image buffer is too short to contain a valid header".to_owned(),
        ));
    }
    Image::new_from_buffer(bytes).map_err(xlsxwriter_error)
}

/// 将内存图片按单元格尺寸插入。
pub fn insert_image_fit_to_cell(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    bytes: &[u8],
    keep_aspect_ratio: bool,
) -> Result<()> {
    let image = image_from_buffer(bytes)?;
    worksheet
        .insert_image_fit_to_cell(row, column, &image, keep_aspect_ratio)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 按缩放尺寸、对象移动策略和偏移量插入图片。
#[allow(clippy::too_many_arguments)]
pub fn insert_scaled_image(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    bytes: &[u8],
    width: u32,
    height: u32,
    movement: ObjectMovement,
    left: u32,
    top: u32,
) -> Result<()> {
    let image = image_from_buffer(bytes)?
        .set_scale_to_size(width, height, false)
        .set_object_movement(movement);
    worksheet
        .insert_image_with_offset(row, column, &image, left, top)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 写入富文本片段及单元格格式。
pub fn write_rich_string(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    runs: &[(Format, String)],
    cell_format: &Format,
) -> Result<()> {
    let references = runs
        .iter()
        .map(|(format, text)| (format, text.as_str()))
        .collect::<Vec<_>>();
    worksheet
        .write_rich_string_with_format(row, column, &references, cell_format)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// XLSX 数字格式描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberFormatSpec {
    /// Excel 内置数字格式编号。
    Builtin(u8),
    /// 自定义数字格式代码。
    Custom(String),
}

/// 与具体门面元数据解耦的 XLSX 字体格式描述。
#[derive(Debug, Clone, Default)]
pub struct FontFormatSpec {
    /// 字体名称。
    pub name: Option<String>,
    /// 字号（磅）。
    pub size: Option<f64>,
    /// 是否斜体。
    pub italic: Option<bool>,
    /// 是否使用删除线。
    pub strikeout: Option<bool>,
    /// 字体颜色。
    pub color: Option<Color>,
    /// 上标或下标格式。
    pub script: Option<FormatScript>,
    /// 下划线格式。
    pub underline: Option<FormatUnderline>,
    /// 字符集编号。
    pub charset: Option<u8>,
    /// 是否粗体。
    pub bold: Option<bool>,
}

/// 与 EasyExcel annotation/handler 类型解耦的 XLSX 单元格格式描述。
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

/// 创建默认 XLSX 格式。
#[must_use]
pub fn new_format() -> Format {
    Format::new()
}

/// 从默认格式构造 XLSX 格式。
#[must_use]
pub fn build_format(spec: &FormatSpec) -> Format {
    apply_format_spec(Format::new(), spec)
}

/// 将中立格式描述应用到已有 XLSX 格式。
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn apply_format_spec(mut format: Format, spec: &FormatSpec) -> Format {
    if let Some(value) = spec.hidden {
        format = if value {
            format.set_hidden()
        } else {
            format.unset_hidden()
        };
    }
    if let Some(value) = spec.locked {
        format = if value {
            format.set_locked()
        } else {
            format.set_unlocked()
        };
    }
    if let Some(value) = spec.quote_prefix {
        format = if value {
            format.set_quote_prefix()
        } else {
            format.unset_quote_prefix()
        };
    }
    if let Some(value) = spec.horizontal_alignment {
        format = format.set_align(value);
    }
    if let Some(value) = spec.vertical_alignment {
        format = format.set_align(value);
    }
    if let Some(value) = spec.wrap_text {
        format = if value {
            format.set_text_wrap()
        } else {
            format.unset_text_wrap()
        };
    }
    if let Some(value) = spec.rotation {
        format = format.set_rotation(value);
    }
    if let Some(value) = spec.indent {
        format = format.set_indent(value);
    }
    if let Some(value) = spec.border_left {
        format = format.set_border_left(value);
    }
    if let Some(value) = spec.border_right {
        format = format.set_border_right(value);
    }
    if let Some(value) = spec.border_top {
        format = format.set_border_top(value);
    }
    if let Some(value) = spec.border_bottom {
        format = format.set_border_bottom(value);
    }
    if let Some(value) = spec.left_border_color {
        format = format.set_border_left_color(value);
    }
    if let Some(value) = spec.right_border_color {
        format = format.set_border_right_color(value);
    }
    if let Some(value) = spec.top_border_color {
        format = format.set_border_top_color(value);
    }
    if let Some(value) = spec.bottom_border_color {
        format = format.set_border_bottom_color(value);
    }
    if let Some(value) = spec.fill_pattern {
        format = format.set_pattern(value);
    }
    if let Some(value) = spec.fill_background_color {
        format = format.set_background_color(value);
    }
    if let Some(value) = spec.fill_foreground_color {
        format = format.set_foreground_color(value);
    }
    if let Some(value) = spec.shrink_to_fit {
        format = if value {
            format.set_shrink()
        } else {
            format.unset_shrink()
        };
    }
    if let Some(value) = &spec.number_format {
        format = match value {
            NumberFormatSpec::Builtin(index) => format.set_num_format_index(*index),
            NumberFormatSpec::Custom(code) => format.set_num_format(code),
        };
    }
    apply_font_format_spec(format, &spec.font)
}

/// 将字体描述应用到已有 XLSX 格式。
#[must_use]
pub fn apply_font_format_spec(mut format: Format, spec: &FontFormatSpec) -> Format {
    if let Some(value) = &spec.name {
        format = format.set_font_name(value);
    }
    if let Some(value) = spec.size {
        format = format.set_font_size(value);
    }
    if let Some(value) = spec.italic {
        format = if value {
            format.set_italic()
        } else {
            format.unset_italic()
        };
    }
    if let Some(value) = spec.strikeout {
        format = if value {
            format.set_font_strikethrough()
        } else {
            format.unset_font_strikethrough()
        };
    }
    if let Some(value) = spec.color {
        format = format.set_font_color(value);
    }
    if let Some(value) = spec.script {
        format = format.set_font_script(value);
    }
    if let Some(value) = spec.underline {
        format = format.set_underline(value);
    }
    if let Some(value) = spec.charset {
        format = format.set_font_charset(value);
    }
    if let Some(value) = spec.bold {
        format = if value {
            format.set_bold()
        } else {
            format.unset_bold()
        };
    }
    format
}

/// 在已有格式上覆盖自定义数字格式。
#[must_use]
pub fn with_number_format(format: Format, number_format: &str) -> Format {
    format.set_num_format(number_format)
}

/// 将 BIFF/OOXML 共享的 Excel 索引色映射为 XLSX 颜色。
#[must_use]
pub fn color_from_indexed(index: u8) -> Color {
    if index == 64 {
        return Color::Automatic;
    }
    let rgb = match index {
        0 | 8 => 0x0000_0000,
        1 | 9 => 0x00ff_ffff,
        2 | 10 => 0x00ff_0000,
        3 | 11 => 0x0000_ff00,
        4 | 12 | 39 => 0x0000_00ff,
        5 | 13 | 34 => 0x00ff_ff00,
        6 | 14 | 33 => 0x00ff_00ff,
        7 | 15 | 35 => 0x0000_ffff,
        16 | 37 => 0x0080_0000,
        17 => 0x0000_8000,
        18 | 32 => 0x0000_0080,
        19 => 0x0080_8000,
        20 | 36 => 0x0080_0080,
        21 | 38 => 0x0000_8080,
        22 => 0x00c0_c0c0,
        23 => 0x0080_8080,
        24 => 0x0099_99ff,
        25 => 0x007f_0000,
        26 => 0x00ff_ffcc,
        27 | 41 => 0x00cc_ffff,
        28 => 0x0066_0066,
        29 => 0x00ff_8080,
        30 => 0x0000_66cc,
        31 => 0x00cc_ccff,
        40 => 0x0000_ccff,
        42 => 0x00cc_ffcc,
        43 => 0x00ff_ff99,
        44 => 0x0099_ccff,
        45 => 0x00ff_99cc,
        46 => 0x00cc_99ff,
        47 => 0x00ff_cc99,
        48 => 0x0033_66ff,
        49 => 0x0033_cccc,
        50 => 0x0099_cc00,
        51 => 0x00ff_cc00,
        52 => 0x00ff_9900,
        53 => 0x00ff_6600,
        54 => 0x0066_6699,
        55 => 0x0096_9696,
        56 => 0x0000_3366,
        57 => 0x0033_9966,
        58 => 0x0000_3300,
        59 => 0x0033_3300,
        60 => 0x0099_3300,
        61 => 0x0099_3366,
        62 => 0x0033_3399,
        63 => 0x0033_3333,
        _ => return Color::Default,
    };
    Color::RGB(rgb)
}

/// 创建 RGB XLSX 颜色。
#[must_use]
pub const fn color_from_rgb(rgb: u32) -> Color {
    Color::RGB(rgb)
}

/// 保存工作簿到文件，可选使用密码加密。
///
/// # Errors
///
/// XLSX 序列化、文件写入或加密失败时返回错误。
pub fn save_workbook(workbook: &mut Workbook, path: &Path, password: Option<&str>) -> Result<()> {
    let Some(password) = password else {
        return workbook.save(path).map_err(xlsxwriter_error);
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    save_encrypted_workbook_to(workbook, password, &mut file)
}

/// 保存工作簿到任意输出流，可选使用密码加密。
///
/// # Errors
///
/// XLSX 序列化、流写入或加密失败时返回错误。
pub fn save_workbook_to_writer(
    workbook: &mut Workbook,
    output: &mut (dyn Write + Send),
    password: Option<&str>,
) -> Result<()> {
    if let Some(password) = password {
        let mut encrypted = std::io::Cursor::new(Vec::new());
        save_encrypted_workbook_to(workbook, password, &mut encrypted)?;
        output.write_all(encrypted.get_ref())?;
    } else {
        workbook
            .save_to_writer(&mut *output)
            .map_err(xlsxwriter_error)?;
    }
    output.flush()?;
    Ok(())
}

/// 将内存工作簿序列化为未加密的 OOXML ZIP 字节。
///
/// 该入口封装 `rust_xlsxwriter` 的具体序列化 API，供模板样式编译、
/// RoundTrip 包合并等基础能力复用，避免上层门面直接操作格式后端。
///
/// # Errors
///
/// 工作簿无法编码为合法 XLSX 包时返回错误。
pub fn serialize_workbook(workbook: &mut Workbook) -> Result<Vec<u8>> {
    workbook.save_to_buffer().map_err(xlsxwriter_error)
}

/// 序列化并加密工作簿到可读写 seek 流。
///
/// # Errors
///
/// XLSX 序列化或加密失败时返回错误。
pub fn save_encrypted_workbook_to(
    workbook: &mut Workbook,
    password: &str,
    output: &mut dyn ReadWriteSeek,
) -> Result<()> {
    let plaintext = serialize_workbook(workbook)?;
    encrypt_package_to(&plaintext, password, output)
}

/// 将已序列化的 OOXML 包保存到路径，可选加密为 CFB 容器。
pub fn save_package_bytes_to_path(
    plaintext: &[u8],
    path: &Path,
    password: Option<&str>,
) -> Result<()> {
    if let Some(password) = password {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        encrypt_package_to(plaintext, password, &mut file)?;
        file.flush()?;
        return Ok(());
    }
    easyexcel_io::io::file_utils::write_to_file(path, plaintext)
}

/// 将已序列化的 OOXML 包写入输出流，可选加密为 CFB 容器。
pub fn save_package_bytes_to_writer(
    plaintext: &[u8],
    output: &mut (dyn Write + Send),
    password: Option<&str>,
) -> Result<()> {
    if let Some(password) = password {
        let mut encrypted = std::io::Cursor::new(Vec::new());
        encrypt_package_to(plaintext, password, &mut encrypted)?;
        return easyexcel_io::io::io_utils::write_all_and_flush(output, encrypted.get_ref());
    }
    easyexcel_io::io::io_utils::write_all_and_flush(output, plaintext)
}

fn xlsxwriter_error(error: impl std::fmt::Display) -> Error {
    Error::Xlsx(error.to_string())
}
