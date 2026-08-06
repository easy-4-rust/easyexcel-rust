/// 对应 Java：无直接对应对象；Rust 架构扩展。 gzip 记录流的可观测状态。
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

