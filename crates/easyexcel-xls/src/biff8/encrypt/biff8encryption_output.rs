/// BIFF8 加密输出：加密字节、salt 与 verifier hash。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type Biff8EncryptionOutput = (Vec<u8>, [u8; 16], [u8; 16]);

