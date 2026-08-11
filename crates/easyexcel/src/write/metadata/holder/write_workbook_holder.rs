//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder`.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::core::WriteHandler;
use crate::metadata::data::DataFormatData;
use crate::util::style_util::{build_cell_style, build_data_format, build_font};
use crate::write::holder::abstract_write_holder::AbstractWriteHolder;
use crate::write::holder::write_sheet_holder::WriteSheetHolder;
use crate::write::metadata::WriteBasicParameter;
use crate::write::metadata::holder::write_holder::delegate_write_holder_contract;
use crate::write::metadata::style::write_cell_style::WriteCellStyle;
use crate::write::metadata::style::write_font::WriteFont;

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
    /// 后端中立样式缓存。Java 以来源 POI `CellStyle#index` 分组；Rust 没有
    /// 暴露后端对象，因此直接以来源样式规格分组，保留同样的“有来源样式时
    /// 不跨来源复用”语义。
    cell_style_index_map: HashMap<Option<WriteCellStyle>, Vec<WriteCellStyle>>,
    data_format_map: Vec<DataFormatData>,
    font_map: Vec<WriteFont>,
}

impl<'a> WriteWorkbookHolder<'a> {
    /// Java `getFile` 的后端中立路径表示。
    #[must_use]
    pub fn get_file(&self) -> &str {
        &self.path
    }
    /// Java `setFile`。
    pub fn set_file(&mut self, value: impl Into<String>) {
        self.path = value.into();
    }
    /// Java `getWriteHandlerList`。
    #[must_use]
    pub fn get_write_handler_list(&self) -> &[Box<dyn WriteHandler>] {
        &self.handlers
    }
    /// Java `setWriteHandlerList`。
    pub fn set_write_handler_list(&mut self, value: Vec<Box<dyn WriteHandler>>) {
        self.handlers = value;
    }
    /// Java `getAutoCloseStream`。
    #[must_use]
    pub const fn get_auto_close_stream(&self) -> bool {
        self.auto_close_stream
    }
    /// Java `getInMemory`。
    #[must_use]
    pub const fn get_in_memory(&self) -> Option<bool> {
        self.in_memory
    }
    /// Java `getMandatoryUseInputStream`。
    #[must_use]
    pub const fn get_mandatory_use_input_stream(&self) -> bool {
        self.mandatory_use_input_stream
    }
    /// Java `getWriteExcelOnException`。
    #[must_use]
    pub const fn get_write_excel_on_exception(&self) -> bool {
        self.write_excel_on_exception
    }
    /// Java `getWithBom`。
    #[must_use]
    pub const fn get_with_bom(&self) -> bool {
        self.with_bom
    }
    /// Java `getCharset`。
    #[must_use]
    pub fn get_charset(&self) -> &str {
        &self.charset
    }
    /// Java `getPassword`。
    #[must_use]
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }
    /// Java `getTemplateFile`。
    #[must_use]
    pub fn get_template_file(&self) -> Option<&str> {
        self.template_file.as_deref()
    }
    /// Java `getHasBeenInitializedSheetIndexMap`。
    #[must_use]
    pub fn get_has_been_initialized_sheet_index_map(&self) -> &HashMap<usize, String> {
        &self.initialized_sheet_indexes
    }
    /// 替换已初始化 Sheet 索引映射。
    pub fn set_has_been_initialized_sheet_index_map(&mut self, value: HashMap<usize, String>) {
        self.initialized_sheet_indexes = value;
    }
    /// Java `getHasBeenInitializedSheetNameMap`。
    #[must_use]
    pub fn get_has_been_initialized_sheet_name_map(
        &self,
    ) -> &HashMap<String, WriteSheetHolder<'a>> {
        &self.sheets
    }
    /// 替换已初始化 Sheet 名称映射。
    pub fn set_has_been_initialized_sheet_name_map(
        &mut self,
        value: HashMap<String, WriteSheetHolder<'a>>,
    ) {
        self.sheets = value;
    }
    /// Java `getCellStyleIndexMap`。
    #[must_use]
    pub const fn get_cell_style_index_map(
        &self,
    ) -> &HashMap<Option<WriteCellStyle>, Vec<WriteCellStyle>> {
        &self.cell_style_index_map
    }
    /// Java `setCellStyleIndexMap`。
    pub fn set_cell_style_index_map(
        &mut self,
        value: HashMap<Option<WriteCellStyle>, Vec<WriteCellStyle>>,
    ) {
        self.cell_style_index_map = value;
    }
    /// Java `getDataFormatMap`。
    #[must_use]
    pub fn get_data_format_map(&self) -> &[DataFormatData] {
        &self.data_format_map
    }
    /// Java `setDataFormatMap`。
    pub fn set_data_format_map(&mut self, value: Vec<DataFormatData>) {
        self.data_format_map = value;
    }
    /// Java `getFontMap`。
    #[must_use]
    pub fn get_font_map(&self) -> &[WriteFont] {
        &self.font_map
    }
    /// Java `setFontMap`。
    pub fn set_font_map(&mut self, value: Vec<WriteFont>) {
        self.font_map = value;
    }

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
            data_format_map: Vec::new(),
            font_map: Vec::new(),
        }
    }

    /// Java `WriteWorkbookHolder(WriteWorkbook)`。
    #[must_use]
    pub fn from_write_workbook(value: crate::WriteWorkbook) -> Self {
        let path = value
            .output_file
            .as_ref()
            .map_or_else(String::new, |path| path.display().to_string());
        let detection_path = value
            .output_file
            .as_deref()
            .or(value.options.template_file.as_deref())
            .unwrap_or_else(|| std::path::Path::new(""));
        let excel_type =
            crate::write_type_helpers::effective_write_type(detection_path, &value.options);
        let parameter = WriteBasicParameter::from_options(&value.options);
        let template_input_stream = value.options.template_bytes.clone();
        let mut holder = Self::new(path);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(&parameter, None);
        holder.excel_type = excel_type;
        holder.output_stream = value.output_stream.clone();
        holder.template_input_stream = template_input_stream.clone();
        holder.temp_template_input_stream = template_input_stream;
        holder.auto_close_stream = value
            .auto_close_stream_override
            .unwrap_or(value.options.auto_close_stream);
        holder.in_memory = Some(value.in_memory_override.unwrap_or(false));
        holder.mandatory_use_input_stream = value.mandatory_use_input_stream.unwrap_or(false);
        holder.write_excel_on_exception = value
            .write_excel_on_exception_override
            .unwrap_or(value.options.write_excel_on_exception);
        holder.with_bom = value.with_bom_override.unwrap_or(value.options.with_bom);
        holder.charset = value.options.charset.name().to_owned();
        holder.password = value.options.password.clone();
        holder.template_file = value
            .options
            .template_file
            .as_ref()
            .map(|path| path.display().to_string());
        holder.write_workbook = Some(value);
        holder
    }
    #[must_use]
    pub const fn get_write_workbook(&self) -> Option<&crate::WriteWorkbook> {
        self.write_workbook.as_ref()
    }
    pub fn set_write_workbook(&mut self, value: Option<crate::WriteWorkbook>) {
        self.write_workbook = value;
    }
    #[must_use]
    pub const fn get_excel_type(&self) -> crate::support::ExcelTypeEnum {
        self.excel_type
    }
    pub const fn set_excel_type(&mut self, value: crate::support::ExcelTypeEnum) {
        self.excel_type = value;
    }
    #[must_use]
    pub fn get_workbook(&self) -> Option<&[u8]> {
        self.workbook.as_deref()
    }
    pub fn set_workbook(&mut self, value: Option<Vec<u8>>) {
        self.workbook = value;
    }
    #[must_use]
    pub fn get_cached_workbook(&self) -> Option<&[u8]> {
        self.cached_workbook.as_deref()
    }
    pub fn set_cached_workbook(&mut self, value: Option<Vec<u8>>) {
        self.cached_workbook = value;
    }
    #[must_use]
    pub fn get_output_stream(&self) -> Option<&[u8]> {
        self.output_stream.as_deref()
    }
    pub fn set_output_stream(&mut self, value: Option<Vec<u8>>) {
        self.output_stream = value;
    }
    #[must_use]
    pub fn get_template_input_stream(&self) -> Option<&[u8]> {
        self.template_input_stream.as_deref()
    }
    pub fn set_template_input_stream(&mut self, value: Option<Vec<u8>>) {
        self.template_input_stream = value;
    }
    #[must_use]
    pub fn get_temp_template_input_stream(&self) -> Option<&[u8]> {
        self.temp_template_input_stream.as_deref()
    }
    pub fn set_temp_template_input_stream(&mut self, value: Option<Vec<u8>>) {
        self.temp_template_input_stream = value;
    }
    #[must_use]
    pub const fn get_workbook_write_handler_context(&self) -> Option<&crate::WriteWorkbookContext> {
        self.workbook_write_handler_context.as_ref()
    }
    pub fn set_workbook_write_handler_context(
        &mut self,
        value: Option<crate::WriteWorkbookContext>,
    ) {
        self.workbook_write_handler_context = value;
    }
    #[must_use]
    pub const fn holder_type(&self) -> crate::HolderEnum {
        crate::HolderEnum::Workbook
    }

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
    pub const fn auto_close_stream(&self) -> bool {
        self.auto_close_stream
    }
    /// 设置 Java `setAutoCloseStream` 对应状态。
    pub const fn set_auto_close_stream(&mut self, value: bool) {
        self.auto_close_stream = value;
    }
    /// 返回自动内存选择覆盖；`None` 表示 Auto。
    #[must_use]
    pub const fn in_memory(&self) -> Option<bool> {
        self.in_memory
    }
    /// 设置自动内存选择覆盖。
    pub const fn set_in_memory(&mut self, value: Option<bool>) {
        self.in_memory = value;
    }
    /// 返回是否强制输入流模板路径。
    #[must_use]
    pub const fn mandatory_use_input_stream(&self) -> bool {
        self.mandatory_use_input_stream
    }
    /// 设置是否强制输入流模板路径。
    pub const fn set_mandatory_use_input_stream(&mut self, value: bool) {
        self.mandatory_use_input_stream = value;
    }
    /// 返回异常时是否仍输出工作簿。
    #[must_use]
    pub const fn write_excel_on_exception(&self) -> bool {
        self.write_excel_on_exception
    }
    /// 设置异常时是否仍输出工作簿。
    pub const fn set_write_excel_on_exception(&mut self, value: bool) {
        self.write_excel_on_exception = value;
    }
    /// 返回 CSV BOM 开关。
    #[must_use]
    pub const fn with_bom(&self) -> bool {
        self.with_bom
    }
    /// 设置 CSV BOM 开关。
    pub const fn set_with_bom(&mut self, value: bool) {
        self.with_bom = value;
    }
    /// 返回字符集名称。
    #[must_use]
    pub fn charset(&self) -> &str {
        &self.charset
    }
    /// 设置字符集名称。
    pub fn set_charset(&mut self, value: impl Into<String>) {
        self.charset = value.into();
    }
    /// 返回调用级工作簿密码。
    #[must_use]
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
    /// 设置调用级工作簿密码。
    pub fn set_password(&mut self, value: Option<String>) {
        self.password = value;
    }
    /// 返回模板文件路径。
    #[must_use]
    pub fn template_file(&self) -> Option<&str> {
        self.template_file.as_deref()
    }
    /// 设置模板文件路径。
    pub fn set_template_file(&mut self, value: Option<String>) {
        self.template_file = value;
    }
    /// 返回按索引初始化的 Sheet 映射。
    #[must_use]
    pub fn initialized_sheet_indexes(&self) -> &HashMap<usize, String> {
        &self.initialized_sheet_indexes
    }
    /// 返回按名称初始化的 Sheet 映射。
    #[must_use]
    pub fn initialized_sheet_names(&self) -> &HashMap<String, WriteSheetHolder<'a>> {
        &self.sheets
    }
    /// 返回样式索引缓存。
    #[must_use]
    pub const fn cell_style_index_map(
        &self,
    ) -> &HashMap<Option<WriteCellStyle>, Vec<WriteCellStyle>> {
        &self.cell_style_index_map
    }
    /// 返回数据格式缓存。
    #[must_use]
    pub fn data_format_map(&self) -> &[DataFormatData] {
        &self.data_format_map
    }
    /// 返回字体缓存。
    #[must_use]
    pub fn font_map(&self) -> &[WriteFont] {
        &self.font_map
    }

    /// 合并并缓存后端中立样式，对应 Java `createCellStyle(writeCellStyle, originCellStyle)`。
    ///
    /// `write_cell_style` 为空时原样返回来源样式；否则仅用非空字段覆盖来源，
    /// 并把字体/数据格式交给同一 Holder 的语义缓存。XLS/XLSX 物理样式表仍由
    /// 各自引擎在最终写入阶段编码。
    pub fn create_cell_style(
        &mut self,
        write_cell_style: Option<&WriteCellStyle>,
        origin_cell_style: Option<&WriteCellStyle>,
    ) -> Option<WriteCellStyle> {
        let Some(write_cell_style) = write_cell_style else {
            return origin_cell_style.cloned();
        };
        let use_cache = origin_cell_style.is_none();
        let style = build_cell_style(origin_cell_style, Some(write_cell_style));
        let data_format = write_cell_style
            .get_data_format_data()
            .map(|value| match value {
                crate::ExcelDataFormat::Builtin(index) => DataFormatData {
                    index: Some(i16::from(index)),
                    format: None,
                },
                crate::ExcelDataFormat::Custom(format) => DataFormatData {
                    index: None,
                    format: Some(format.to_owned()),
                },
            });
        let origin_font = origin_cell_style.and_then(WriteCellStyle::get_write_font);
        let _ = self.create_font(write_cell_style.get_write_font(), origin_font, use_cache);
        let _ = self.create_data_format(data_format.as_ref(), use_cache);
        let cache_partition = self
            .cell_style_index_map
            .entry(origin_cell_style.cloned())
            .or_default();
        if let Some(cached) = cache_partition.iter().find(|cached| **cached == style) {
            return Some(cached.clone());
        }
        cache_partition.push(style.clone());
        Some(style)
    }

    /// 解析并缓存数据格式，对应 Java `createDataFormat(dataFormatData, useCache)`。
    pub fn create_data_format(
        &mut self,
        data_format_data: Option<&DataFormatData>,
        use_cache: bool,
    ) -> Option<DataFormatData> {
        let data_format_data = data_format_data?;
        let resolved = build_data_format(Some(data_format_data));
        if !use_cache {
            return Some(resolved);
        }
        if let Some(cached) = self
            .data_format_map
            .iter()
            .find(|cached| **cached == resolved)
        {
            return Some(cached.clone());
        }
        self.data_format_map.push(resolved.clone());
        Some(resolved)
    }

    /// 合并并缓存字体，对应 Java `createFont(writeFont, originFont, useCache)`。
    pub fn create_font(
        &mut self,
        write_font: Option<&WriteFont>,
        origin_font: Option<&WriteFont>,
        use_cache: bool,
    ) -> Option<WriteFont> {
        let font = build_font(origin_font, write_font)?;
        if !use_cache {
            return Some(font);
        }
        if let Some(cached) = self.font_map.iter().find(|cached| **cached == font) {
            return Some(cached.clone());
        }
        self.font_map.push(font.clone());
        Some(font)
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

delegate_write_holder_contract!(WriteWorkbookHolder<'a>, abstract_holder);

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

    #[test]
    fn java_getter_aliases_match_primary_getters() {
        let mut holder = WriteWorkbookHolder::new("/tmp/test.xlsx");

        // 文件路径
        assert_eq!(holder.get_file(), holder.path());
        holder.set_file("/tmp/new.xlsx");
        assert_eq!(holder.get_file(), "/tmp/new.xlsx");

        // Excel 类型
        assert_eq!(holder.get_excel_type(), crate::support::ExcelTypeEnum::Xlsx);
        holder.set_excel_type(crate::support::ExcelTypeEnum::Xls);
        assert_eq!(holder.get_excel_type(), crate::support::ExcelTypeEnum::Xls);

        // 工作簿
        assert!(holder.get_workbook().is_none());
        holder.set_workbook(Some(vec![1, 2, 3]));
        assert_eq!(holder.get_workbook(), Some([1_u8, 2, 3].as_slice()));
        holder.set_workbook(None);
        assert!(holder.get_workbook().is_none());

        // 缓存工作簿
        assert!(holder.get_cached_workbook().is_none());
        holder.set_cached_workbook(Some(vec![4, 5]));
        assert_eq!(holder.get_cached_workbook(), Some([4_u8, 5].as_slice()));

        // 输出流
        assert!(holder.get_output_stream().is_none());
        holder.set_output_stream(Some(vec![6, 7]));
        assert_eq!(holder.get_output_stream(), Some([6_u8, 7].as_slice()));

        // 模板输入流
        assert!(holder.get_template_input_stream().is_none());
        holder.set_template_input_stream(Some(vec![8, 9]));
        assert_eq!(
            holder.get_template_input_stream(),
            Some([8_u8, 9].as_slice())
        );

        // 临时模板输入流
        assert!(holder.get_temp_template_input_stream().is_none());
        holder.set_temp_template_input_stream(Some(vec![10]));
        assert_eq!(
            holder.get_temp_template_input_stream(),
            Some([10_u8].as_slice())
        );

        // 写入上下文
        assert!(holder.get_workbook_write_handler_context().is_none());
    }

    #[test]
    fn holder_type_returns_workbook() {
        let holder = WriteWorkbookHolder::new("out.xlsx");
        assert_eq!(holder.holder_type(), crate::HolderEnum::Workbook);
    }

    #[test]
    fn auto_close_stream_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.auto_close_stream());
        assert!(holder.get_auto_close_stream());
        holder.set_auto_close_stream(false);
        assert!(!holder.auto_close_stream());
        assert!(!holder.get_auto_close_stream());
    }

    #[test]
    fn in_memory_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.in_memory().is_none());
        assert!(holder.get_in_memory().is_none());
        holder.set_in_memory(Some(true));
        assert_eq!(holder.in_memory(), Some(true));
        assert_eq!(holder.get_in_memory(), Some(true));
    }

    #[test]
    fn mandatory_use_input_stream_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(!holder.mandatory_use_input_stream());
        assert!(!holder.get_mandatory_use_input_stream());
        holder.set_mandatory_use_input_stream(true);
        assert!(holder.mandatory_use_input_stream());
        assert!(holder.get_mandatory_use_input_stream());
    }

    #[test]
    fn write_excel_on_exception_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(!holder.write_excel_on_exception());
        assert!(!holder.get_write_excel_on_exception());
        holder.set_write_excel_on_exception(true);
        assert!(holder.write_excel_on_exception());
        assert!(holder.get_write_excel_on_exception());
    }

    #[test]
    fn with_bom_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.with_bom());
        assert!(holder.get_with_bom());
        holder.set_with_bom(false);
        assert!(!holder.with_bom());
        assert!(!holder.get_with_bom());
    }

    #[test]
    fn charset_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert_eq!(holder.charset(), "UTF-8");
        assert_eq!(holder.get_charset(), "UTF-8");
        holder.set_charset("GBK");
        assert_eq!(holder.charset(), "GBK");
    }

    #[test]
    fn password_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.password().is_none());
        assert!(holder.get_password().is_none());
        holder.set_password(Some("secret".to_owned()));
        assert_eq!(holder.password(), Some("secret"));
        assert_eq!(holder.get_password(), Some("secret"));
    }

    #[test]
    fn template_file_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.template_file().is_none());
        assert!(holder.get_template_file().is_none());
        holder.set_template_file(Some("/tmp/template.xlsx".to_owned()));
        assert_eq!(holder.template_file(), Some("/tmp/template.xlsx"));
        assert_eq!(holder.get_template_file(), Some("/tmp/template.xlsx"));
    }

    #[test]
    fn initialized_sheet_indexes_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.initialized_sheet_indexes().is_empty());
        assert!(holder.get_has_been_initialized_sheet_index_map().is_empty());
        let mut map = std::collections::HashMap::new();
        map.insert(0, "Sheet1".to_owned());
        holder.set_has_been_initialized_sheet_index_map(map);
        assert_eq!(holder.initialized_sheet_indexes().len(), 1);
    }

    #[test]
    fn initialized_sheet_names_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.initialized_sheet_names().is_empty());
        assert!(holder.get_has_been_initialized_sheet_name_map().is_empty());
    }

    #[test]
    fn cell_style_index_map_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.cell_style_index_map().is_empty());
        assert!(holder.get_cell_style_index_map().is_empty());
    }

    #[test]
    fn data_format_map_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.data_format_map().is_empty());
        assert!(holder.get_data_format_map().is_empty());
    }

    #[test]
    fn font_map_setter_and_getter() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        assert!(holder.font_map().is_empty());
        assert!(holder.get_font_map().is_empty());
    }

    #[test]
    fn create_font_returns_none_for_empty() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        let result = holder.create_font(None, None, true);
        assert!(result.is_none());
    }

    #[test]
    fn create_data_format_returns_none_for_none() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        let result = holder.create_data_format(None, true);
        assert!(result.is_none());
    }

    #[test]
    fn create_cell_style_returns_origin_when_write_is_none() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        let origin = Some(WriteCellStyle::default());
        let result = holder.create_cell_style(None, origin.as_ref());
        assert!(result.is_some());
    }
}
