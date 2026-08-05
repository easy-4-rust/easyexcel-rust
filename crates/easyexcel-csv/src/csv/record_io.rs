//! CSV 增量记录读写后端。

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use encoding_rs_io::DecodeReaderBytes;
use easyexcel_io::{Error, Result};

use super::{
    CsvCharset, CsvEncodingWriter, csv_bom, csv_encoding, decode_reader,
};

/// 带字符集转码和 BOM 策略的 CSV 增量记录写入器。
pub struct CsvRecordWriter {
    inner: csv::Writer<CsvEncodingWriter>,
}

impl CsvRecordWriter {
    /// 从任意输出流创建记录写入器。
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

    /// 创建写入文件路径的记录写入器。
    ///
    /// # Errors
    ///
    /// 文件创建、字符集解析或 BOM 写入失败时返回错误。
    pub fn from_path(path: &Path, charset: &CsvCharset, with_bom: bool) -> Result<Self> {
        Self::new(Box::new(File::create(path)?), charset, with_bom)
    }

    /// 写入一条 CSV 记录。
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

    /// 刷新 CSV 状态并终结字符集编码器。
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

/// 带字符集解码的 CSV 增量记录读取器。
pub struct CsvRecordReader<R: Read> {
    inner: csv::Reader<DecodeReaderBytes<R, Vec<u8>>>,
}

impl<R: Read> CsvRecordReader<R> {
    /// 从字节输入和字符集创建记录读取器。
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

    /// 返回逐行 UTF-8 字段记录。
    pub fn records(&mut self) -> impl Iterator<Item = Result<Vec<String>>> + '_ {
        self.inner.records().map(|record| {
            record
                .map(|record| record.iter().map(str::to_owned).collect())
                .map_err(Error::from)
        })
    }
}

impl CsvRecordReader<File> {
    /// 打开文件路径并创建记录读取器。
    ///
    /// # Errors
    ///
    /// 文件打开或字符集解析失败时返回错误。
    pub fn from_path(path: &Path, charset: &CsvCharset) -> Result<Self> {
        Self::new(File::open(path)?, charset)
    }
}
