/// 对应 Java：无直接对应对象；Rust 架构扩展。 带字符集解码的 CSV 增量记录读取器。
pub struct CsvRecordReader<R: Read> {
    inner: csv::Reader<DecodeReaderBytes<R, Vec<u8>>>,
}

impl<R: Read> CsvRecordReader<R> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从字节输入和字符集创建记录读取器。
    ///
    /// # Errors
    ///
    /// 字符集不受支持时返回错误。
    pub fn new(reader: R, charset: &CsvCharset) -> Result<Self> {
        Ok(Self {
            inner: csv::ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .from_reader(decode_reader(reader, charset)?),
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回逐行 UTF-8 字段记录。
    pub fn records(&mut self) -> impl Iterator<Item = Result<Vec<String>>> + '_ {
        self.inner.records().map(|record| {
            record
                .map(|record| record.iter().map(str::to_owned).collect())
                .map_err(Error::from)
        })
    }
}

impl CsvRecordReader<File> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 打开文件路径并创建记录读取器。
    ///
    /// # Errors
    ///
    /// 文件打开或字符集解析失败时返回错误。
    pub fn from_path(path: &Path, charset: &CsvCharset) -> Result<Self> {
        Self::new(File::open(path)?, charset)
    }
}

