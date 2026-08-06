//! 受超时约束的远程二进制资源读取。

use std::io::Read;
use std::time::Duration;

use super::{Error, Result};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 下载 URL 指向的完整二进制内容。
///
/// # Errors
///
/// 连接、响应读取或 TLS 失败时返回 I/O 错误。
pub fn download_bytes(
    url: &str,
    connect_timeout: Duration,
    read_timeout: Duration,
) -> Result<Vec<u8>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(connect_timeout))
        .timeout_recv_body(Some(read_timeout))
        .build()
        .into();
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| Error::Io(std::io::Error::other(error.to_string())))?;
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut bytes)
        .map_err(Error::from)?;
    Ok(bytes)
}
