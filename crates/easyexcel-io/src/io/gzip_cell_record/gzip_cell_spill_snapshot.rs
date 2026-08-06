/// 对应 Java：无直接对应对象；Rust 架构扩展。 带逻辑工作表名称的 gzip spill 可观测状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GzipCellSpillSnapshot {
    /// spill 所属的逻辑工作表名称。
    pub sheet_name: String,
    /// gzip 临时文件路径。
    pub path: PathBuf,
    /// 文件是否包含 gzip 魔数。
    pub is_gzip: bool,
    /// 压缩后字节数。
    pub compressed_len: u64,
    /// 写入压缩器前的字节数。
    pub uncompressed_len: u64,
}

