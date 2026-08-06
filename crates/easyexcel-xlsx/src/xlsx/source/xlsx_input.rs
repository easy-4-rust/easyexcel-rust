/// 对应 Java：无直接对应对象；Rust 架构扩展。 可 seek 的 XLSX 输入流。
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

