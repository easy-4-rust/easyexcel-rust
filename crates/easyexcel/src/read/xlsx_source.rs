//! XLSX input/source abstraction supporting encrypted (compound-document) workbooks.

use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::core::{ExcelError, Result};
use crate::read::read_helpers::validate_read_options;
use crate::read::read_options::ReadOptions;

pub(crate) enum XlsxInput {
    File(BufReader<File>),
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

pub(crate) enum XlsxSource {
    File(PathBuf),
    Memory(Arc<[u8]>),
}

impl XlsxSource {
    pub(crate) fn open(path: &Path, password: Option<&str>) -> Result<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        // If the lightweight probe itself fails, the XLSX parser below still
        // returns the authoritative workbook error from the unchanged stream.
        if !is_compound_document(&mut reader) {
            return Ok(Self::File(path.to_owned()));
        }
        let password = password.ok_or_else(|| {
            ExcelError::Unsupported("encrypted OOXML workbook requires a password".to_owned())
        })?;
        let decrypted = easyexcel_xlsx::decrypt_file(path, password)?;
        Ok(Self::Memory(Arc::from(decrypted)))
    }

    pub(crate) fn reader(&self) -> Result<XlsxInput> {
        match self {
            Self::File(path) => Ok(XlsxInput::File(BufReader::new(File::open(path)?))),
            Self::Memory(bytes) => Ok(XlsxInput::Memory(Cursor::new(Arc::clone(bytes)))),
        }
    }
}

pub(crate) fn open_xlsx_source(path: &Path, options: &ReadOptions) -> Result<XlsxSource> {
    validate_read_options(options)?;
    XlsxSource::open(path, options.password.as_deref())
}

pub(crate) fn is_compound_document(reader: &mut dyn BufRead) -> bool {
    reader
        .fill_buf()
        .is_ok_and(easyexcel_io::looks_like_cfb)
}
