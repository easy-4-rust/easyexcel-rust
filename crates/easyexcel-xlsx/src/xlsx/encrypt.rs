//! OOXML ECMA-376 Agile 加密写入原语。

use std::io::{Read, Seek, Write};

use easyexcel_io::Result;
use ms_offcrypto_writer::Ecma376AgileWriter;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 同时支持读取、写入和定位的加密容器输出。
pub trait ReadWriteSeek: Read + Write + Seek {}

impl<T> ReadWriteSeek for T where T: Read + Write + Seek {}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将完整 OOXML ZIP 包加密写入 OLE/CFB 容器。
///
/// # Errors
///
/// 随机数初始化、加密写入或容器终结失败时返回统一 I/O 错误。
pub fn encrypt_package_to(
    plaintext: &[u8],
    password: &str,
    output: &mut dyn ReadWriteSeek,
) -> Result<()> {
    let mut random = rand::rng();
    let mut writer = Ecma376AgileWriter::create(&mut random, password, output)?;
    writer.write_all(plaintext)?;
    writer.finalize()?;
    Ok(())
}
