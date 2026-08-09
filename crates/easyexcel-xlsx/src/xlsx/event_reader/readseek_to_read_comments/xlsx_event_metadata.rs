/// 对应 Java：无直接对应对象；Rust 架构扩展。 工作簿级事件读取元数据与 OOXML 包句柄。
pub struct XlsxEventMetadata<R: Read + Seek> {
    package: XlsxPackageReader<R>,
    sheet_paths: HashMap<String, String>,
    sheet_names: Vec<String>,
    cell_formats: Vec<XlsxNumberFormat>,
    shared_strings: Box<dyn SharedStringCacheReader>,
}

impl<R: Read + Seek> XlsxEventMetadata<R> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用指定缓存模式打开 XLSX 包。
    ///
    /// # Errors
    ///
    /// 包关系、工作簿、样式或共享字符串无效时返回错误。
    pub fn new(input: R, cache_mode: ReadCacheMode) -> Result<Self> {
        Self::new_with_cache_factory(input, |xml_size| create_cache(cache_mode, xml_size))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用调用方提供的共享字符串缓存工厂打开 XLSX 包。
    ///
    /// 工厂接收 `sharedStrings.xml` 的未压缩大小。
    ///
    /// # Errors
    ///
    /// 包关系、工作簿、样式、共享字符串或缓存初始化失败时返回错误。
    pub fn new_with_cache_factory<F>(input: R, mut cache_factory: F) -> Result<Self>
    where
        F: FnMut(u64) -> Result<Box<dyn SharedStringCache>>,
    {
        let mut package = XlsxPackageReader::new(input)?;
        let package_relationships = package.relationships("_rels/.rels")?;
        let workbook_target = package_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/officeDocument"))
            .map(|(target, _)| target)
            .ok_or_else(|| Error::Xlsx("officeDocument relationship not found".to_owned()))?;
        let workbook_path = resolve_target("", workbook_target)?;
        let workbook_relationships_path = relationship_part_name(&workbook_path);
        let workbook_relationships = package.relationships(&workbook_relationships_path)?;
        let (sheets, _) =
            read_workbook_metadata(&mut package, &workbook_path, &workbook_relationships)?;
        let sheet_names = sheets.iter().map(|(name, _)| name.clone()).collect();
        let sheet_paths = sheets.into_iter().collect::<HashMap<_, _>>();
        let cell_formats = workbook_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/styles"))
            .map(|(target, _)| resolve_target(&workbook_path, target))
            .transpose()?
            .map(|styles_path| read_cell_formats(&mut package, &styles_path))
            .transpose()?
            .unwrap_or_else(|| vec![XlsxNumberFormat::Builtin(0)]);
        let shared_strings_path = workbook_relationships
            .values()
            .find(|(_, relationship_type)| relationship_type.ends_with("/sharedStrings"))
            .map(|(target, _)| resolve_target(&workbook_path, target))
            .transpose()?;
        let shared_strings = match shared_strings_path {
            Some(path) => read_shared_strings(&mut package, &path, &mut cache_factory)?,
            None => memory_cache(),
        };
        for path in sheet_paths.values() {
            if !package.contains(path) {
                return Err(Error::Xlsx(format!("worksheet part not found: {path}")));
            }
        }
        Ok(Self {
            package,
            sheet_paths,
            sheet_names,
            cell_formats,
            shared_strings,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表名称，顺序与工作簿一致。
    #[must_use]
    pub fn sheet_names(&self) -> &[String] {
        &self.sheet_names
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 打开一个工作表的单元格事件游标。
    ///
    /// # Errors
    ///
    /// 工作表不存在或 XML 无效时返回错误。
    pub fn cells(
        &mut self,
        sheet_name: &str,
        options: XlsxDisplayOptions,
    ) -> Result<XlsxCellEventReader<'_>> {
        let path = self
            .sheet_paths
            .get(sheet_name)
            .cloned()
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        let file = self.package.open_part(&path)?;
        let reader = boxed_xml_reader(BufReader::new(file));
        XlsxCellEventReader::new(
            reader,
            &self.cell_formats,
            options,
            self.shared_strings.as_ref(),
        )
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 扫描工作表最后一个显式行号。
    ///
    /// # Errors
    ///
    /// 工作表不存在或 XML 无效时返回错误。
    pub fn last_explicit_row(&mut self, sheet_name: &str) -> Result<Option<u32>> {
        let path = self
            .sheet_paths
            .get(sheet_name)
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        let file = self.package.open_part(path)?;
        scan_last_row(BufReader::new(file))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 读取合并区域、超链接和批注。
    ///
    /// # Errors
    ///
    /// 工作表、关系或附加 XML 无效时返回错误。
    pub fn extras(
        &mut self,
        sheet_name: &str,
        enabled: &HashSet<XlsxExtraKind>,
    ) -> Result<Vec<XlsxExtra>> {
        let sheet_path = self
            .sheet_paths
            .get(sheet_name)
            .ok_or_else(|| Error::Other(format!("sheet not found: {sheet_name}")))?;
        if enabled.is_empty() {
            return Ok(Vec::new());
        }
        let sheet_path = sheet_path.clone();
        let relationships_path = relationship_part_name(&sheet_path);
        let requires_relationships = enabled.contains(&XlsxExtraKind::Hyperlink)
            || enabled.contains(&XlsxExtraKind::Comment);
        let relationships =
            if requires_relationships && self.package.contains(&relationships_path) {
                self.package.raw_relationships(&relationships_path)?
            } else {
                RawRelationships::new()
            };
        let mut extras = if enabled.contains(&XlsxExtraKind::Merge)
            || enabled.contains(&XlsxExtraKind::Hyperlink)
        {
            read_worksheet_extras(&mut self.package, &sheet_path, &relationships, enabled)?
        } else {
            Vec::new()
        };
        if enabled.contains(&XlsxExtraKind::Comment)
            && let Some((target, _, false)) = relationships
                .values()
                .find(|(_, relationship_type, _)| relationship_type.ends_with("/comments"))
        {
            let comments_path = resolve_target(&sheet_path, target)?;
            extras.extend(read_comments(&mut self.package, &comments_path)?);
        }
        Ok(extras)
    }
}
