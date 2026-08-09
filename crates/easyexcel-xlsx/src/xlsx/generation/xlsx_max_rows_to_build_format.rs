/// XLSX worksheet row count limit defined by ECMA-376.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XLSX_MAX_ROWS: u32 = 1_048_576;

/// XLSX worksheet column count limit defined by ECMA-376.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XLSX_MAX_COLUMNS: u16 = 16_384;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建 XLSX 生成工作簿。
#[must_use]
pub fn new_workbook() -> Workbook {
    Workbook::new()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在工作簿末尾创建工作表。
pub fn add_worksheet(workbook: &mut Workbook) -> &mut Worksheet {
    workbook.add_worksheet()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建命名工作表，可选择常量内存写入模式。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按名称获取可变工作表句柄。
///
/// # Errors
///
/// 工作表不存在时返回错误。
pub fn worksheet_by_name<'a>(workbook: &'a mut Workbook, name: &str) -> Result<&'a mut Worksheet> {
    workbook.worksheet_from_name(name).map_err(xlsxwriter_error)
}

/// 对工作表启用密码保护。
///
/// 对应 Java：`Sheet.protectSheet(String)`；具体 OOXML protection 属性由 XLSX 引擎编码。
pub fn protect_worksheet(worksheet: &mut Worksheet, password: &str) {
    worksheet.protect_with_password(password);
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 设置工作表名称。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按像素设置列宽。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按 Excel 字符单位设置列宽，并保持 OOXML 中的精确字符宽度。
///
/// # Errors
///
/// 列坐标或宽度无效时返回错误。
pub fn set_column_width_chars(worksheet: &mut Worksheet, column: u16, chars: u16) -> Result<()> {
    set_column_width_pixels(worksheet, column, u32::from(chars).saturating_mul(7))
}

/// 将 Java/POI 字符列宽换算为图片布局像素宽度。
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn column_width_pixels(width: u16) -> u32 {
    if width == 0 { 0 } else { width as u32 * 7 + 5 }
}

/// 将 Java/POI 行高换算为图片布局像素高度。
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn row_height_pixels(height: Option<u16>) -> u32 {
    match height {
        Some(height) => (height as u32 * 4 + 1) / 3,
        None => 20,
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将动态列索引收窄为 XLSX 列坐标。
///
/// # Errors
///
/// 索引超出 `u16` 范围时返回错误。
pub fn column_index(index: usize) -> Result<u16> {
    u16::try_from(index).map_err(|_| Error::Xlsx("column index exceeds XLSX limit".to_owned()))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates a zero-based XLSX row index.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Validates a zero-based XLSX column index.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 设置行高（磅）。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 冻结指定行列之前的窗格。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 自动调整已写入工作表的行列尺寸。
pub fn autofit(worksheet: &mut Worksheet) {
    worksheet.autofit();
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 合并单元格区域。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入空白单元格及格式。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入无格式字符串。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式字符串。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入无格式布尔值。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn write_boolean(worksheet: &mut Worksheet, row: u32, column: u16, value: bool) -> Result<()> {
    worksheet
        .write_boolean(row, column, value)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式布尔值。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入无格式数字。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn write_number(worksheet: &mut Worksheet, row: u32, column: u16, value: f64) -> Result<()> {
    worksheet
        .write_number(row, column, value)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式数字。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入整数；超出 Excel 可精确表示范围时保留为文本。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入无格式公式。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式公式。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带显示文本及格式的超链接。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式日期。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入带格式日期时间。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 插入单元格批注。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn insert_note(worksheet: &mut Worksheet, row: u32, column: u16, text: &str) -> Result<()> {
    worksheet
        .insert_note(row, column, &Note::new(text))
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 插入带原作者与对象移动语义的单元格批注。
pub fn insert_note_with_metadata(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    text: &str,
    author: Option<&str>,
    movement: Option<ObjectMovement>,
    visible: Option<bool>,
) -> Result<()> {
    let mut note = Note::new(text).add_author_prefix(false);
    if let Some(author) = author {
        note = note.set_author(author);
    }
    if let Some(movement) = movement {
        note = note.set_object_movement(movement);
    }
    if let Some(visible) = visible {
        note = note.set_visible(visible);
    }
    worksheet
        .insert_note(row, column, &note)
        .map(|_| ())
        .map_err(xlsxwriter_error)
}

/// 使用引擎中立移动策略插入带元数据的批注。
pub fn insert_note_with_policy(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    text: &str,
    author: Option<&str>,
    movement: Option<super::TemplateImageMovement>,
    visible: Option<bool>,
) -> Result<()> {
    insert_note_with_metadata(
        worksheet,
        row,
        column,
        text,
        author,
        movement.map(object_movement),
        visible,
    )
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 从内存图片创建 XLSX 图片对象。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将内存图片按单元格尺寸插入。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按缩放尺寸、对象移动策略和偏移量插入图片。
#[allow(clippy::too_many_arguments)]
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 使用引擎中立移动策略插入缩放图片。
#[allow(clippy::too_many_arguments)]
pub fn insert_scaled_image_with_policy(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    bytes: &[u8],
    width: u32,
    height: u32,
    movement: super::TemplateImageMovement,
    left: u32,
    top: u32,
) -> Result<()> {
    insert_scaled_image(
        worksheet,
        row,
        column,
        bytes,
        width,
        height,
        object_movement(movement),
        left,
        top,
    )
}

const fn object_movement(value: super::TemplateImageMovement) -> ObjectMovement {
    match value {
        super::TemplateImageMovement::MoveAndResize => ObjectMovement::MoveAndSizeWithCells,
        super::TemplateImageMovement::MoveDontResize => {
            ObjectMovement::MoveButDontSizeWithCells
        }
        super::TemplateImageMovement::DontMoveOrResize => {
            ObjectMovement::DontMoveOrSizeWithCells
        }
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 写入富文本片段及单元格格式。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 使用中立字体规格写入富文本片段及单元格格式。
///
/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。调用方负责按照 Java 的字符
/// 区间语义切分文本，具体后端字体对象只在 `easyexcel-xlsx` 内构造。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn write_rich_string_with_font_specs(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    runs: &[(FontFormatSpec, String)],
    cell_format: &Format,
) -> Result<()> {
    let compiled_runs = runs
        .iter()
        .map(|(font, text)| {
            (
                build_format(&FormatSpec {
                    font: font.clone(),
                    ..FormatSpec::default()
                }),
                text.clone(),
            )
        })
        .collect::<Vec<_>>();
    write_rich_string(worksheet, row, column, &compiled_runs, cell_format)
}

/// 将一组已编译单元格格式编码为可供模板样式导入的最小 XLSX 工作簿。
///
/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。每个格式写入第一列的独立空白
/// 单元格，模板层随后只需复制对应 style index，无需感知底层 workbook API。
///
/// # Errors
///
/// 格式数量超出 XLSX 行上限，或工作簿序列化失败时返回错误。
pub fn compile_blank_format_workbook(formats: &[Format]) -> Result<Vec<u8>> {
    let mut workbook = new_workbook();
    let worksheet = workbook.add_worksheet();
    for (index, format) in formats.iter().enumerate() {
        let row = u32::try_from(index)
            .map_err(|_| Error::Xlsx("too many template fill styles".to_owned()))?;
        write_blank(worksheet, row, 0, format)?;
    }
    serialize_workbook(&mut workbook)
}

include!("xlsx_max_rows_to_build_format/number_format_spec.rs");

include!("xlsx_max_rows_to_build_format/font_format_spec.rs");

include!("xlsx_max_rows_to_build_format/format_spec.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 创建默认 XLSX 格式。
#[must_use]
pub fn new_format() -> Format {
    Format::new()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 从默认格式构造 XLSX 格式。
#[must_use]
pub fn build_format(spec: &FormatSpec) -> Format {
    apply_format_spec(Format::new(), spec)
}
