//! Synchronous collecting reader builder.
//!
//! 对应 Java：`EasyExcel.readSync(...)`——一次性把所有转换好的行收集到
//! `Vec<T>` 返回，Java 端无单独的 builder 类型，由 Rust facade 提供等价入口。

use std::marker::PhantomData;
use std::path::PathBuf;

use crate::IntoSheetSelector;
use crate::collect_listener::{collect_listener, drain_listener};
use crate::core::{
    CellExtraType, Converter, CsvCharset, CustomReadObject, ExcelRow, NullableObjectConverter,
    ReadDefaultReturn, Result,
};
use crate::read::{
    ExcelLocale, ReadCacheMode, ReadOptions, ScientificFormatMode, SheetSelector,
    StoredReadCacheSelector, read_csv, read_xls, read_xlsx,
};
use crate::write_type_helpers::{is_csv_path, is_xls_path};

/// Synchronous collecting reader builder.
pub struct ExcelSyncReaderBuilder<T> {
    pub(crate) path: PathBuf,
    pub(crate) options: ReadOptions,
    marker: PhantomData<T>,
}

impl<T> ExcelSyncReaderBuilder<T>
where
    T: ExcelRow,
{
    /// 从路径构造一个默认配置的同步读取 builder。
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            options: ReadOptions::default(),
            marker: PhantomData,
        }
    }

    /// Selects a worksheet by name or zero-based index.
    #[must_use]
    pub fn sheet(mut self, sheet: impl IntoSheetSelector) -> Self {
        self.options.sheet = sheet.into_sheet_selector();
        self
    }

    /// Selects every worksheet in workbook order.
    #[must_use]
    pub fn all_sheets(mut self) -> Self {
        self.options.sheet = SheetSelector::All;
        self
    }

    /// Sets the number of header rows.
    #[must_use]
    pub const fn head_row_number(mut self, rows: u32) -> Self {
        self.options.head_row_number = rows;
        self
    }

    /// Includes or skips rows containing no values.
    #[must_use]
    pub const fn ignore_empty_row(mut self, ignore: bool) -> Self {
        self.options.ignore_empty_row = ignore;
        self
    }

    /// Enables or disables Java EasyExcel-compatible string trimming.
    #[must_use]
    pub const fn auto_trim(mut self, enabled: bool) -> Self {
        self.options.auto_trim = enabled;
        self
    }

    /// Selects Excel's 1904 date windowing system while collecting rows.
    #[must_use]
    pub const fn use_1904_windowing(mut self, enabled: bool) -> Self {
        self.options.use_1904_windowing = enabled;
        self
    }

    /// Controls scientific notation while collecting extreme General-format numbers.
    #[must_use]
    pub const fn use_scientific_format(mut self, enabled: bool) -> Self {
        self.options.scientific_format = if enabled {
            ScientificFormatMode::Scientific
        } else {
            ScientificFormatMode::Plain
        };
        self
    }

    /// Sets the locale used while collecting formatted number and date values.
    #[must_use]
    pub fn locale(mut self, locale: ExcelLocale) -> Self {
        self.options.locale = locale;
        self
    }

    /// Registers a Java-style global converter while collecting rows.
    #[must_use]
    pub fn register_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: Converter<V> + Send + Sync + 'static,
    {
        self.options.converters.register::<V, C>(converter);
        self
    }

    /// Registers a nullable converter while collecting rows.
    #[must_use]
    pub fn register_nullable_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: NullableObjectConverter<V> + Send + Sync + 'static,
    {
        self.options.converters.register_nullable::<V, C>(converter);
        self
    }

    /// Selects the XLSX shared-string cache backend while collecting rows.
    #[must_use]
    pub fn read_cache(mut self, mode: ReadCacheMode) -> Self {
        self.options.read_cache = mode;
        self.options.read_cache_selector = None;
        self
    }

    /// Installs a Java-style cache selector while collecting rows.
    #[must_use]
    pub fn read_cache_selector(mut self, selector: StoredReadCacheSelector) -> Self {
        self.options.read_cache_selector = Some(selector);
        self
    }

    /// Sets the first physical data row to collect, zero-based and inclusive.
    ///
    /// Configured header rows are still analysed for name-based mapping.
    #[must_use]
    pub const fn start_row(mut self, row: u32) -> Self {
        self.options.start_row = Some(row);
        self
    }

    /// Sets the last physical data row to collect, zero-based and inclusive.
    ///
    /// Configured header rows are still analysed for name-based mapping.
    #[must_use]
    pub const fn end_row(mut self, row: u32) -> Self {
        self.options.end_row = Some(row);
        self
    }

    /// Limits collected data to an inclusive physical row range.
    #[must_use]
    pub const fn read_rows(mut self, start: u32, end: u32) -> Self {
        self.options.start_row = Some(start);
        self.options.end_row = Some(end);
        self
    }

    /// Maps a workbook header name to the name used by typed row mapping.
    #[must_use]
    pub fn header_alias(mut self, header: impl Into<String>, alias: impl Into<String>) -> Self {
        self.options
            .header_aliases
            .insert(header.into(), alias.into());
        self
    }

    /// Stores a type-safe value exposed while synchronously collecting rows.
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
    pub const fn read_default_return(mut self, mode: ReadDefaultReturn) -> Self {
        self.options.read_default_return = mode;
        self
    }

    /// Enables a Java `extraRead` metadata category.
    #[must_use]
    pub fn extra_read(mut self, extra_type: CellExtraType) -> Self {
        self.options.extra_read.insert(extra_type);
        self
    }

    /// Sets the password for an encrypted OOXML workbook.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.options.password = Some(password.into());
        self
    }

    /// Sets the character encoding used for CSV input.
    #[must_use]
    pub fn charset(mut self, charset: impl Into<CsvCharset>) -> Self {
        self.options.charset = charset.into();
        self
    }

    /// Reads all rows into memory.
    ///
    /// # Errors
    ///
    /// Returns a workbook, sheet-selection, or row-conversion error.
    pub fn do_read_sync(self) -> Result<Vec<T>> {
        let mut listener = collect_listener::<T>();
        if is_csv_path(&self.path) {
            read_csv::<T, _>(&self.path, &self.options, &mut listener)?;
        } else if is_xls_path(&self.path) {
            read_xls::<T, _>(&self.path, &self.options, &mut listener)?;
        } else {
            read_xlsx::<T, _>(&self.path, &self.options, &mut listener)?;
        }
        Ok(drain_listener(listener))
    }
}
