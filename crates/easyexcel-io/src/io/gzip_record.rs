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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

include!("gzip_record/gzip_record_snapshot.rs");

include!("gzip_record/gzip_record_writer.rs");

include!("gzip_record/gzip_record_reader.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断文件是否以 gzip 魔数开头。
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
