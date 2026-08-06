/// 对应 Java：无直接对应对象；Rust 架构扩展。 采用 `u32 little-endian length + payload` 帧格式的 gzip 记录写入器。
pub struct GzipRecordWriter {
    path: PathBuf,
    encoder: GzEncoder<File>,
    uncompressed_len: u64,
    dir: Option<TempDir>,
}

impl GzipRecordWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定目录创建 gzip 记录文件。
    ///
    /// # Errors
    ///
    /// 临时文件无法创建或持久化时返回错误。
    pub fn create(dir: &Path, prefix: &str, suffix: &str) -> Result<Self> {
        let tmp = Builder::new()
            .prefix(prefix)
            .suffix(suffix)
            .tempfile_in(dir)?;
        let (file, path) = tmp.keep().map_err(|error| Error::Io(error.error))?;
        Ok(Self {
            path,
            encoder: GzEncoder::new(file, Compression::default()),
            uncompressed_len: 0,
            dir: None,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建自持有临时目录的 gzip 记录文件。
    ///
    /// # Errors
    ///
    /// 临时目录或记录文件无法创建时返回错误。
    pub fn create_owned(prefix: &str, suffix: &str) -> Result<Self> {
        let dir = TempDir::new()?;
        let mut writer = Self::create(dir.path(), prefix, suffix)?;
        writer.dir = Some(dir);
        Ok(writer)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 写入一条长度前缀记录。
    ///
    /// # Errors
    ///
    /// 记录超过 `u32` 长度或压缩流写入失败时返回错误。
    pub fn write_record(&mut self, payload: &[u8]) -> Result<()> {
        let len = u32::try_from(payload.len())
            .map_err(|_| Error::Other("gzip record exceeds u32 length".to_owned()))?;
        self.encoder.write_all(&len.to_le_bytes())?;
        self.encoder.write_all(payload)?;
        self.uncompressed_len = self
            .uncompressed_len
            .saturating_add(4)
            .saturating_add(u64::from(len));
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 刷新压缩器缓冲区。
    ///
    /// # Errors
    ///
    /// 压缩流刷新失败时返回错误。
    pub fn flush(&mut self) -> Result<()> {
        self.encoder.flush()?;
        Ok(())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回当前记录流状态。
    ///
    /// # Errors
    ///
    /// 刷新压缩流失败时返回错误。
    pub fn snapshot(&mut self) -> Result<GzipRecordSnapshot> {
        self.flush()?;
        Ok(snapshot_for(&self.path, self.uncompressed_len))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 完成压缩并返回流式读取器。
    ///
    /// # Errors
    ///
    /// 压缩收尾或文件重新打开失败时返回错误。
    pub fn finish(self) -> Result<GzipRecordReader> {
        let path = self.path;
        let uncompressed_len = self.uncompressed_len;
        let dir = self.dir;
        self.encoder.finish()?;
        let compressed_len = std::fs::metadata(&path).map_or(0, |meta| meta.len());
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(GzipRecordReader {
            path,
            decoder: GzDecoder::new(file),
            uncompressed_len,
            compressed_len,
            dir,
        })
    }
}

