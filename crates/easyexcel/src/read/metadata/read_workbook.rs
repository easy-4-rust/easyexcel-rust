//! 对应 Java：`com.alibaba.excel.read.metadata.ReadWorkbook`.
//!
//! Java signature: 47 members (18 fields × 3 each for get/set/equals,
//! equals/hashCode and 5 ctor overloads). The Rust port stores the
//! configuration in [`crate::ReadOptions`] and exposes a 1:1 named
//! wrapper struct for callers that mirror the Java shape.
//!
//! Fields not present in [`crate::ReadOptions`] (POJO `InputStream`,
//! `File`, `ReadCache`/`ReadCacheSelector` raw types) are exposed as
//! typed accessors that return the underlying engine handles when
//! they are available.

use std::path::{Path, PathBuf};

use crate::ReadOptions;

/// 对应 Java：`ReadWorkbook extends ReadBasicParameter`.
///
/// The Java side carries 18 fields (file, outputStream, charset,
/// mandatoryUseInputStream, autoCloseStream, customObject, etc.).
/// Rust reuses [`ReadOptions`] as the backing config and exposes
/// the Java-shaped getters/setters as thin pass-throughs.
#[derive(Debug, Clone)]
pub struct ReadWorkbook {
    /// Java 父类 `ReadBasicParameter`。
    parameter: crate::read::metadata::ReadBasicParameter,
    /// Input workbook path. (Java `ReadWorkbook.file`)
    file: Option<PathBuf>,
    /// Backing configuration. (Java `ReadWorkbook` getter surface)
    pub options: ReadOptions,
    /// Explicit workbook type selected by the caller.
    excel_type: Option<crate::support::ExcelTypeEnum>,
    /// Whether an owned input is closed after analysis.
    auto_close_stream: bool,
    /// 后端中立输入流字节。
    input_stream: Option<Vec<u8>>,
    /// Java nullable 配置覆盖。
    auto_close_stream_override: Option<bool>,
    ignore_empty_row_override: Option<bool>,
    mandatory_use_input_stream: Option<bool>,
    use_default_listener: Option<bool>,
    xlsx_sax_parser_factory_name: Option<String>,
}

impl ReadWorkbook {
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Creates a `ReadWorkbook` with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameter: crate::read::metadata::ReadBasicParameter::new(),
            file: None,
            options: ReadOptions::default(),
            excel_type: None,
            auto_close_stream: true,
            input_stream: None,
            auto_close_stream_override: None,
            ignore_empty_row_override: None,
            mandatory_use_input_stream: None,
            use_default_listener: Some(true),
            xlsx_sax_parser_factory_name: None,
        }
    }

    /// 返回输入工作簿文件。
    ///
    /// 对应 Java：`ReadWorkbook#getFile()`。
    #[must_use]
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// 设置输入工作簿文件。
    ///
    /// 对应 Java：`ReadWorkbook#setFile(File)`。
    pub fn set_file(&mut self, file: impl Into<PathBuf>) -> &mut Self {
        self.file = Some(file.into());
        self
    }

    /// Returns the Excel file type. (Java `getExcelType()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn excel_type(&self) -> Option<crate::support::ExcelTypeEnum> {
        self.excel_type
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the Excel file type. (Java `setExcelType(ExcelTypeEnum)`)
    pub fn set_excel_type(&mut self, excel_type: crate::support::ExcelTypeEnum) -> &mut Self {
        self.excel_type = Some(excel_type);
        self
    }

    /// Returns the ignore-empty-row flag. (Java `getIgnoreEmptyRow()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn ignore_empty_row(&self) -> bool {
        self.options.ignore_empty_row
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the ignore-empty-row flag. (Java `setIgnoreEmptyRow(Boolean)`)
    pub fn set_ignore_empty_row(&mut self, value: bool) -> &mut Self {
        self.options.ignore_empty_row = value;
        self.ignore_empty_row_override = Some(value);
        self
    }

    /// Returns the auto-close-stream flag. (Java `getAutoCloseStream()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn auto_close_stream(&self) -> bool {
        self.auto_close_stream
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the auto-close-stream flag. (Java `setAutoCloseStream(Boolean)`)
    /// Path-based readers own their file handle; borrowed stream entrypoints
    /// retain caller ownership independently of this metadata value.
    pub fn set_auto_close_stream(&mut self, value: bool) -> &mut Self {
        self.auto_close_stream = value;
        self.auto_close_stream_override = Some(value);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Returns the custom object. (Java `getCustomObject()`)
    #[must_use]
    pub fn custom_object(&self) -> Option<&crate::CustomReadObject> {
        self.options.custom_object.as_ref()
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the custom object. (Java `setCustomObject(Object)`)
    pub fn set_custom_object(&mut self, custom_object: crate::CustomReadObject) -> &mut Self {
        self.options.custom_object = Some(custom_object);
        self
    }

    /// Returns the charset. (Java `getCharset()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn charset(&self) -> &crate::CsvCharset {
        &self.options.charset
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the charset. (Java `setCharset(Charset)`)
    pub fn set_charset(&mut self, charset: crate::CsvCharset) -> &mut Self {
        self.options.charset = charset;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Returns the password. (Java `getPassword()`)
    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.options.password.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the password. (Java `setPassword(String)`)
    pub fn set_password(&mut self, password: impl Into<String>) -> &mut Self {
        self.options.password = Some(password.into());
        self
    }

    /// Returns the head row number. (Java `getHeadRowNumber()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn head_row_number(&self) -> u32 {
        self.options.head_row_number
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the head row number. (Java `setHeadRowNumber(Integer)`)
    pub fn set_head_row_number(&mut self, value: u32) -> &mut Self {
        self.options.head_row_number = value;
        self.parameter.head_row_number = value;
        self
    }

    /// Returns the read cache mode. (Java `getReadCache()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn read_cache(&self) -> crate::ReadCacheMode {
        self.options.read_cache
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the read cache mode. (Java `setReadCache(ReadCache)`)
    pub fn set_read_cache(&mut self, value: crate::ReadCacheMode) -> &mut Self {
        self.options.read_cache = value;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Returns the read cache selector, if any.
    /// (Java `getReadCacheSelector()`)
    #[must_use]
    pub fn read_cache_selector(&self) -> Option<&crate::StoredReadCacheSelector> {
        self.options.read_cache_selector.as_ref()
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。 Sets the read cache selector. (Java `setReadCacheSelector(ReadCacheSelector)`)
    pub fn set_read_cache_selector(&mut self, value: crate::StoredReadCacheSelector) -> &mut Self {
        self.options.read_cache_selector = Some(value);
        self
    }

    /// Returns the underlying options. (Java `getReadWorkbookHolder()`-style)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadWorkbook。
    pub const fn options(&self) -> &ReadOptions {
        &self.options
    }

    /// 返回 Java 父类参数。
    #[must_use]
    pub const fn get_read_basic_parameter(&self) -> &crate::read::metadata::ReadBasicParameter {
        &self.parameter
    }

    /// 返回可变 Java 父类参数。
    pub const fn get_read_basic_parameter_mut(&mut self) -> &mut crate::read::metadata::ReadBasicParameter {
        &mut self.parameter
    }

    /// Java `getFile` 别名。
    #[must_use]
    pub fn get_file(&self) -> Option<&Path> { self.file() }
    /// Java `getExcelType` 别名。
    #[must_use]
    pub const fn get_excel_type(&self) -> Option<crate::support::ExcelTypeEnum> {
        self.excel_type
    }
    /// Java `getInputStream` 的后端中立字节表示。
    #[must_use]
    pub fn get_input_stream(&self) -> Option<&[u8]> { self.input_stream.as_deref() }
    /// Java `setInputStream`。
    pub fn set_input_stream(&mut self, value: Option<Vec<u8>>) -> &mut Self {
        self.input_stream = value;
        self
    }
    /// Java nullable `getAutoCloseStream`。
    #[must_use]
    pub const fn get_auto_close_stream(&self) -> Option<bool> {
        self.auto_close_stream_override
    }
    /// Java nullable `getIgnoreEmptyRow`。
    #[must_use]
    pub const fn get_ignore_empty_row(&self) -> Option<bool> {
        self.ignore_empty_row_override
    }
    /// Java nullable `getMandatoryUseInputStream`。
    #[must_use]
    pub const fn get_mandatory_use_input_stream(&self) -> Option<bool> {
        self.mandatory_use_input_stream
    }
    /// Java `setMandatoryUseInputStream`。
    pub const fn set_mandatory_use_input_stream(&mut self, value: bool) -> &mut Self {
        self.mandatory_use_input_stream = Some(value);
        self
    }
    /// Java `getCharset` 别名。
    #[must_use]
    pub const fn get_charset(&self) -> &crate::CsvCharset { &self.options.charset }
    /// Java `getCustomObject` 别名。
    #[must_use]
    pub fn get_custom_object(&self) -> Option<&crate::CustomReadObject> {
        self.options.custom_object.as_ref()
    }
    /// Java `getPassword` 别名。
    #[must_use]
    pub fn get_password(&self) -> Option<&str> { self.options.password.as_deref() }
    /// Java `getReadCache` 别名。
    #[must_use]
    pub const fn get_read_cache(&self) -> crate::ReadCacheMode { self.options.read_cache }
    /// Java `getReadCacheSelector` 别名。
    #[must_use]
    pub fn get_read_cache_selector(&self) -> Option<&crate::StoredReadCacheSelector> {
        self.options.read_cache_selector.as_ref()
    }
    /// Java `getReadDefaultReturn`。
    #[must_use]
    pub const fn get_read_default_return(&self) -> crate::ReadDefaultReturn {
        self.options.read_default_return
    }
    /// Java `setReadDefaultReturn`。
    pub const fn set_read_default_return(&mut self, value: crate::ReadDefaultReturn) -> &mut Self {
        self.options.read_default_return = value;
        self
    }
    /// Java `getExtraReadSet`。
    #[must_use]
    pub fn get_extra_read_set(&self) -> &std::collections::HashSet<crate::CellExtraType> {
        &self.options.extra_read
    }
    /// Java `setExtraReadSet`。
    pub fn set_extra_read_set(
        &mut self,
        value: std::collections::HashSet<crate::CellExtraType>,
    ) -> &mut Self {
        self.options.extra_read = value;
        self
    }
    /// Java nullable `getUseDefaultListener`。
    #[must_use]
    pub const fn get_use_default_listener(&self) -> Option<bool> { self.use_default_listener }
    /// Java `setUseDefaultListener`。
    pub const fn set_use_default_listener(&mut self, value: bool) -> &mut Self {
        self.use_default_listener = Some(value);
        self.options.use_default_listener = value;
        self
    }
    /// Java `getXlsxSAXParserFactoryName`。
    #[must_use]
    pub fn get_xlsx_sax_parser_factory_name(&self) -> Option<&str> {
        self.xlsx_sax_parser_factory_name.as_deref()
    }
    /// Java `getXlsxSAXParserFactoryName()` 原始缩写兼容入口。
    #[must_use]
    pub fn get_xlsx_saxparser_factory_name(&self) -> Option<&str> {
        self.get_xlsx_sax_parser_factory_name()
    }
    /// Java `setXlsxSAXParserFactoryName`。
    pub fn set_xlsx_sax_parser_factory_name(
        &mut self,
        value: Option<String>,
    ) -> &mut Self {
        self.xlsx_sax_parser_factory_name = value;
        self.options.xlsx_sax_parser_factory_name = self.xlsx_sax_parser_factory_name.clone();
        self
    }
    /// Java `setXlsxSAXParserFactoryName()` 原始缩写兼容入口。
    pub fn set_xlsx_saxparser_factory_name(&mut self, value: Option<String>) -> &mut Self {
        self.set_xlsx_sax_parser_factory_name(value)
    }
}

impl Default for ReadWorkbook {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ReadOptions> for ReadWorkbook {
    fn from(options: ReadOptions) -> Self {
        let parameter = crate::read::metadata::ReadBasicParameter::from_options(&options);
        let use_default_listener = options.use_default_listener;
        let xlsx_sax_parser_factory_name = options.xlsx_sax_parser_factory_name.clone();
        Self {
            parameter,
            file: None,
            options,
            excel_type: None,
            auto_close_stream: true,
            input_stream: None,
            auto_close_stream_override: None,
            ignore_empty_row_override: None,
            mandatory_use_input_stream: None,
            use_default_listener: Some(use_default_listener),
            xlsx_sax_parser_factory_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::SimpleReadCacheSelector;
    use crate::{ReadCacheMode, StoredReadCacheSelector};

    #[test]
    fn java_shaped_getters_and_setters_round_trip() {
        // 对应 Java：ReadWorkbook 的 47 个成员访问器表面
        let mut workbook = ReadWorkbook::new();
        assert_eq!(workbook.excel_type(), None);

        workbook.set_excel_type(crate::support::ExcelTypeEnum::Xls);
        assert_eq!(
            workbook.excel_type(),
            Some(crate::support::ExcelTypeEnum::Xls)
        );
        assert!(workbook.auto_close_stream());
        workbook.set_auto_close_stream(false);
        assert!(!workbook.auto_close_stream());

        workbook.set_ignore_empty_row(false);
        assert!(!workbook.ignore_empty_row());

        workbook.set_custom_object(crate::CustomReadObject::new(1_u32));
        assert!(workbook.custom_object().is_some());

        workbook.set_charset(crate::CsvCharset::from("gbk"));
        assert_eq!(workbook.charset().name(), "gbk");

        workbook.set_password("secret");
        assert_eq!(workbook.password(), Some("secret"));

        workbook.set_head_row_number(3);
        assert_eq!(workbook.head_row_number(), 3);

        workbook.set_read_cache(ReadCacheMode::File);
        assert_eq!(workbook.read_cache(), ReadCacheMode::File);

        assert!(workbook.read_cache_selector().is_none());
        workbook.set_read_cache_selector(StoredReadCacheSelector::Simple(
            SimpleReadCacheSelector::new(),
        ));
        assert!(workbook.read_cache_selector().is_some());

        assert!(!workbook.options().ignore_empty_row);
    }

    #[test]
    fn excel_type_is_independent_from_sheet_selection() {
        let workbook = ReadWorkbook::from(ReadOptions {
            sheet: crate::SheetSelector::Index(3),
            ..ReadOptions::default()
        });
        assert_eq!(workbook.excel_type(), None);
        assert_eq!(ReadWorkbook::default().excel_type(), None);
    }
}
