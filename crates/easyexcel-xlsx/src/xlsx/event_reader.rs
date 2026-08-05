//! 面向上层框架的 XLSX 单元格事件读取引擎。
//!
//! 本模块拥有 OOXML 包关系、XML 事件、共享字符串、样式格式和工作表附加
//! 信息解析。上层门面只需把中立事件映射到自己的 metadata/listener 类型。

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Seek};

use bigdecimal::BigDecimal;
use easyexcel_cache::{
    ReadCacheMode, SharedStringCache, SharedStringCacheReader, create_cache, memory_cache,
};
use easyexcel_format::{
    SpreadsheetLocale, excel_display_number, format_with_code, is_date_format_code,
    is_scientific_magnitude, java_plain_extreme_format, java_scientific_format,
    resolve_builtin_format_code,
};
use easyexcel_io::{Error, Result};
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Decoder, Reader as XmlReader, XmlVersion};

use super::package::{RawRelationships, Relationships, relationship_part_name, resolve_target};
use super::package_reader::XlsxPackageReader;
use super::{
    decode_ooxml_escape, dimension_last_row as parse_dimension_last_row, parse_a1_cell_range,
    parse_a1_cell_reference,
};

/// 可被 XLSX 事件读取器持有的输入流。
pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

/// XLSX 数字格式描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxNumberFormat {
    /// 内建格式编号。
    Builtin(u32),
    /// 自定义格式代码。
    Custom(String),
}

impl XlsxNumberFormat {
    fn code(&self) -> Option<&str> {
        match self {
            Self::Builtin(id) => resolve_builtin_format_code(*id),
            Self::Custom(code) => Some(code.as_str()),
        }
    }

    /// 判断是否为 General 格式。
    #[must_use]
    pub fn is_general(&self) -> bool {
        match self {
            Self::Builtin(id) => *id == 0,
            Self::Custom(code) => code.trim().eq_ignore_ascii_case("general"),
        }
    }

    /// 判断是否为日期或日期时间格式。
    #[must_use]
    pub fn is_date_format(&self) -> bool {
        self.code().is_some_and(is_date_format_code)
    }

    fn display(
        &self,
        value: f64,
        date_1904: bool,
        use_scientific_format: bool,
        locale: &SpreadsheetLocale,
    ) -> Option<String> {
        if self.is_general() && is_scientific_magnitude(value) {
            return Some(if use_scientific_format {
                java_scientific_format(value, locale.decimal_separator)
            } else {
                java_plain_extreme_format(value)
            });
        }
        self.code()
            .and_then(|code| format_with_code(value, code, date_1904, locale))
    }
}

/// 中立的 XLSX 单元格缓存值。
#[derive(Debug, Clone, PartialEq)]
pub enum XlsxCellValue {
    /// 空单元格。
    Empty,
    /// 字符串。
    String(String),
    /// 布尔值。
    Bool(bool),
    /// Excel 错误文本。
    Error(String),
    /// 数字值。
    Number(f64),
}

/// 一个按文档顺序产生的 XLSX 单元格事件。
#[derive(Debug, Clone, PartialEq)]
pub struct XlsxCellEvent {
    /// 零基 `(row, column)` 坐标。
    pub position: (u32, usize),
    /// 缓存值。
    pub value: XlsxCellValue,
    /// 公式文本，不含强制添加的等号。
    pub formula: Option<String>,
    /// 按 Excel 数字格式渲染的显示值。
    pub display_value: Option<String>,
    /// 数字的十进制表示。
    pub decimal_value: Option<BigDecimal>,
    /// 当前样式是否是日期格式。
    pub date_formatted: bool,
}

/// 单元格显示配置。
#[derive(Debug, Clone)]
pub struct XlsxDisplayOptions {
    /// 是否按 1904 日期系统解释序列值。
    pub date_1904: bool,
    /// General 极值是否使用科学计数法。
    pub use_scientific_format: bool,
    /// 数字和日期显示区域设置。
    pub locale: SpreadsheetLocale,
}

impl Default for XlsxDisplayOptions {
    fn default() -> Self {
        Self {
            date_1904: false,
            use_scientific_format: false,
            locale: SpreadsheetLocale::default(),
        }
    }
}

/// 工作表附加信息种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XlsxExtraKind {
    /// 合并区域。
    Merge,
    /// 超链接。
    Hyperlink,
    /// 批注。
    Comment,
}

/// 中立的工作表附加信息事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XlsxExtra {
    /// 附加信息种类。
    pub kind: XlsxExtraKind,
    /// 超链接目标或批注文本；合并区域为空。
    pub text: Option<String>,
    /// 起始行。
    pub first_row: u32,
    /// 结束行。
    pub last_row: u32,
    /// 起始列。
    pub first_column: usize,
    /// 结束列。
    pub last_column: usize,
}

/// 工作簿级事件读取元数据与 OOXML 包句柄。
pub struct XlsxEventMetadata<R: Read + Seek> {
    package: XlsxPackageReader<R>,
    sheet_paths: HashMap<String, String>,
    sheet_names: Vec<String>,
    cell_formats: Vec<XlsxNumberFormat>,
    shared_strings: Box<dyn SharedStringCacheReader>,
}

impl<R: Read + Seek> XlsxEventMetadata<R> {
    /// 使用指定缓存模式打开 XLSX 包。
    ///
    /// # Errors
    ///
    /// 包关系、工作簿、样式或共享字符串无效时返回错误。
    pub fn new(input: R, cache_mode: ReadCacheMode) -> Result<Self> {
        Self::new_with_cache_factory(input, |xml_size| create_cache(cache_mode, xml_size))
    }

    /// 使用调用方提供的共享字符串缓存工厂打开 XLSX 包。
    ///
    /// 工厂接收 `sharedStrings.xml` 的未压缩大小。
    ///
    /// # Errors
    ///
    /// 包关系、工作簿、样式、共享字符串或缓存初始化失败时返回错误。
    pub fn new_with_cache_factory<F>(input: R, mut cache_factory: F) -> Result<Self>
    where
        F: FnMut(u64) -> Result<Box<dyn SharedStringCache>>,
    {
        let mut package = XlsxPackageReader::new(input)?;
        let package_relationships = package.relationships("_rels/.rels")?;
        let workbook_target = package_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/officeDocument"))
            .map(|(target, _)| target)
            .ok_or_else(|| Error::Xlsx("officeDocument relationship not found".to_owned()))?;
        let workbook_path = resolve_target("", workbook_target)?;
        let workbook_relationships_path = relationship_part_name(&workbook_path);
        let workbook_relationships = package.relationships(&workbook_relationships_path)?;
        let (sheets, _) = read_workbook_metadata(
            &mut package,
            &workbook_path,
            &workbook_relationships,
        )?;
        let sheet_names = sheets.iter().map(|(name, _)| name.clone()).collect();
        let sheet_paths = sheets.into_iter().collect::<HashMap<_, _>>();
        let cell_formats = workbook_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/styles"))
            .map(|(target, _)| resolve_target(&workbook_path, target))
            .transpose()?
            .map(|styles_path| read_cell_formats(&mut package, &styles_path))
            .transpose()?
            .unwrap_or_else(|| vec![XlsxNumberFormat::Builtin(0)]);
        let shared_strings_path = workbook_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/sharedStrings"))
            .map(|(target, _)| resolve_target(&workbook_path, target))
            .transpose()?;
        let shared_strings = match shared_strings_path {
            Some(path) => read_shared_strings(&mut package, &path, &mut cache_factory)?,
            None => memory_cache(),
        };
        for path in sheet_paths.values() {
            if !package.contains(path) {
                return Err(Error::Xlsx(format!("worksheet part not found: {path}")));
            }
        }
        Ok(Self {
            package,
            sheet_paths,
            sheet_names,
            cell_formats,
            shared_strings,
        })
    }

    /// 返回工作表名称，顺序与工作簿一致。
    #[must_use]
    pub fn sheet_names(&self) -> &[String] {
        &self.sheet_names
    }

    /// 打开一个工作表的单元格事件游标。
    ///
    /// # Errors
    ///
    /// 工作表不存在或 XML 无效时返回错误。
    pub fn cells(
        &mut self,
        sheet_name: &str,
        options: XlsxDisplayOptions,
    ) -> Result<XlsxCellEventReader<'_>> {
        let path = self
            .sheet_paths
            .get(sheet_name)
            .cloned()
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        let file = self.package.open_part(&path)?;
        let reader = boxed_xml_reader(BufReader::new(file));
        XlsxCellEventReader::new(
            reader,
            &self.cell_formats,
            options,
            self.shared_strings.as_ref(),
        )
    }

    /// 扫描工作表最后一个显式行号。
    ///
    /// # Errors
    ///
    /// 工作表不存在或 XML 无效时返回错误。
    pub fn last_explicit_row(&mut self, sheet_name: &str) -> Result<Option<u32>> {
        let path = self
            .sheet_paths
            .get(sheet_name)
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        let file = self.package.open_part(path)?;
        scan_last_row(BufReader::new(file))
    }

    /// 读取合并区域、超链接和批注。
    ///
    /// # Errors
    ///
    /// 工作表、关系或附加 XML 无效时返回错误。
    pub fn extras(
        &mut self,
        sheet_name: &str,
        enabled: &HashSet<XlsxExtraKind>,
    ) -> Result<Vec<XlsxExtra>> {
        let sheet_path = self
            .sheet_paths
            .get(sheet_name)
            .cloned()
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        let relationships_path = relationship_part_name(&sheet_path);
        let relationships = if self.package.contains(&relationships_path) {
            self.package.raw_relationships(&relationships_path)?
        } else {
            RawRelationships::new()
        };
        let mut extras = read_worksheet_extras(
            &mut self.package,
            &sheet_path,
            &relationships,
            enabled,
        )?;
        if enabled.contains(&XlsxExtraKind::Comment)
            && let Some((target, _, false)) = relationships
                .values()
                .find(|(_, relationship_type, _)| relationship_type.ends_with("/comments"))
        {
            let comments_path = resolve_target(&sheet_path, target)?;
            extras.extend(read_comments(&mut self.package, &comments_path)?);
        }
        Ok(extras)
    }
}

/// 单个工作表的拉取式单元格事件读取器。
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

type ParsedCell = (
    XlsxCellValue,
    Option<String>,
    Option<String>,
    Option<BigDecimal>,
    bool,
);

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

    /// 读取下一个单元格事件。
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
                    let position = values.get("r").map_or(
                        Ok((self.row_index, self.column_index)),
                        |reference| parse_a1_cell_reference(reference),
                    )?;
                    let style_index = values
                        .get("s")
                        .filter(|value| !value.is_empty())
                        .map(|value| value.parse::<usize>().map_err(xlsx_error))
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
                    let text = value.xml_content(XmlVersion::Implicit1_0).map_err(xlsx_error)?;
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
                    let text = value.xml_content(XmlVersion::Implicit1_0).map_err(xlsx_error)?;
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
                return Err(Error::Xlsx(
                    "non-finite XLSX numeric cell value".to_owned(),
                ));
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
                    let index = raw_value.parse::<usize>().map_err(xlsx_error)?;
                    XlsxCellValue::String(self.shared_strings.get(index)?)
                }
            }
            Some("inlineStr" | "str") => XlsxCellValue::String(decode_ooxml_escape(if inline_value.is_empty() {
                raw_value
            } else {
                inline_value
            })),
            Some("b") => XlsxCellValue::Bool(matches!(raw_value, "1" | "true")),
            Some("e") => XlsxCellValue::Error(raw_value.to_owned()),
            Some("d") => XlsxCellValue::String(raw_value.to_owned()),
            Some("n") | None => number.map_or(XlsxCellValue::Empty, XlsxCellValue::Number),
            Some(other) => {
                return Err(Error::Xlsx(format!(
                    "unsupported XLSX cell type: {other}"
                )));
            }
        };
        let format = self.cell_formats.get(style_index);
        let date_formatted = number.is_some() && format.is_some_and(XlsxNumberFormat::is_date_format);
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
                    let replacement = resolve_predefined_entity(&name).ok_or_else(|| {
                        Error::Xlsx(format!("unrecognized XML entity: {name}"))
                    })?;
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
                return Err(Error::Xlsx(
                    "unexpected end of XML in styles".to_owned(),
                ));
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

fn parse_shared_strings(
    input: &mut dyn BufRead,
    cache: &mut dyn SharedStringCache,
) -> Result<()> {
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
                let (target, relationship_type) = relationships
                    .get(relationship_id)
                    .ok_or_else(|| {
                        Error::Xlsx(format!(
                            "sheet relationship not found: {relationship_id}"
                        ))
                    })?;
                if relationship_type.ends_with("/worksheet") {
                    sheets.push((name.clone(), resolve_target(workbook_path, target)?));
                }
            }
            Event::End(element) if element.local_name().as_ref() == b"workbook" => break,
            Event::Eof => {
                return Err(Error::Xlsx(
                    "unexpected end of XML in workbook".to_owned(),
                ));
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
                return Err(Error::Xlsx(
                    "unexpected end of XML in worksheet".to_owned(),
                ));
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
