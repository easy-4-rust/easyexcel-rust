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
/// `RoundTrip` 包合并等基础能力复用，避免上层门面直接操作格式后端。
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
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

#[cfg(test)]
#[path = "../generation_tests/tests.rs"]
mod tests;
