include!("readseek_to_read_comments/read_seek.rs");



include!("readseek_to_read_comments/xlsx_number_format.rs");



include!("readseek_to_read_comments/xlsx_cell_value.rs");

include!("readseek_to_read_comments/xlsx_cell_event.rs");

include!("readseek_to_read_comments/xlsx_display_options.rs");

include!("readseek_to_read_comments/xlsx_extra_kind.rs");

include!("readseek_to_read_comments/xlsx_extra.rs");

include!("readseek_to_read_comments/xlsx_event_metadata.rs");



include!("readseek_to_read_comments/xlsx_cell_event_reader.rs");

type ParsedCell = (
    XlsxCellValue,
    Option<String>,
    Option<String>,
    Option<BigDecimal>,
    bool,
);



fn append_cell_text(
    text: &str,
    in_value: bool,
    in_formula: bool,
    in_text: bool,
    raw_value: &mut String,
    formula: &mut String,
    inline_value: &mut String,
) {
    if in_value {
        raw_value.push_str(text);
    } else if in_formula {
        formula.push_str(text);
    } else if in_text {
        inline_value.push_str(text);
    }
}

fn read_worksheet_extras<R: Read + Seek>(
    package: &mut XlsxPackageReader<R>,
    sheet_path: &str,
    relationships: &RawRelationships,
    enabled: &HashSet<XlsxExtraKind>,
) -> Result<Vec<XlsxExtra>> {
    let file = package.open_part(sheet_path)?;
    parse_worksheet_extras(
        &mut BufReader::new(file),
        sheet_path,
        relationships,
        enabled,
    )
}

fn parse_worksheet_extras(
    input: &mut dyn BufRead,
    sheet_path: &str,
    relationships: &RawRelationships,
    enabled: &HashSet<XlsxExtraKind>,
) -> Result<Vec<XlsxExtra>> {
    let mut reader = configured_xml_reader(input);
    let mut extras = Vec::new();
    let mut buffer = Vec::with_capacity(256);
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"mergeCell" => {
                if enabled.contains(&XlsxExtraKind::Merge) {
                    let values = attributes(&element, reader.decoder())?;
                    let reference = required_attribute(&values, "ref", "mergeCell")?;
                    let (first_row, last_row, first_column, last_column) =
                        parse_a1_cell_range(reference)?;
                    extras.push(XlsxExtra {
                        kind: XlsxExtraKind::Merge,
                        text: None,
                        first_row,
                        last_row,
                        first_column,
                        last_column,
                    });
                }
            }
            Event::Start(element) if element.local_name().as_ref() == b"hyperlink" => {
                if enabled.contains(&XlsxExtraKind::Hyperlink) {
                    let values = attributes(&element, reader.decoder())?;
                    let reference = required_attribute(&values, "ref", "hyperlink")?;
                    let target = if let Some(location) = values.get("location") {
                        location.clone()
                    } else {
                        let relationship_id = required_attribute(&values, "id", "hyperlink")?;
                        relationships
                            .get(relationship_id)
                            .filter(|(_, relationship_type, _)| {
                                relationship_type.ends_with("/hyperlink")
                            })
                            .map(|(target, _, _)| target.clone())
                            .ok_or_else(|| {
                                Error::Xlsx(format!(
                                    "hyperlink relationship not found: {relationship_id}"
                                ))
                            })?
                    };
                    let (first_row, last_row, first_column, last_column) =
                        parse_a1_cell_range(reference)?;
                    extras.push(XlsxExtra {
                        kind: XlsxExtraKind::Hyperlink,
                        text: Some(target),
                        first_row,
                        last_row,
                        first_column,
                        last_column,
                    });
                }
            }
            Event::End(element) if element.local_name().as_ref() == b"worksheet" => break,
            Event::Eof => {
                return Err(Error::Xlsx(format!(
                    "unexpected end of XML in {sheet_path}"
                )));
            }
            _ => {}
        }
    }
    Ok(extras)
}

fn read_comments<R: Read + Seek>(
    package: &mut XlsxPackageReader<R>,
    comments_path: &str,
) -> Result<Vec<XlsxExtra>> {
    let file = package.open_part(comments_path)?;
    parse_comments(&mut BufReader::new(file), comments_path)
}

