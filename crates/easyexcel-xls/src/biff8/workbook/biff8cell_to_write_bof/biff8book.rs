/// 对应 Java：无直接对应对象；Rust 架构扩展。 Multi-sheet BIFF8 workbook buffer.
#[derive(Debug, Clone, Default)]
pub struct Biff8Book {
    /// Ordered worksheets (emission order = BOUNDSHEET order).
    pub sheets: Vec<Biff8Sheet>,
    /// Workbook-global FONT / XF registry (Java HSSF style table).
    pub styles: Biff8StyleTable,
    /// When `true`, BIFF8 `DATEMODE` uses the 1904 date windowing system.
    pub use_1904_windowing: bool,
    /// Raw bytes appended after the BIFF8 Workbook stream (images etc.)
    pub extra_bytes: Vec<u8>,
}

impl Biff8Book {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends raw bytes to be written after the BIFF8 stream in the
    /// OLE container. Used for embedding image data in the output.
    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.extra_bytes.extend_from_slice(bytes);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Encodes image bytes as BIFF8 Obj + `MSODrawing` records (Escher BSE
    /// container) and appends them to `extra_bytes`. This produces output
    /// compatible with POI's `HSSFWorkbook` image writing.
    // 语义敏感：Escher 容器长度与 BIFF 记录长度按规范为 u32/u16 字段，
    // 与原实现（POI HSSF 单记录嵌入）行为一致，保留 as 转换。
    #[allow(clippy::cast_possible_truncation)]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn write_image(&mut self, image_data: &[u8], _col: u8, _row: u32) {
        // Determine image type from magic bytes
        let blip_type: u8 = if image_data.len() >= 2 {
            match &image_data[..2] {
                [0x89, b'P'] => 0x09, // PNG
                _ => 0x07,            // JPEG / default
            }
        } else {
            0x07
        };

        let obj_id: u16 = 1;
        let image_size = image_data.len() as u32;

        // --- Obj Record (0x005D, common object) ---
        let mut obj = Vec::with_capacity(26);
        // ftCmo (Common object header, 18 bytes)
        obj.extend_from_slice(&0x0015u16.to_le_bytes()); // ftCmo
        obj.extend_from_slice(&0x0012u16.to_le_bytes()); // cbCmo = 18
        obj.extend_from_slice(&0x0008u16.to_le_bytes()); // ot = Picture
        obj.extend_from_slice(&obj_id.to_le_bytes()); // id
        obj.extend_from_slice(&0x6011u16.to_le_bytes()); // grbit
        obj.extend_from_slice(&[0u8; 4]); // reserved
        obj.extend_from_slice(&[0u8; 4]); // reserved
        obj.extend_from_slice(&[0u8; 2]); // reserved
        // ftEnd
        obj.extend_from_slice(&0x0000u16.to_le_bytes()); // ftEnd
        obj.extend_from_slice(&0x0000u16.to_le_bytes()); // cbEnd

        // --- MSODrawing Record (0x00EC) with Escher BSE ---
        let mut drawing = Vec::new();

        // === MsofbtDggContainer ===
        let dgg_start = drawing.len();
        drawing.extend_from_slice(&[0x0F, 0x00, 0x00, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0u32.to_le_bytes()); // length placeholder
        // MsofbtDgg
        drawing.extend_from_slice(&[0x00, 0x00, 0x00, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0x0000_0008u32.to_le_bytes()); // length
        drawing.extend_from_slice(&(-1i32).to_le_bytes()); // idclusters
        drawing.extend_from_slice(&1u32.to_le_bytes()); // cSavedDrawings
        drawing.extend_from_slice(&1u32.to_le_bytes()); // cSavedShapes
        let dgg_end = drawing.len();
        let dgg_len = (dgg_end - dgg_start - 4) as u32;
        drawing[dgg_start..dgg_start + 4].copy_from_slice(&dgg_len.to_le_bytes());

        // === MsofbtDgContainer ===
        drawing.extend_from_slice(&[0x0F, 0x00, 0x02, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0u32.to_le_bytes()); // length placeholder
        // MsofbtDg
        drawing.extend_from_slice(&[0x00, 0x00, 0x08, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0x0000_0008u32.to_le_bytes()); // length
        drawing.extend_from_slice(&1u32.to_le_bytes()); // drawingId
        drawing.extend_from_slice(&1u32.to_le_bytes()); // cLastSpId
        // MsofbtSpgrContainer
        drawing.extend_from_slice(&[0x0F, 0x00, 0x03, 0xF0]); // ver+inst+type
        let spgr_start = drawing.len();
        drawing.extend_from_slice(&0u32.to_le_bytes()); // length placeholder
        // MsofbtSpContainer
        drawing.extend_from_slice(&[0x0F, 0x00, 0x04, 0xF0]); // ver+inst+type
        let sp_start = drawing.len();
        drawing.extend_from_slice(&0u32.to_le_bytes()); // length placeholder
        // MsofbtSp
        drawing.extend_from_slice(&[0x00, 0x00, 0x0A, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0x0000_0008u32.to_le_bytes()); // length
        drawing.extend_from_slice(&obj_id.to_le_bytes()); // shapeId
        drawing.extend_from_slice(&0x0A00u16.to_le_bytes()); // flags
        let sp_end = drawing.len();
        let sp_len = (sp_end - sp_start - 4) as u32;
        drawing[sp_start..sp_start + 4].copy_from_slice(&sp_len.to_le_bytes());

        // MsofbtClientAnchor
        drawing.extend_from_slice(&[0x00, 0x00, 0x10, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0x0000_0008u32.to_le_bytes()); // length
        drawing.extend_from_slice(&[0u8; 8]); // 8 bytes anchor

        // MsofbtClientData
        drawing.extend_from_slice(&[0x00, 0x00, 0x11, 0xF0]); // ver+inst+type
        drawing.extend_from_slice(&0x0000_0000u32.to_le_bytes()); // length

        // Close spgr
        let spgr_end = drawing.len();
        let spgr_len = (spgr_end - spgr_start - 4) as u32;
        drawing[spgr_start..spgr_start + 4].copy_from_slice(&spgr_len.to_le_bytes());

        // Close Dg
        let dg_end2 = drawing.len();
        let dg_start2 = dgg_end; // dg container starts right after dgg
        let dg_len2 = (dg_end2 - dg_start2 - 4) as u32;
        drawing[dg_start2..dg_start2 + 4].copy_from_slice(&dg_len2.to_le_bytes());

        // === MsofbtBSE (Blip Store Entry) with embedded image ===
        drawing.extend_from_slice(&[0x02, 0x00, 0x07, 0xF0]); // ver+inst+type (BlipType depends)
        let bse_start = drawing.len();
        drawing.extend_from_slice(&0u32.to_le_bytes()); // length placeholder
        drawing.push(blip_type); // btWin32
        drawing.push(0x00); // btMacOS
        drawing.extend_from_slice(&[0u8; 16]); // rgbUid (dummy)
        drawing.extend_from_slice(&0x0000u16.to_le_bytes()); // tag
        drawing.extend_from_slice(&image_size.to_le_bytes()); // size
        drawing.extend_from_slice(&1u32.to_le_bytes()); // cRef
        drawing.extend_from_slice(&0u32.to_le_bytes()); // foDelay
        drawing.push(0x00); // usage
        drawing.push(0x00); // cbName
        drawing.push(0x00); // cbSave
        drawing.extend_from_slice(image_data); // image bytes
        // Padding to 4-byte boundary
        while drawing.len() % 4 != 0 {
            drawing.push(0x00);
        }
        let bse_end = drawing.len();
        let bse_len = (bse_end - bse_start - 4) as u32;
        drawing[bse_start..bse_start + 4].copy_from_slice(&bse_len.to_le_bytes());

        // Write as BIFF records
        let mut record_data = Vec::new();
        // Obj record
        record_data.extend_from_slice(&OBJ.to_le_bytes());
        record_data.extend_from_slice(&(obj.len() as u16).to_le_bytes());
        record_data.extend_from_slice(&obj);
        // MSODrawing record
        record_data.extend_from_slice(&MSODRAWING.to_le_bytes());
        record_data.extend_from_slice(&(drawing.len() as u16).to_le_bytes());
        record_data.extend_from_slice(&drawing);

        self.extra_bytes.extend_from_slice(&record_data);
    }
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Serializes this book to an OLE Compound File containing a `Workbook` stream.
    ///
    /// # Errors
    ///
    /// Returns I/O or CFB construction errors.
    pub fn to_cfb_bytes(&self) -> Result<Vec<u8>> {
        // 写入前对全部工作表公式求值，得到缓存值表（借用 xls 求值引擎）
        let caches = super::cached::recalc_cached_values(&self.sheets);
        let stream = build_workbook_stream(self, &caches);
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
            if !self.extra_bytes.is_empty() {
                #[rustfmt::skip]
                let mut images = cf.create_stream("Images").map_err(|error| ExcelError::Cfb(format!("cannot create Images stream: {error}")))?;
                images.write_all(&self.extra_bytes)?;
            }
            #[rustfmt::skip]
            cf.flush().map_err(|error| ExcelError::Cfb(format!("cannot flush OLE2 container: {error}")))?;
        }
        Ok(mem.into_inner())
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将 BIFF8/OLE2 工作簿写入路径，并可选应用兼容层 RC4 加密。
    ///
    /// # Errors
    ///
    /// 工作簿序列化、随机盐生成、加密或目标文件写入失败时返回错误。
    pub fn save_to_path_with_password(&self, path: &Path, password: Option<&str>) -> Result<()> {
        let Some(password) = password else {
            return self.save_to_path(path);
        };
        let bytes = self.to_cfb_bytes()?;
        let (encrypted, _, _) = super::encrypt::encrypt_biff8_stream(&bytes, password)
            .map_err(easyexcel_io::Error::Other)?;
        easyexcel_io::io::file_utils::write_to_file(path, &encrypted)
    }
}

