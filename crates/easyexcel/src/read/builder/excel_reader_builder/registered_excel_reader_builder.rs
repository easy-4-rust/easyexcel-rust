/// An [`ExcelReaderBuilder`] carrying its registered listener.
///
/// Java stores listeners inside `ReadWorkbook`; this wrapper provides the
/// same lifecycle without erasing the Rust row or listener types.
/// 对应 Java：`Math.max(headRowNumber, 0)`。
pub struct RegisteredExcelReaderBuilder<T> {
    builder: ExcelReaderBuilder,
    listeners: ReadListenerList<T>,
}

impl<T> RegisteredExcelReaderBuilder<T>
where
    T: ExcelRow + Clone,
{
    /// Sets the file path.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.builder = self.builder.file(path);
        self
    }

    /// Selects a worksheet by zero-based index.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn sheet(mut self, index: usize) -> Self {
        self.builder = self.builder.sheet(index);
        self
    }

    /// Selects a worksheet by name.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn sheet_name(mut self, name: impl Into<String>) -> Self {
        self.builder = self.builder.sheet_name(name);
        self
    }

    /// Sets the number of header rows.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn head_row_number(mut self, rows: u32) -> Self {
        self.builder = self.builder.head_row_number(rows);
        self
    }

    /// Sets the CSV character encoding.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.builder = self.builder.charset(charset);
        self
    }

    /// Controls whether empty rows are skipped.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn ignore_empty_row(mut self, ignore: bool) -> Self {
        self.builder = self.builder.ignore_empty_row(ignore);
        self
    }

    /// Stores a custom context value.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn custom_object<C>(mut self, custom_object: C) -> Self
    where
        C: std::any::Any + Send + Sync,
    {
        self.builder = self.builder.custom_object(custom_object);
        self
    }

    /// Sets the encrypted OOXML password.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.builder = self.builder.password(password);
        self
    }

    /// Enables an extra metadata category.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn extra_read(mut self, extra_type: CellExtraType) -> Self {
        self.builder = self.builder.extra_read(extra_type);
        self
    }

    /// Selects the no-model value representation.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn read_default_return(mut self, mode: ReadDefaultReturn) -> Self {
        self.builder = self.builder.read_default_return(mode);
        self
    }

    /// Controls scientific formatting.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.builder = self.builder.use_scientific_format(enabled);
        self
    }

    /// Registers another listener after all listeners already present.
    #[must_use]
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn register_read_listener<Next>(mut self, listener: Next) -> Self
    where
        Next: ReadListener<T> + 'static,
    {
        self.listeners.push(listener);
        self
    }

    /// Builds an event-driven reader using the registered listener chain.
    ///
    /// # Errors
    ///
    /// 当未设置 `file`（对应 Java 抛异常）或工作簿打开失败时返回 `ExcelError`。
    pub fn build(self) -> Result<ExcelReader<T, ReadListenerList<T>>> {
        self.builder.build(self.listeners)
    }

    /// Builds, reads, and finishes all configured sheets.
    ///
    /// # Errors
    ///
    /// 当构建失败（未设置 `file`）或任一工作表解析失败时返回 `ExcelError`。
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn do_read_all(self) -> Result<()> {
        self.builder.do_read_all(self.listeners)
    }

    /// Reads synchronously while retaining all previously registered listeners.
    ///
    /// # Errors
    ///
    /// 当构建失败（未设置 `file`）或任一工作表解析失败时返回 `ExcelError`。
    /// 对应 Java：`Math.max(headRowNumber, 0)`。
    pub fn do_read_all_sync(self) -> Result<Vec<T>> {
        let rows = Rc::new(RefCell::new(Vec::new()));
        let mut collector = SharedCollectListener(Rc::clone(&rows));
        let mut reader = self.build()?;
        reader.read_all_with_additional_listener(&mut collector)?;
        reader.finish();
        drop(reader);
        let collected = std::mem::take(&mut *rows.borrow_mut());
        Ok(collected)
    }
}

impl<T> AbstractExcelReaderParameterBuilder<T> for RegisteredExcelReaderBuilder<T>
where
    T: ExcelRow + Clone,
{
    // 对应 Java：`Math.max(headRowNumber, 0)` 保证非负后再存入 u32 字段，
    // 符号位必然为 0，`as u32` 不会丢失符号。
    #[allow(clippy::cast_sign_loss)]
    fn head_row_number(&mut self, head_row_number: i32) -> &mut Self {
        self.builder.options.head_row_number = head_row_number.max(0) as u32;
        self
    }

    fn use_scientific_format(&mut self, enabled: bool) -> &mut Self {
        self.builder.options.scientific_format = if enabled {
            crate::ScientificFormatMode::Scientific
        } else {
            crate::ScientificFormatMode::Plain
        };
        self
    }

    fn register_read_listener(&mut self, listener: Box<dyn ReadListener<T>>) -> &mut Self {
        self.listeners.push_boxed(listener);
        self
    }
}

