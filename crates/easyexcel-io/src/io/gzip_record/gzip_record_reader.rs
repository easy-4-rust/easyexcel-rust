/// 对应 Java：无直接对应对象；Rust 架构扩展。 完成后的 gzip 长度前缀记录读取器。
pub struct GzipRecordReader {
    path: PathBuf,
    decoder: GzDecoder<File>,
    uncompressed_len: u64,
    compressed_len: u64,
    #[allow(dead_code)]
    dir: Option<TempDir>,
}

impl GzipRecordReader {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 打开已有 gzip 记录文件，主要用于恢复未完成任务或诊断损坏文件。
    ///
    /// # Errors
    ///
    /// 文件元数据读取或打开失败时返回错误。
    pub fn open_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let compressed_len = std::fs::metadata(&path).map_or(0, |meta| meta.len());
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(Self {
            path,
            decoder: GzDecoder::new(file),
            uncompressed_len: 0,
            compressed_len,
            dir: None,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回完成后的记录流状态。
    #[must_use]
    pub fn snapshot(&self) -> GzipRecordSnapshot {
        GzipRecordSnapshot {
            path: self.path.clone(),
            is_gzip: file_has_gzip_magic(&self.path),
            compressed_len: self.compressed_len,
            uncompressed_len: self.uncompressed_len,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 读取下一条记录；流结束时返回 `None`。
    ///
    /// # Errors
    ///
    /// gzip 解码失败、长度前缀损坏或记录内容截断时返回错误。
    pub fn next_record(&mut self) -> Result<Option<Vec<u8>>> {
        let mut len_buf = [0u8; 4];
        match self.decoder.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(Error::Io(error)),
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.decoder.read_exact(&mut payload)?;
        Ok(Some(payload))
    }
}

