/// One framed BIFF record (`type` + payload).
#[derive(Debug, Clone)]
struct RawRecord {
    typ: u16,
    data: Vec<u8>,
}

/// Worksheet location inside the globals / sheet record list.
#[derive(Debug, Clone)]
struct SheetSpan {
    name: String,
    /// 在全部 BOUNDSHEET（含 chart/macro sheet）中的索引。
    bound_sheet_index: u16,
    /// Index of the worksheet `BOF` record.
    bof_index: usize,
    /// Index of the worksheet `EOF` record (exclusive insert point is this index).
    eof_index: usize,
    /// Index of the `DIMENSION` record inside this sheet, when present.
    dimension_index: Option<usize>,
}

/// 对应 Java：HSSFSheet#getLastRowNum。 In-memory `.xls` template with record-preserving cell writes.
///
/// Corresponds to a loaded POI `HSSFWorkbook` used only for appending / overlay
/// cells while leaving the rest of the BIFF stream intact.
#[derive(Debug, Clone)]
pub struct Biff8TemplatePackage {
    /// Full OLE/CFB bytes (all streams); only `Workbook` is rewritten on save.
    ole_bytes: Vec<u8>,
    /// Workbook stream path (`Workbook` or `Book`).
    workbook_path: String,
    /// Parsed BIFF records from the Workbook stream.
    records: Vec<RawRecord>,
    /// Bound sheets in workbook order.
    sheets: Vec<SheetSpan>,
    /// 模板加载时捕获的占位符锚点；填充值覆写后仍用于后续批次定位。
    placeholders: Vec<(String, u16, u8, String)>,
    /// 每个 sheet/wrapper/锚点的下一次集合填充行或列偏移。
    collection_cursors: BTreeMap<(String, String, u16, u8, bool), u16>,
}

impl Biff8TemplatePackage {
    /// 对应 Java：HSSFSheet#getLastRowNum。 Loads an OLE `.xls` template from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Xls`] when the bytes are not a readable BIFF8
    /// workbook, or [`ExcelError::Unsupported`] for empty / unusable templates.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes_with_password(bytes, None)
    }

    /// 从 OLE `.xls` 字节加载模板，并以调用级密码解密 `CryptoAPI` Workbook stream。
    ///
    /// # Errors
    ///
    /// 模板无效、密码缺失/错误或加密类型不支持时返回错误。
    pub fn from_bytes_with_password(bytes: &[u8], password: Option<&str>) -> Result<Self> {
        if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            return Err(ExcelError::Xls(
                "xls template is not an OLE Compound File".to_owned(),
            ));
        }
        let (workbook_path, encrypted_workbook) = read_workbook_stream(bytes)?;
        let encrypted_records = split_records(&encrypted_workbook)?;
        let workbook = if encrypted_records.iter().any(|record| record.typ == FILEPASS) {
            let password = password.ok_or_else(|| {
                ExcelError::PasswordProtected("legacy XLS (BIFF8) CryptoAPI RC4".to_owned())
            })?;
            decrypt_crypto_api_workbook_stream(&encrypted_workbook, password)?
        } else {
            encrypted_workbook
        };
        let records = split_records(&workbook)?;
        let sheets = discover_sheets(&records)?;
        if sheets.is_empty() {
            return Err(ExcelError::Xls(
                "xls template Workbook contains no worksheets".to_owned(),
            ));
        }
        let mut package = Self {
            ole_bytes: bytes.to_vec(),
            workbook_path,
            records,
            sheets,
            placeholders: Vec::new(),
            collection_cursors: BTreeMap::new(),
        };
        package.placeholders = package.scan_placeholders();
        Ok(package)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Loads an OLE `.xls` template from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors from [`Self::from_bytes`].
    pub fn from_path(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(ExcelError::from)?;
        Self::from_bytes(&bytes)
    }

    /// 从路径加载模板，并以调用级密码解密 `CryptoAPI` Workbook stream。
    ///
    /// # Errors
    ///
    /// I/O、模板格式或密码验证失败时返回错误。
    pub fn from_path_with_password(path: &Path, password: Option<&str>) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(ExcelError::from)?;
        Self::from_bytes_with_password(&bytes, password)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Returns worksheet names in `BoundSheet` order.
    #[must_use]
    pub fn sheet_names(&self) -> Vec<String> {
        self.sheets.iter().map(|sheet| sheet.name.clone()).collect()
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Returns the next zero-based append row for a sheet (Java `lastRowNum + 1`).
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::SheetNotFound`] when the sheet is absent.
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        let sheet = self.sheet(sheet_name)?;
        Ok(sheet_max_row(&self.records, sheet).map_or(0, |row| u32::from(row).saturating_add(1)))
    }

    /// 从工作表当前最后一行之后追加中立 BIFF8 单元格行。
    /// 对应 Java：`HSSFSheet#getLastRowNum` 与逐行 `createRow`/`createCell`。
    ///
    /// 返回追加完成后的下一可写行号。
    ///
    /// # Errors
    ///
    /// 工作表不存在、坐标越界或单元格无法编码时返回错误。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, Biff8Cell)>],
    ) -> Result<u32> {
        let mut next_row = self.next_row_for_sheet(sheet_name)?;
        for row in rows {
            for (column, cell) in row {
                self.set_cell(sheet_name, next_row, *column, cell)?;
            }
            next_row = next_row.saturating_add(1);
        }
        Ok(next_row)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Writes a cell value at `(row, col)`, replacing any existing cell record.
    ///
    /// Existing XF indexes are reused when overwriting a cell; new cells use
    /// [`XF_GENERAL`]. Unrelated records are left untouched.
    ///
    /// # Errors
    ///
    /// Returns format errors for out-of-range coordinates or unsupported values.
    pub fn set_cell(
        &mut self,
        sheet_name: &str,
        row: u32,
        col: usize,
        cell: &Biff8Cell,
    ) -> Result<()> {
        let row = u16::try_from(row)
            .map_err(|_| ExcelError::Xls("BIFF8 supports at most 65536 rows".to_owned()))?;
        let col = u8::try_from(col)
            .map_err(|_| ExcelError::Xls("BIFF8 supports at most 256 columns".to_owned()))?;
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = self.sheets[sheet_index].clone();
        let existing = find_cell_record(&self.records, &sheet, row, col);
        let xf = if let Some(index) = existing {
            // Preserve the template cell's XF (styles) when overwriting a value.
            if self.records[index].data.len() >= 6 {
                u16::from_le_bytes([self.records[index].data[4], self.records[index].data[5]])
            } else {
                cell.xf
            }
        } else {
            cell.xf
        };
        let payload = encode_cell_record(row, col, xf, &cell.value)?;
        if let Some(index) = existing {
            self.records[index] = payload;
        } else {
            let insert_at = sheet_cell_insert_index(&self.records, &self.sheets[sheet_index]);
            self.records.insert(insert_at, payload);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Adds one inclusive merge range while preserving all existing BIFF records.
    ///
    /// Java `HSSFSheet.addMergedRegionUnsafe` permits multiple MERGECELLS
    /// records, so a one-range record can be inserted directly before the
    /// target worksheet EOF without rewriting pre-existing merge tables.
    ///
    /// # Errors
    ///
    /// Returns a format error when the sheet does not exist.
    pub fn add_merge_range(&mut self, sheet_name: &str, range: Biff8Merge) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let mut data = Vec::with_capacity(10);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&pack_merge_range(
            range.first_row,
            range.last_row,
            u16::from(range.first_col),
            u16::from(range.last_col),
        ));
        let insert_at = self.sheets[sheet_index].eof_index;
        self.records.insert(
            insert_at,
            RawRecord {
                typ: MERGECELLS,
                data,
            },
        );
        self.adjust_indices_after_insert(sheet_index, insert_at);
        Ok(())
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Serializes the package back to OLE/CFB bytes.
    ///
    /// # Errors
    ///
    /// Returns format or I/O errors when the Workbook stream cannot be rewritten.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.to_bytes_with_password_and_macro_policy(None, &Biff8MacroPolicy::Preserve)
    }

    /// 将修改后的模板序列化，并按调用级密码重新生成 `FILEPASS` 与加密流。
    ///
    /// # Errors
    ///
    /// BIFF 重组、随机材料生成、加密或 OLE 重写失败时返回错误。
    pub fn to_bytes_with_password(&self, password: Option<&str>) -> Result<Vec<u8>> {
        self.to_bytes_with_password_and_macro_policy(password, &Biff8MacroPolicy::Preserve)
    }

    /// 将修改后的模板按指定 VBA 策略序列化。
    ///
    /// 对应 Java：`HSSFWorkbook#write` 后对 `_VBA_PROJECT_CUR` 的显式保留、删除或替换。
    /// 本方法只复制 opaque CFB 数据，绝不解析或执行宏。
    ///
    /// # Errors
    ///
    /// BIFF 重组、密码加密、OLE 重写或替换项目格式无效时返回错误。
    pub fn to_bytes_with_password_and_macro_policy(
        &self,
        password: Option<&str>,
        macro_policy: &Biff8MacroPolicy,
    ) -> Result<Vec<u8>> {
        let mut records = self.records.clone();
        let workbook = if let Some(password) = password {
            let encryption = prepare_crypto_api_encryption(password).map_err(ExcelError::Xls)?;
            if let Some(filepass) = records.iter_mut().find(|record| record.typ == FILEPASS) {
                filepass.data = encryption.filepass_payload().to_vec();
            } else {
                let insert_at = usize::from(!records.is_empty());
                records.insert(
                    insert_at,
                    RawRecord {
                        typ: FILEPASS,
                        data: encryption.filepass_payload().to_vec(),
                    },
                );
            }
            let plaintext = assemble_workbook(&records)?;
            encrypt_crypto_api_workbook_stream(&plaintext, &encryption).map_err(ExcelError::Xls)?
        } else {
            records.retain(|record| record.typ != FILEPASS);
            assemble_workbook(&records)?
        };
        let rewritten = rewrite_workbook_stream(&self.ole_bytes, &self.workbook_path, &workbook)?;
        apply_macro_policy(&rewritten, macro_policy)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Returns all cell placeholders (`{key}` patterns) found in
    /// LABEL/LABELSST records across the workbook, resolving SST
    /// references when an SST record is present.
    ///
    /// Each entry is `(sheet_name, row, col, placeholder_text)`.
    #[must_use]
    pub fn scan_placeholders(&self) -> Vec<(String, u16, u8, String)> {
        let sst_strings = parse_sst(&self.records);
        let mut placeholders = Vec::new();
        for sheet in &self.sheets {
            for (idx, record) in self.records.iter().enumerate() {
                if idx < sheet.bof_index || idx >= sheet.eof_index {
                    continue;
                }
                let (row, col, text) = match record.typ {
                    LABEL => decode_label_payload(&record.data),
                    LABELSST => {
                        let (row, col, sst_idx) = decode_labelsst_index(&record.data);
                        let text = sst_idx.and_then(|i| sst_strings.get(i as usize).cloned());
                        (row, col, text)
                    }
                    _ => continue,
                };
                if let Some(ref text) = text
                    && text.contains('{')
                    && text.contains('}')
                {
                    placeholders.push((sheet.name.clone(), row, col, text.clone()));
                }
            }
        }
        placeholders
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 使用格式无关文本映射替换标量 `{key}` 占位符。
    ///
    /// 替换过程固定在 BIFF8 引擎内部，避免 LABEL、LABELSST、XF 保留和
    /// record 修复逻辑泄漏到门面层。返回实际替换的单元格数量。
    ///
    /// # Errors
    ///
    /// 匹配单元格无法重写时返回 BIFF8 格式错误。
    pub fn replace_scalar_placeholders(
        &mut self,
        values: &BTreeMap<String, String>,
    ) -> Result<usize> {
        let replacements = self
            .scan_placeholders()
            .into_iter()
            .filter_map(|(sheet_name, row, col, text)| {
                let key = scalar_placeholder_key(&text);
                values
                    .get(key)
                    .cloned()
                    .map(|replacement| (sheet_name, row, col, replacement))
            })
            .collect::<Vec<_>>();
        let replacement_count = replacements.len();
        for (sheet_name, row, col, replacement) in replacements {
            self.replace_label(&sheet_name, row, col, &replacement)?;
        }
        Ok(replacement_count)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 使用格式无关文本行替换集合占位符。
    ///
    /// 未命名集合匹配 `{.field}`，命名集合匹配 `{name.field}`。为保持既有
    /// XLS 仅替换值的行为，首个包含字段的输入行提供替换值。本方法不会插入
    /// BIFF 行，结构化扩展仍明确不支持。返回实际替换的单元格数量。
    ///
    /// # Errors
    ///
    /// 匹配单元格无法重写时返回 BIFF8 格式错误。
    pub fn replace_collection_placeholders(
        &mut self,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, String>],
    ) -> Result<usize> {
        self.fill_collection_placeholders(None, collection_name, rows, false, false, true)
    }

    /// 按工作表、方向和扩行策略执行 BIFF8 集合占位符填充。
    ///
    /// 对应 Java：`ExcelWriteFillExecutor#doFill` 的 `FillWrapper` 分支。
    /// `horizontal=false` 为纵向填充；`force_new_row` 会迁移锚点后的记录；
    /// `auto_style` 为真时复制模板锚点 XF。
    ///
    /// # Errors
    ///
    /// 工作表不存在、BIFF8 行列越界或记录迁移失败时返回错误。
    pub fn fill_collection_placeholders(
        &mut self,
        selected_sheet: Option<&str>,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, String>],
        horizontal: bool,
        force_new_row: bool,
        auto_style: bool,
    ) -> Result<usize> {
        if rows.is_empty() {
            return Ok(0);
        }
        let wrapper = collection_name.unwrap_or("").to_owned();
        let mut groups = BTreeMap::<(String, u16), Vec<(u8, String)>>::new();
        for (sheet_name, row, col, text) in self.placeholders.clone() {
            if selected_sheet.is_some_and(|selected| selected != sheet_name) {
                continue;
            }
            let Some(key) = collection_placeholder_key(&text, collection_name) else {
                continue;
            };
            if !key.is_empty() {
                groups.entry((sheet_name, row)).or_default().push((col, key.to_owned()));
            }
        }
        let mut replacement_count = 0usize;
        for ((sheet_name, anchor_row), fields) in groups {
            let anchor_col = fields.iter().map(|(col, _)| *col).min().unwrap_or(0);
            let cursor_key = (
                sheet_name.clone(),
                wrapper.clone(),
                anchor_row,
                anchor_col,
                horizontal,
            );
            let cursor = self.collection_cursors.get(&cursor_key).copied().unwrap_or(0);
            if horizontal {
                for (offset, values) in rows.iter().enumerate() {
                    let offset = u16::try_from(offset).map_err(|_| {
                        ExcelError::Xls("BIFF8 horizontal fill exceeds 256 columns".to_owned())
                    })?;
                    for (field_col, key) in &fields {
                        let target = u16::from(*field_col)
                            .checked_add(cursor)
                            .and_then(|value| value.checked_add(offset))
                            .ok_or_else(|| {
                                ExcelError::Xls(
                                    "BIFF8 horizontal fill exceeds 256 columns".to_owned(),
                                )
                            })?;
                        let target = u8::try_from(target).map_err(|_| {
                            ExcelError::Xls("BIFF8 horizontal fill exceeds 256 columns".to_owned())
                        })?;
                        if let Some(value) = values.get(key) {
                            let xf = if auto_style {
                                self.cell_xf(&sheet_name, anchor_row, *field_col)
                            } else {
                                XF_GENERAL
                            };
                            self.replace_label_with_xf(
                                &sheet_name,
                                anchor_row,
                                target,
                                value,
                                xf,
                            )?;
                            replacement_count += 1;
                        }
                    }
                }
                let advance = u16::try_from(rows.len()).map_err(|_| {
                    ExcelError::Xls("BIFF8 horizontal fill exceeds 256 columns".to_owned())
                })?;
                self.collection_cursors.insert(cursor_key, cursor.saturating_add(advance));
                continue;
            }

            let target_start = anchor_row.checked_add(cursor).ok_or_else(|| {
                ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
            })?;
            let available_anchor_row = usize::from(cursor == 0);
            let rows_to_insert = rows.len().saturating_sub(available_anchor_row);
            if force_new_row && rows_to_insert > 0 {
                let delta = u16::try_from(rows_to_insert).map_err(|_| {
                    ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
                })?;
                self.shift_rows(&sheet_name, target_start.saturating_add(available_anchor_row as u16), delta)?;
            }
            for (offset, values) in rows.iter().enumerate() {
                let offset = u16::try_from(offset).map_err(|_| {
                    ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
                })?;
                let target_row = target_start.checked_add(offset).ok_or_else(|| {
                    ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
                })?;
                for (field_col, key) in &fields {
                    if let Some(value) = values.get(key) {
                        let xf = if auto_style {
                            self.cell_xf(&sheet_name, anchor_row, *field_col)
                        } else {
                            XF_GENERAL
                        };
                        self.replace_label_with_xf(
                            &sheet_name,
                            target_row,
                            *field_col,
                            value,
                            xf,
                        )?;
                        replacement_count += 1;
                    }
                }
            }
            let advance = u16::try_from(rows.len()).map_err(|_| {
                ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
            })?;
            self.collection_cursors.insert(cursor_key, cursor.saturating_add(advance));
        }
        Ok(replacement_count)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Replaces a cell value at `(row, col)` on the given sheet with
    /// a new BIFF8 LABEL record containing the replacement text.
    /// If the original record was a LABELSST (SST reference), it is
    /// replaced with a LABEL record carrying the inline string value.
    ///
    /// # Errors
    ///
    /// Returns format errors for out-of-range coordinates.
    pub fn replace_label(
        &mut self,
        sheet_name: &str,
        row: u16,
        col: u8,
        replacement: &str,
    ) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = &self.sheets[sheet_index];
        let existing = find_cell_record(&self.records, sheet, row, col);
        let xf = if let Some(index) = existing {
            if self.records[index].data.len() >= 6 {
                u16::from_le_bytes([self.records[index].data[4], self.records[index].data[5]])
            } else {
                XF_GENERAL
            }
        } else {
            XF_GENERAL
        };
        // Always use LABEL (inline string) for replacements, even when
        // the original was LABELSST — this avoids SST mutation and
        // ensures the replacement text is self-contained.
        let _cell = Biff8Cell {
            value: Biff8Value::Text(replacement.to_owned()),
            xf,
        };
        // Force LABEL record type for replacement
        let payload = encode_label_record(row, col, xf, replacement)?;
        if let Some(index) = existing {
            self.records[index] = payload;
        } else {
            let insert_at = sheet_cell_insert_index(&self.records, &self.sheets[sheet_index]);
            self.records.insert(insert_at, payload);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    fn replace_label_with_xf(
        &mut self,
        sheet_name: &str,
        row: u16,
        col: u8,
        replacement: &str,
        xf: u16,
    ) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = &self.sheets[sheet_index];
        let existing = find_cell_record(&self.records, sheet, row, col);
        let payload = encode_label_record(row, col, xf, replacement)?;
        if let Some(index) = existing {
            self.records[index] = payload;
        } else {
            let insert_at = sheet_cell_insert_index(&self.records, &self.sheets[sheet_index]);
            self.records.insert(insert_at, payload);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    fn cell_xf(&self, sheet_name: &str, row: u16, col: u8) -> u16 {
        self.sheet(sheet_name)
            .ok()
            .and_then(|sheet| find_cell_record(&self.records, sheet, row, col))
            .and_then(|index| self.records[index].data.get(4..6))
            .map_or(XF_GENERAL, |bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn shift_rows(&mut self, sheet_name: &str, start_row: u16, delta: u16) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = self.sheets[sheet_index].clone();
        let extern_sheet_ranges = internal_extern_sheet_ranges(&self.records);
        for record in &mut self.records {
            if record.typ == NAME_SID {
                shift_name_references(
                    record,
                    start_row,
                    delta,
                    sheet.bound_sheet_index,
                    &extern_sheet_ranges,
                )?;
            }
        }
        let mut conditional_format_base = None;
        for record in &mut self.records[sheet.bof_index..sheet.eof_index] {
            match record.typ {
                FORMULA => {
                    shift_formula_references(
                        record,
                        start_row,
                        delta,
                        sheet.bound_sheet_index,
                        &extern_sheet_ranges,
                    )?;
                    shift_record_row(record, start_row, delta)?;
                }
                CHART_AI_SID => {
                    shift_chart_ai_references(
                        record,
                        start_row,
                        delta,
                        sheet.bound_sheet_index,
                        &extern_sheet_ranges,
                    )?;
                }
                LABEL | LABELSST | NUMBER | RK | BOOLERR | BLANK | ROW_SID | NOTE_SID => {
                    shift_record_row(record, start_row, delta)?;
                }
                HYPERLINK_SID => shift_range_rows(&mut record.data, start_row, delta)?,
                MERGECELLS => shift_merge_rows(&mut record.data, start_row, delta)?,
                CONDITIONAL_FORMATTING_HEADER_SID => {
                    conditional_format_base = Some(shift_conditional_format_header(
                        &mut record.data,
                        start_row,
                        delta,
                    )?);
                }
                CONDITIONAL_FORMATTING_RULE_SID => {
                    let (formula_row, shifted_formula_row) = conditional_format_base.ok_or_else(|| {
                        ExcelError::Xls(
                            "BIFF8 CF record is missing its preceding CONDFMT record".to_owned(),
                        )
                    })?;
                    shift_conditional_format_rule(
                        &mut record.data,
                        formula_row,
                        shifted_formula_row,
                        start_row,
                        delta,
                        sheet.bound_sheet_index,
                        &extern_sheet_ranges,
                    )?;
                }
                DATA_VALIDATION_SID => shift_data_validation(
                    &mut record.data,
                    start_row,
                    delta,
                    sheet.bound_sheet_index,
                    &extern_sheet_ranges,
                )?,
                MSO_DRAWING_SID => {
                    shift_msodrawing_anchors(&mut record.data, start_row, delta)?;
                }
                _ => {}
            }
        }
        for (placeholder_sheet, row, _, _) in &mut self.placeholders {
            if placeholder_sheet == sheet_name && *row >= start_row {
                *row = row.checked_add(delta).ok_or_else(|| {
                    ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
                })?;
            }
        }
        self.refresh_dimension(sheet_index);
        Ok(())
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Writes the package to a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors.
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.save_to_path_with_password(path, None)
    }

    /// 将模板保存到路径，并按调用级密码加密输出。
    ///
    /// # Errors
    ///
    /// 序列化、加密或 I/O 失败时返回错误。
    pub fn save_to_path_with_password(&self, path: &Path, password: Option<&str>) -> Result<()> {
        let bytes = self.to_bytes_with_password(password)?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes).map_err(ExcelError::from)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 Writes the package to an arbitrary writer.
    ///
    /// # Errors
    ///
    /// Returns I/O or format errors.
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        self.save_to_writer_with_password(output, None)
    }

    /// 将模板写入任意 writer，并按调用级密码加密输出。
    ///
    /// # Errors
    ///
    /// 序列化、加密或 I/O 失败时返回错误。
    pub fn save_to_writer_with_password(
        &self,
        output: &mut dyn Write,
        password: Option<&str>,
    ) -> Result<()> {
        let bytes = self.to_bytes_with_password(password)?;
        output.write_all(&bytes)?;
        output.flush()?;
        Ok(())
    }

    fn sheet(&self, name: &str) -> Result<&SheetSpan> {
        self.sheets
            .iter()
            .find(|sheet| sheet.name == name)
            .ok_or_else(|| ExcelError::SheetNotFound(name.to_owned()))
    }

    fn sheet_index(&self, name: &str) -> Result<usize> {
        self.sheets
            .iter()
            .position(|sheet| sheet.name == name)
            .ok_or_else(|| ExcelError::SheetNotFound(name.to_owned()))
    }

    /// After inserting a record at `insert_at`, shift later sheet indices.
    fn adjust_indices_after_insert(&mut self, sheet_index: usize, insert_at: usize) {
        for (index, sheet) in self.sheets.iter_mut().enumerate() {
            if sheet.bof_index >= insert_at {
                sheet.bof_index += 1;
            }
            if sheet.eof_index >= insert_at {
                sheet.eof_index += 1;
            }
            if let Some(dim) = sheet.dimension_index.as_mut()
                && *dim >= insert_at
            {
                *dim += 1;
            }
            if index == sheet_index {
                // Insert is always before EOF of this sheet.
                debug_assert!(sheet.eof_index > insert_at || sheet.eof_index == insert_at + 1);
            }
        }
    }

    fn refresh_dimension(&mut self, sheet_index: usize) {
        let sheet = self.sheets[sheet_index].clone();
        let (max_row, max_col) = sheet_dimensions(&self.records, &sheet);
        let mut data = Vec::with_capacity(14);
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&u32::from(max_row).to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&u16::from(max_col).to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        if let Some(dim_index) = sheet.dimension_index {
            self.records[dim_index] = RawRecord {
                typ: DIMENSION,
                data,
            };
        } else {
            let insert_at = sheet.bof_index + 1;
            self.records.insert(
                insert_at,
                RawRecord {
                    typ: DIMENSION,
                    data,
                },
            );
            self.sheets[sheet_index].dimension_index = Some(insert_at);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
    }
}

fn scalar_placeholder_key(text: &str) -> &str {
    text.trim_start_matches('{').trim_end_matches('}')
}
