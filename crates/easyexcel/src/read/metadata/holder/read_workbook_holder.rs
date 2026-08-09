//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadWorkbookHolder`.

use crate::context::read_sheet::ReadSheet;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use super::abstract_read_holder::AbstractReadHolder;
use super::read_holder::delegate_read_holder_contract;

/// 对应 Java：`ReadWorkbookHolder extends AbstractReadHolder`.
///
/// Java carries 17 fields. Rust collapses them into the `ReadOptions`
/// struct that already lives in the reader facade. This struct exists
/// for 1:1 API parity.
#[derive(Debug, Clone)]
pub struct ReadWorkbookHolder {
    abstract_holder: AbstractReadHolder,
    /// 原始 Java `ReadWorkbook` 参数快照。
    pub read_workbook: Option<crate::ReadWorkbook>,
    /// 显式或识别出的工作簿格式。
    pub excel_type: Option<crate::support::ExcelTypeEnum>,
    /// Owned input bytes used as a backend-neutral InputStream handle.
    pub input_stream: Option<Vec<u8>>,
    /// Mirrors `ReadWorkbookHolder.charset`.
    pub charset: crate::core::CsvCharset,
    /// Mirrors `ReadWorkbookHolder.autoCloseStream`.
    pub auto_close_stream: bool,
    /// Mirrors `ReadWorkbookHolder.ignoreEmptyRow`.
    pub ignore_empty_row: bool,
    /// Mirrors `ReadWorkbookHolder.password`.
    pub password: Option<String>,
    /// Workbooks sheets discovered by the format executor.
    ///
    /// Mirrors `ReadWorkbookHolder.actualSheetDataList`.
    pub actual_sheet_data_list: Option<Vec<ReadSheet>>,
    /// Mirrors `mandatoryUseInputStream`。
    pub mandatory_use_input_stream: bool,
    /// Mirrors `readDefaultReturn`。
    pub read_default_return: crate::core::ReadDefaultReturn,
    /// Mirrors `customObject`。
    pub custom_object: Option<crate::core::CustomReadObject>,
    /// Mirrors `readCache`。
    pub read_cache: crate::read::read_cache::ReadCacheMode,
    /// Mirrors `readCacheSelector`。
    pub read_cache_selector: Option<crate::read::stored_read_cache_selector::StoredReadCacheSelector>,
    /// Mirrors `extraReadSet`。
    pub extra_read_set: HashSet<crate::core::CellExtraType>,
    /// Sheets explicitly requested by the caller。
    pub parameter_sheet_data_list: Option<Vec<ReadSheet>>,
    /// Whether the reader executes every discovered sheet。
    pub read_all: bool,
    /// Sheet numbers already consumed。
    pub has_read_sheet: HashSet<u32>,
    /// Source file retained for holder observability。
    pub file: Option<PathBuf>,
    /// Temporary source file retained for holder observability。
    pub temp_file: Option<PathBuf>,
}

impl Default for ReadWorkbookHolder {
    /// Java `ReadWorkbookHolder(ReadWorkbook)`：`autoCloseStream` 未指定时为
    /// `Boolean.TRUE`（`if (readWorkbook.getAutoCloseStream() == null) ... TRUE`），
    /// 因此 Default 与 `new()` 的自动关闭语义保持一致。
    fn default() -> Self {
        Self {
            abstract_holder: AbstractReadHolder::default(),
            read_workbook: None,
            excel_type: None,
            input_stream: None,
            charset: crate::core::CsvCharset::default(),
            auto_close_stream: true,
            ignore_empty_row: false,
            password: None,
            actual_sheet_data_list: None,
            mandatory_use_input_stream: false,
            read_default_return: crate::core::ReadDefaultReturn::default(),
            custom_object: None,
            read_cache: crate::read::read_cache::ReadCacheMode::default(),
            read_cache_selector: None,
            extra_read_set: HashSet::new(),
            parameter_sheet_data_list: None,
            read_all: false,
            has_read_sheet: HashSet::new(),
            file: None,
            temp_file: None,
        }
    }
}

impl ReadWorkbookHolder {
    /// Java 无参构造器。
    #[must_use] pub fn new() -> Self { Self::default() }
    /// Java `ReadWorkbookHolder(ReadWorkbook)`。
    #[must_use]
    pub fn from_read_workbook(value: crate::ReadWorkbook) -> Self {
        let mut holder = Self::from_options(&value.options);
        holder.abstract_holder = AbstractReadHolder::from_parameter(
            value.get_read_basic_parameter(),
            None,
            crate::HolderEnum::Workbook,
        );
        holder.input_stream = value.get_input_stream().map(<[u8]>::to_vec);
        holder.file = value.file().map(Path::to_path_buf);
        holder.excel_type = value.excel_type();
        holder.auto_close_stream = value.get_auto_close_stream().unwrap_or(true);
        holder.ignore_empty_row = value.get_ignore_empty_row().unwrap_or(true);
        holder.mandatory_use_input_stream = value
            .get_mandatory_use_input_stream()
            .unwrap_or(false);
        holder.read_workbook = Some(value);
        holder
    }
    #[must_use] pub const fn get_read_workbook(&self) -> Option<&crate::ReadWorkbook> {
        self.read_workbook.as_ref()
    }
    pub fn set_read_workbook(&mut self, value: Option<crate::ReadWorkbook>) {
        self.read_workbook = value;
    }
    #[must_use] pub const fn get_excel_type(&self) -> Option<crate::support::ExcelTypeEnum> {
        self.excel_type
    }
    pub const fn set_excel_type(&mut self, value: Option<crate::support::ExcelTypeEnum>) {
        self.excel_type = value;
    }
    #[must_use] pub fn get_input_stream(&self) -> Option<&[u8]> { self.input_stream.as_deref() }
    pub fn set_input_stream(&mut self, value: Option<Vec<u8>>) { self.input_stream = value; }
    /// Java `getCharset`。
    #[must_use] pub const fn get_charset(&self) -> &crate::core::CsvCharset { &self.charset }
    /// Java `setCharset`。
    pub fn set_charset(&mut self, value: crate::core::CsvCharset) { self.charset = value; }
    /// Java `getAutoCloseStream`。
    #[must_use] pub const fn get_auto_close_stream(&self) -> bool { self.auto_close_stream }
    /// Java `getIgnoreEmptyRow`。
    #[must_use] pub const fn get_ignore_empty_row(&self) -> bool { self.ignore_empty_row }
    /// Java `getPassword`。
    #[must_use] pub fn get_password(&self) -> Option<&str> { self.password.as_deref() }
    /// Java `getMandatoryUseInputStream`。
    #[must_use] pub const fn get_mandatory_use_input_stream(&self) -> bool { self.mandatory_use_input_stream }
    /// Java `getReadDefaultReturn`。
    #[must_use] pub const fn get_read_default_return(&self) -> crate::core::ReadDefaultReturn { self.read_default_return }
    /// Java `setReadDefaultReturn`。
    pub const fn set_read_default_return(&mut self, value: crate::core::ReadDefaultReturn) { self.read_default_return = value; }
    /// Java `getCustomObject`。
    #[must_use] pub const fn get_custom_object(&self) -> Option<&crate::core::CustomReadObject> { self.custom_object.as_ref() }
    /// Java `setCustomObject`。
    pub fn set_custom_object(&mut self, value: Option<crate::core::CustomReadObject>) { self.custom_object = value; }
    /// Java `getReadCache`。
    #[must_use] pub const fn get_read_cache(&self) -> crate::read::read_cache::ReadCacheMode { self.read_cache }
    /// Java `setReadCache`。
    pub const fn set_read_cache(&mut self, value: crate::read::read_cache::ReadCacheMode) { self.read_cache = value; }
    /// Java `getReadCacheSelector`。
    #[must_use] pub const fn get_read_cache_selector(&self) -> Option<&crate::read::stored_read_cache_selector::StoredReadCacheSelector> { self.read_cache_selector.as_ref() }
    /// Java `setReadCacheSelector`。
    pub fn set_read_cache_selector(&mut self, value: Option<crate::read::stored_read_cache_selector::StoredReadCacheSelector>) { self.read_cache_selector = value; }
    /// Java `getExtraReadSet`。
    #[must_use] pub const fn get_extra_read_set(&self) -> &HashSet<crate::core::CellExtraType> { &self.extra_read_set }
    /// Java `setExtraReadSet`。
    pub fn set_extra_read_set(&mut self, value: HashSet<crate::core::CellExtraType>) { self.extra_read_set = value; }
    /// Java `getActualSheetDataList`。
    #[must_use] pub fn get_actual_sheet_data_list(&self) -> Option<&[ReadSheet]> { self.actual_sheet_data_list.as_deref() }
    /// Java `getParameterSheetDataList`。
    #[must_use] pub fn get_parameter_sheet_data_list(&self) -> Option<&[ReadSheet]> { self.parameter_sheet_data_list.as_deref() }
    /// Java `getHasReadSheet`。
    #[must_use] pub const fn get_has_read_sheet(&self) -> &HashSet<u32> { &self.has_read_sheet }
    /// Java `setHasReadSheet`。
    pub fn set_has_read_sheet(&mut self, value: HashSet<u32>) { self.has_read_sheet = value; }
    /// Java `getReadAll`。
    #[must_use] pub const fn get_read_all(&self) -> bool { self.read_all }
    /// Java `getFile`。
    #[must_use] pub fn get_file(&self) -> Option<&Path> { self.file.as_deref() }
    /// Java `getTempFile`。
    #[must_use] pub fn get_temp_file(&self) -> Option<&Path> { self.temp_file.as_deref() }
    #[must_use] pub const fn holder_type(&self) -> crate::HolderEnum {
        crate::HolderEnum::Workbook
    }
    /// 返回父类读取 Holder。
    #[must_use] pub const fn abstract_holder(&self) -> &AbstractReadHolder { &self.abstract_holder }
    /// 返回可变父类读取 Holder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractReadHolder { &mut self.abstract_holder }

    /// Resolves workbook-level holder state from the public read options.
    ///
    /// 对应 Java：`ReadWorkbookHolder(ReadWorkbook, ...)` propagation before
    /// a format-specific context is constructed.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            abstract_holder: AbstractReadHolder::from_parameter(
                &crate::read::metadata::ReadBasicParameter::from_options(options),
                None,
                crate::HolderEnum::Workbook,
            ),
            read_workbook: None,
            excel_type: None,
            input_stream: None,
            charset: options.charset.clone(),
            auto_close_stream: true,
            ignore_empty_row: options.ignore_empty_row,
            password: options.password.clone(),
            actual_sheet_data_list: None,
            mandatory_use_input_stream: false,
            read_default_return: options.read_default_return,
            custom_object: options.custom_object.clone(),
            read_cache: options.read_cache,
            read_cache_selector: options.read_cache_selector.clone(),
            extra_read_set: options.extra_read.clone(),
            parameter_sheet_data_list: None,
            read_all: false,
            has_read_sheet: HashSet::new(),
            file: None,
            temp_file: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.holder.ReadWorkbookHolder。 Returns format-discovered sheets in workbook order.
    #[must_use]
    pub fn actual_sheet_data_list(&self) -> Option<&[ReadSheet]> {
        self.actual_sheet_data_list.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.holder.ReadWorkbookHolder。 Stores format-discovered sheets.
    pub fn set_actual_sheet_data_list(
        &mut self,
        sheets: impl Into<Option<Vec<ReadSheet>>>,
    ) {
        self.actual_sheet_data_list = sheets.into();
    }

    /// 返回调用参数 Sheet 列表。
    #[must_use]
    pub fn parameter_sheet_data_list(&self) -> Option<&[ReadSheet]> {
        self.parameter_sheet_data_list.as_deref()
    }

    /// 设置调用参数 Sheet 列表。
    pub fn set_parameter_sheet_data_list(
        &mut self,
        sheets: impl Into<Option<Vec<ReadSheet>>>,
    ) {
        self.parameter_sheet_data_list = sheets.into();
    }

    /// 标记一个 Sheet 已读取，返回是否首次插入。
    pub fn mark_sheet_read(&mut self, sheet_no: u32) -> bool {
        self.has_read_sheet.insert(sheet_no)
    }

    /// 返回已经读取的 Sheet 编号集合。
    #[must_use]
    pub fn has_read_sheet(&self) -> &HashSet<u32> { &self.has_read_sheet }
    /// 返回读取全部 Sheet 开关。
    #[must_use]
    pub const fn read_all(&self) -> bool { self.read_all }
    /// 设置读取全部 Sheet 开关。
    pub const fn set_read_all(&mut self, value: bool) { self.read_all = value; }
    /// 返回强制输入流开关。
    #[must_use]
    pub const fn mandatory_use_input_stream(&self) -> bool { self.mandatory_use_input_stream }
    /// 设置强制输入流开关。
    pub const fn set_mandatory_use_input_stream(&mut self, value: bool) { self.mandatory_use_input_stream = value; }
    /// 返回自动关闭输入流开关。
    #[must_use]
    pub const fn auto_close_stream(&self) -> bool { self.auto_close_stream }
    /// 设置自动关闭输入流开关。
    pub const fn set_auto_close_stream(&mut self, value: bool) { self.auto_close_stream = value; }
    /// 返回忽略空行开关。
    #[must_use]
    pub const fn ignore_empty_row(&self) -> bool { self.ignore_empty_row }
    /// 设置忽略空行开关。
    pub const fn set_ignore_empty_row(&mut self, value: bool) { self.ignore_empty_row = value; }
    /// 返回调用级密码。
    #[must_use]
    pub fn password(&self) -> Option<&str> { self.password.as_deref() }
    /// 设置调用级密码。
    pub fn set_password(&mut self, value: Option<String>) { self.password = value; }
    /// 返回源文件。
    #[must_use]
    pub fn file(&self) -> Option<&Path> { self.file.as_deref() }
    /// 设置源文件。
    pub fn set_file(&mut self, value: Option<PathBuf>) { self.file = value; }
    /// 返回临时文件。
    #[must_use]
    pub fn temp_file(&self) -> Option<&Path> { self.temp_file.as_deref() }
    /// 设置临时文件。
    pub fn set_temp_file(&mut self, value: Option<PathBuf>) { self.temp_file = value; }
}

impl Deref for ReadWorkbookHolder {
    type Target = AbstractReadHolder;
    fn deref(&self) -> &Self::Target { &self.abstract_holder }
}

impl DerefMut for ReadWorkbookHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.abstract_holder }
}

delegate_read_holder_contract!(ReadWorkbookHolder, abstract_holder);
