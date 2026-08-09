/// 对应 Java：无直接对应对象；Rust 架构扩展。 Multi-sheet BIFF8 workbook buffer.
#[derive(Debug, Clone, Default)]
pub struct Biff8Book {
    /// Ordered worksheets (emission order = BOUNDSHEET order).
    pub sheets: Vec<Biff8Sheet>,
    /// Workbook-global FONT / XF registry (Java HSSF style table).
    pub styles: Biff8StyleTable,
    /// When `true`, BIFF8 `DATEMODE` uses the 1904 date windowing system.
    pub use_1904_windowing: bool,
    /// 活动工作表索引；写入 WINDOW1，并与各工作表 WINDOW2 选择状态同步。
    pub active_sheet: usize,
}

impl Biff8Book {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a new worksheet and rejects duplicate names.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when `name` is already in use.
    pub fn create_sheet(&mut self, name: impl Into<String>) -> Result<&mut Biff8Sheet> {
        let name = name.into();
        if self.sheets.iter().any(|sheet| sheet.name == name) {
            return Err(ExcelError::Xls(format!(
                "worksheet name is already in use: {name}"
            )));
        }
        self.sheets.push(Biff8Sheet::new(name));
        self.sheets
            .last_mut()
            .ok_or_else(|| ExcelError::Xls("worksheet append produced no sheet".to_owned()))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns a mutable sheet by name, creating it if missing.
    ///
    /// # Panics
    ///
    /// Never in practice; `last_mut` is only reached right after the new
    /// sheet was pushed onto the list.
    pub fn sheet_mut(&mut self, name: &str) -> &mut Biff8Sheet {
        if let Some(index) = self.sheets.iter().position(|s| s.name == name) {
            return &mut self.sheets[index];
        }
        self.sheets.push(Biff8Sheet::new(name.to_owned()));
        self.sheets.last_mut().expect("just pushed")
    }

    /// 将后端中立图表请求编译为 BIFF8 图表记录模型。
    ///
    /// # Errors
    ///
    /// 目标工作表不存在、系列为空、文本或坐标超出 BIFF8 限制时返回错误。
    pub fn add_chart_mutation(
        &mut self,
        mutation: &easyexcel_model::ChartMutation,
    ) -> Result<()> {
        if mutation.series.is_empty() {
            return Err(ExcelError::Xls(
                "chart mutation requires at least one data series".to_owned(),
            ));
        }
        if mutation.last_row < mutation.first_row || mutation.last_column < mutation.first_column {
            return Err(ExcelError::Xls(
                "chart mutation anchor end must not precede its start".to_owned(),
            ));
        }
        for text in mutation
            .title
            .iter()
            .chain(mutation.series.iter().filter_map(|series| series.name.as_ref()))
        {
            if text.contains('\0') || text.encode_utf16().count() > usize::from(u8::MAX) {
                return Err(ExcelError::Xls(
                    "BIFF8 chart titles and series names must contain at most 255 UTF-16 units and no NUL"
                        .to_owned(),
                ));
            }
        }
        let target_index = self
            .sheets
            .iter()
            .position(|sheet| sheet.name == mutation.sheet_name)
            .ok_or_else(|| {
                ExcelError::Xls(format!(
                    "chart target sheet '{}' does not exist",
                    mutation.sheet_name
                ))
            })?;
        let known_sheets = self
            .sheets
            .iter()
            .map(|sheet| sheet.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        let kind = match mutation.chart_type {
            easyexcel_model::ChartType::Bar => Biff8ChartKind::Bar,
            easyexcel_model::ChartType::Line => Biff8ChartKind::Line,
            easyexcel_model::ChartType::Pie => Biff8ChartKind::Pie,
        };
        let mut chart = Biff8Chart::new(
            kind,
            checked_chart_row(mutation.first_row)?,
            checked_chart_column(mutation.first_column)?,
            checked_chart_row(mutation.last_row)?,
            checked_chart_column(mutation.last_column)?,
        );
        if let Some(title) = &mutation.title {
            chart = chart.with_title(title.clone());
        }
        for source in &mutation.series {
            let values = chart_range(&source.values, &known_sheets)?;
            let mut series = Biff8ChartSeries::new(values);
            if let Some(name) = &source.name {
                series = series.with_name(name.clone());
            }
            if let Some(categories) = &source.categories {
                series = series.with_categories(chart_range(categories, &known_sheets)?);
            }
            chart = chart.with_series(series);
        }
        self.sheets[target_index].add_chart(chart);
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Serializes this book to an OLE Compound File containing a `Workbook` stream.
    ///
    /// # Errors
    ///
    /// Returns I/O or CFB construction errors.
    pub fn to_cfb_bytes(&self) -> Result<Vec<u8>> {
        self.to_cfb_bytes_with_password(None)
    }

    /// 使用与 Java `EasyExcel`/POI 5.2.5 一致的 BIFF8 RC4 `CryptoAPI` 密码写出。
    ///
    /// # Errors
    ///
    /// 随机源、BIFF8 加密、I/O 或 CFB 构造失败时返回错误。
    pub fn to_cfb_bytes_with_password(&self, password: Option<&str>) -> Result<Vec<u8>> {
        self.validate_generated_charts()?;
        // 写入前对全部工作表公式求值，得到缓存值表（借用 xls 求值引擎）
        let caches = super::cached::recalc_cached_values(&self.sheets);
        let stream = if let Some(password) = password {
            let encryption = super::encrypt::prepare_crypto_api_encryption(password)
                .map_err(ExcelError::Xls)?;
            let plain = build_workbook_stream_with_filepass(
                self,
                &caches,
                Some(encryption.filepass_payload()),
            )?;
            let plain = super::model::Biff8WorkbookModel::from_workbook_stream(&plain)?
                .to_workbook_stream()?;
            super::encrypt::encrypt_crypto_api_workbook_stream(&plain, &encryption)
                .map_err(ExcelError::Xls)?
        } else {
            let plain = build_workbook_stream_result(self, &caches)?;
            super::model::Biff8WorkbookModel::from_workbook_stream(&plain)?
                .to_workbook_stream()?
        };
        let mut mem = Cursor::new(Vec::<u8>::new());
        {
            #[rustfmt::skip]
            // 使用 V3（512 字节扇区）：与 Excel / LibreOffice 生成的 .xls 一致，
            // 兼容性最广（部分解析器不支持 V4 的 4096 扇区）。
            let mut cf = cfb::CompoundFile::create_with_version(cfb::Version::V3, &mut mem)
                .map_err(|error| ExcelError::Cfb(format!("cannot create OLE2 container: {error}")))?;
            {
                #[rustfmt::skip]
                let mut workbook = cf.create_stream("Workbook").map_err(|error| ExcelError::Cfb(format!("cannot create Workbook stream: {error}")))?;
                workbook.write_all(&stream)?;
            }
            #[rustfmt::skip]
            cf.flush().map_err(|error| ExcelError::Cfb(format!("cannot flush OLE2 container: {error}")))?;
        }
        Ok(mem.into_inner())
    }

    fn validate_generated_charts(&self) -> Result<()> {
        let sheet_names = self
            .sheets
            .iter()
            .map(|sheet| sheet.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        for sheet in &self.sheets {
            for chart in &sheet.charts {
                for text in chart
                    .title
                    .iter()
                    .chain(chart.series.iter().filter_map(|series| series.name.as_ref()))
                {
                    if text.contains('\0') || text.encode_utf16().count() > usize::from(u8::MAX) {
                        return Err(ExcelError::Xls(
                            "BIFF8 chart titles and series names must contain at most 255 UTF-16 units and no NUL"
                                .to_owned(),
                        ));
                    }
                }
                if chart.last_row < chart.first_row || chart.last_column < chart.first_column {
                    return Err(ExcelError::Xls(
                        "BIFF8 chart anchor end must not precede its start".to_owned(),
                    ));
                }
                if chart.series.is_empty() {
                    return Err(ExcelError::Xls(
                        "BIFF8 chart requires at least one data series".to_owned(),
                    ));
                }
                for range in chart.series.iter().flat_map(|series| {
                    series.categories.iter().chain(std::iter::once(&series.values))
                }) {
                    if range.last_row < range.first_row
                        || range.last_column < range.first_column
                    {
                        return Err(ExcelError::Xls(format!(
                            "BIFF8 chart range on sheet '{}' is reversed",
                            range.sheet_name
                        )));
                    }
                    if !sheet_names.contains(range.sheet_name.as_str()) {
                        return Err(ExcelError::Xls(format!(
                            "BIFF8 chart range sheet '{}' does not exist",
                            range.sheet_name
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes the CFB bytes to `writer`.
    ///
    /// # Errors
    ///
    /// Returns serialization or I/O errors.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<()> {
        let bytes = self.to_cfb_bytes()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// 将可选密码保护的 CFB 工作簿写入调用方提供的 writer。
    ///
    /// # Errors
    ///
    /// 序列化、密码加密或输出失败时返回错误。
    pub fn write_to_with_password<W: Write>(
        &self,
        mut writer: W,
        password: Option<&str>,
    ) -> Result<()> {
        let bytes = self.to_cfb_bytes_with_password(password)?;
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes the CFB bytes and flushes the caller-owned writer.
    ///
    /// # Errors
    ///
    /// Returns serialization, write, or flush errors.
    pub fn write_to_and_flush<W: Write>(&self, mut writer: W) -> Result<()> {
        self.write_to(&mut writer)?;
        writer.flush()?;
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将 BIFF8/OLE2 工作簿写入文件路径。
    ///
    /// # Errors
    ///
    /// 父目录创建、工作簿序列化、文件创建或写入失败时返回错误。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        self.write_to_and_flush(&mut file)?;
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将可选密码保护的 BIFF8/OLE2 工作簿写入路径。
    ///
    /// # Errors
    ///
    /// 父目录、随机源、BIFF8/CFB 序列化或输出失败时返回错误。
    pub fn save_to_path_with_password(&self, path: &Path, password: Option<&str>) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        self.write_to_with_password(&mut file, password)
    }
}

fn chart_range(
    range: &easyexcel_model::ChartRange,
    known_sheets: &std::collections::HashSet<&str>,
) -> Result<Biff8ChartRange> {
    if range.last_row < range.first_row || range.last_column < range.first_column {
        return Err(ExcelError::Xls(format!(
            "chart range on sheet '{}' has an end before its start",
            range.sheet_name
        )));
    }
    if !known_sheets.contains(range.sheet_name.as_str()) {
        return Err(ExcelError::Xls(format!(
            "chart range sheet '{}' does not exist",
            range.sheet_name
        )));
    }
    Ok(Biff8ChartRange::new(
        range.sheet_name.clone(),
        checked_chart_row(range.first_row)?,
        checked_chart_column(range.first_column)?,
        checked_chart_row(range.last_row)?,
        checked_chart_column(range.last_column)?,
    ))
}

fn checked_chart_row(row: u32) -> Result<u16> {
    u16::try_from(row).map_err(|_| {
        ExcelError::Xls(format!(
            "BIFF8 chart row {row} exceeds the 65535 row index limit"
        ))
    })
}

fn checked_chart_column(column: u16) -> Result<u8> {
    u8::try_from(column).map_err(|_| {
        ExcelError::Xls(format!(
            "BIFF8 chart column {column} exceeds the 255 column index limit"
        ))
    })
}
