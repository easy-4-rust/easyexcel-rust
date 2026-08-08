use super::{Biff8Rc4State, REKEY_INTERVAL, crypto_api_block_key};

/// BIFF8 `CryptoAPI` 以 Workbook stream 绝对位置每 1024 字节重新派生密钥的 RC4 状态。
///
/// 对应 Java：POI `CryptoAPICipherOutputStream` 的 chunk 状态。
pub(super) struct ChunkedRc4<'a> {
    secret_key: &'a [u8; 20],
    position: usize,
    cipher: Biff8Rc4State,
}

impl<'a> ChunkedRc4<'a> {
    pub(super) fn new(secret_key: &'a [u8; 20]) -> Self {
        Self {
            secret_key,
            position: 0,
            cipher: Biff8Rc4State::new(&crypto_api_block_key(secret_key, 0)),
        }
    }

    fn next_keystream(&mut self) -> u8 {
        if self.position > 0 && self.position.is_multiple_of(REKEY_INTERVAL) {
            let block = u32::try_from(self.position / REKEY_INTERVAL).unwrap_or(u32::MAX);
            self.cipher = Biff8Rc4State::new(&crypto_api_block_key(self.secret_key, block));
        }
        self.position = self.position.saturating_add(1);
        self.cipher.next_byte()
    }

    pub(super) fn advance_plain(&mut self, len: usize) {
        for _ in 0..len {
            let _ = self.next_keystream();
        }
    }

    pub(super) fn apply(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            *byte ^= self.next_keystream();
        }
    }
}
