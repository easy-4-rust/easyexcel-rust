//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder`.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::core::WriteHandler;
use crate::write::holder::abstract_write_holder::AbstractWriteHolder;
use crate::write::holder::write_sheet_holder::WriteSheetHolder;
use crate::write::metadata::WriteBasicParameter;

/// 对应 Java：`WriteWorkbookHolder extends AbstractWriteHolder`.
///
/// The Java side aggregates the `rust_xlsxwriter::Workbook` POI handle, the
/// in-progress sheet holders, and the writer's handler list. Rust holds the
/// same data inside [`crate::ExcelWriter`]; this owned builder-side mirror is
/// retained for Java package/API parity. Runtime callbacks expose the actual
/// logical state through [`crate::core::WriteWorkbookHolderView`].
pub struct WriteWorkbookHolder<'a> {
    write_workbook: Option<crate::WriteWorkbook>,
    excel_type: crate::support::ExcelTypeEnum,
    workbook: Option<Vec<u8>>,
    cached_workbook: Option<Vec<u8>>,
    output_stream: Option<Vec<u8>>,
    template_input_stream: Option<Vec<u8>>,
    temp_template_input_stream: Option<Vec<u8>>,
    workbook_write_handler_context: Option<crate::WriteWorkbookContext>,
    abstract_holder: AbstractWriteHolder,
    path: String,
    sheets: HashMap<String, WriteSheetHolder<'a>>,
    handlers: Vec<Box<dyn WriteHandler>>,
    auto_close_stream: bool,
    in_memory: Option<bool>,
    mandatory_use_input_stream: bool,
    write_excel_on_exception: bool,
    with_bom: bool,
    charset: String,
    password: Option<String>,
    template_file: Option<String>,
    initialized_sheet_indexes: HashMap<usize, String>,
    cell_style_index_map: HashMap<String, u32>,
    data_format_map: HashMap<String, u16>,
    font_map: HashMap<String, u16>,
}

impl<'a> WriteWorkbookHolder<'a> {
    /// Java `getFile` 的后端中立路径表示。
    #[must_use] pub fn get_file(&self) -> &str { &self.path }
    /// Java `setFile`。
    pub fn set_file(&mut self, value: impl Into<String>) { self.path = value.into(); }
    /// Java `getWriteHandlerList`。
    #[must_use] pub fn get_write_handler_list(&self) -> &[Box<dyn WriteHandler>] { &self.handlers }
    /// Java `setWriteHandlerList`。
    pub fn set_write_handler_list(&mut self, value: Vec<Box<dyn WriteHandler>>) { self.handlers = value; }
    /// Java `getAutoCloseStream`。
    #[must_use] pub const fn get_auto_close_stream(&self) -> bool { self.auto_close_stream }
    /// Java `getInMemory`。
    #[must_use] pub const fn get_in_memory(&self) -> Option<bool> { self.in_memory }
    /// Java `getMandatoryUseInputStream`。
    #[must_use] pub const fn get_mandatory_use_input_stream(&self) -> bool { self.mandatory_use_input_stream }
    /// Java `getWriteExcelOnException`。
    #[must_use] pub const fn get_write_excel_on_exception(&self) -> bool { self.write_excel_on_exception }
    /// Java `getWithBom`。
    #[must_use] pub const fn get_with_bom(&self) -> bool { self.with_bom }
    /// Java `getCharset`。
    #[must_use] pub fn get_charset(&self) -> &str { &self.charset }
    /// Java `getPassword`。
    #[must_use] pub fn get_password(&self) -> Option<&str> { self.password.as_deref() }
    /// Java `getTemplateFile`。
    #[must_use] pub fn get_template_file(&self) -> Option<&str> { self.template_file.as_deref() }
    /// Java `getHasBeenInitializedSheetIndexMap`。
    #[must_use] pub fn get_has_been_initialized_sheet_index_map(&self) -> &HashMap<usize, String> { &self.initialized_sheet_indexes }
    /// 替换已初始化 Sheet 索引映射。
    pub fn set_has_been_initialized_sheet_index_map(&mut self, value: HashMap<usize, String>) { self.initialized_sheet_indexes = value; }
    /// Java `getHasBeenInitializedSheetNameMap`。
    #[must_use] pub fn get_has_been_initialized_sheet_name_map(&self) -> &HashMap<String, WriteSheetHolder<'a>> { &self.sheets }
    /// 替换已初始化 Sheet 名称映射。
    pub fn set_has_been_initialized_sheet_name_map(&mut self, value: HashMap<String, WriteSheetHolder<'a>>) { self.sheets = value; }
    /// Java `getCellStyleIndexMap`。
    #[must_use] pub fn get_cell_style_index_map(&self) -> &HashMap<String, u32> { &self.cell_style_index_map }
    /// Java `setCellStyleIndexMap`。
    pub fn set_cell_style_index_map(&mut self, value: HashMap<String, u32>) { self.cell_style_index_map = value; }
    /// Java `getDataFormatMap`。
    #[must_use] pub fn get_data_format_map(&self) -> &HashMap<String, u16> { &self.data_format_map }
    /// Java `setDataFormatMap`。
    pub fn set_data_format_map(&mut self, value: HashMap<String, u16>) { self.data_format_map = value; }
    /// Java `getFontMap`。
    #[must_use] pub fn get_font_map(&self) -> &HashMap<String, u16> { &self.font_map }
    /// Java `setFontMap`。
    pub fn set_font_map(&mut self, value: HashMap<String, u16>) { self.font_map = value; }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Creates a holder matching the Java `WriteWorkbookHolder(WriteWorkbook)`
    /// initialiser.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            write_workbook: None,
            excel_type: crate::support::ExcelTypeEnum::Xlsx,
            workbook: None,
            cached_workbook: None,
            output_stream: None,
            template_input_stream: None,
            temp_template_input_stream: None,
            workbook_write_handler_context: None,
            abstract_holder: AbstractWriteHolder::default(),
            path: path.into(),
            sheets: HashMap::new(),
            handlers: Vec::new(),
            auto_close_stream: true,
            in_memory: None,
            mandatory_use_input_stream: false,
            write_excel_on_exception: false,
            with_bom: true,
            charset: "UTF-8".to_owned(),
            password: None,
            template_file: None,
            initialized_sheet_indexes: HashMap::new(),
            cell_style_index_map: HashMap::new(),
            data_format_map: HashMap::new(),
            font_map: HashMap::new(),
        }
    }

    /// Java `WriteWorkbookHolder(WriteWorkbook)`。
    #[must_use]
    pub fn from_write_workbook(value: crate::WriteWorkbook) -> Self {
        let path = value.output_file.as_ref().map_or_else(String::new, |path| path.display().to_string());
        let mut holder = Self::new(path);
        holder.excel_type = value.excel_type;
        holder.write_workbook = Some(value);
        holder
    }
    #[must_use] pub const fn get_write_workbook(&self) -> Option<&crate::WriteWorkbook> { self.write_workbook.as_ref() }
    pub fn set_write_workbook(&mut self, value: Option<crate::WriteWorkbook>) { self.write_workbook = value; }
    #[must_use] pub const fn get_excel_type(&self) -> crate::support::ExcelTypeEnum { self.excel_type }
    pub const fn set_excel_type(&mut self, value: crate::support::ExcelTypeEnum) { self.excel_type = value; }
    #[must_use] pub fn get_workbook(&self) -> Option<&[u8]> { self.workbook.as_deref() }
    pub fn set_workbook(&mut self, value: Option<Vec<u8>>) { self.workbook = value; }
    #[must_use] pub fn get_cached_workbook(&self) -> Option<&[u8]> { self.cached_workbook.as_deref() }
    pub fn set_cached_workbook(&mut self, value: Option<Vec<u8>>) { self.cached_workbook = value; }
    #[must_use] pub fn get_output_stream(&self) -> Option<&[u8]> { self.output_stream.as_deref() }
    pub fn set_output_stream(&mut self, value: Option<Vec<u8>>) { self.output_stream = value; }
    #[must_use] pub fn get_template_input_stream(&self) -> Option<&[u8]> { self.template_input_stream.as_deref() }
    pub fn set_template_input_stream(&mut self, value: Option<Vec<u8>>) { self.template_input_stream = value; }
    #[must_use] pub fn get_temp_template_input_stream(&self) -> Option<&[u8]> { self.temp_template_input_stream.as_deref() }
    pub fn set_temp_template_input_stream(&mut self, value: Option<Vec<u8>>) { self.temp_template_input_stream = value; }
    #[must_use] pub const fn get_workbook_write_handler_context(&self) -> Option<&crate::WriteWorkbookContext> {
        self.workbook_write_handler_context.as_ref()
    }
    pub fn set_workbook_write_handler_context(&mut self, value: Option<crate::WriteWorkbookContext>) {
        self.workbook_write_handler_context = value;
    }
    #[must_use] pub const fn holder_type(&self) -> crate::HolderEnum { crate::HolderEnum::Workbook }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Creates a workbook holder from nullable write parameters.
    #[must_use]
    pub fn from_parameter(path: impl Into<String>, parameter: &WriteBasicParameter) -> Self {
        let mut holder = Self::new(path);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(parameter, None);
        holder
    }

    /// Returns the inherited write-holder state.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。
    pub const fn abstract_holder(&self) -> &AbstractWriteHolder {
        &self.abstract_holder
    }

    /// Returns mutable inherited write-holder state.
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractWriteHolder {
        &mut self.abstract_holder
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Returns the output path. (Java `getFile()`)
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Returns the in-progress sheet holders. (Java `getHasBeenInitializedSheetNameMap()`)
    #[must_use]
    pub fn sheets(&self) -> &HashMap<String, WriteSheetHolder<'a>> {
        &self.sheets
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Returns a mutable handle on the in-progress sheet holders.
    pub fn sheets_mut(&mut self) -> &mut HashMap<String, WriteSheetHolder<'a>> {
        &mut self.sheets
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Returns the ordered write handler list. (Java `getWriteHandlerList()`)
    #[must_use]
    pub fn handlers(&self) -> &[Box<dyn WriteHandler>] {
        &self.handlers
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder。 Appends a handler. (Java `setWriteHandlerList` step)
    pub fn push_handler(&mut self, handler: Box<dyn WriteHandler>) {
        self.handlers.push(handler);
    }

    /// 返回 Java `getAutoCloseStream` 对应状态。
    #[must_use]
    pub const fn auto_close_stream(&self) -> bool { self.auto_close_stream }
    /// 设置 Java `setAutoCloseStream` 对应状态。
    pub const fn set_auto_close_stream(&mut self, value: bool) { self.auto_close_stream = value; }
    /// 返回自动内存选择覆盖；`None` 表示 Auto。
    #[must_use]
    pub const fn in_memory(&self) -> Option<bool> { self.in_memory }
    /// 设置自动内存选择覆盖。
    pub const fn set_in_memory(&mut self, value: Option<bool>) { self.in_memory = value; }
    /// 返回是否强制输入流模板路径。
    #[must_use]
    pub const fn mandatory_use_input_stream(&self) -> bool { self.mandatory_use_input_stream }
    /// 设置是否强制输入流模板路径。
    pub const fn set_mandatory_use_input_stream(&mut self, value: bool) { self.mandatory_use_input_stream = value; }
    /// 返回异常时是否仍输出工作簿。
    #[must_use]
    pub const fn write_excel_on_exception(&self) -> bool { self.write_excel_on_exception }
    /// 设置异常时是否仍输出工作簿。
    pub const fn set_write_excel_on_exception(&mut self, value: bool) { self.write_excel_on_exception = value; }
    /// 返回 CSV BOM 开关。
    #[must_use]
    pub const fn with_bom(&self) -> bool { self.with_bom }
    /// 设置 CSV BOM 开关。
    pub const fn set_with_bom(&mut self, value: bool) { self.with_bom = value; }
    /// 返回字符集名称。
    #[must_use]
    pub fn charset(&self) -> &str { &self.charset }
    /// 设置字符集名称。
    pub fn set_charset(&mut self, value: impl Into<String>) { self.charset = value.into(); }
    /// 返回调用级工作簿密码。
    #[must_use]
    pub fn password(&self) -> Option<&str> { self.password.as_deref() }
    /// 设置调用级工作簿密码。
    pub fn set_password(&mut self, value: Option<String>) { self.password = value; }
    /// 返回模板文件路径。
    #[must_use]
    pub fn template_file(&self) -> Option<&str> { self.template_file.as_deref() }
    /// 设置模板文件路径。
    pub fn set_template_file(&mut self, value: Option<String>) { self.template_file = value; }
    /// 返回按索引初始化的 Sheet 映射。
    #[must_use]
    pub fn initialized_sheet_indexes(&self) -> &HashMap<usize, String> { &self.initialized_sheet_indexes }
    /// 返回按名称初始化的 Sheet 映射。
    #[must_use]
    pub fn initialized_sheet_names(&self) -> &HashMap<String, WriteSheetHolder<'a>> { &self.sheets }
    /// 返回样式索引缓存。
    #[must_use]
    pub fn cell_style_index_map(&self) -> &HashMap<String, u32> { &self.cell_style_index_map }
    /// 返回数据格式缓存。
    #[must_use]
    pub fn data_format_map(&self) -> &HashMap<String, u16> { &self.data_format_map }
    /// 返回字体缓存。
    #[must_use]
    pub fn font_map(&self) -> &HashMap<String, u16> { &self.font_map }

    /// 分配一个稳定的样式索引，语义对应 Java `createCellStyle` 的 holder 缓存。
    pub fn create_cell_style(&mut self, key: impl Into<String>) -> u32 {
        let key = key.into();
        if let Some(index) = self.cell_style_index_map.get(&key) { return *index; }
        let index = u32::try_from(self.cell_style_index_map.len()).unwrap_or(u32::MAX);
        self.cell_style_index_map.insert(key, index);
        index
    }

    /// 分配一个稳定的数据格式索引。
    pub fn create_data_format(&mut self, key: impl Into<String>) -> u16 {
        let key = key.into();
        if let Some(index) = self.data_format_map.get(&key) { return *index; }
        let index = u16::try_from(self.data_format_map.len()).unwrap_or(u16::MAX);
        self.data_format_map.insert(key, index);
        index
    }

    /// 分配一个稳定的字体索引。
    pub fn create_font(&mut self, key: impl Into<String>) -> u16 {
        let key = key.into();
        if let Some(index) = self.font_map.get(&key) { return *index; }
        let index = u16::try_from(self.font_map.len()).unwrap_or(u16::MAX);
        self.font_map.insert(key, index);
        index
    }
}

impl Deref for WriteWorkbookHolder<'_> {
    type Target = AbstractWriteHolder;

    fn deref(&self) -> &Self::Target {
        &self.abstract_holder
    }
}

impl DerefMut for WriteWorkbookHolder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.abstract_holder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::write::metadata::write_basic_parameter::WriteBasicParameter;

    #[test]
    fn write_workbook_holder_new() {
        let holder = WriteWorkbookHolder::new("/tmp/out.xlsx");
        assert_eq!(holder.path(), "/tmp/out.xlsx");
        assert!(holder.sheets().is_empty());
        assert!(holder.handlers().is_empty());
    }

    #[test]
    fn write_workbook_holder_from_parameter() {
        let param = WriteBasicParameter::default();
        let holder = WriteWorkbookHolder::from_parameter("/tmp/p.xlsx", &param);
        assert_eq!(holder.path(), "/tmp/p.xlsx");
    }

    #[test]
    fn write_workbook_holder_abstract_holder_accessors() {
        let mut holder = WriteWorkbookHolder::new("/tmp/a.xlsx");
        let _ = holder.abstract_holder();
        let _ = holder.abstract_holder_mut();
    }

    #[test]
    fn write_workbook_holder_sheets_mut() {
        let mut holder = WriteWorkbookHolder::new("/tmp/b.xlsx");
        let _ = holder.sheets_mut();
    }

    #[test]
    fn write_workbook_holder_push_handler() {
        /// No-op `WriteHandler` for testing.
        struct NoopHandler;
        impl WriteHandler for NoopHandler {
            fn order(&self) -> i32 {
                0
            }
        }
        let mut holder = WriteWorkbookHolder::new("/tmp/c.xlsx");
        holder.push_handler(Box::new(NoopHandler));
        assert_eq!(holder.handlers().len(), 1);
        assert_eq!(NoopHandler.order(), 0);
    }

    #[test]
    fn write_workbook_holder_deref() {
        let holder = WriteWorkbookHolder::new("/tmp/d.xlsx");
        let _ = holder.abstract_holder();
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::{ExcelWriteHeadProperty, WriteHandler};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn workbook_holder_deref_mut_reaches_abstract_holder() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        holder.set_excel_write_head_property(ExcelWriteHeadProperty::new());
        assert_eq!(NoopHandler.order(), 0);
    }
}
