//! 通用 gzip 临时记录流。
//!
//! 该模块只处理临时文件、gzip 压缩和长度前缀记录，不理解工作表或单元格，
//! 可被 XLSX 常量内存写入及其他表格后端复用。

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tempfile::{Builder, TempDir};

use crate::{Error, Result};

/// gzip 文件头魔数。
pub const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// gzip 记录流的可观测状态。
#[derive(Debug, Clone)]
pub struct GzipRecordSnapshot {
    /// 临时文件路径。
    pub path: PathBuf,
    /// 文件是否包含 gzip 魔数。
    pub is_gzip: bool,
    /// 压缩后字节数。
    pub compressed_len: u64,
    /// 写入压缩器前的字节数，包含每条记录的长度前缀。
    pub uncompressed_len: u64,
}

/// 采用 `u32 little-endian length + payload` 帧格式的 gzip 记录写入器。
pub struct GzipRecordWriter {
    path: PathBuf,
    encoder: GzEncoder<File>,
    uncompressed_len: u64,
    dir: Option<TempDir>,
}

impl GzipRecordWriter {
    /// 在指定目录创建 gzip 记录文件。
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

    /// 创建自持有临时目录的 gzip 记录文件。
    pub fn create_owned(prefix: &str, suffix: &str) -> Result<Self> {
        let dir = TempDir::new()?;
        let mut writer = Self::create(dir.path(), prefix, suffix)?;
        writer.dir = Some(dir);
        Ok(writer)
    }

    /// 写入一条长度前缀记录。
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

    /// 刷新压缩器缓冲区。
    pub fn flush(&mut self) -> Result<()> {
        self.encoder.flush()?;
        Ok(())
    }

    /// 返回当前记录流状态。
    pub fn snapshot(&mut self) -> Result<GzipRecordSnapshot> {
        self.flush()?;
        Ok(snapshot_for(&self.path, self.uncompressed_len))
    }

    /// 完成压缩并返回流式读取器。
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

/// 完成后的 gzip 长度前缀记录读取器。
pub struct GzipRecordReader {
    path: PathBuf,
    decoder: GzDecoder<File>,
    uncompressed_len: u64,
    compressed_len: u64,
    #[allow(dead_code)]
    dir: Option<TempDir>,
}

impl GzipRecordReader {
    /// 打开已有 gzip 记录文件，主要用于恢复未完成任务或诊断损坏文件。
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

    /// 返回完成后的记录流状态。
    #[must_use]
    pub fn snapshot(&self) -> GzipRecordSnapshot {
        GzipRecordSnapshot {
            path: self.path.clone(),
            is_gzip: file_has_gzip_magic(&self.path),
            compressed_len: self.compressed_len,
            uncompressed_len: self.uncompressed_len,
        }
    }

    /// 读取下一条记录；流结束时返回 `None`。
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

/// 判断文件是否以 gzip 魔数开头。
#[must_use]
pub fn file_has_gzip_magic(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 2];
    matches!(file.read_exact(&mut magic), Ok(())) && magic == GZIP_MAGIC
}

fn snapshot_for(path: &Path, uncompressed_len: u64) -> GzipRecordSnapshot {
    GzipRecordSnapshot {
        path: path.to_path_buf(),
        is_gzip: file_has_gzip_magic(path),
        compressed_len: std::fs::metadata(path).map_or(0, |meta| meta.len()),
        uncompressed_len,
    }
}
