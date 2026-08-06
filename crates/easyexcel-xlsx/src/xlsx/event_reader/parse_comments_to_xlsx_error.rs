fn parse_comments(input: &mut dyn BufRead, comments_path: &str) -> Result<Vec<XlsxExtra>> {
    let mut reader = configured_xml_reader(input);
    let mut extras = Vec::new();
    let mut buffer = Vec::with_capacity(256);
    let mut current = None;
    let mut text = String::new();
    let mut in_text_run = false;
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"comment" => {
                let values = attributes(&element, reader.decoder())?;
                let reference = required_attribute(&values, "ref", "comment")?;
                current = Some(parse_a1_cell_range(reference)?);
                text.clear();
            }
            Event::Start(element) if current.is_some() && element.local_name().as_ref() == b"t" => {
                in_text_run = true;
            }
            Event::Text(value) if in_text_run => {
                text.push_str(
                    &value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?,
                );
            }
            Event::CData(value) if in_text_run => {
                text.push_str(
                    &value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?,
                );
            }
            Event::GeneralRef(value) if in_text_run => {
                if let Some(character) = value.resolve_char_ref().map_err(xlsx_error)? {
                    text.push(character);
                } else {
                    let name = String::from_utf8_lossy(value.as_ref());
                    let replacement = resolve_predefined_entity(&name)
                        .ok_or_else(|| Error::Xlsx(format!("unrecognized XML entity: {name}")))?;
                    text.push_str(replacement);
                }
            }
            Event::End(element) if element.local_name().as_ref() == b"t" => {
                in_text_run = false;
            }
            Event::End(element) if element.local_name().as_ref() == b"comment" => {
                let (first_row, last_row, first_column, last_column) = current
                    .take()
                    .ok_or_else(|| Error::Xlsx("comment start is missing".to_owned()))?;
                extras.push(XlsxExtra {
                    kind: XlsxExtraKind::Comment,
                    text: Some(text.clone()),
                    first_row,
                    last_row,
                    first_column,
                    last_column,
                });
            }
            Event::End(element) if element.local_name().as_ref() == b"comments" => break,
            Event::Eof => {
                return Err(Error::Xlsx(format!(
                    "unexpected end of XML in {comments_path}"
                )));
            }
            _ => {}
        }
    }
    Ok(extras)
}

fn read_cell_formats<R: Read + Seek>(
    package: &mut XlsxPackageReader<R>,
    styles_path: &str,
) -> Result<Vec<XlsxNumberFormat>> {
    let mut reader = xml_reader(package, styles_path)?;
    let mut custom_formats = HashMap::new();
    let mut cell_formats = Vec::new();
    let mut in_cell_formats = false;
    let mut buffer = Vec::with_capacity(256);
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"numFmt" => {
                let values = attributes(&element, reader.decoder())?;
                if let (Some(id), Some(code)) = (values.get("numFmtId"), values.get("formatCode")) {
                    custom_formats.insert(id.parse::<u32>().map_err(xlsx_error)?, code.clone());
                }
            }
            Event::Start(element) if element.local_name().as_ref() == b"cellXfs" => {
                in_cell_formats = true;
            }
            Event::Start(element) if in_cell_formats && element.local_name().as_ref() == b"xf" => {
                let values = attributes(&element, reader.decoder())?;
                let id = values
                    .get("numFmtId")
                    .map(|value| value.parse::<u32>().map_err(xlsx_error))
                    .transpose()?
                    .unwrap_or_default();
                cell_formats.push(custom_formats.get(&id).map_or_else(
                    || XlsxNumberFormat::Builtin(id),
                    |code| XlsxNumberFormat::Custom(code.clone()),
                ));
            }
            Event::End(element) if element.local_name().as_ref() == b"cellXfs" => {
                in_cell_formats = false;
            }
            Event::End(element) if element.local_name().as_ref() == b"styleSheet" => break,
            Event::Eof => {
                return Err(Error::Xlsx("unexpected end of XML in styles".to_owned()));
            }
            _ => {}
        }
    }
    if cell_formats.is_empty() {
        cell_formats.push(XlsxNumberFormat::Builtin(0));
    }
    Ok(cell_formats)
}

fn read_shared_strings<R, F>(
    package: &mut XlsxPackageReader<R>,
    path: &str,
    cache_factory: &mut F,
) -> Result<Box<dyn SharedStringCacheReader>>
where
    R: Read + Seek,
    F: FnMut(u64) -> Result<Box<dyn SharedStringCache>>,
{
    let xml_size = package.part_size(path)?;
    let mut cache = cache_factory(xml_size)?;
    let file = package.open_part(path)?;
    parse_shared_strings(&mut BufReader::new(file), cache.as_mut())?;
    cache.finish()
}

fn parse_shared_strings(input: &mut dyn BufRead, cache: &mut dyn SharedStringCache) -> Result<()> {
    let mut reader = XmlReader::from_reader(input);
    reader.config_mut().expand_empty_elements = true;
    let mut buffer = Vec::with_capacity(256);
    let mut current = String::new();
    let mut in_si = false;
    let mut in_text = false;
    let mut phonetic_depth = 0_u32;
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"si" => {
                current.clear();
                in_si = true;
            }
            Event::Start(element) if in_si && element.local_name().as_ref() == b"rPh" => {
                phonetic_depth = phonetic_depth.saturating_add(1);
            }
            Event::Start(element)
                if in_si && phonetic_depth == 0 && element.local_name().as_ref() == b"t" =>
            {
                in_text = true;
            }
            Event::Text(value) if in_text => {
                current.push_str(
                    &value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?,
                );
            }
            Event::CData(value) if in_text => {
                current.push_str(
                    &value
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(xlsx_error)?,
                );
            }
            Event::End(element) if element.local_name().as_ref() == b"t" => in_text = false,
            Event::End(element) if element.local_name().as_ref() == b"rPh" => {
                phonetic_depth = phonetic_depth.saturating_sub(1);
            }
            Event::End(element) if element.local_name().as_ref() == b"si" => {
                in_si = false;
                cache.put(decode_ooxml_escape(&current))?;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

fn read_workbook_metadata<R: Read + Seek>(
    package: &mut XlsxPackageReader<R>,
    workbook_path: &str,
    relationships: &Relationships,
) -> Result<(Vec<(String, String)>, bool)> {
    let mut reader = xml_reader(package, workbook_path)?;
    let mut sheets = Vec::new();
    let mut date_1904 = false;
    let mut buffer = Vec::with_capacity(256);
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"workbookPr" => {
                let values = attributes(&element, reader.decoder())?;
                date_1904 = values
                    .get("date1904")
                    .is_some_and(|value| matches!(value.as_str(), "1" | "true"));
            }
            Event::Start(element) if element.local_name().as_ref() == b"sheet" => {
                let values = attributes(&element, reader.decoder())?;
                let name = values
                    .get("name")
                    .ok_or_else(|| Error::Xlsx("sheet name is missing".to_owned()))?;
                let relationship_id = values
                    .get("id")
                    .ok_or_else(|| Error::Xlsx("sheet relationship is missing".to_owned()))?;
                let (target, relationship_type) =
                    relationships.get(relationship_id).ok_or_else(|| {
                        Error::Xlsx(format!("sheet relationship not found: {relationship_id}"))
                    })?;
                if relationship_type.ends_with("/worksheet") {
                    sheets.push((name.clone(), resolve_target(workbook_path, target)?));
                }
            }
            Event::End(element) if element.local_name().as_ref() == b"workbook" => break,
            Event::Eof => {
                return Err(Error::Xlsx("unexpected end of XML in workbook".to_owned()));
            }
            _ => {}
        }
    }
    Ok((sheets, date_1904))
}

fn scan_last_row<R: BufRead>(input: R) -> Result<Option<u32>> {
    let mut reader = XmlReader::from_reader(input);
    reader.config_mut().expand_empty_elements = true;
    let mut buffer = Vec::with_capacity(256);
    let mut in_sheet_data = false;
    let mut current_row = 0_u32;
    let mut last_row = None;
    let mut dimension_last_row = None;
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"dimension" => {
                let values = attributes(&element, reader.decoder())?;
                if let Some(reference) = values.get("ref") {
                    dimension_last_row = Some(parse_dimension_last_row(reference)?);
                }
            }
            Event::Start(element) if element.local_name().as_ref() == b"sheetData" => {
                in_sheet_data = true;
            }
            Event::Start(element) if in_sheet_data && element.local_name().as_ref() == b"row" => {
                let row = attributes(&element, reader.decoder())?
                    .get("r")
                    .map_or(Ok(current_row), |value| parse_row_number(value))?;
                current_row = row;
                last_row = Some(row);
            }
            Event::End(element) if element.local_name().as_ref() == b"row" => {
                current_row = current_row.saturating_add(1);
            }
            Event::End(element) if element.local_name().as_ref() == b"sheetData" => {
                return Ok(last_row.or(dimension_last_row));
            }
            Event::Eof => {
                return Err(Error::Xlsx("unexpected end of XML in worksheet".to_owned()));
            }
            _ => {}
        }
    }
}

fn attributes(element: &BytesStart<'_>, decoder: Decoder) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.map_err(xlsx_error)?;
        let key = std::str::from_utf8(attribute.key.local_name().as_ref())
            .map_err(xlsx_error)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, decoder)
            .map_err(xlsx_error)?
            .into_owned();
        values.insert(key, value);
    }
    Ok(values)
}

fn required_attribute<'a>(
    attributes: &'a HashMap<String, String>,
    name: &str,
    element: &str,
) -> Result<&'a str> {
    attributes
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| Error::Xlsx(format!("{element} {name} is missing")))
}

fn xml_reader<'a, R: Read + Seek>(
    package: &'a mut XlsxPackageReader<R>,
    path: &str,
) -> Result<XmlReader<BufReader<Box<dyn Read + 'a>>>> {
    let file = package.open_part(path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(file));
    configure_xml(&mut reader);
    Ok(reader)
}

fn boxed_xml_reader<'a>(input: impl BufRead + 'a) -> XmlReader<Box<dyn BufRead + 'a>> {
    let mut reader = XmlReader::from_reader(Box::new(input) as Box<dyn BufRead + 'a>);
    configure_xml(&mut reader);
    reader
}

fn configured_xml_reader(input: &mut dyn BufRead) -> XmlReader<&mut dyn BufRead> {
    let mut reader = XmlReader::from_reader(input);
    configure_xml(&mut reader);
    reader
}

fn configure_xml<R: BufRead>(reader: &mut XmlReader<R>) {
    let config = reader.config_mut();
    config.check_end_names = false;
    config.check_comments = false;
    config.expand_empty_elements = true;
}

fn xlsx_error(error: impl std::fmt::Display) -> Error {
    Error::Xlsx(error.to_string())
}
