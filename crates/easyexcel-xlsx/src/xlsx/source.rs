//! 可重复打开的 XLSX 输入源，透明支持 MS-OFFCRYPTO 容器解密。

use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use easyexcel_io::io::file_utils::TemporaryInput;
use easyexcel_io::{Error, Result};

/// 可 seek 的 XLSX 输入流。
pub enum XlsxInput {
    /// 直接从文件读取。
    File(BufReader<File>),
    /// 从已解密的共享内存读取。
    Memory(Cursor<Arc<[u8]>>),
}

impl Read for XlsxInput {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::File(reader) => reader.read(buffer),
            Self::Memory(reader) => reader.read(buffer),
        }
    }
}

impl Seek for XlsxInput {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(reader) => reader.seek(position),
            Self::Memory(reader) => reader.seek(position),
        }
    }
}

/// 可重复创建读取器的 XLSX 源。
pub enum XlsxSource {
    /// 未加密 OOXML 文件。
    File(PathBuf),
    /// 已解密 OOXML ZIP 字节。
    Memory(Arc<[u8]>),
}

impl XlsxSource {
    /// 打开普通或加密 XLSX 文件。
    ///
    /// # Errors
    ///
    /// 文件无法读取、缺少密码、密码错误或加密格式不受支持时返回错误。
    pub fn open(path: &Path, password: Option<&str>) -> Result<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        if !is_compound_document(&mut reader) {
            return Ok(Self::File(path.to_owned()));
        }
        let password = password
            .ok_or_else(|| Error::PasswordProtected("MS-OFFCRYPTO OOXML container".to_owned()))?;
        let decrypted = super::crypto::decrypt_file(path, password)?;
        Ok(Self::Memory(Arc::from(decrypted)))
    }

    /// 创建一个位于起始位置的新读取器。
    ///
    /// # Errors
    ///
    /// 原始文件无法重新打开时返回错误。
    pub fn reader(&self) -> Result<XlsxInput> {
        match self {
            Self::File(path) => Ok(XlsxInput::File(BufReader::new(File::open(path)?))),
            Self::Memory(bytes) => Ok(XlsxInput::Memory(Cursor::new(Arc::clone(bytes)))),
        }
    }
}

/// 判断缓冲输入是否为 OLE2/CFB 容器。
#[must_use]
pub fn is_compound_document(reader: &mut dyn BufRead) -> bool {
    reader.fill_buf().is_ok_and(easyexcel_io::looks_like_cfb)
}

/// 根据输入魔数选择物化文件后缀。
#[must_use]
pub fn excel_input_suffix(bytes: &[u8]) -> &'static str {
    match easyexcel_io::Format::from_magic(bytes) {
        easyexcel_io::Format::Xlsx => ".xlsx",
        easyexcel_io::Format::Xls if super::crypto::is_encrypted_ooxml(bytes) => ".xlsx",
        easyexcel_io::Format::Xls => ".xls",
        easyexcel_io::Format::Csv => ".csv",
        _ => ".csv",
    }
}

/// 将 Java 风格非随机访问输入流物化为自动删除的 Excel 输入文件。
///
/// # Errors
///
/// 输入读取、格式探测或临时文件写入失败时返回错误。
pub fn materialize_excel_input<R>(mut input: R) -> Result<TemporaryInput>
where
    R: Read,
{
    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;
    let suffix = excel_input_suffix(&bytes);
    TemporaryInput::from_bytes(&bytes, suffix).map_err(Error::from)
}
