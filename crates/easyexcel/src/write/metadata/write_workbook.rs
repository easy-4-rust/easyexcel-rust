//! 对应 Java：`com.alibaba.excel.write.metadata.WriteWorkbook`.

use crate::core::CsvCharset;

use crate::WriteOptions;

/// 对应 Java：`WriteWorkbook extends WriteBasicParameter`.
///
/// The Java side carries 11 fields (file, outputStream, templateFile, etc.).
/// Rust reuses the existing [`WriteOptions`] struct that already models the
/// same data; this newtype exists so the public API carries a 1:1 named
/// class and lets builders accept either `WriteOptions` or `WriteWorkbook`.
#[derive(Debug, Clone)]
pub struct WriteWorkbook {
    /// Backing configuration. (Java `WriteWorkbook` getter surface)
    pub options: WriteOptions,
    /// Mirrors `WriteWorkbook.excelType`. (Java `getExcelType()`)
    pub excel_type: crate::support::ExcelTypeEnum,
    /// Java nullable `excelType` 原始状态；`excel_type` 保存引擎有效默认值。
    pub excel_type_override: Option<crate::support::ExcelTypeEnum>,
    /// Final output file. (Java `WriteWorkbook.file`)
    pub output_file: Option<std::path::PathBuf>,
    /// 后端中立输出流缓冲。对应 Java `outputStream`。
    pub output_stream: Option<Vec<u8>>,
    /// Java nullable `charset` 原始状态；`options.charset` 保存引擎有效值。
    pub charset_override: Option<CsvCharset>,
    /// Java nullable 配置覆盖；Holder 初始化时再应用有效默认值。
    pub auto_close_stream_override: Option<bool>,
    /// Java nullable `inMemory`，`None` 表示自动选择。
    pub in_memory_override: Option<bool>,
    /// Java nullable `mandatoryUseInputStream`。
    pub mandatory_use_input_stream: Option<bool>,
    /// Java nullable `withBom`。
    pub with_bom_override: Option<bool>,
    /// Java nullable `writeExcelOnException`。
    pub write_excel_on_exception_override: Option<bool>,
}

impl WriteWorkbook {
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Creates a new `WriteWorkbook` with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: WriteOptions::default(),
            excel_type: crate::support::ExcelTypeEnum::Xlsx,
            excel_type_override: None,
            output_file: None,
            output_stream: None,
            charset_override: None,
            auto_close_stream_override: None,
            in_memory_override: None,
            mandatory_use_input_stream: None,
            with_bom_override: None,
            write_excel_on_exception_override: None,
        }
    }

    /// Returns the effective write options.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。
    pub const fn options(&self) -> &WriteOptions {
        &self.options
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Returns the Excel file type. (Java `getExcelType()`)
    #[must_use]
    pub fn excel_type(&self) -> crate::support::ExcelTypeEnum {
        self.excel_type
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the Excel file type. (Java `setExcelType(ExcelTypeEnum)`)
    pub fn set_excel_type(&mut self, excel_type: crate::support::ExcelTypeEnum) -> &mut Self {
        self.excel_type = excel_type;
        self.excel_type_override = Some(excel_type);
        self.options.excel_type = Some(excel_type);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Returns the output file path. (Java `getFile()`)
    ///
    #[must_use]
    pub fn file(&self) -> Option<&std::path::Path> {
        self.output_file.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the output file path. (Java `setFile(File)`)
    pub fn set_file(&mut self, file: impl Into<std::path::PathBuf>) -> &mut Self {
        self.output_file = Some(file.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Returns the template file path. (Java `getTemplateFile()`)
    #[must_use]
    pub fn template_file(&self) -> Option<&std::path::Path> {
        self.options.template_file.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the template file and clears an input-stream template.
    /// (Java `setTemplateFile(File)`)
    pub fn set_template_file(&mut self, template_file: impl Into<std::path::PathBuf>) -> &mut Self {
        self.options.template_file = Some(template_file.into());
        self.options.template_bytes = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets an already-buffered input-stream template and clears the file.
    /// (Java `setTemplateInputStream(InputStream)`)
    pub fn set_template_bytes(&mut self, template_bytes: impl Into<Vec<u8>>) -> &mut Self {
        self.options.template_bytes = Some(template_bytes.into());
        self.options.template_file = None;
        self
    }

    /// Java `getTemplateInputStream` 的后端中立字节表示。
    #[must_use]
    pub fn get_template_input_stream(&self) -> Option<&[u8]> {
        self.options.template_bytes.as_deref()
    }
    /// Java `setTemplateInputStream`。
    pub fn set_template_input_stream(&mut self, value: Option<Vec<u8>>) -> &mut Self {
        self.options.template_bytes = value;
        if self.options.template_bytes.is_some() {
            self.options.template_file = None;
        }
        self
    }
    /// Java `getOutputStream` 的后端中立字节表示。
    #[must_use]
    pub fn get_output_stream(&self) -> Option<&[u8]> {
        self.output_stream.as_deref()
    }
    /// Java `setOutputStream`。
    pub fn set_output_stream(&mut self, value: Option<Vec<u8>>) -> &mut Self {
        self.output_stream = value;
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Returns the charset. (Java `getCharset()`)
    #[must_use]
    pub fn charset(&self) -> &CsvCharset {
        &self.options.charset
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the charset. (Java `setCharset(Charset)`)
    pub fn set_charset(&mut self, charset: CsvCharset) -> &mut Self {
        self.options.charset = charset;
        self.charset_override = Some(self.options.charset.clone());
        self
    }

    /// Returns the BOM flag. (Java `getWithBom()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。
    pub const fn with_bom(&self) -> bool {
        self.options.with_bom
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the BOM flag. (Java `setWithBom(Boolean)`)
    pub fn set_with_bom(&mut self, with_bom: bool) -> &mut Self {
        self.options.with_bom = with_bom;
        self.with_bom_override = Some(with_bom);
        self
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Returns the password, if any. (Java `getPassword()`)
    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.options.password.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the password. (Java `setPassword(String)`)
    pub fn set_password(&mut self, password: impl Into<String>) -> &mut Self {
        self.options.password = Some(password.into());
        self
    }

    /// Returns the in-memory flag. (Java `getInMemory()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。
    pub const fn in_memory(&self) -> bool {
        !self.options.constant_memory
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the in-memory flag. (Java `setInMemory(boolean)`)
    pub fn set_in_memory(&mut self, in_memory: bool) -> &mut Self {
        self.options.constant_memory = !in_memory;
        self.in_memory_override = Some(in_memory);
        self
    }

    /// Returns the write-on-exception flag. (Java `getWriteExcelOnException()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。
    pub const fn write_excel_on_exception(&self) -> bool {
        self.options.write_excel_on_exception
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the write-on-exception flag. (Java `setWriteExcelOnException(boolean)`)
    pub fn set_write_excel_on_exception(&mut self, value: bool) -> &mut Self {
        self.options.write_excel_on_exception = value;
        self.write_excel_on_exception_override = Some(value);
        self
    }

    /// Returns the auto-close-stream flag. (Java `getAutoCloseStream()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。
    pub const fn auto_close_stream(&self) -> bool {
        self.options.auto_close_stream
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.WriteWorkbook。 Sets the auto-close-stream flag. (Java `setAutoCloseStream(boolean)`)
    pub fn set_auto_close_stream(&mut self, value: bool) -> &mut Self {
        self.options.auto_close_stream = value;
        self.auto_close_stream_override = Some(value);
        self
    }

    /// Java `getExcelType` 别名。
    #[must_use]
    pub const fn get_excel_type(&self) -> Option<crate::support::ExcelTypeEnum> {
        self.excel_type_override
    }
    /// Java `getFile` 别名。
    #[must_use]
    pub fn get_file(&self) -> Option<&std::path::Path> {
        self.file()
    }
    /// Java `getTemplateFile` 别名。
    #[must_use]
    pub fn get_template_file(&self) -> Option<&std::path::Path> {
        self.template_file()
    }
    /// Java `getCharset` 别名。
    #[must_use]
    pub fn get_charset(&self) -> Option<&CsvCharset> {
        self.charset_override.as_ref()
    }
    /// Java `getPassword` 别名。
    #[must_use]
    pub fn get_password(&self) -> Option<&str> {
        self.password()
    }
    /// Java nullable `getAutoCloseStream`。
    #[must_use]
    pub const fn get_auto_close_stream(&self) -> Option<bool> {
        self.auto_close_stream_override
    }
    /// Java nullable `getInMemory`。
    #[must_use]
    pub const fn get_in_memory(&self) -> Option<bool> {
        self.in_memory_override
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
    /// Java nullable `getWithBom`。
    #[must_use]
    pub const fn get_with_bom(&self) -> Option<bool> {
        self.with_bom_override
    }
    /// Java nullable `getWriteExcelOnException`。
    #[must_use]
    pub const fn get_write_excel_on_exception(&self) -> Option<bool> {
        self.write_excel_on_exception_override
    }
}

impl Default for WriteWorkbook {
    fn default() -> Self {
        Self::new()
    }
}

// Java Lombok 默认 `callSuper = false`，只比较 WriteWorkbook 自身声明的十二个字段。
impl PartialEq for WriteWorkbook {
    fn eq(&self, other: &Self) -> bool {
        self.excel_type_override == other.excel_type_override
            && self.output_file == other.output_file
            && self.output_stream == other.output_stream
            && self.charset_override == other.charset_override
            && self.with_bom_override == other.with_bom_override
            && self.options.template_bytes == other.options.template_bytes
            && self.options.template_file == other.options.template_file
            && self.auto_close_stream_override == other.auto_close_stream_override
            && self.mandatory_use_input_stream == other.mandatory_use_input_stream
            && self.options.password == other.options.password
            && self.in_memory_override == other.in_memory_override
            && self.write_excel_on_exception_override == other.write_excel_on_exception_override
    }
}

impl Eq for WriteWorkbook {}

impl std::hash::Hash for WriteWorkbook {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(
            &self
                .excel_type_override
                .map(crate::support::ExcelTypeEnum::java_name),
            state,
        );
        std::hash::Hash::hash(&self.output_file, state);
        std::hash::Hash::hash(&self.output_stream, state);
        std::hash::Hash::hash(&self.charset_override.as_ref().map(CsvCharset::name), state);
        std::hash::Hash::hash(&self.with_bom_override, state);
        std::hash::Hash::hash(&self.options.template_bytes, state);
        std::hash::Hash::hash(&self.options.template_file, state);
        std::hash::Hash::hash(&self.auto_close_stream_override, state);
        std::hash::Hash::hash(&self.mandatory_use_input_stream, state);
        std::hash::Hash::hash(&self.options.password, state);
        std::hash::Hash::hash(&self.in_memory_override, state);
        std::hash::Hash::hash(&self.write_excel_on_exception_override, state);
    }
}

impl From<WriteOptions> for WriteWorkbook {
    fn from(options: WriteOptions) -> Self {
        let excel_type_override = options.excel_type;
        let excel_type = excel_type_override.unwrap_or(crate::support::ExcelTypeEnum::Xlsx);
        let charset_override = Some(options.charset.clone());
        Self {
            options,
            excel_type,
            excel_type_override,
            output_file: None,
            output_stream: None,
            charset_override,
            auto_close_stream_override: None,
            in_memory_override: None,
            mandatory_use_input_stream: None,
            with_bom_override: None,
            write_excel_on_exception_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::ExcelTypeEnum;

    #[test]
    fn write_workbook_new_defaults() {
        let wb = WriteWorkbook::new();
        assert_eq!(wb.excel_type(), ExcelTypeEnum::Xlsx);
        assert!(wb.file().is_none());
        assert!(wb.template_file().is_none());
        // Default WriteOptions has with_bom=true and auto_close_stream=true
        assert!(wb.with_bom());
        assert!(wb.password().is_none());
        assert!(wb.in_memory());
        assert!(!wb.write_excel_on_exception());
        assert!(wb.auto_close_stream());
    }

    #[test]
    fn write_workbook_default_impl() {
        let wb = WriteWorkbook::default();
        assert_eq!(wb.excel_type(), ExcelTypeEnum::Xlsx);
    }

    #[test]
    fn write_workbook_set_excel_type() {
        let mut wb = WriteWorkbook::new();
        wb.set_excel_type(ExcelTypeEnum::Xls);
        assert_eq!(wb.excel_type(), ExcelTypeEnum::Xls);
    }

    #[test]
    fn write_workbook_set_file() {
        let mut wb = WriteWorkbook::new();
        wb.set_file("/tmp/wb.xlsx");
        assert_eq!(wb.file().unwrap().to_str().unwrap(), "/tmp/wb.xlsx");
    }

    #[test]
    fn write_workbook_set_template_file() {
        let mut wb = WriteWorkbook::new();
        wb.set_template_file("/tmp/tpl.xlsx");
        assert!(wb.template_file().is_some());
    }

    #[test]
    fn write_workbook_set_template_bytes() {
        let mut wb = WriteWorkbook::new();
        wb.set_template_bytes(vec![1u8, 2, 3]);
        assert!(wb.template_file().is_none());
    }

    #[test]
    fn write_workbook_set_charset() {
        let mut wb = WriteWorkbook::new();
        let charset = CsvCharset::new("UTF-8");
        wb.set_charset(charset.clone());
        assert_eq!(wb.charset(), &charset);
    }

    #[test]
    fn write_workbook_set_with_bom() {
        let mut wb = WriteWorkbook::new();
        wb.set_with_bom(true);
        assert!(wb.with_bom());
    }

    #[test]
    fn write_workbook_set_password() {
        let mut wb = WriteWorkbook::new();
        wb.set_password("secret");
        assert_eq!(wb.password(), Some("secret"));
    }

    #[test]
    fn write_workbook_set_in_memory() {
        let mut wb = WriteWorkbook::new();
        wb.set_in_memory(false);
        assert!(!wb.in_memory());
    }

    #[test]
    fn write_workbook_set_write_excel_on_exception() {
        let mut wb = WriteWorkbook::new();
        wb.set_write_excel_on_exception(true);
        assert!(wb.write_excel_on_exception());
    }

    #[test]
    fn write_workbook_set_auto_close_stream() {
        let mut wb = WriteWorkbook::new();
        wb.set_auto_close_stream(true);
        assert!(wb.auto_close_stream());
    }

    #[test]
    fn write_workbook_from_write_options() {
        let opts = WriteOptions::default();
        let wb = WriteWorkbook::from(opts);
        assert_eq!(wb.excel_type(), ExcelTypeEnum::Xlsx);
    }

    #[test]
    fn write_workbook_options_accessor() {
        let wb = WriteWorkbook::new();
        let _ = wb.options();
    }
}
