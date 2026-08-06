/// 对应 Java：无直接对应对象；Rust 架构扩展。 一个需要原样保留的 OOXML ZIP 条目。
#[derive(Debug, Clone)]
pub struct OoxmlZipEntry {
    /// ZIP 内路径。
    pub name: String,
    /// 是否为目录标记。
    pub is_dir: bool,
    /// 原压缩方式。
    pub compression: CompressionMethod,
    /// 可选 UNIX 权限位。
    pub unix_mode: Option<u32>,
    /// 条目原始内容。
    pub bytes: Vec<u8>,
}

