/// 对应 Java：无直接对应对象；Rust 架构扩展。 Runtime executor selected by `ExcelAnalyserImpl.choiceExcelExecutor`.
///
/// Java stores one `ExcelReadExecutor` interface object. Rust uses an enum so
/// callers can inspect the same concrete XLSX/XLS/CSV executor that owns sheet
/// discovery while typed listener execution remains statically dispatched.
// 对应 Java：三个变体与 Java 具体 executor 对象一一对应；体积差异来自 SAX 解析器
// 持有的缓存/上下文字段，为保持直接字段访问语义（与 Java 相同）不做 Box 装箱。
#[allow(clippy::large_enum_variant)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub enum ExcelReadExecutorKind {
    /// OOXML SAX executor.
    Xlsx(XlsxSaxAnalyser),
    /// BIFF event executor.
    Xls(XlsSaxAnalyser),
    /// CSV record executor.
    Csv(CsvExcelReadExecutor),
}

impl ExcelReadExecutorKind {
    /// Constructs the concrete executor selected from the resolved workbook type.
    ///
    /// # Errors
    ///
    /// 当工作簿无法打开或解析（对应 Java 抛异常）时返回 `ExcelError`。
    pub fn new(
        excel_type: ExcelTypeEnum,
        path: impl Into<PathBuf>,
        options: ReadOptions,
    ) -> Result<Self> {
        let path = path.into();
        match excel_type {
            ExcelTypeEnum::Xlsx => XlsxSaxAnalyser::from_path(path, options).map(Self::Xlsx),
            ExcelTypeEnum::Xls => XlsSaxAnalyser::from_path(path, options).map(Self::Xls),
            ExcelTypeEnum::Csv => Ok(Self::Csv(CsvExcelReadExecutor::from_path(path))),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Executes through the selected real parser with the current analyser options.
    ///
    /// # Errors
    ///
    /// 当工作簿解析失败时返回 `ExcelError`。
    pub fn execute_with_listener<T, L>(
        &mut self,
        options: &ReadOptions,
        listener: &mut L,
    ) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        ExcelReadExecutor::execute::<T, L>(self, options, listener)
    }

    /// Returns the concrete executor variant's resolved workbook type.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn excel_type(&self) -> ExcelTypeEnum {
        match self {
            Self::Xlsx(_) => ExcelTypeEnum::Xlsx,
            Self::Xls(_) => ExcelTypeEnum::Xls,
            Self::Csv(_) => ExcelTypeEnum::Csv,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the selected executor's discovered worksheet list.
    #[must_use]
    pub fn sheet_list(&self) -> &[ReadSheet] {
        ExcelReadExecutor::sheet_list(self)
    }
}

impl ExcelReadExecutor for ExcelReadExecutorKind {
    fn sheet_list(&self) -> &[ReadSheet] {
        match self {
            Self::Xlsx(executor) => executor.sheet_list(),
            Self::Xls(executor) => executor.sheet_list(),
            Self::Csv(executor) => executor.sheet_list(),
        }
    }

    fn execute<T, L>(&mut self, options: &ReadOptions, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        match self {
            Self::Xlsx(executor) => executor.execute::<T, L>(options, listener),
            Self::Xls(executor) => executor.execute::<T, L>(options, listener),
            Self::Csv(executor) => executor.execute::<T, L>(options, listener),
        }
    }
}

