/// 对应 Java：无直接对应对象；Rust 架构扩展。 单个工作表的拉取式单元格事件读取器。
pub struct XlsxCellEventReader<'a> {
    reader: XmlReader<Box<dyn BufRead + 'a>>,
    cell_formats: &'a [XlsxNumberFormat],
    options: XlsxDisplayOptions,
    row_index: u32,
    column_index: usize,
    buffer: Vec<u8>,
    cell_buffer: Vec<u8>,
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
            options,
            row_index: 0,
            column_index: 0,
            buffer,
            cell_buffer: Vec::with_capacity(256),
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
                    let values = attributes(&element, self.reader.decoder())?;
                    self.row_index = values
                        .get("r")
                        .map_or(Ok(self.row_index), |value| parse_row_number(value))?;
                    self.column_index = 0;
                }
                Event::Start(element) if element.local_name().as_ref() == b"c" => {
                    let values = attributes(&element, self.reader.decoder())?;
                    let position = values
                        .get("r")
                        .map_or(Ok((self.row_index, self.column_index)), |reference| {
                            parse_a1_cell_reference(reference)
                        })?;
                    let style_index = values
                        .get("s")
                        .filter(|value| !value.is_empty())
                        .map(|value| parse_xlsx_index(value, "style"))
                        .transpose()?
                        .unwrap_or_default();
                    let cell_type = values.get("t").map(String::as_str);
                    let (value, formula, display_value, decimal_value, date_formatted) =
                        self.read_cell(style_index, cell_type)?;
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

    fn read_cell(&mut self, style_index: usize, cell_type: Option<&str>) -> Result<ParsedCell> {
        let mut raw_value = String::new();
        let mut inline_value = String::new();
        let mut formula = String::new();
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
                        &mut raw_value,
                        &mut formula,
                        &mut inline_value,
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
                        &mut raw_value,
                        &mut formula,
                        &mut inline_value,
                    );
                }
                Event::End(element) if element.local_name().as_ref() == b"v" => in_value = false,
                Event::End(element) if element.local_name().as_ref() == b"f" => in_formula = false,
                Event::End(element) if element.local_name().as_ref() == b"t" => in_text = false,
                Event::End(element) if element.local_name().as_ref() == b"rPh" => {
                    phonetic_depth = phonetic_depth.saturating_sub(1);
                }
                Event::End(element) if element.local_name().as_ref() == b"c" => {
                    let formula = (!formula.is_empty()).then_some(formula);
                    return self.finish_cell(
                        style_index,
                        cell_type,
                        &raw_value,
                        &inline_value,
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
        let date_formatted =
            number.is_some() && format.is_some_and(XlsxNumberFormat::is_date_format);
        let (display_value, decimal_value) = number.map_or((None, None), |number| {
            let decimal = number.to_string().parse::<BigDecimal>().ok();
            let display = format.and_then(|format| {
                format.display(
                    number,
                    self.options.date_1904,
                    self.options.use_scientific_format,
                    &self.options.locale,
                )
            });
            (display, decimal)
        });
        Ok((value, formula, display_value, decimal_value, date_formatted))
    }
}

