/// 对应 Java：无直接对应对象；Rust 架构扩展。 带字符集转码和 BOM 策略的 CSV 增量记录写入器。
pub struct CsvRecordWriter {
    inner: csv::Writer<CsvEncodingWriter>,
}

impl CsvRecordWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 从任意输出流创建记录写入器。
    ///
    /// # Errors
    ///
    /// 字符集不受支持或 BOM 写入失败时返回错误。
    pub fn new(
        mut output: Box<dyn Write + Send>,
        charset: &CsvCharset,
        with_bom: bool,
    ) -> Result<Self> {
        let encoding = csv_encoding(charset)?;
        if with_bom {
            output.write_all(csv_bom(encoding))?;
        }
        Ok(Self {
            inner: csv::WriterBuilder::new()
                .flexible(true)
                .from_writer(CsvEncodingWriter::new(output, encoding)),
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建写入文件路径的记录写入器。
    ///
    /// # Errors
    ///
    /// 文件创建、字符集解析或 BOM 写入失败时返回错误。
    pub fn from_path(path: &Path, charset: &CsvCharset, with_bom: bool) -> Result<Self> {
        Self::new(Box::new(File::create(path)?), charset, with_bom)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 写入一条 CSV 记录。
    ///
    /// # Errors
    ///
    /// CSV 转义、字符集转码或底层输出失败时返回错误。
    pub fn write_record<I, T>(&mut self, record: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        self.inner.write_record(record).map_err(Error::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 刷新 CSV 状态并终结字符集编码器。
    ///
    /// # Errors
    ///
    /// CSV 或底层输出刷新失败时返回错误。
    pub fn finish(mut self) -> Result<()> {
        self.inner.flush()?;
        let mut output = self
            .inner
            .into_inner()
            .map_err(|error| Error::Csv(error.to_string()))?;
        output.finish()?;
        Ok(())
    }
}

