/// 对应 Java：无直接对应对象；Rust 架构扩展。 可被 XLSX 事件读取器持有的输入流。
pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

