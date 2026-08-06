/// 对应 Java：无直接对应对象；Rust 架构扩展。 `EasyExcel` 门面持有的 XLSX 工作簿事件元数据。
pub(crate) struct XlsxRowMetadata {
    inner: XlsxEventMetadata<Box<dyn ReadSeek>>,
}

impl XlsxRowMetadata {
    #[cfg(test)]
    pub(crate) fn new(input: impl Read + Seek + 'static) -> Result<Self> {
        Self::new_with_cache(input, &ReadOptions::default())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn new_with_cache(
        input: impl Read + Seek + 'static,
        options: &ReadOptions,
    ) -> Result<Self> {
        let mode = options.read_cache;
        let selector = options
            .read_cache_selector
            .as_ref()
            .map(|stored| stored as &dyn crate::cache::ReadCacheSelector);
        let inner = XlsxEventMetadata::new_with_cache_factory(
            Box::new(input) as Box<dyn ReadSeek>,
            |xml_size| {
                selector.map_or_else(
                    || create_cache(mode, xml_size),
                    |selector| selector.create_cache(xml_size),
                )
            },
        )
        .map_err(ExcelError::from)?;
        Ok(Self { inner })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn sheet_names(&self) -> Vec<String> {
        self.inner.sheet_names().to_vec()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn display_cells(
        &mut self,
        sheet_name: &str,
        use_1904_windowing: bool,
        use_scientific_format: bool,
        locale: SpreadsheetLocale,
    ) -> Result<XlsxDisplayCellReader<'_>> {
        let inner = self
            .inner
            .cells(
                sheet_name,
                XlsxDisplayOptions {
                    date_1904: use_1904_windowing,
                    use_scientific_format,
                    locale,
                },
            )
            .map_err(ExcelError::from)?;
        Ok(XlsxDisplayCellReader {
            inner,
            use_1904_windowing,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn last_explicit_row(&mut self, sheet_name: &str) -> Result<Option<u32>> {
        self.inner
            .last_explicit_row(sheet_name)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn extras(
        &mut self,
        sheet_name: &str,
        enabled: &HashSet<CellExtraType>,
    ) -> Result<Vec<CellExtra>> {
        let engine_enabled = enabled
            .iter()
            .map(|kind| match kind {
                CellExtraType::Merge => XlsxExtraKind::Merge,
                CellExtraType::Hyperlink => XlsxExtraKind::Hyperlink,
                CellExtraType::Comment => XlsxExtraKind::Comment,
            })
            .collect();
        self.inner
            .extras(sheet_name, &engine_enabled)
            .map(|extras| {
                extras
                    .into_iter()
                    .map(|extra| {
                        let kind = match extra.kind {
                            XlsxExtraKind::Merge => CellExtraType::Merge,
                            XlsxExtraKind::Hyperlink => CellExtraType::Hyperlink,
                            XlsxExtraKind::Comment => CellExtraType::Comment,
                        };
                        CellExtra::new(
                            kind,
                            extra.text,
                            extra.first_row,
                            extra.last_row,
                            extra.first_column,
                            extra.last_column,
                        )
                    })
                    .collect()
            })
            .map_err(ExcelError::from)
    }
}

