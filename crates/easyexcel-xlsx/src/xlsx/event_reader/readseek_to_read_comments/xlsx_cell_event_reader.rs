/// 对应 Java：无直接对应对象；Rust 架构扩展。 单个工作表的拉取式单元格事件读取器。
pub struct XlsxCellEventReader<'a> {
    reader: XmlReader<Box<dyn BufRead + 'a>>,
    cell_formats: &'a [XlsxNumberFormat],
    compiled_cell_formats: Vec<Option<CompiledExcelFormat>>,
    options: XlsxDisplayOptions,
    row_index: u32,
    column_index: usize,
    buffer: Vec<u8>,
    cell_buffer: Vec<u8>,
    raw_value: String,
    inline_value: String,
    formula: String,
    shared_strings: &'a dyn SharedStringCacheReader,
}

impl<'a> XlsxCellEventReader<'a> {
    fn new(
        mut reader: XmlReader<Box<dyn BufRead + 'a>>,
        cell_formats: &'a [XlsxNumberFormat],
        options: XlsxDisplayOptions,
        shared_strings: &'a dyn SharedStringCacheReader,
    ) -> Result<Self> {
        let mut buffer = Vec::with_capacity(256);
        loop {
            buffer.clear();
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) if element.local_name().as_ref() == b"sheetData" => break,
                Event::Eof => {
                    return Err(Error::Xlsx(
                        "unexpected end of XML before worksheet data".to_owned(),
                    ));
                }
                _ => {}
            }
        }
        Ok(Self {
            reader,
            cell_formats,
            compiled_cell_formats: cell_formats
                .iter()
                .map(XlsxNumberFormat::compile)
                .collect(),
            options,
            row_index: 0,
            column_index: 0,
            buffer,
            cell_buffer: Vec::with_capacity(256),
            raw_value: String::with_capacity(32),
            inline_value: String::with_capacity(64),
            formula: String::with_capacity(32),
            shared_strings,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 读取下一个单元格事件。
    ///
    /// # Errors
    ///
    /// XML、坐标、共享字符串或数字无效时返回错误。
    pub fn next_cell(&mut self) -> Result<Option<XlsxCellEvent>> {
        loop {
            self.buffer.clear();
            match self.reader.read_event_into(&mut self.buffer)? {
                Event::Start(element) if element.local_name().as_ref() == b"row" => {
                    self.row_index = worksheet_row_index(&element, self.row_index)?;
                    self.column_index = 0;
                }
                Event::Start(element) if element.local_name().as_ref() == b"c" => {
                    let (position, style_index, cell_type) = worksheet_cell_attributes(
                        &element,
                        (self.row_index, self.column_index),
                    )?;
                    let (value, formula, display_value, decimal_value, date_formatted) =
                        self.read_cell(style_index, position.1, cell_type)?;
                    self.row_index = position.0;
                    self.column_index = position.1.saturating_add(1);
                    return Ok(Some(XlsxCellEvent {
                        position,
                        value,
                        formula,
                        display_value,
                        decimal_value,
                        date_formatted,
                    }));
                }
                Event::End(element) if element.local_name().as_ref() == b"row" => {
                    self.row_index = self.row_index.saturating_add(1);
                    self.column_index = 0;
                }
                Event::End(element) if element.local_name().as_ref() == b"sheetData" => {
                    return Ok(None);
                }
                Event::Eof => {
                    return Err(Error::Xlsx(
                        "unexpected end of XML in worksheet data".to_owned(),
                    ));
                }
                _ => {}
            }
        }
    }

    fn read_cell(
        &mut self,
        style_index: usize,
        column_index: usize,
        cell_type: Option<&str>,
    ) -> Result<ParsedCell> {
        self.raw_value.clear();
        self.inline_value.clear();
        self.formula.clear();
        let mut in_value = false;
        let mut in_formula = false;
        let mut in_text = false;
        let mut phonetic_depth = 0_u32;
        loop {
            self.cell_buffer.clear();
            match self.reader.read_event_into(&mut self.cell_buffer)? {
                Event::Start(element) if element.local_name().as_ref() == b"v" => in_value = true,
                Event::Start(element) if element.local_name().as_ref() == b"f" => in_formula = true,
                Event::Start(element) if element.local_name().as_ref() == b"rPh" => {
                    phonetic_depth = phonetic_depth.saturating_add(1);
                }
                Event::Start(element)
                    if phonetic_depth == 0 && element.local_name().as_ref() == b"t" =>
                {
                    in_text = true;
                }
                Event::Text(value) => {
                    let text = value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?;
                    append_cell_text(
                        &text,
                        in_value,
                        in_formula,
                        in_text,
                        &mut self.raw_value,
                        &mut self.formula,
                        &mut self.inline_value,
                    );
                }
                Event::CData(value) => {
                    let text = value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?;
                    append_cell_text(
                        &text,
                        in_value,
                        in_formula,
                        in_text,
                        &mut self.raw_value,
                        &mut self.formula,
                        &mut self.inline_value,
                    );
                }
                Event::End(element) if element.local_name().as_ref() == b"v" => in_value = false,
                Event::End(element) if element.local_name().as_ref() == b"f" => in_formula = false,
                Event::End(element) if element.local_name().as_ref() == b"t" => in_text = false,
                Event::End(element) if element.local_name().as_ref() == b"rPh" => {
                    phonetic_depth = phonetic_depth.saturating_sub(1);
                }
                Event::End(element) if element.local_name().as_ref() == b"c" => {
                    // 公式缓冲在下一单元格开始时本就会清空；直接转移其所有权，
                    // 避免每个公式单元格复制一次 String。无公式时保留已分配容量
                    // 供后续单元格复用。
                    let formula = if self.formula.is_empty() {
                        None
                    } else {
                        Some(std::mem::take(&mut self.formula))
                    };
                    return self.finish_cell(
                        style_index,
                        column_index,
                        cell_type,
                        &self.raw_value,
                        &self.inline_value,
                        formula,
                    );
                }
                Event::Eof => {
                    return Err(Error::Xlsx(
                        "unexpected end of XML in worksheet cell".to_owned(),
                    ));
                }
                _ => {}
            }
        }
    }

    fn finish_cell(
        &self,
        style_index: usize,
        column_index: usize,
        cell_type: Option<&str>,
        raw_value: &str,
        inline_value: &str,
        formula: Option<String>,
    ) -> Result<ParsedCell> {
        let number = if matches!(cell_type, Some("n") | None) && !raw_value.is_empty() {
            let number = excel_display_number(raw_value.parse::<f64>().map_err(xlsx_error)?);
            if !number.is_finite() {
                return Err(Error::Xlsx("non-finite XLSX numeric cell value".to_owned()));
            }
            Some(number)
        } else {
            None
        };
        let value = match cell_type {
            Some("s") => {
                if raw_value.is_empty() {
                    XlsxCellValue::Empty
                } else {
                    let index = parse_xlsx_index(raw_value, "shared string")?;
                    XlsxCellValue::String(self.shared_strings.get(index)?)
                }
            }
            Some("inlineStr" | "str") => {
                XlsxCellValue::String(decode_ooxml_escape(if inline_value.is_empty() {
                    raw_value
                } else {
                    inline_value
                }))
            }
            Some("b") => XlsxCellValue::Bool(matches!(raw_value, "1" | "true")),
            Some("e") => XlsxCellValue::Error(raw_value.to_owned()),
            Some("d") => XlsxCellValue::String(raw_value.to_owned()),
            Some("n") | None => number.map_or(XlsxCellValue::Empty, XlsxCellValue::Number),
            Some(other) => {
                return Err(Error::Xlsx(format!("unsupported XLSX cell type: {other}")));
            }
        };
        let format = self.cell_formats.get(style_index);
        let compiled_format = self
            .compiled_cell_formats
            .get(style_index)
            .and_then(Option::as_ref);
        let date_formatted = number.is_some()
            && compiled_format.is_some_and(CompiledExcelFormat::is_date_format);
        let (display_value, decimal_value) = number.map_or((None, None), |number| {
            // `retain_decimal_values` 为 true 时才构造 BigDecimal；高层 API
            // `EasyExcel::read` 会按 schema 自动覆盖此值（见 `requires_decimal_metadata`），
            // 低层 API 用户按需显式传 false 避免逐格构造。
            let decimal = if self.options.retain_decimal_values {
                #[cold]
                #[inline(never)]
                fn parse_big_decimal(number: f64) -> Option<BigDecimal> {
                    number.to_string().parse::<BigDecimal>().ok()
                }
                parse_big_decimal(number)
            } else {
                None
            };
            let retain_display = self
                .options
                .retain_display_columns
                .as_ref()
                .is_none_or(|columns| columns.contains(&column_index));
            let display = retain_display.then(|| {
                format.and_then(|format| {
                    format.display_compiled(
                        compiled_format,
                        number,
                        self.options.date_1904,
                        self.options.use_scientific_format,
                        &self.options.locale,
                    )
                })
            }).flatten();
            (display, decimal)
        });
        Ok((value, formula, display_value, decimal_value, date_formatted))
    }
}

fn worksheet_row_index(element: &BytesStart<'_>, fallback: u32) -> Result<u32> {
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.map_err(xlsx_error)?;
        if attribute.key.local_name().as_ref() == b"r" {
            let value = std::str::from_utf8(attribute.value.as_ref()).map_err(xlsx_error)?;
            return parse_row_number(value);
        }
    }
    Ok(fallback)
}

fn worksheet_cell_attributes(
    element: &BytesStart<'_>,
    fallback: (u32, usize),
) -> Result<((u32, usize), usize, Option<&'static str>)> {
    let mut position = fallback;
    let mut style_index = 0;
    let mut cell_type = None;
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.map_err(xlsx_error)?;
        let key = attribute.key.local_name();
        if !matches!(key.as_ref(), b"r" | b"s" | b"t") {
            continue;
        }
        let value = std::str::from_utf8(attribute.value.as_ref()).map_err(xlsx_error)?;
        match key.as_ref() {
            b"r" => position = parse_a1_cell_reference(value)?,
            b"s" if !value.is_empty() => style_index = parse_xlsx_index(value, "style")?,
            b"t" => {
                cell_type = Some(match value {
                    "s" => "s",
                    "inlineStr" => "inlineStr",
                    "str" => "str",
                    "b" => "b",
                    "e" => "e",
                    "d" => "d",
                    "n" => "n",
                    other => {
                        return Err(Error::Xlsx(format!(
                            "unsupported XLSX cell type: {other}"
                        )));
                    }
                });
            }
            _ => {}
        }
    }
    Ok((position, style_index, cell_type))
}
