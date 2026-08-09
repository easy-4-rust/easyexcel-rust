//! 对应 Java：`com.alibaba.excel.read.builder.ExcelReaderBuilder`.

use std::cell::RefCell;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use easyexcel_io::io::file_utils::TemporaryInput;

use crate::core::{
    AnalysisContext, CellExtraType, Converter, CsvCharset, CustomReadObject, ExcelRow,
    NullableObjectConverter, ReadDefaultReturn, ReadListener, ReadListenerList, Result,
};

use crate::cache::SimpleReadCacheSelector;
use crate::read::builder::abstract_excel_reader_parameter_builder::AbstractExcelReaderParameterBuilder;
use crate::read::excel_reader::ExcelReader;
use crate::{
    ExcelLocale, ExcelTypeEnum, ReadCacheMode, ReadOptions, SheetSelector,
    StoredReadCacheSelector,
};

/// 对应 Java：`ExcelReaderBuilder extends AbstractExcelReaderParameterBuilder`.
#[derive(Debug, Clone, Default)]
pub struct ExcelReaderBuilder {
    /// Mirrors `ReadWorkbook.file`.
    pub file: Option<PathBuf>,
    /// Guard for a caller-supplied Java-style `InputStream`.
    temporary_input: Option<Arc<TemporaryInput>>,
    /// Java `ReadWorkbook.excelType`；`None` 时由格式探测引擎决定。
    explicit_excel_type: Option<ExcelTypeEnum>,
    /// Java 流关闭开关；Rust 中实际关闭行为由传值或 `&mut` 借用所有权决定。
    auto_close_stream: Option<bool>,
    /// Java 强制流路径开关；Rust 的非 seekable 输入始终物化，因此记录该意图即可。
    mandatory_use_input_stream: Option<bool>,
    /// Collapsed read options from Java `ReadWorkbook` + parameter builders.
    pub options: ReadOptions,
}

impl ExcelReaderBuilder {
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Creates a builder. (Java `new ExcelReaderBuilder()`)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the file path. (Java `file(String pathName)`)
    #[must_use]
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file = Some(path.into());
        self.temporary_input = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Materialises a caller-supplied input stream into an automatically
    /// deleted file and selects the existing XLSX/XLS/CSV parsing engine.
    ///
    /// Java accepts a non-seekable `InputStream`, while XLSX and XLS readers
    /// need random access. The temporary file is retained by the resulting
    /// [`ExcelReader`] and deleted only after that reader is dropped.
    ///
    /// # Errors
    ///
    /// 当输入流读取失败或临时文件创建/写入失败时返回 `ExcelError`。
    pub fn input_stream<R>(mut self, input: R) -> Result<Self>
    where
        R: Read,
    {
        let temporary_input = Arc::new(easyexcel_xlsx::materialize_excel_input(input)?);
        self.file = Some(temporary_input.path().to_path_buf());
        self.temporary_input = Some(temporary_input);
        Ok(self)
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Selects a worksheet by zero-based index. (Java `sheet(Integer)`)
    #[must_use]
    pub fn sheet(mut self, index: usize) -> Self {
        self.options.sheet = SheetSelector::Index(index);
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Selects a worksheet by name. (Java `sheet(String)`)
    #[must_use]
    pub fn sheet_name(mut self, name: impl Into<String>) -> Self {
        self.options.sheet = SheetSelector::Name(name.into());
        self
    }

    /// Sets the number of header rows. (Java `headRowNumber(Integer)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn head_row_number(mut self, rows: u32) -> Self {
        self.options.head_row_number = rows;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the character encoding used for CSV input. (Java `charset(Charset)`)
    #[must_use]
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.options.charset = charset.into();
        self
    }

    /// 显式指定工作簿格式并优先于路径扩展名推断。
    /// 对应 Java：`ExcelReaderBuilder.excelType(ExcelTypeEnum)`。
    #[must_use]
    pub const fn excel_type(mut self, excel_type: ExcelTypeEnum) -> Self {
        self.explicit_excel_type = Some(excel_type);
        self
    }

    /// 记录 Java `autoCloseStream(Boolean)` 配置。
    ///
    /// Rust 调用者传入拥有值时随值释放，传入 `&mut R` 时底层流仍归调用者所有。
    #[must_use]
    pub const fn auto_close_stream(mut self, enabled: bool) -> Self {
        self.auto_close_stream = Some(enabled);
        self
    }

    /// 记录 Java `mandatoryUseInputStream(Boolean)` 配置。
    ///
    /// Rust 格式引擎对非 seekable 输入始终采用临时文件物化，等价于安全的强制流路径。
    #[must_use]
    pub const fn mandatory_use_input_stream(mut self, enabled: bool) -> Self {
        self.mandatory_use_input_stream = Some(enabled);
        self
    }

    /// 设置字符串自动裁剪。对应 Java：`AbstractParameterBuilder.autoTrim(Boolean)`。
    #[must_use]
    pub const fn auto_trim(mut self, enabled: bool) -> Self {
        self.options.auto_trim = enabled;
        self
    }

    /// 设置 1904 日期窗口。对应 Java：`AbstractParameterBuilder.use1904windowing(Boolean)`。
    #[must_use]
    pub const fn use_1904_windowing(mut self, enabled: bool) -> Self {
        self.options.use_1904_windowing = enabled;
        self
    }

    /// 设置数字和日期格式化区域。
    #[must_use]
    pub fn locale(mut self, locale: ExcelLocale) -> Self {
        self.options.locale = locale;
        self
    }

    /// 注册强类型读取转换器。
    #[must_use]
    pub fn register_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: Converter<V> + Send + Sync + 'static,
    {
        self.options.converters.register::<V, C>(converter);
        self
    }

    /// 注册可接收空单元格的强类型读取转换器。
    #[must_use]
    pub fn register_nullable_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: NullableObjectConverter<V> + Send + Sync + 'static,
    {
        self.options.converters.register_nullable::<V, C>(converter);
        self
    }

    /// Controls whether physically empty rows are skipped.
    /// (Java `ignoreEmptyRow(Boolean)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn ignore_empty_row(mut self, ignore: bool) -> Self {
        self.options.ignore_empty_row = ignore;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Stores a value exposed through [`AnalysisContext::custom_object`].
    /// (Java `customObject(Object)`)
    #[must_use]
    pub fn custom_object<C>(mut self, custom_object: C) -> Self
    where
        C: std::any::Any + Send + Sync,
    {
        self.options.custom_object = Some(CustomReadObject::new(custom_object));
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the workbook password used by encrypted OOXML input.
    /// (Java `password(String)`)
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.options.password = Some(password.into());
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Enables one additional metadata category.
    /// (Java `extraRead(CellExtraTypeEnum)`)
    #[must_use]
    pub fn extra_read(mut self, extra_type: CellExtraType) -> Self {
        self.options.extra_read.insert(extra_type);
        self
    }

    /// Selects the no-model value representation.
    /// (Java `readDefaultReturn(ReadDefaultReturnEnum)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。
    pub const fn read_default_return(mut self, mode: ReadDefaultReturn) -> Self {
        self.options.read_default_return = mode;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Controls scientific formatting.
    #[must_use]
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.options.scientific_format = if enabled {
            crate::ScientificFormatMode::Scientific
        } else {
            crate::ScientificFormatMode::Plain
        };
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Sets the shared-string cache mode directly. (Java `readCache(ReadCache)`)
    #[must_use]
    pub fn read_cache(mut self, mode: ReadCacheMode) -> Self {
        self.options.read_cache = mode;
        self.options.read_cache_selector = None;
        self
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Installs a cache selector. (Java `readCacheSelector(ReadCacheSelector)`)
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

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Installs Java's default simple selector.
    #[must_use]
    pub fn simple_read_cache_selector(self, selector: SimpleReadCacheSelector) -> Self {
        self.read_cache_selector(StoredReadCacheSelector::Simple(selector))
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Registers the first typed listener and returns a builder that owns it.
    ///
    /// This is the Rust equivalent of Java
    /// `ExcelReaderBuilder.registerReadListener(...)`: the listener becomes
    /// part of the builder state and is passed into the built [`ExcelReader`].
    /// The older [`Self::build`] overload that accepts a listener explicitly
    /// remains available for source compatibility.
    #[must_use]
    pub fn register_read_listener<T, L>(self, listener: L) -> RegisteredExcelReaderBuilder<T>
    where
        T: ExcelRow + Clone,
        L: ReadListener<T> + 'static,
    {
        RegisteredExcelReaderBuilder {
            builder: self,
            listeners: ReadListenerList::new(listener),
        }
    }

    /// Builds an event-driven reader. (Java `build()`)
    ///
    /// # Errors
    ///
    /// 当未设置 `file`（对应 Java 抛异常）或工作簿打开失败时返回 `ExcelError`。
    pub fn build<T, L>(self, listener: L) -> Result<ExcelReader<T, L>>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        // Java 的开关影响 InputStream 生命周期/是否先落盘；Rust 分别由所有权和
        // 非 seekable 输入必经的 TemporaryInput 协议承载，保留配置意图但不伪造句柄。
        let _stream_policy = (
            self.auto_close_stream.unwrap_or(true),
            self.mandatory_use_input_stream.unwrap_or(false),
        );
        let path = self.file.ok_or_else(|| {
            crate::core::ExcelError::Format(
                "ExcelReaderBuilder.file must be set before build()".to_owned(),
            )
        })?;
        match self.temporary_input {
            Some(temporary_input) => ExcelReader::from_temporary_input_with_explicit_type(
                path,
                temporary_input,
                self.options,
                listener,
                self.explicit_excel_type,
            ),
            None => ExcelReader::new_with_explicit_type(
                path,
                self.options,
                listener,
                self.explicit_excel_type,
            ),
        }
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Builds and immediately reads all configured sheets. (Java `doReadAll()`)
    ///
    /// # Errors
    ///
    /// 当构建失败（未设置 `file`）或任一工作表解析失败时返回 `ExcelError`。
    pub fn do_read_all<T, L>(self, listener: L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        let mut reader = self.build(listener)?;
        reader.read_all()
    }

    /// 对应 Java：com.alibaba.excel.read.builder.ExcelReaderBuilder。 Reads synchronously and returns all converted rows.
    /// (Java `doReadAllSync()`)
    ///
    /// # Errors
    ///
    /// 当构建失败（未设置 `file`）或任一工作表解析失败时返回 `ExcelError`。
    pub fn do_read_all_sync<T>(self) -> Result<Vec<T>>
    where
        T: ExcelRow,
    {
        let rows = Rc::new(RefCell::new(Vec::new()));
        let listener = SharedCollectListener(Rc::clone(&rows));
        let mut reader = self.build(listener)?;
        reader.read_all()?;
        reader.finish();
        drop(reader);
        let collected = std::mem::take(&mut *rows.borrow_mut());
        Ok(collected)
    }
}

include!("excel_reader_builder/registered_excel_reader_builder.rs");

include!("excel_reader_builder/shared_collect_listener.rs");

#[cfg(test)]
fn input_suffix(bytes: &[u8]) -> &'static str {
    easyexcel_xlsx::excel_input_suffix(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::EternalReadCacheSelector;
    use crate::core::DynamicRow;
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::NamedTempFile;

    #[derive(Default)]
    struct CollectListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for CollectListener {
        fn invoke(
            &mut self,
            data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    struct CountingListener(Arc<AtomicUsize>);

    impl ReadListener<DynamicRow> for CountingListener {
        fn invoke(
            &mut self,
            _data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn builder_applies_eternal_cache_selector_via_read_cache() {
        let builder = ExcelReaderBuilder::new().read_cache_selector(
            StoredReadCacheSelector::Eternal(EternalReadCacheSelector::map_cache()),
        );
        assert!(builder.options.read_cache_selector.is_some());
    }

    #[test]
    fn builder_reads_csv_file() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "bob,22")?;
        ExcelReaderBuilder::new()
            .file(file.path())
            .head_row_number(1)
            .do_read_all(CollectListener::default())?;
        Ok(())
    }

    #[test]
    fn registered_listeners_are_owned_and_invoked_by_the_builder() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "bob,22")?;
        let first = Arc::new(AtomicUsize::new(0));
        let second = Arc::new(AtomicUsize::new(0));

        let mut builder = ExcelReaderBuilder::new()
            .file(file.path())
            .register_read_listener::<DynamicRow, _>(CountingListener(Arc::clone(&first)));
        AbstractExcelReaderParameterBuilder::<DynamicRow>::register_read_listener(
            &mut builder,
            Box::new(CountingListener(Arc::clone(&second))),
        );
        builder.do_read_all()?;

        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[test]
    fn builder_do_read_all_sync_returns_rows() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "bob,22")?;

        let rows = ExcelReaderBuilder::new()
            .file(file.path())
            .do_read_all_sync::<DynamicRow>()?;

        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get(0),
            Some(&crate::core::DynamicValue::String("bob".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn registered_builder_sync_read_keeps_existing_listeners() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "bob,22")?;
        let invoked = Arc::new(AtomicUsize::new(0));

        let rows = ExcelReaderBuilder::new()
            .file(file.path())
            .register_read_listener::<DynamicRow, _>(CountingListener(Arc::clone(&invoked)))
            .do_read_all_sync()?;

        assert_eq!(invoked.load(Ordering::SeqCst), 1);
        assert_eq!(rows.len(), 1);
        Ok(())
    }

    #[test]
    fn builder_stores_java_workbook_options_in_real_read_options() {
        let builder = ExcelReaderBuilder::new()
            .charset("gbk")
            .ignore_empty_row(false)
            .custom_object(42_u32)
            .password("secret")
            .extra_read(CellExtraType::Comment)
            .read_default_return(ReadDefaultReturn::ActualData);

        assert!(!builder.options.ignore_empty_row);
        assert_eq!(builder.options.password.as_deref(), Some("secret"));
        assert!(builder.options.custom_object.is_some());
        assert!(builder.options.extra_read.contains(&CellExtraType::Comment));
        assert_eq!(
            builder.options.read_default_return,
            ReadDefaultReturn::ActualData
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::core::DynamicRow;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Default)]
    struct ExtraCollectListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for ExtraCollectListener {
        fn invoke(
            &mut self,
            data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    #[test]
    fn sheet_and_sheet_name_selectors_are_stored() {
        // 对应 Java：ExcelReaderBuilder.sheet(Integer)/sheet(String)
        let builder = ExcelReaderBuilder::new().sheet(2);
        assert_eq!(builder.options.sheet, SheetSelector::Index(2));
        let builder = ExcelReaderBuilder::new().sheet_name("Sheet2");
        assert_eq!(
            builder.options.sheet,
            SheetSelector::Name("Sheet2".to_owned())
        );
    }

    #[test]
    fn use_scientific_format_toggles_the_mode() {
        // 对应 Java：ExcelReaderBuilder.useScientificFormat(Boolean)
        assert_eq!(
            ExcelReaderBuilder::new()
                .use_scientific_format(true)
                .options
                .scientific_format,
            crate::ScientificFormatMode::Scientific
        );
        assert_eq!(
            ExcelReaderBuilder::new()
                .use_scientific_format(false)
                .options
                .scientific_format,
            crate::ScientificFormatMode::Plain
        );
    }

    #[test]
    fn read_cache_and_simple_selector_wire_into_options() {
        // 对应 Java：ExcelReaderBuilder.readCache(ReadCache)/readCacheSelector(...)
        let builder = ExcelReaderBuilder::new().read_cache(ReadCacheMode::Memory);
        assert_eq!(builder.options.read_cache, ReadCacheMode::Memory);
        assert!(builder.options.read_cache_selector.is_none());

        let builder = ExcelReaderBuilder::new()
            .read_cache(ReadCacheMode::File)
            .simple_read_cache_selector(SimpleReadCacheSelector::new());
        assert_eq!(builder.options.read_cache, ReadCacheMode::File);
        assert!(matches!(
            builder.options.read_cache_selector,
            Some(StoredReadCacheSelector::Simple(_))
        ));
    }

    #[test]
    fn build_without_file_fails_like_java() {
        // 对应 Java：build() 在未设置 file 时抛出异常
        let Err(error) =
            ExcelReaderBuilder::new().build::<DynamicRow, _>(ExtraCollectListener::default())
        else {
            panic!("missing file must fail");
        };
        assert!(error.to_string().contains("file must be set"));

        let Err(error) =
            ExcelReaderBuilder::new().do_read_all::<DynamicRow, _>(ExtraCollectListener::default())
        else {
            panic!("missing file must fail");
        };
        assert!(error.to_string().contains("file must be set"));
    }

    #[test]
    fn input_suffix_detects_ole_xls_workbooks() -> Result<()> {
        // 对应 Java：根据文件头识别格式；OLE(CFB) 且不含加密流 → xls
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("plain.xls");
        // 仅写 OLE 魔数（D0 CF 11 E0 A1 B1 1A E1），cfb 无法解析 → 视为普通 .xls
        std::fs::write(&path, [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])?;
        let bytes = std::fs::read(&path)?;
        assert_eq!(input_suffix(&bytes), ".xls");
        // PK 头 → xlsx，其余 → csv（对应 Java：XSSF/HSSF/CSV 识别顺序）
        assert_eq!(input_suffix(b"PK\x03\x04rest"), ".xlsx");
        assert_eq!(input_suffix(b"not a workbook"), ".csv");
        Ok(())
    }

    #[test]
    fn input_stream_materialises_callers_bytes_to_a_temp_file() -> Result<()> {
        // 对应 Java：InputStream 写入临时文件后按内容识别格式
        let builder = ExcelReaderBuilder::new().input_stream(std::io::Cursor::new(b"a,b\n1,2"))?;
        assert!(builder.file.is_some());
        let path = builder.file.clone().expect("materialised file");
        assert_eq!(
            std::fs::read_to_string(&path)?,
            "a,b\n1,2",
            "temporary file must carry the input bytes"
        );
        Ok(())
    }

    #[test]
    fn registered_builder_forwards_every_option() {
        // 对应 Java：ExcelReaderBuilder 链式 API 全部转发到内部 builder
        let mut builder = ExcelReaderBuilder::new()
            .register_read_listener::<DynamicRow, _>(ExtraCollectListener::default())
            .file("out.csv")
            .sheet(1)
            .sheet_name("Sheet2")
            .head_row_number(2)
            .charset("gbk")
            .ignore_empty_row(false)
            .custom_object(42_u32)
            .password("secret")
            .extra_read(CellExtraType::Comment)
            .read_default_return(ReadDefaultReturn::ActualData)
            .use_scientific_format(true)
            .register_read_listener(ExtraCollectListener::default());

        assert_eq!(
            builder.builder.file.as_deref(),
            Some(PathBuf::from("out.csv").as_path())
        );
        assert_eq!(
            builder.builder.options.sheet,
            SheetSelector::Name("Sheet2".to_owned())
        );
        assert_eq!(builder.builder.options.head_row_number, 2);
        assert_eq!(builder.builder.options.charset, CsvCharset::from("gbk"));
        assert!(!builder.builder.options.ignore_empty_row);
        assert!(builder.builder.options.custom_object.is_some());
        assert_eq!(builder.builder.options.password.as_deref(), Some("secret"));
        assert!(
            builder
                .builder
                .options
                .extra_read
                .contains(&CellExtraType::Comment)
        );
        assert_eq!(
            builder.builder.options.read_default_return,
            ReadDefaultReturn::ActualData
        );
        assert_eq!(
            builder.builder.options.scientific_format,
            crate::ScientificFormatMode::Scientific
        );

        // AbstractExcelReaderParameterBuilder 继承方法（对应 Java 继承链）
        AbstractExcelReaderParameterBuilder::<DynamicRow>::head_row_number(&mut builder, 5);
        assert_eq!(builder.builder.options.head_row_number, 5);
        AbstractExcelReaderParameterBuilder::<DynamicRow>::head_row_number(&mut builder, -3);
        assert_eq!(builder.builder.options.head_row_number, 0);
        AbstractExcelReaderParameterBuilder::<DynamicRow>::use_scientific_format(
            &mut builder,
            false,
        );
        assert_eq!(
            builder.builder.options.scientific_format,
            crate::ScientificFormatMode::Plain
        );
        AbstractExcelReaderParameterBuilder::<DynamicRow>::register_read_listener(
            &mut builder,
            Box::new(ExtraCollectListener::default()),
        );
    }

    #[test]
    fn registered_builder_build_uses_forwarded_file() -> Result<()> {
        // 对应 Java：registerReadListener 后 build() 读取转发后的文件
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "carol,33")?;
        let builder = ExcelReaderBuilder::new()
            .register_read_listener::<DynamicRow, _>(ExtraCollectListener::default())
            .file(file.path());
        let mut reader = builder.build()?;
        reader.read_all()?;
        reader.finish();
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;
    use crate::core::DynamicRow;

    #[derive(Default)]
    struct ParameterListener;

    impl ReadListener<DynamicRow> for ParameterListener {
        fn invoke(
            &mut self,
            _data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn parameter_builder_scientific_format_true_arm() {
        // 对应 Java：AbstractExcelReaderParameterBuilder.useScientificFormat(true)
        let mut builder =
            ExcelReaderBuilder::new().register_read_listener::<DynamicRow, _>(ParameterListener);
        AbstractExcelReaderParameterBuilder::<DynamicRow>::use_scientific_format(
            &mut builder,
            true,
        );
        assert_eq!(
            builder.builder.options.scientific_format,
            crate::ScientificFormatMode::Scientific
        );
    }
}
