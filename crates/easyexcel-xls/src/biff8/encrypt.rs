//! BIFF8 `FILEPASS` RC4 `CryptoAPI` 加密。
//!
//! Java `EasyExcel` 4.0.3 依赖 POI 5.2.5；其
//! `Biff8EncryptionKey.setCurrentUserPassword` 写出 `EncryptionMode.cryptoAPI`
//! （RC4-40/SHA-1），而不是把整个 OLE 或 Workbook stream 一次性 RC4。
//! 本模块逐项对齐 POI 的 `HSSFWorkbook#encryptBytes`：1024 字节重置密钥、
//! BIFF 记录头明文、BOF/INTERFACEHDR/FILEPASS payload 明文，以及
//! BOUNDSHEET 的 `lbPlyPos` 四字节明文。

use sha1::{Digest, Sha1};

use super::record_sid::{BOF_SID, BOUND_SHEET_SID, FILE_PASS_SID, INTERFACE_HEADER_SID};

mod biff8_rc4_state;
use biff8_rc4_state::Biff8Rc4State;

mod biff8_crypto_api_encryption;
pub use biff8_crypto_api_encryption::Biff8CryptoApiEncryption;

mod chunked_rc4;
use chunked_rc4::ChunkedRc4;

const REKEY_INTERVAL: usize = 1024;
const CRYPTO_API_VERSION_MAJOR: u16 = 4;
const CRYPTO_API_VERSION_MINOR: u16 = 2;
const CRYPTO_API_FLAGS: u32 = 0x04;
const RC4_ALGORITHM_ID: u32 = 0x6801;
const SHA1_ALGORITHM_ID: u32 = 0x8004;
const RC4_KEY_BITS: u32 = 40;
const RC4_PROVIDER_TYPE: u32 = 1;
const RC4_PROVIDER_NAME: &str = "Microsoft Base Cryptographic Provider v1.0";

/// 为密码生成随机 salt/verifier、`FILEPASS` payload 与工作簿加密密钥。
///
/// # Errors
///
/// 操作系统安全随机源不可用时返回错误。
pub fn prepare_crypto_api_encryption(password: &str) -> Result<Biff8CryptoApiEncryption, String> {
    let mut salt = [0u8; 16];
    let mut verifier = [0u8; 16];
    getrandom::fill(&mut salt)
        .map_err(|error| format!("failed to generate BIFF8 salt: {error}"))?;
    getrandom::fill(&mut verifier)
        .map_err(|error| format!("failed to generate BIFF8 verifier: {error}"))?;
    Ok(prepare_crypto_api_encryption_with_material(
        password, salt, verifier,
    ))
}

fn prepare_crypto_api_encryption_with_material(
    password: &str,
    salt: [u8; 16],
    verifier: [u8; 16],
) -> Biff8CryptoApiEncryption {
    let secret_key = crypto_api_secret_key(password, &salt);
    let block_key = crypto_api_block_key(&secret_key, 0);
    let mut cipher = Biff8Rc4State::new(&block_key);
    let mut encrypted_verifier = verifier;
    cipher.apply(&mut encrypted_verifier);
    let mut encrypted_verifier_hash: [u8; 20] = Sha1::digest(verifier).into();
    cipher.apply(&mut encrypted_verifier_hash);

    Biff8CryptoApiEncryption {
        filepass_payload: crypto_api_filepass_payload(
            &salt,
            &encrypted_verifier,
            &encrypted_verifier_hash,
        ),
        secret_key,
    }
}

fn crypto_api_secret_key(password: &str, salt: &[u8; 16]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(salt);
    for code_unit in password.encode_utf16().take(255) {
        hasher.update(code_unit.to_le_bytes());
    }
    hasher.finalize().into()
}

fn crypto_api_block_key(secret_key: &[u8; 20], block: u32) -> [u8; 16] {
    let mut hasher = Sha1::new();
    hasher.update(secret_key);
    hasher.update(block.to_le_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 16];
    // POI CryptoAPIDecryptor#getBlock0: RC4-40 keeps five bytes and pads the
    // JCE RC4 key to sixteen bytes with zeros.
    key[..5].copy_from_slice(&digest[..5]);
    key
}

/// 使用调用方提供的密码解密 BIFF8 `CryptoAPI` Workbook stream。
///
/// 该函数不使用 Java POI 的线程局部全局密码；密码只在本次调用内派生并验证。
///
/// # Errors
///
/// 缺少 `FILEPASS`、加密类型不是 POI 默认的 `CryptoAPI` RC4、记录损坏或密码错误时返回错误。
pub fn decrypt_crypto_api_workbook_stream(
    workbook_stream: &[u8],
    password: &str,
) -> Result<Vec<u8>, easyexcel_io::Error> {
    let filepass = find_filepass_payload(workbook_stream)?;
    let secret_key = verify_crypto_api_password(filepass, password)?;
    transform_crypto_api_workbook_stream(workbook_stream, &secret_key)
        .map_err(easyexcel_io::Error::Xls)
}

fn find_filepass_payload(workbook_stream: &[u8]) -> Result<&[u8], easyexcel_io::Error> {
    let mut cursor = 0usize;
    while cursor < workbook_stream.len() {
        let header = workbook_stream
            .get(cursor..cursor.saturating_add(4))
            .ok_or_else(|| easyexcel_io::Error::Xls("truncated BIFF record header".to_owned()))?;
        let sid = u16::from_le_bytes([header[0], header[1]]);
        let len = usize::from(u16::from_le_bytes([header[2], header[3]]));
        let payload_start = cursor + 4;
        let payload_end = payload_start
            .checked_add(len)
            .ok_or_else(|| easyexcel_io::Error::Xls("BIFF record length overflow".to_owned()))?;
        let payload = workbook_stream
            .get(payload_start..payload_end)
            .ok_or_else(|| {
                easyexcel_io::Error::Xls(format!("truncated BIFF record 0x{sid:04X}"))
            })?;
        if sid == FILE_PASS_SID {
            return Ok(payload);
        }
        cursor = payload_end;
    }
    Err(easyexcel_io::Error::Xls(
        "BIFF8 CryptoAPI stream is missing FILEPASS".to_owned(),
    ))
}

fn verify_crypto_api_password(
    filepass: &[u8],
    password: &str,
) -> Result<[u8; 20], easyexcel_io::Error> {
    if read_u16(filepass, 0) != Some(1)
        || read_u16(filepass, 2) != Some(CRYPTO_API_VERSION_MAJOR)
        || read_u16(filepass, 4) != Some(CRYPTO_API_VERSION_MINOR)
    {
        return Err(easyexcel_io::Error::Unsupported(
            "BIFF8 password encryption is not CryptoAPI RC4".to_owned(),
        ));
    }
    let header_size = read_u32(filepass, 10)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| easyexcel_io::Error::Xls("invalid FILEPASS header size".to_owned()))?;
    let verifier_start = 14usize
        .checked_add(header_size)
        .ok_or_else(|| easyexcel_io::Error::Xls("FILEPASS header overflow".to_owned()))?;
    if read_u32(filepass, verifier_start) != Some(16) {
        return Err(easyexcel_io::Error::Xls(
            "invalid FILEPASS salt size".to_owned(),
        ));
    }
    let salt_start = verifier_start + 4;
    let salt: [u8; 16] = filepass
        .get(salt_start..salt_start + 16)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| easyexcel_io::Error::Xls("truncated FILEPASS salt".to_owned()))?;
    let encrypted_verifier_start = salt_start + 16;
    let mut verifier: [u8; 16] = filepass
        .get(encrypted_verifier_start..encrypted_verifier_start + 16)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| easyexcel_io::Error::Xls("truncated FILEPASS verifier".to_owned()))?;
    let hash_size_offset = encrypted_verifier_start + 16;
    let hash_size = read_u32(filepass, hash_size_offset)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            easyexcel_io::Error::Xls("invalid FILEPASS verifier hash size".to_owned())
        })?;
    if hash_size < 20 {
        return Err(easyexcel_io::Error::Xls(
            "FILEPASS verifier hash is shorter than SHA-1".to_owned(),
        ));
    }
    let hash_start = hash_size_offset + 4;
    let mut verifier_hash = filepass
        .get(hash_start..hash_start + hash_size)
        .ok_or_else(|| easyexcel_io::Error::Xls("truncated FILEPASS verifier hash".to_owned()))?
        .to_vec();

    let secret_key = crypto_api_secret_key(password, &salt);
    let mut cipher = Biff8Rc4State::new(&crypto_api_block_key(&secret_key, 0));
    cipher.apply(&mut verifier);
    cipher.apply(&mut verifier_hash);
    let expected: [u8; 20] = Sha1::digest(verifier).into();
    if !constant_time_eq(&expected, &verifier_hash[..20]) {
        return Err(easyexcel_io::Error::WrongPassword);
    }
    Ok(secret_key)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let value = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let value = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            == 0
}

fn crypto_api_filepass_payload(
    salt: &[u8; 16],
    encrypted_verifier: &[u8; 16],
    encrypted_verifier_hash: &[u8; 20],
) -> Vec<u8> {
    let mut header_body = Vec::new();
    header_body.extend_from_slice(&CRYPTO_API_FLAGS.to_le_bytes());
    header_body.extend_from_slice(&0u32.to_le_bytes());
    header_body.extend_from_slice(&RC4_ALGORITHM_ID.to_le_bytes());
    header_body.extend_from_slice(&SHA1_ALGORITHM_ID.to_le_bytes());
    header_body.extend_from_slice(&RC4_KEY_BITS.to_le_bytes());
    header_body.extend_from_slice(&RC4_PROVIDER_TYPE.to_le_bytes());
    header_body.extend_from_slice(&0u32.to_le_bytes());
    header_body.extend_from_slice(&0u32.to_le_bytes());
    for code_unit in RC4_PROVIDER_NAME.encode_utf16().chain(std::iter::once(0)) {
        header_body.extend_from_slice(&code_unit.to_le_bytes());
    }

    let mut payload = Vec::with_capacity(18 + header_body.len() + 56);
    payload.extend_from_slice(&1u16.to_le_bytes());
    payload.extend_from_slice(&CRYPTO_API_VERSION_MAJOR.to_le_bytes());
    payload.extend_from_slice(&CRYPTO_API_VERSION_MINOR.to_le_bytes());
    payload.extend_from_slice(&CRYPTO_API_FLAGS.to_le_bytes());
    payload.extend_from_slice(&(header_body.len() as u32).to_le_bytes());
    payload.extend_from_slice(&header_body);
    payload.extend_from_slice(&16u32.to_le_bytes());
    payload.extend_from_slice(salt);
    payload.extend_from_slice(encrypted_verifier);
    payload.extend_from_slice(&20u32.to_le_bytes());
    payload.extend_from_slice(encrypted_verifier_hash);
    payload
}

/// 按 POI `HSSFWorkbook#encryptBytes` 对已含 `FILEPASS` 的 Workbook stream 加密。
///
/// # Errors
///
/// BIFF record 截断、长度越界或 `FILEPASS` 缺失时返回错误。
pub fn encrypt_crypto_api_workbook_stream(
    workbook_stream: &[u8],
    encryption: &Biff8CryptoApiEncryption,
) -> Result<Vec<u8>, String> {
    transform_crypto_api_workbook_stream(workbook_stream, &encryption.secret_key)
}

fn transform_crypto_api_workbook_stream(
    workbook_stream: &[u8],
    secret_key: &[u8; 20],
) -> Result<Vec<u8>, String> {
    let mut output = workbook_stream.to_vec();
    let mut crypt = ChunkedRc4::new(secret_key);
    let mut cursor = 0usize;
    let mut saw_filepass = false;
    while cursor < output.len() {
        let header_end = cursor
            .checked_add(4)
            .ok_or_else(|| "BIFF8 encryption cursor overflow".to_owned())?;
        let header = output
            .get(cursor..header_end)
            .ok_or_else(|| "BIFF8 encryption truncated record header".to_owned())?;
        let sid = u16::from_le_bytes([header[0], header[1]]);
        let len = usize::from(u16::from_le_bytes([header[2], header[3]]));
        crypt.advance_plain(4);
        cursor = header_end;
        let payload_end = cursor
            .checked_add(len)
            .ok_or_else(|| "BIFF8 encryption payload overflow".to_owned())?;
        let payload = output
            .get_mut(cursor..payload_end)
            .ok_or_else(|| format!("BIFF8 encryption truncated record 0x{sid:04X}"))?;
        match sid {
            BOF_SID | INTERFACE_HEADER_SID | FILE_PASS_SID => {
                if sid == FILE_PASS_SID {
                    saw_filepass = true;
                }
                crypt.advance_plain(len);
            }
            BOUND_SHEET_SID if len >= 4 => {
                crypt.advance_plain(4);
                crypt.apply(&mut payload[4..]);
            }
            _ => crypt.apply(payload),
        }
        cursor = payload_end;
    }
    if !saw_filepass {
        return Err("BIFF8 CryptoAPI stream is missing FILEPASS".to_owned());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_api_material_is_deterministic_for_golden_inputs() {
        let encryption =
            prepare_crypto_api_encryption_with_material("123456", [0x11; 16], [0x22; 16]);
        assert_eq!(
            &encryption.filepass_payload[..10],
            &[1, 0, 4, 0, 2, 0, 4, 0, 0, 0]
        );
        assert_eq!(encryption.filepass_payload[10..14], (118u32).to_le_bytes());
        assert_eq!(encryption.filepass_payload.len(), 192);
    }

    #[test]
    fn workbook_transform_keeps_headers_filepass_bof_and_boundsheet_offset_plain() {
        let encryption = prepare_crypto_api_encryption_with_material("pwd", [3; 16], [7; 16]);
        let mut stream = Vec::new();
        super::super::encode::record(&mut stream, BOF_SID, &[1, 2, 3, 4]);
        super::super::encode::record(&mut stream, FILE_PASS_SID, encryption.filepass_payload());
        super::super::encode::record(&mut stream, BOUND_SHEET_SID, &[9, 8, 7, 6, 5, 4]);
        super::super::encode::record(&mut stream, 0x0203, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let encrypted = encrypt_crypto_api_workbook_stream(&stream, &encryption).unwrap();
        assert_eq!(&encrypted[..8], &stream[..8]);
        let filepass_start = 8usize;
        let filepass_end = filepass_start + 4 + encryption.filepass_payload.len();
        assert_eq!(
            &encrypted[filepass_start..filepass_end],
            &stream[filepass_start..filepass_end]
        );
        assert_eq!(
            &encrypted[filepass_end..filepass_end + 8],
            &stream[filepass_end..filepass_end + 8]
        );
        assert_ne!(&encrypted[filepass_end + 8..], &stream[filepass_end + 8..]);
        let decrypted = decrypt_crypto_api_workbook_stream(&encrypted, "pwd").unwrap();
        assert_eq!(decrypted, stream);
        assert!(matches!(
            decrypt_crypto_api_workbook_stream(&encrypted, "wrong"),
            Err(easyexcel_io::Error::WrongPassword)
        ));
    }
}
