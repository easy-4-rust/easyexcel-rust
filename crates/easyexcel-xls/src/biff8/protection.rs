//! BIFF8/OOXML 传统工作表密码校验值。

/// 生成 Excel `PASSWORD` 记录使用的 16 位 XOR verifier。
///
/// 对应 Java：Apache POI `CryptoFunctions#createXorVerifier1`。
#[must_use]
pub fn legacy_password_hash(password: &str) -> u16 {
    let utf16: Vec<u16> = password.encode_utf16().collect();
    let mut hash = 0_u16;
    for value in utf16.iter().rev() {
        hash = hash.rotate_left(1) ^ *value;
    }
    hash ^= u16::try_from(utf16.len()).unwrap_or(u16::MAX);
    hash ^ 0xCE4B
}
