/// 模板编辑链沿用统一 BIFF8 record model；旧名称仅保留在本私有模块内，
/// 避免为 placeholder 算法复制第二种 record 容器。
type RawRecord = super::model::Biff8Record;

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

    /// 在模板工作簿中创建一个空 BIFF8 worksheet，同时保留所有既有记录与 CFB 流。
    ///
    /// 对应 Java：`HSSFWorkbook#createSheet(String)`。新增 BOUNDSHEET 放在
    /// globals EOF 前，worksheet 子流追加在 Workbook stream 末尾；保存时
    /// `assemble_workbook` 会统一修复全部 `lbPlyPos`。
    pub fn ensure_sheet(&mut self, sheet_name: &str) -> Result<bool> {
        if self.sheets.iter().any(|sheet| sheet.name == sheet_name) {
            return Ok(false);
        }
        validate_new_sheet_name(sheet_name, &self.sheets)?;
        let global_eof = top_level_substreams(&self.records)
            .first()
            .map(|(_, eof)| *eof)
            .ok_or_else(|| ExcelError::Xls("BIFF8 template has no workbook globals EOF".to_owned()))?;
        self.records.insert(global_eof, RawRecord {
            typ: BOUNDSHEET,
            data: encode_boundsheet_record_data(sheet_name)?,
        });

        self.records.extend(empty_worksheet_records());
        self.sheets = discover_sheets(&self.records)?;
        Ok(true)
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
        let snapshot = self.clone();
        let result = self.append_rows_inner(sheet_name, rows);
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    fn append_rows_inner(
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

    /// 在模板工作表中写入 BIFF8 PROTECT/OBJECTPROTECT/SCENPROTECT/PASSWORD 记录。
    pub fn protect_sheet(&mut self, sheet_name: &str, password: &str) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let records = [
            (PROTECT_SID, 1_u16),
            (OBJECT_PROTECT_SID, 1_u16),
            (SCENARIO_PROTECT_SID, 1_u16),
            (PASSWORD_SID, legacy_password_hash(password)),
        ];
        let mut insert_at = self.sheets[sheet_index].bof_index.saturating_add(1);
        for (typ, value) in records {
            let sheet = self.sheets[sheet_index].clone();
            if let Some(existing) = self.records[sheet.bof_index..sheet.eof_index]
                .iter()
                .position(|record| record.typ == typ)
                .map(|offset| sheet.bof_index + offset)
            {
                self.records[existing].data = value.to_le_bytes().to_vec();
                insert_at = insert_at.max(existing.saturating_add(1));
            } else {
                self.records.insert(
                    insert_at,
                    RawRecord {
                        typ,
                        data: value.to_le_bytes().to_vec(),
                    },
                );
                self.adjust_indices_after_insert(sheet_index, insert_at);
                insert_at = insert_at.saturating_add(1);
            }
        }
        Ok(())
    }

    /// 在模板工作表中原位追加 HLINK 记录，不把链接降级为显示文本。
    ///
    /// 对应 Java：`HSSFCell#setHyperlink`。
    #[allow(clippy::too_many_arguments)]
    pub fn add_hyperlink_range(
        &mut self,
        sheet_name: &str,
        first_row: u32,
        last_row: u32,
        first_col: usize,
        last_col: usize,
        address: String,
        label: String,
        kind: Biff8HyperlinkKind,
    ) -> Result<()> {
        let first_row = u16::try_from(first_row)
            .map_err(|_| ExcelError::Xls("BIFF8 hyperlink row exceeds 65535".to_owned()))?;
        let last_row = u16::try_from(last_row)
            .map_err(|_| ExcelError::Xls("BIFF8 hyperlink row exceeds 65535".to_owned()))?;
        let first_col = u8::try_from(first_col)
            .map_err(|_| ExcelError::Xls("BIFF8 hyperlink column exceeds 255".to_owned()))?;
        let last_col = u8::try_from(last_col)
            .map_err(|_| ExcelError::Xls("BIFF8 hyperlink column exceeds 255".to_owned()))?;
        let hyperlink = Biff8Hyperlink::new_range(
            first_row, last_row, first_col, last_col, address, label, kind,
        )?;
        let sheet_index = self.sheet_index(sheet_name)?;
        let insert_at = self.sheets[sheet_index].eof_index;
        self.records.insert(
            insert_at,
            RawRecord {
                typ: HYPERLINK_SID,
                data: hyperlink.encode_record_data(),
            },
        );
        self.adjust_indices_after_insert(sheet_index, insert_at);
        Ok(())
    }

    /// 向模板工作表批量加入批注记录组；已有 Drawing/OBJ 子流会原位扩展
    /// Escher DG/SPGR 计数和对象编号，而不是创建重复 drawing group。
    pub fn add_comments(&mut self, sheet_name: &str, comments: &[Biff8Comment]) -> Result<()> {
        let snapshot = self.clone();
        if let Err(error) = self.add_comments_inner(sheet_name, comments) {
            *self = snapshot;
            return Err(error);
        }
        Ok(())
    }

    fn add_comments_inner(&mut self, sheet_name: &str, comments: &[Biff8Comment]) -> Result<()> {
        if comments.is_empty() {
            return Ok(());
        }
        let mut effective = Vec::with_capacity(comments.len());
        for comment in comments {
            effective.retain(|existing: &Biff8Comment| {
                existing.row != comment.row || existing.col != comment.col
            });
            effective.push(comment.clone());
        }
        for comment in &effective {
            self.remove_comment(
                sheet_name,
                u32::from(comment.row),
                usize::from(comment.col),
            )?;
        }
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = self.sheets[sheet_index].clone();
        let has_drawing = self.records[sheet.bof_index..sheet.eof_index]
            .iter()
            .any(|record| record.typ == MSO_DRAWING_SID);
        let first_sheet_bof = self
            .sheets
            .iter()
            .map(|candidate| candidate.bof_index)
            .min()
            .ok_or_else(|| ExcelError::Xls("XLS template has no worksheet BOF".to_owned()))?;
        let existing_dgg = self.records[..first_sheet_bof]
            .iter()
            .position(|record| record.typ == MSODRAWINGGROUP);
        let first_shape_id = next_sheet_shape_id(&self.records, &sheet);
        let mut framed = Vec::new();
        if has_drawing {
            let drawing_id = sheet_drawing_id(&self.records, &sheet)?;
            let dgg_index = existing_dgg.ok_or_else(|| {
                ExcelError::Xls(
                    "XLS worksheet has a drawing but Workbook globals has no DGG".to_owned(),
                )
            })?;
            extend_sheet_escher_for_comments(
                &mut self.records,
                &sheet,
                &effective,
                first_shape_id,
            )?;
            extend_existing_dgg_shapes(
                &mut self.records[dgg_index].data,
                drawing_id,
                effective.len(),
                first_shape_id.saturating_add(
                    u32::try_from(effective.len()).unwrap_or(u32::MAX).saturating_sub(1),
                ),
            )?;
            super::workbook::write_appended_comments(&mut framed, &effective, first_shape_id);
        } else {
            let used_shapes = u32::try_from(effective.len())
                .unwrap_or(u32::MAX)
                .saturating_add(1);
            let drawing_id = if let Some(index) = existing_dgg {
                append_dgg_drawing(&mut self.records[index].data, used_shapes)?
            } else {
                let boundsheet_at = self.records[..first_sheet_bof]
                    .iter()
                    .position(|record| record.typ == BOUNDSHEET)
                    .unwrap_or(first_sheet_bof);
                self.records.insert(
                    boundsheet_at,
                    RawRecord {
                        typ: MSODRAWINGGROUP,
                        data: super::workbook::drawing_group_for_clusters(&[(1, used_shapes)]),
                    },
                );
                self.adjust_indices_after_global_insert(boundsheet_at);
                1
            };
            super::workbook::write_comments_with_drawing_id(
                &mut framed,
                &effective,
                drawing_id,
            );
        }
        for record in split_records(&framed)? {
            let insert_at = self.sheets[sheet_index].eof_index;
            self.records.insert(insert_at, record);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
        Ok(())
    }

    /// 删除模板工作表中指定单元格的 BIFF8 批注对象链。
    ///
    /// 对应 Java：`HSSFCell#removeCellComment()`。删除 NOTE、OBJ、TXO、文本/
    /// 格式 CONTINUE 和 Escher comment shape，并同步修正 DG/SPGR 容器长度与
    /// shape count。未知或截断的对象链 fail-closed，避免保存出部分删除文件。
    pub fn remove_comment(&mut self, sheet_name: &str, row: u32, col: usize) -> Result<bool> {
        let row = u16::try_from(row)
            .map_err(|_| ExcelError::Xls("BIFF8 comment row exceeds 65535".to_owned()))?;
        let col = u16::try_from(col)
            .map_err(|_| ExcelError::Xls("BIFF8 comment column exceeds 65535".to_owned()))?;
        if col > u16::from(u8::MAX) {
            return Err(ExcelError::Xls(
                "BIFF8 comment column exceeds 255".to_owned(),
            ));
        }
        let mut removed = false;
        loop {
            let sheet_index = self.sheet_index(sheet_name)?;
            let sheet = self.sheets[sheet_index].clone();
            let Some(note_index) = (sheet.bof_index..sheet.eof_index).find(|index| {
                let record = &self.records[*index];
                record.typ == NOTE_SID
                    && record.data.len() >= 8
                    && u16::from_le_bytes([record.data[0], record.data[1]]) == row
                    && u16::from_le_bytes([record.data[2], record.data[3]]) == col
            }) else {
                return Ok(removed);
            };
            let shape_id = u16::from_le_bytes([
                self.records[note_index].data[6],
                self.records[note_index].data[7],
            ]);
            self.remove_comment_object_chain(sheet_index, note_index, shape_id)?;
            removed = true;
        }
    }

    fn remove_comment_object_chain(
        &mut self,
        sheet_index: usize,
        note_index: usize,
        shape_id: u16,
    ) -> Result<()> {
        let sheet = self.sheets[sheet_index].clone();
        let drawing_id = sheet_drawing_id(&self.records, &sheet)?;
        let object_index = (sheet.bof_index..sheet.eof_index)
            .find(|index| {
                let record = &self.records[*index];
                record.typ == OBJ_SID
                    && record.data.len() >= 8
                    && u16::from_le_bytes([record.data[4], record.data[5]]) == 0x0019
                    && u16::from_le_bytes([record.data[6], record.data[7]]) == shape_id
            })
            .ok_or_else(|| {
                ExcelError::Xls(format!(
                    "BIFF8 comment NOTE shape {shape_id} has no matching OBJ"
                ))
            })?;
        let text_object_index = (object_index + 1..sheet.eof_index)
            .take_while(|index| {
                !matches!(self.records[*index].typ, OBJ_SID | NOTE_SID | EOF)
            })
            .find(|index| self.records[*index].typ == TEXT_OBJECT_SID)
            .ok_or_else(|| {
                ExcelError::Xls(format!(
                    "BIFF8 comment OBJ {shape_id} has no matching TXO"
                ))
            })?;

        let mut updated = self.records.clone();
        let mut escher_removed = false;
        for record in &mut updated[sheet.bof_index..sheet.eof_index] {
            if record.typ != MSO_DRAWING_SID {
                continue;
            }
            let (data, removed_shape) = remove_escher_comment_shape(&record.data, u32::from(shape_id))?;
            if removed_shape {
                record.data = data;
                escher_removed = true;
                break;
            }
        }
        if !escher_removed {
            return Err(ExcelError::Xls(format!(
                "BIFF8 comment OBJ {shape_id} has no matching Escher shape"
            )));
        }
        let mut dg_updated = false;
        for record in &mut updated[sheet.bof_index..sheet.eof_index] {
            if record.typ == MSO_DRAWING_SID
                && decrement_escher_dg_count(&mut record.data)?
            {
                dg_updated = true;
                break;
            }
        }
        if !dg_updated {
            return Err(ExcelError::Xls(
                "BIFF8 comment drawing has no Escher DG shape count".to_owned(),
            ));
        }
        let first_sheet_bof = self
            .sheets
            .iter()
            .map(|candidate| candidate.bof_index)
            .min()
            .ok_or_else(|| ExcelError::Xls("XLS template has no worksheet BOF".to_owned()))?;
        let dgg_index = updated[..first_sheet_bof]
            .iter()
            .position(|record| record.typ == MSODRAWINGGROUP)
            .ok_or_else(|| {
                ExcelError::Xls(
                    "XLS comment drawing exists but Workbook globals has no DGG".to_owned(),
                )
            })?;
        decrement_existing_dgg_shapes(&mut updated[dgg_index].data, drawing_id, 1)?;

        let mut remove = vec![note_index, object_index, text_object_index];
        let mut continuation = text_object_index + 1;
        while continuation < sheet.eof_index && updated[continuation].typ == CONTINUE_SID {
            remove.push(continuation);
            continuation += 1;
        }
        if object_index + 1 < text_object_index
            && updated[object_index + 1].typ == MSO_DRAWING_SID
            && is_empty_client_textbox_record(&updated[object_index + 1].data)
        {
            remove.push(object_index + 1);
        }
        for index in sheet.bof_index..sheet.eof_index {
            if updated[index].typ == MSO_DRAWING_SID && updated[index].data.is_empty() {
                remove.push(index);
            }
        }
        remove.sort_unstable();
        remove.dedup();
        for index in remove.into_iter().rev() {
            updated.remove(index);
        }
        self.records = updated;
        self.sheets = discover_sheets(&self.records)?;
        Ok(())
    }

    /// 返回模板中下一个可分配的 BIFF8 FONT 索引（索引 4 按规范保留）。
    #[must_use]
    pub fn next_custom_font_index(&self) -> u16 {
        let first_sheet_bof = self
            .sheets
            .iter()
            .map(|sheet| sheet.bof_index)
            .min()
            .unwrap_or(self.records.len());
        let count = self.records[..first_sheet_bof]
            .iter()
            .filter(|record| record.typ == FONT)
            .count();
        let logical = if count >= 4 { count.saturating_add(1) } else { count };
        u16::try_from(logical).unwrap_or(u16::MAX)
    }

    /// 在模板当前 FONT 表尾部追加由 EasyExcel 富文本声明分配的 FONT。
    pub fn append_custom_fonts(&mut self, fonts: &[Vec<u8>]) -> Result<()> {
        if fonts.is_empty() {
            return Ok(());
        }
        let first_sheet_bof = self
            .sheets
            .iter()
            .map(|sheet| sheet.bof_index)
            .min()
            .ok_or_else(|| ExcelError::Xls("XLS template has no worksheet BOF".to_owned()))?;
        let mut insert_at = self.records[..first_sheet_bof]
            .iter()
            .position(|record| record.typ == XF)
            .unwrap_or(first_sheet_bof);
        for font in fonts {
            if font.len() > MAX_RECORD_DATA {
                return Err(ExcelError::Xls("BIFF8 FONT record exceeds record limit".to_owned()));
            }
            self.records.insert(
                insert_at,
                RawRecord {
                    typ: FONT,
                    data: font.clone(),
                },
            );
            self.adjust_indices_after_global_insert(insert_at);
            insert_at += 1;
        }
        Ok(())
    }

    /// 向没有既有 Escher 对象表的模板工作表追加生成式 BIFF8 图表。
    ///
    /// 同时在 Workbook globals 中写入 DGG、SUPBOOK 与 EXTERNSHEET，保证
    /// 图表 AI 记录的跨 Sheet `ixti` 与模板 BOUNDSHEET 顺序一致。
    pub fn add_charts(&mut self, sheet_name: &str, charts: &[Biff8Chart]) -> Result<()> {
        if charts.is_empty() {
            return Ok(());
        }
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = self.sheets[sheet_index].clone();
        let has_sheet_drawing = self.records[sheet.bof_index..sheet.eof_index]
            .iter()
            .any(|record| record.typ == MSO_DRAWING_SID);
        let sheet_names = self.sheet_names();
        let references = charts
            .iter()
            .flat_map(|chart| chart.series.iter())
            .flat_map(|series| {
                series
                    .categories
                    .iter()
                    .map(|range| (range.sheet_name.as_str(), range.sheet_name.as_str()))
                    .chain(std::iter::once((
                        series.values.sheet_name.as_str(),
                        series.values.sheet_name.as_str(),
                    )))
            })
            .collect::<Vec<_>>();
        let existing_ixti_count = self.records[..sheet.bof_index]
            .iter()
            .filter(|record| record.typ == EXTERNAL_SHEET_SID && record.data.len() >= 2)
            .map(|record| u16::from_le_bytes([record.data[0], record.data[1]]))
            .try_fold(0_u16, |total, count| {
                total.checked_add(count).ok_or_else(|| {
                    ExcelError::Xls("template EXTERNSHEET entry count exceeds u16".to_owned())
                })
            })?;
        let existing_supbook_count = u16::try_from(
            self.records[..sheet.bof_index]
                .iter()
                .filter(|record| record.typ == SUP_BOOK_SID)
                .count(),
        )
        .map_err(|_| ExcelError::Xls("template SUPBOOK count exceeds u16".to_owned()))?;
        let link_table = super::ptg::Biff8LinkTable::from_formulas_and_references(
            &sheet_names,
            &[],
            &references,
        )
        .with_template_offsets(existing_ixti_count, existing_supbook_count);
        let first_sheet_bof = self
            .sheets
            .iter()
            .map(|candidate| candidate.bof_index)
            .min()
            .ok_or_else(|| ExcelError::Xls("XLS template has no worksheet BOF".to_owned()))?;
        let existing_dgg = self.records[..first_sheet_bof]
            .iter()
            .position(|record| record.typ == MSODRAWINGGROUP);
        let first_drawing_id = if has_sheet_drawing {
            let drawing_id = sheet_drawing_id(&self.records, &sheet)?;
            let first_shape_id = next_sheet_shape_id(&self.records, &sheet);
            extend_sheet_escher_for_charts(
                &mut self.records,
                &sheet,
                charts,
                drawing_id,
                first_shape_id,
            )?;
            if let Some(index) = existing_dgg {
                extend_existing_dgg_shapes(
                    &mut self.records[index].data,
                    drawing_id,
                    charts.len(),
                    first_shape_id.saturating_add(
                        u32::try_from(charts.len()).unwrap_or(u32::MAX).saturating_sub(1),
                    ),
                )?;
            }
            drawing_id
        } else if let Some(index) = existing_dgg {
            extend_chart_drawing_group(&mut self.records[index].data, charts.len())?
        } else {
            let boundsheet_at = self.records[..first_sheet_bof]
                .iter()
                .position(|record| record.typ == BOUNDSHEET)
                .unwrap_or(first_sheet_bof);
            self.records.insert(
                boundsheet_at,
                RawRecord {
                    typ: MSODRAWINGGROUP,
                    data: super::workbook::chart_drawing_group_for_range(1, charts.len()),
                },
            );
            self.adjust_indices_after_global_insert(boundsheet_at);
            1
        };
        if !link_table.is_empty() {
            let first_sheet_bof = self
                .sheets
                .iter()
                .map(|candidate| candidate.bof_index)
                .min()
                .unwrap_or(self.records.len());
            let global_eof = self.records[..first_sheet_bof]
                .iter()
                .rposition(|record| record.typ == EOF)
                .ok_or_else(|| ExcelError::Xls("XLS globals EOF is missing".to_owned()))?;
            for (offset, record) in [
                RawRecord {
                    typ: SUPBOOK,
                    data: link_table.supbook_payload().to_vec(),
                },
                RawRecord {
                    typ: EXTERNSHEET,
                    data: link_table.externsheet_payload(),
                },
            ]
            .into_iter()
            .enumerate()
            {
                let insert_at = global_eof + offset;
                self.records.insert(insert_at, record);
                self.adjust_indices_after_global_insert(insert_at);
            }
        }
        let mut framed = Vec::new();
        let first_object_id = next_sheet_object_id(&self.records, &self.sheets[sheet_index]);
        if has_sheet_drawing {
            super::workbook::write_appended_charts(
                &mut framed,
                charts,
                &link_table,
                first_drawing_id,
                next_sheet_shape_id(&self.records, &self.sheets[sheet_index]),
                first_object_id,
            );
        } else {
            super::workbook::write_charts_with_drawing_ids(
                &mut framed,
                charts,
                &link_table,
                first_drawing_id,
                first_object_id,
            );
        }
        for record in split_records(&framed)? {
            let insert_at = self.sheets[sheet_index].eof_index;
            self.records.insert(insert_at, record);
            self.adjust_indices_after_insert(sheet_index, insert_at);
        }
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
        self.replace_scalar_placeholders_on_sheet(None, values)
    }

    /// 仅在选定工作表内替换标量占位符；`None` 表示全部工作表。
    ///
    /// 对应 Java：`ExcelWriter.fill(data, WriteSheet)` 的 Sheet 选择语义。
    pub fn replace_scalar_placeholders_on_sheet(
        &mut self,
        selected_sheet: Option<&str>,
        values: &BTreeMap<String, String>,
    ) -> Result<usize> {
        let cells = values
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    Biff8Cell::general(Biff8Value::Text(value.clone())),
                )
            })
            .collect::<BTreeMap<_, _>>();
        self.replace_scalar_cells_on_sheet(selected_sheet, &cells)
            .map(|placements| placements.len())
    }

    /// 使用类型化 BIFF8 单元格替换标量占位符，并返回最终物理位置。
    ///
    /// 该入口承载模板定位、原样式保留与事务回滚；上层只负责把领域值适配为
    /// [`Biff8Cell`]，并根据返回位置写入超链接、批注等独立记录。
    pub fn replace_scalar_cells_on_sheet(
        &mut self,
        selected_sheet: Option<&str>,
        values: &BTreeMap<String, Biff8Cell>,
    ) -> Result<Vec<(String, u16, u8, String)>> {
        let snapshot = self.clone();
        let result = self.replace_scalar_cells_on_sheet_inner(selected_sheet, values);
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    fn replace_scalar_cells_on_sheet_inner(
        &mut self,
        selected_sheet: Option<&str>,
        values: &BTreeMap<String, Biff8Cell>,
    ) -> Result<Vec<(String, u16, u8, String)>> {
        if let Some(sheet_name) = selected_sheet {
            self.sheet_index(sheet_name)?;
        }
        let replacements = self
            .scan_placeholders()
            .into_iter()
            .filter(|(sheet_name, _, _, _)| {
                selected_sheet.is_none_or(|selected| selected == sheet_name)
            })
            .filter_map(|(sheet_name, row, col, text)| {
                let key = scalar_placeholder_key(&text);
                values.get(key).cloned().map(|replacement| {
                    (sheet_name, row, col, key.to_owned(), replacement)
                })
            })
            .collect::<Vec<_>>();
        let mut placements = Vec::with_capacity(replacements.len());
        for (sheet_name, row, col, key, replacement) in replacements {
            self.set_cell(&sheet_name, u32::from(row), usize::from(col), &replacement)?;
            placements.push((sheet_name, row, col, key));
        }
        Ok(placements)
    }

    /// 对应 Java：HSSFSheet#getLastRowNum。 使用格式无关文本行替换集合占位符。
    ///
    /// 未命名集合匹配 `{.field}`，命名集合匹配 `{name.field}`。该兼容入口
    /// 使用纵向、不强制迁移尾部行的默认配置；集合仍会按输入行数扩展并复用
    /// 模板锚点样式。返回实际替换的单元格数量。
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
        let cells = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(key, value)| {
                        (
                            key.clone(),
                            Biff8Cell::general(Biff8Value::Text(value.clone())),
                        )
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();
        self.fill_collection_cells(
            selected_sheet,
            collection_name,
            &cells,
            horizontal,
            force_new_row,
            auto_style,
        )
        .map(|placements| placements.len())
    }

    /// 使用类型化 BIFF8 单元格执行集合填充，并返回输入行、字段与最终坐标。
    pub fn fill_collection_cells(
        &mut self,
        selected_sheet: Option<&str>,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, Biff8Cell>],
        horizontal: bool,
        force_new_row: bool,
        auto_style: bool,
    ) -> Result<Vec<(String, u16, u8, usize, String)>> {
        let snapshot = self.clone();
        let result = self.fill_collection_cells_inner(
            selected_sheet,
            collection_name,
            rows,
            horizontal,
            force_new_row,
            auto_style,
        );
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    fn fill_collection_cells_inner(
        &mut self,
        selected_sheet: Option<&str>,
        collection_name: Option<&str>,
        rows: &[BTreeMap<String, Biff8Cell>],
        horizontal: bool,
        force_new_row: bool,
        auto_style: bool,
    ) -> Result<Vec<(String, u16, u8, usize, String)>> {
        if let Some(sheet_name) = selected_sheet {
            self.sheet_index(sheet_name)?;
        }
        if rows.is_empty() {
            return Ok(Vec::new());
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
        let mut placements = Vec::new();
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
                for (input_row, values) in rows.iter().enumerate() {
                    let offset = u16::try_from(input_row).map_err(|_| {
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
                            self.set_cell_with_xf(
                                &sheet_name,
                                anchor_row,
                                target,
                                value,
                                xf,
                            )?;
                            placements.push((
                                sheet_name.clone(),
                                anchor_row,
                                target,
                                input_row,
                                key.clone(),
                            ));
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
            for (input_row, values) in rows.iter().enumerate() {
                let offset = u16::try_from(input_row).map_err(|_| {
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
                        self.set_cell_with_xf(
                            &sheet_name,
                            target_row,
                            *field_col,
                            value,
                            xf,
                        )?;
                        placements.push((
                            sheet_name.clone(),
                            target_row,
                            *field_col,
                            input_row,
                            key.clone(),
                        ));
                    }
                }
            }
            let advance = u16::try_from(rows.len()).map_err(|_| {
                ExcelError::Xls("BIFF8 collection fill exceeds 65536 rows".to_owned())
            })?;
            self.collection_cursors.insert(cursor_key, cursor.saturating_add(advance));
        }
        Ok(placements)
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

    /// 使用调用方解析后的 XF 写入类型化单元格。
    ///
    /// 集合填充的 `auto_style` 决策属于模板引擎：开启时复制锚点 XF，关闭时
    /// 固定使用通用 XF；值类型仍由 [`Biff8Cell`] 决定。
    fn set_cell_with_xf(
        &mut self,
        sheet_name: &str,
        row: u16,
        col: u8,
        cell: &Biff8Cell,
        xf: u16,
    ) -> Result<()> {
        let sheet_index = self.sheet_index(sheet_name)?;
        let sheet = &self.sheets[sheet_index];
        let existing = find_cell_record(&self.records, sheet, row, col);
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

    fn adjust_indices_after_global_insert(&mut self, insert_at: usize) {
        for sheet in &mut self.sheets {
            if sheet.bof_index >= insert_at {
                sheet.bof_index += 1;
            }
            if sheet.eof_index >= insert_at {
                sheet.eof_index += 1;
            }
            if let Some(dimension_index) = sheet.dimension_index.as_mut()
                && *dimension_index >= insert_at
            {
                *dimension_index += 1;
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

fn validate_new_sheet_name(sheet_name: &str, sheets: &[SheetSpan]) -> Result<()> {
    let utf16_len = sheet_name.encode_utf16().count();
    if utf16_len == 0 || utf16_len > 31 {
        return Err(ExcelError::Xls("BIFF8 sheet name must contain 1..=31 UTF-16 units".to_owned()));
    }
    if sheet_name.chars().any(|value| matches!(value, '\0' | ':' | '\\' | '/' | '?' | '*' | '[' | ']')) {
        return Err(ExcelError::Xls(format!("invalid BIFF8 sheet name: {sheet_name}")));
    }
    if sheets.iter().any(|sheet| sheet.name.eq_ignore_ascii_case(sheet_name)) {
        return Err(ExcelError::Xls(format!("duplicate BIFF8 sheet name: {sheet_name}")));
    }
    Ok(())
}

fn encode_boundsheet_record_data(sheet_name: &str) -> Result<Vec<u8>> {
    let units = sheet_name.encode_utf16().collect::<Vec<_>>();
    let compressed = units.iter().all(|unit| *unit <= 0xFF);
    let mut data = Vec::with_capacity(8 + units.len().saturating_mul(2));
    data.extend_from_slice(&0u32.to_le_bytes());
    data.push(0); // visible
    data.push(0); // worksheet
    data.push(u8::try_from(units.len()).map_err(|_| ExcelError::Xls("BIFF8 sheet name is too long".to_owned()))?);
    data.push(u8::from(!compressed));
    if compressed {
        data.extend(units.into_iter().map(|unit| u8::try_from(unit).unwrap_or(b'?')));
    } else {
        for unit in units { data.extend_from_slice(&unit.to_le_bytes()); }
    }
    Ok(data)
}

fn empty_worksheet_records() -> Vec<RawRecord> {
    let mut bof = Vec::with_capacity(16);
    bof.extend_from_slice(&0x0600u16.to_le_bytes());
    bof.extend_from_slice(&DT_WORKSHEET.to_le_bytes());
    bof.extend_from_slice(&0x0DBBu16.to_le_bytes());
    bof.extend_from_slice(&0x07CCu16.to_le_bytes());
    bof.extend_from_slice(&0x0000_0041u32.to_le_bytes());
    bof.extend_from_slice(&0x0000_0006u32.to_le_bytes());
    let mut dimension = Vec::with_capacity(14);
    dimension.extend_from_slice(&0u32.to_le_bytes());
    dimension.extend_from_slice(&0u32.to_le_bytes());
    dimension.extend_from_slice(&0u16.to_le_bytes());
    dimension.extend_from_slice(&0u16.to_le_bytes());
    dimension.extend_from_slice(&0u16.to_le_bytes());
    let mut window2 = vec![0u8; 18];
    window2[0] = 0xB6;
    window2[1] = 0x06;
    vec![
        RawRecord { typ: BOF, data: bof },
        RawRecord { typ: DIMENSION, data: dimension },
        RawRecord { typ: WINDOW2, data: window2 },
        RawRecord { typ: EOF, data: Vec::new() },
    ]
}

fn next_sheet_shape_id(records: &[RawRecord], sheet: &SheetSpan) -> u32 {
    let mut maximum = 1_024_u32;
    for record in &records[sheet.bof_index..sheet.eof_index] {
        if record.typ == OBJ_SID && record.data.len() >= 8 {
            maximum = maximum.max(u32::from(u16::from_le_bytes([
                record.data[6],
                record.data[7],
            ])));
        }
        if record.typ == NOTE_SID && record.data.len() >= 8 {
            maximum = maximum.max(u32::from(u16::from_le_bytes([
                record.data[6],
                record.data[7],
            ])));
        }
        if record.typ == MSO_DRAWING_SID {
            let mut offset: usize = 0;
            while offset.saturating_add(12) <= record.data.len() {
                let record_type = u16::from_le_bytes([
                    record.data[offset + 2],
                    record.data[offset + 3],
                ]);
                if record_type == 0xF00A {
                    maximum = maximum.max(u32::from_le_bytes([
                        record.data[offset + 8],
                        record.data[offset + 9],
                        record.data[offset + 10],
                        record.data[offset + 11],
                    ]));
                }
                offset = offset.saturating_add(1);
            }
        }
    }
    maximum.saturating_add(1).max(1_025)
}

fn is_empty_client_textbox_record(data: &[u8]) -> bool {
    data.len() == 8
        && u16::from_le_bytes([data[2], data[3]]) == 0xF00D
        && u32::from_le_bytes([data[4], data[5], data[6], data[7]]) == 0
}

fn remove_escher_comment_shape(data: &[u8], shape_id: u32) -> Result<(Vec<u8>, bool)> {
    remove_escher_records(data, shape_id)
}

fn remove_escher_records(data: &[u8], shape_id: u32) -> Result<(Vec<u8>, bool)> {
    let mut output = Vec::with_capacity(data.len());
    let mut offset = 0usize;
    let mut removed = false;
    while offset < data.len() {
        if offset.saturating_add(8) > data.len() {
            return Err(ExcelError::Xls(
                "truncated Escher record while removing BIFF8 comment".to_owned(),
            ));
        }
        let options = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let record_type = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let payload_len = usize::try_from(u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]))
        .unwrap_or(usize::MAX);
        let end = offset
            .checked_add(8)
            .and_then(|value| value.checked_add(payload_len))
            .filter(|end| *end <= data.len())
            .ok_or_else(|| {
                ExcelError::Xls("Escher record length exceeds drawing payload".to_owned())
            })?;
        let payload = &data[offset + 8..end];
        if record_type == 0xF004 && escher_shape_container_id(payload) == Some(shape_id) {
            removed = true;
            offset = end;
            continue;
        }

        let is_container = options & 0x000F == 0x000F;
        let (next_payload, child_removed) = if is_container {
            remove_escher_records(payload, shape_id)?
        } else {
            (payload.to_vec(), false)
        };
        output.extend_from_slice(&options.to_le_bytes());
        output.extend_from_slice(&record_type.to_le_bytes());
        output.extend_from_slice(
            &u32::try_from(next_payload.len())
                .map_err(|_| ExcelError::Xls("Escher payload exceeds 4 GiB".to_owned()))?
                .to_le_bytes(),
        );
        output.extend_from_slice(&next_payload);
        removed |= child_removed;
        offset = end;
    }
    Ok((output, removed))
}

fn decrement_escher_dg_count(data: &mut [u8]) -> Result<bool> {
    let mut offset = 0usize;
    while offset < data.len() {
        if offset.saturating_add(8) > data.len() {
            return Err(ExcelError::Xls(
                "truncated Escher record while updating comment shape count".to_owned(),
            ));
        }
        let options = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let record_type = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let payload_len = usize::try_from(u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]))
        .unwrap_or(usize::MAX);
        let payload_start = offset + 8;
        let end = payload_start
            .checked_add(payload_len)
            .filter(|end| *end <= data.len())
            .ok_or_else(|| {
                ExcelError::Xls("Escher record length exceeds drawing payload".to_owned())
            })?;
        if record_type == 0xF008 && payload_len >= 8 {
            let count = u32::from_le_bytes([
                data[payload_start],
                data[payload_start + 1],
                data[payload_start + 2],
                data[payload_start + 3],
            ]);
            data[payload_start..payload_start + 4]
                .copy_from_slice(&count.saturating_sub(1).max(1).to_le_bytes());
            return Ok(true);
        }
        if options & 0x000F == 0x000F
            && decrement_escher_dg_count(&mut data[payload_start..end])?
        {
            return Ok(true);
        }
        offset = end;
    }
    Ok(false)
}

fn escher_shape_container_id(payload: &[u8]) -> Option<u32> {
    let mut offset = 0usize;
    while offset.saturating_add(16) <= payload.len() {
        let record_type = u16::from_le_bytes([payload[offset + 2], payload[offset + 3]]);
        let length = usize::try_from(u32::from_le_bytes([
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
        ]))
        .ok()?;
        let end = offset.checked_add(8)?.checked_add(length)?;
        if end > payload.len() {
            return None;
        }
        if record_type == 0xF00A && length >= 4 {
            return Some(u32::from_le_bytes([
                payload[offset + 8],
                payload[offset + 9],
                payload[offset + 10],
                payload[offset + 11],
            ]));
        }
        offset = end;
    }
    None
}

fn next_sheet_object_id(records: &[RawRecord], sheet: &SheetSpan) -> u16 {
    records[sheet.bof_index..sheet.eof_index]
        .iter()
        .filter(|record| record.typ == OBJ_SID && record.data.len() >= 8)
        .map(|record| u16::from_le_bytes([record.data[6], record.data[7]]))
        .max()
        .unwrap_or(1)
        .saturating_add(1)
}

fn sheet_drawing_id(records: &[RawRecord], sheet: &SheetSpan) -> Result<u16> {
    for record in &records[sheet.bof_index..sheet.eof_index] {
        if record.typ != MSO_DRAWING_SID {
            continue;
        }
        for offset in 0..record.data.len().saturating_sub(8) {
            if u16::from_le_bytes([record.data[offset + 2], record.data[offset + 3]]) == 0xF008 {
                let options = u16::from_le_bytes([record.data[offset], record.data[offset + 1]]);
                return Ok(options >> 4);
            }
        }
    }
    Err(ExcelError::Xls(
        "existing XLS drawing stream has no Escher DG record".to_owned(),
    ))
}

fn extend_existing_dgg_shapes(
    data: &mut [u8],
    drawing_id: u16,
    shape_count: usize,
    last_shape_id: u32,
) -> Result<()> {
    let offset = (0..data.len().saturating_sub(24))
        .find(|offset| u16::from_le_bytes([data[offset + 2], data[offset + 3]]) == 0xF006)
        .ok_or_else(|| ExcelError::Xls("existing XLS DGG has no Escher Dgg record".to_owned()))?;
    let payload = offset.saturating_add(8);
    let payload_len = usize::try_from(u32::from_le_bytes([
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]))
    .unwrap_or(0);
    if payload.saturating_add(payload_len) > data.len() || payload_len < 24 {
        return Err(ExcelError::Xls("existing XLS DGG payload is truncated".to_owned()));
    }
    let current_max = u32::from_le_bytes([
        data[payload], data[payload + 1], data[payload + 2], data[payload + 3],
    ]);
    data[payload..payload + 4]
        .copy_from_slice(&current_max.max(last_shape_id.saturating_add(1)).to_le_bytes());
    let saved_shapes = u32::from_le_bytes([
        data[payload + 8], data[payload + 9], data[payload + 10], data[payload + 11],
    ]);
    data[payload + 8..payload + 12].copy_from_slice(
        &saved_shapes
            .saturating_add(u32::try_from(shape_count).unwrap_or(u32::MAX))
            .to_le_bytes(),
    );
    for cluster in (payload + 16..payload + payload_len).step_by(8) {
        if cluster.saturating_add(8) > data.len() {
            break;
        }
        if u32::from_le_bytes([
            data[cluster], data[cluster + 1], data[cluster + 2], data[cluster + 3],
        ]) == u32::from(drawing_id)
        {
            let used = u32::from_le_bytes([
                data[cluster + 4], data[cluster + 5], data[cluster + 6], data[cluster + 7],
            ]);
            data[cluster + 4..cluster + 8].copy_from_slice(
                &used
                    .saturating_add(u32::try_from(shape_count).unwrap_or(u32::MAX))
                    .to_le_bytes(),
            );
            return Ok(());
        }
    }
    Err(ExcelError::Xls(
        "existing XLS DGG has no cluster for the worksheet drawing".to_owned(),
    ))
}

fn decrement_existing_dgg_shapes(
    data: &mut [u8],
    drawing_id: u16,
    shape_count: usize,
) -> Result<()> {
    let offset = (0..data.len().saturating_sub(24))
        .find(|offset| u16::from_le_bytes([data[offset + 2], data[offset + 3]]) == 0xF006)
        .ok_or_else(|| ExcelError::Xls("existing XLS DGG has no Escher Dgg record".to_owned()))?;
    let payload = offset.saturating_add(8);
    let payload_len = usize::try_from(u32::from_le_bytes([
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ]))
    .unwrap_or(0);
    if payload.saturating_add(payload_len) > data.len() || payload_len < 24 {
        return Err(ExcelError::Xls("existing XLS DGG payload is truncated".to_owned()));
    }
    let decrement = u32::try_from(shape_count).unwrap_or(u32::MAX);
    let saved_shapes = u32::from_le_bytes([
        data[payload + 8], data[payload + 9], data[payload + 10], data[payload + 11],
    ]);
    data[payload + 8..payload + 12]
        .copy_from_slice(&saved_shapes.saturating_sub(decrement).to_le_bytes());
    for cluster in (payload + 16..payload + payload_len).step_by(8) {
        if cluster.saturating_add(8) > data.len() {
            break;
        }
        if u32::from_le_bytes([
            data[cluster], data[cluster + 1], data[cluster + 2], data[cluster + 3],
        ]) == u32::from(drawing_id)
        {
            let used = u32::from_le_bytes([
                data[cluster + 4], data[cluster + 5], data[cluster + 6], data[cluster + 7],
            ]);
            data[cluster + 4..cluster + 8]
                .copy_from_slice(&used.saturating_sub(decrement).max(1).to_le_bytes());
            return Ok(());
        }
    }
    Err(ExcelError::Xls(
        "existing XLS DGG has no cluster for the worksheet drawing".to_owned(),
    ))
}

fn extend_sheet_escher_for_charts(
    records: &mut [RawRecord],
    sheet: &SheetSpan,
    charts: &[Biff8Chart],
    drawing_id: u16,
    first_shape_id: u32,
) -> Result<()> {
    let added_shapes = u32::try_from(charts.len()).unwrap_or(u32::MAX);
    let added_bytes = charts
        .iter()
        .enumerate()
        .map(|(index, chart)| {
            super::workbook::appended_chart_shape_len(
                chart,
                drawing_id,
                first_shape_id.saturating_add(u32::try_from(index).unwrap_or(u32::MAX)),
            )
        })
        .sum::<usize>();
    extend_sheet_escher_headers(
        records,
        sheet,
        u32::try_from(added_bytes).unwrap_or(u32::MAX),
        added_shapes,
        first_shape_id.saturating_add(added_shapes.saturating_sub(1)),
    )
}

fn extend_sheet_escher_headers(
    records: &mut [RawRecord],
    sheet: &SheetSpan,
    added_bytes: u32,
    added_shapes: u32,
    last_shape_id: u32,
) -> Result<()> {
    let mut updated_dg_container = false;
    let mut updated_spgr_container = false;
    let mut updated_dg = false;
    for record in &mut records[sheet.bof_index..sheet.eof_index] {
        if record.typ != MSO_DRAWING_SID {
            continue;
        }
        for offset in 0..record.data.len().saturating_sub(8) {
            let options = u16::from_le_bytes([record.data[offset], record.data[offset + 1]]);
            let record_type = u16::from_le_bytes([
                record.data[offset + 2], record.data[offset + 3],
            ]);
            let declared = u32::from_le_bytes([
                record.data[offset + 4], record.data[offset + 5],
                record.data[offset + 6], record.data[offset + 7],
            ]);
            if record_type == 0xF002 && options & 0x000F == 0x000F && !updated_dg_container {
                record.data[offset + 4..offset + 8]
                    .copy_from_slice(&declared.saturating_add(added_bytes).to_le_bytes());
                updated_dg_container = true;
            } else if record_type == 0xF003
                && options & 0x000F == 0x000F
                && !updated_spgr_container
            {
                record.data[offset + 4..offset + 8]
                    .copy_from_slice(&declared.saturating_add(added_bytes).to_le_bytes());
                updated_spgr_container = true;
            } else if record_type == 0xF008
                && offset.saturating_add(16) <= record.data.len()
                && !updated_dg
            {
                let shape_count = u32::from_le_bytes([
                    record.data[offset + 8], record.data[offset + 9],
                    record.data[offset + 10], record.data[offset + 11],
                ]);
                record.data[offset + 8..offset + 12]
                    .copy_from_slice(&shape_count.saturating_add(added_shapes).to_le_bytes());
                record.data[offset + 12..offset + 16]
                    .copy_from_slice(&last_shape_id.to_le_bytes());
                updated_dg = true;
            }
        }
    }
    if updated_dg_container && updated_spgr_container && updated_dg {
        Ok(())
    } else {
        Err(ExcelError::Xls(
            "existing XLS drawing stream has no extensible Escher DG/SPGR headers".to_owned(),
        ))
    }
}

fn append_dgg_drawing(data: &mut Vec<u8>, used_shapes: u32) -> Result<u16> {
    let offset = (0..data.len().saturating_sub(8))
        .find(|offset| {
            u16::from_le_bytes([data[offset + 2], data[offset + 3]]) == 0xF006
                && offset.saturating_add(24) <= data.len()
        })
        .ok_or_else(|| ExcelError::Xls("existing XLS DGG has no Escher Dgg record".to_owned()))?;
    let payload = offset.saturating_add(8);
    let payload_len = u32::from_le_bytes([
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    let payload_end = payload.saturating_add(usize::try_from(payload_len).unwrap_or(usize::MAX));
    if payload_end > data.len() || payload.saturating_add(16) > data.len() {
        return Err(ExcelError::Xls("existing XLS DGG payload is truncated".to_owned()));
    }
    let saved_drawings = u32::from_le_bytes([
        data[payload + 12],
        data[payload + 13],
        data[payload + 14],
        data[payload + 15],
    ]);
    let maximum_drawing_id = (payload + 16..payload_end)
        .step_by(8)
        .filter(|cluster| cluster.saturating_add(8) <= data.len())
        .map(|cluster| {
            u32::from_le_bytes([
                data[cluster],
                data[cluster + 1],
                data[cluster + 2],
                data[cluster + 3],
            ])
        })
        .max()
        .unwrap_or(0);
    let drawing_id = maximum_drawing_id.saturating_add(1);
    let mut cluster = Vec::with_capacity(8);
    cluster.extend_from_slice(&drawing_id.to_le_bytes());
    cluster.extend_from_slice(&used_shapes.to_le_bytes());
    data.splice(payload_end..payload_end, cluster.iter().copied());
    let added_len = 8_u32;
    data[offset + 4..offset + 8]
        .copy_from_slice(&payload_len.saturating_add(added_len).to_le_bytes());
    if data.len() >= 8 && u16::from_le_bytes([data[2], data[3]]) == 0xF000 {
        let container_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        data[4..8].copy_from_slice(&container_len.saturating_add(added_len).to_le_bytes());
    }
    let current_max = u32::from_le_bytes([
        data[payload], data[payload + 1], data[payload + 2], data[payload + 3],
    ]);
    let next_shape_id = drawing_id
        .saturating_mul(1_024)
        .saturating_add(used_shapes);
    data[payload..payload + 4]
        .copy_from_slice(&current_max.max(next_shape_id).to_le_bytes());
    let cluster_count = u32::from_le_bytes([
        data[payload + 4], data[payload + 5], data[payload + 6], data[payload + 7],
    ]);
    data[payload + 4..payload + 8]
        .copy_from_slice(&cluster_count.saturating_add(1).to_le_bytes());
    let saved_shapes = u32::from_le_bytes([
        data[payload + 8], data[payload + 9], data[payload + 10], data[payload + 11],
    ]);
    data[payload + 8..payload + 12]
        .copy_from_slice(&saved_shapes.saturating_add(used_shapes).to_le_bytes());
    data[payload + 12..payload + 16]
        .copy_from_slice(&saved_drawings.saturating_add(1).to_le_bytes());
    u16::try_from(drawing_id)
        .map_err(|_| ExcelError::Xls("XLS drawing id exceeds BIFF8 range".to_owned()))
}

fn extend_chart_drawing_group(data: &mut Vec<u8>, chart_count: usize) -> Result<u16> {
    let mut first_drawing_id = None;
    for _ in 0..chart_count {
        let drawing_id = append_dgg_drawing(data, 3)?;
        first_drawing_id.get_or_insert(drawing_id);
    }
    first_drawing_id.ok_or_else(|| ExcelError::Xls("cannot add an empty chart drawing group".to_owned()))
}

fn extend_sheet_escher_for_comments(
    records: &mut [RawRecord],
    sheet: &SheetSpan,
    comments: &[Biff8Comment],
    first_shape_id: u32,
) -> Result<()> {
    let added_shapes = u32::try_from(comments.len()).unwrap_or(u32::MAX);
    let added_bytes = comments
        .iter()
        .enumerate()
        .map(|(index, comment)| {
            super::workbook::appended_comment_shape_len(
                comment,
                first_shape_id.saturating_add(u32::try_from(index).unwrap_or(u32::MAX)),
            )
        })
        .sum::<usize>();
    let added_bytes = u32::try_from(added_bytes)
        .map_err(|_| ExcelError::Xls("BIFF8 comment Escher stream exceeds 4 GiB".to_owned()))?;
    let mut updated_dg_container = false;
    let mut updated_spgr_container = false;
    let mut updated_dg = false;
    for record in &mut records[sheet.bof_index..sheet.eof_index] {
        if record.typ != MSO_DRAWING_SID {
            continue;
        }
        let mut offset: usize = 0;
        while offset.saturating_add(8) <= record.data.len() {
            let options = u16::from_le_bytes([record.data[offset], record.data[offset + 1]]);
            let record_type = u16::from_le_bytes([
                record.data[offset + 2],
                record.data[offset + 3],
            ]);
            let declared = u32::from_le_bytes([
                record.data[offset + 4],
                record.data[offset + 5],
                record.data[offset + 6],
                record.data[offset + 7],
            ]);
            if record_type == 0xF002 && options & 0x000F == 0x000F && !updated_dg_container {
                record.data[offset + 4..offset + 8]
                    .copy_from_slice(&declared.saturating_add(added_bytes).to_le_bytes());
                updated_dg_container = true;
            } else if record_type == 0xF003
                && options & 0x000F == 0x000F
                && !updated_spgr_container
            {
                record.data[offset + 4..offset + 8]
                    .copy_from_slice(&declared.saturating_add(added_bytes).to_le_bytes());
                updated_spgr_container = true;
            } else if record_type == 0xF008
                && offset.saturating_add(16) <= record.data.len()
                && !updated_dg
            {
                let shape_count = u32::from_le_bytes([
                    record.data[offset + 8],
                    record.data[offset + 9],
                    record.data[offset + 10],
                    record.data[offset + 11],
                ]);
                record.data[offset + 8..offset + 12]
                    .copy_from_slice(&shape_count.saturating_add(added_shapes).to_le_bytes());
                let last_shape_id = first_shape_id.saturating_add(added_shapes.saturating_sub(1));
                record.data[offset + 12..offset + 16]
                    .copy_from_slice(&last_shape_id.to_le_bytes());
                updated_dg = true;
            }
            offset = offset.saturating_add(1);
        }
    }
    if updated_dg_container && updated_spgr_container && updated_dg {
        Ok(())
    } else {
        Err(ExcelError::Xls(
            "existing XLS drawing stream has no extensible Escher DG/SPGR headers".to_owned(),
        ))
    }
}

fn scalar_placeholder_key(text: &str) -> &str {
    text.trim_start_matches('{').trim_end_matches('}')
}
