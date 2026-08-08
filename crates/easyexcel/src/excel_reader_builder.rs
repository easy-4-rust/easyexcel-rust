//! Event-driven reader builder.
//!
//! 对应 Java：`com.alibaba.excel.read.builder.ExcelReaderBuilder`
//! （typed `read()` 路径专用；no-model Java 风格入口见
//! [`crate::read::CompatibleExcelReaderBuilder`]）。

use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;

use easyexcel_io::io::file_utils::TemporaryInput;

use crate::ExcelReader;
use crate::IntoSheetSelector;
use crate::core::{
    CellExtraType, CompositeReadListener, Converter, CsvCharset, CustomReadObject, ExcelRow,
    NullableObjectConverter, ReadDefaultReturn, ReadListener, Result,
};
use crate::read::{
    ExcelLocale, ReadCacheMode, ReadOptions, ScientificFormatMode, SheetSelector,
    StoredReadCacheSelector,
};

/// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Event-driven reader builder.
pub struct ExcelReaderBuilder<T, L> {
    pub(crate) path: PathBuf,
    pub(crate) options: ReadOptions,
    pub(crate) listener: L,
    pub(crate) temporary_input: Option<Arc<TemporaryInput>>,
    pub(crate) marker: PhantomData<T>,
}

impl<T, L> ExcelReaderBuilder<T, L>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 从路径与监听器构造一个默认配置的事件读取 builder。
    pub(crate) fn new(path: PathBuf, listener: L) -> Self {
        Self {
            path,
            options: ReadOptions::default(),
            listener,
            temporary_input: None,
            marker: PhantomData,
        }
    }

    /// 从 Java 风格的非 seekable `InputStream` 与监听器构造事件读取 builder。
    ///
    /// 输入会物化到临时文件；guard 随 builder 移交给 reader，直至读取完成后才删除。
    /// 对应 Java：`EasyExcelFactory.read(InputStream, Class, ReadListener)`。
    ///
    /// # Errors
    ///
    /// 输入读取或临时文件创建失败时返回 `ExcelError`。
    pub fn from_input_stream<R>(input: R, listener: L) -> Result<Self>
    where
        R: std::io::Read,
    {
        let temporary_input = Arc::new(easyexcel_xlsx::materialize_excel_input(input)?);
        Ok(Self {
            path: temporary_input.path().to_path_buf(),
            options: ReadOptions::default(),
            listener,
            temporary_input: Some(temporary_input),
            marker: PhantomData,
        })
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Registers another listener after the listener supplied to
    /// [`crate::EasyExcel::read`].
    ///
    /// Java appends listeners to `ReadBasicParameter.customReadListenerList`.
    /// Rust returns a builder carrying an ordered composite listener. `T:
    /// Clone` is required because Rust listeners take ownership of each row,
    /// whereas Java listeners share the same object reference.
    #[must_use]
    pub fn register_read_listener<Next>(
        self,
        listener: Next,
    ) -> ExcelReaderBuilder<T, CompositeReadListener<T, L, Next>>
    where
        T: Clone,
        Next: ReadListener<T>,
    {
        ExcelReaderBuilder {
            path: self.path,
            options: self.options,
            listener: CompositeReadListener::new(self.listener, listener),
            temporary_input: self.temporary_input,
            marker: PhantomData,
        }
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Selects a worksheet by name or zero-based index.
    #[must_use]
    pub fn sheet(mut self, sheet: impl IntoSheetSelector) -> Self {
        self.options.sheet = sheet.into_sheet_selector();
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Selects every worksheet in workbook order.
    #[must_use]
    pub fn all_sheets(mut self) -> Self {
        self.options.sheet = SheetSelector::All;
        self
    }

    /// Sets the number of header rows.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn head_row_number(mut self, rows: u32) -> Self {
        self.options.head_row_number = rows;
        self
    }

    /// Configures empty-row filtering.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn ignore_empty_row(mut self, ignore: bool) -> Self {
        self.options.ignore_empty_row = ignore;
        self
    }

    /// Enables or disables Java EasyExcel-compatible string trimming.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn auto_trim(mut self, enabled: bool) -> Self {
        self.options.auto_trim = enabled;
        self
    }

    /// Selects Excel's 1904 date windowing system for numeric date cells.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn use_1904_windowing(mut self, enabled: bool) -> Self {
        self.options.use_1904_windowing = enabled;
        self
    }

    /// Controls scientific notation for extreme General-format numeric cells.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.options.scientific_format = if enabled {
            ScientificFormatMode::Scientific
        } else {
            ScientificFormatMode::Plain
        };
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the locale used for formatted number and date display values.
    #[must_use]
    pub fn locale(mut self, locale: ExcelLocale) -> Self {
        self.options.locale = locale;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Registers a Java-style global converter for this read operation.
    #[must_use]
    pub fn register_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: Converter<V> + Send + Sync + 'static,
    {
        self.options.converters.register::<V, C>(converter);
        self
    }

    /// Registers a converter that receives empty cells.
    ///
    /// 对应 Java：'s `registerConverter(NullableObjectConverter)`.
    #[must_use]
    pub fn register_nullable_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: NullableObjectConverter<V> + Send + Sync + 'static,
    {
        self.options.converters.register_nullable::<V, C>(converter);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Selects the XLSX shared-string cache backend.
    #[must_use]
    pub fn read_cache(mut self, mode: ReadCacheMode) -> Self {
        self.options.read_cache = mode;
        self.options.read_cache_selector = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Installs a Java-style cache selector. (Java `readCacheSelector(ReadCacheSelector)`)
    #[must_use]
    pub fn read_cache_selector(mut self, selector: StoredReadCacheSelector) -> Self {
        self.options.read_cache_selector = Some(selector);
        self
    }

    /// 控制是否安装默认模型构建监听器。对应 Java：`useDefaultListener(Boolean)`。
    #[must_use]
    pub const fn use_default_listener(mut self, enabled: bool) -> Self {
        self.options.use_default_listener = enabled;
        self
    }

    /// 设置 XLSX SAX 解析器工厂名。对应 Java：`xlsxSAXParserFactoryName(String)`。
    #[must_use]
    pub fn xlsx_saxparser_factory_name(mut self, name: impl Into<String>) -> Self {
        self.options.xlsx_sax_parser_factory_name = Some(name.into());
        self
    }

    /// Sets the first physical data row to dispatch, zero-based and inclusive.
    ///
    /// Configured header rows are still analysed for name-based mapping.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn start_row(mut self, row: u32) -> Self {
        self.options.start_row = Some(row);
        self
    }

    /// Sets the last physical data row to dispatch, zero-based and inclusive.
    ///
    /// Configured header rows are still analysed for name-based mapping.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn end_row(mut self, row: u32) -> Self {
        self.options.end_row = Some(row);
        self
    }

    /// Limits data callbacks to an inclusive physical row range.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn read_rows(mut self, start: u32, end: u32) -> Self {
        self.options.start_row = Some(start);
        self.options.end_row = Some(end);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Maps a workbook header name to the name used by typed row mapping.
    #[must_use]
    pub fn header_alias(mut self, header: impl Into<String>, alias: impl Into<String>) -> Self {
        self.options
            .header_aliases
            .insert(header.into(), alias.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Stores a type-safe value exposed by every read callback context.
    #[must_use]
    pub fn custom_object<C>(mut self, custom_object: C) -> Self
    where
        C: std::any::Any + Send + Sync,
    {
        self.options.custom_object = Some(CustomReadObject::new(custom_object));
        self
    }

    /// Selects the Java-compatible no-model return mode.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn read_default_return(mut self, mode: ReadDefaultReturn) -> Self {
        self.options.read_default_return = mode;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Enables a Java `extraRead` metadata category.
    #[must_use]
    pub fn extra_read(mut self, extra_type: CellExtraType) -> Self {
        self.options.extra_read.insert(extra_type);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the password for an encrypted OOXML workbook.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.options.password = Some(password.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the character encoding used for CSV input.
    #[must_use]
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.options.charset = charset.into();
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Executes the read and consumes the builder.
    ///
    /// # Errors
    ///
    /// Returns a workbook, sheet-selection, conversion, or listener error.
    pub fn do_read(self) -> Result<()> {
        let mut reader = match self.temporary_input {
            Some(temporary_input) => ExcelReader::from_temporary_input(
                self.path,
                temporary_input,
                self.options,
                self.listener,
            )?,
            None => ExcelReader::new(self.path, self.options, self.listener)?,
        };
        reader.read_all()
    }
}

#[cfg(test)]
mod unified_dispatch_tests {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::core::{AnalysisContext, DynamicRow};

    struct CountingListener(Arc<AtomicUsize>);

    impl ReadListener<DynamicRow> for CountingListener {
        fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn extensionless_csv() -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "name")?;
        writeln!(file, "alice")?;
        Ok(file)
    }

    #[test]
    fn event_and_sync_facades_share_magic_based_executor_selection() -> Result<()> {
        let file = extensionless_csv()?;
        let calls = Arc::new(AtomicUsize::new(0));
        crate::EasyExcel::read_dynamic(file.path(), CountingListener(Arc::clone(&calls)))
            .do_read()?;
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let rows = crate::EasyExcel::read_dynamic_sync(file.path()).do_read_sync()?;
        assert_eq!(rows.len(), 1);
        Ok(())
    }
}
