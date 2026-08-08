/// POI 5.2.x 默认 BIFF8 `CryptoAPI` 输出所需的 `FILEPASS` 与工作簿密钥。
///
/// 对应 Java：`EncryptionInfo(EncryptionMode.cryptoAPI)`、
/// `CryptoAPIEncryptionHeader` 和 `CryptoAPIEncryptionVerifier`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8CryptoApiEncryption {
    pub(crate) filepass_payload: Vec<u8>,
    pub(crate) secret_key: [u8; 20],
}

impl Biff8CryptoApiEncryption {
    /// 返回完整 `FILEPASS` 记录 payload（不含 SID/长度头）。
    #[must_use]
    pub fn filepass_payload(&self) -> &[u8] {
        &self.filepass_payload
    }
}
