//! Decryption of password-protected OOXML workbooks (MS-OFFCRYPTO).
//!
//! A password-protected `.xlsx` is not a ZIP: it's an OLE2/CFB compound file
//! holding an `EncryptionInfo` stream (describing the scheme) and an
//! `EncryptedPackage` stream (the real ZIP, encrypted). We implement the two
//! ECMA-376 password schemes from [MS-OFFCRYPTO] directly:
//!
//! * **Agile** (Excel 2010+, the default): an XML descriptor; AES-CBC with a
//!   configurable hash (SHA-1/256/384/512) and a high spin-count key stretch.
//! * **Standard** (Excel 2007): a binary descriptor; AES-ECB with SHA-1.
//!
//! Legacy binary RC4/XOR (`.xls`) and the rare "extensible" scheme are detected
//! and named but not decrypted.

use std::io::{Cursor, Read};

use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};

use easyexcel_io::{Error, Result};

// blockKey constants from [MS-OFFCRYPTO] §2.3.4.7 used to derive per-purpose
// keys in agile encryption.
const BLOCK_VERIFIER_INPUT: [u8; 8] = [0xfe, 0xa7, 0xd2, 0x76, 0x3b, 0x4b, 0x9e, 0x79];
const BLOCK_VERIFIER_VALUE: [u8; 8] = [0xd7, 0xaa, 0x0f, 0x6d, 0x30, 0x61, 0x34, 0x4e];
const BLOCK_KEY_VALUE: [u8; 8] = [0x14, 0x6e, 0x0b, 0xe7, 0xab, 0xac, 0xd0, 0xd6];

/// Read a CFB stream by path (e.g. `/EncryptionInfo`) into a byte vector.
fn read_stream(comp: &mut cfb::CompoundFile<Cursor<&[u8]>>, path: &str) -> Result<Vec<u8>> {
    let mut stream = comp
        .open_stream(path)
        .map_err(|e| Error::Cfb(format!("missing {path} stream: {e}")))?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Read just the `EncryptionInfo` stream out of the CFB container.
fn read_encryption_info(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut comp = cfb::CompoundFile::open(Cursor::new(bytes))
        .map_err(|e| Error::Cfb(format!("not a valid OLE2 container: {e}")))?;
    read_stream(&mut comp, "/EncryptionInfo")
}

/// Human-readable name of the encryption scheme, parsed from `EncryptionInfo`.
///
/// Used to tell the user *what* protects the file when they haven't supplied a
/// password.
pub fn describe_scheme(bytes: &[u8]) -> Result<String> {
    let info = read_encryption_info(bytes)?;
    if info.len() < 4 {
        return Ok("encrypted (unrecognized EncryptionInfo)".to_string());
    }
    let major = u16::from_le_bytes([info[0], info[1]]);
    let minor = u16::from_le_bytes([info[2], info[3]]);

    Ok(match (major, minor) {
        (4, 4) => {
            let xml = String::from_utf8_lossy(&info[8.min(info.len())..]);
            let cipher = attr_in(&xml, "<keyData", "cipherAlgorithm").unwrap_or("AES");
            let bits = attr_in(&xml, "<keyData", "keyBits").unwrap_or("?");
            let mode = attr_in(&xml, "<keyData", "cipherChaining")
                .map(|c| c.strip_prefix("ChainingMode").unwrap_or(c))
                .unwrap_or("CBC");
            let hash = attr_in(&xml, "<keyData", "hashAlgorithm").unwrap_or("?");
            format!("ECMA-376 agile encryption: {cipher}-{bits}-{mode}, {hash}")
        }
        (2..=4, 2) => {
            // EncryptionHeader: Flags(4),SizeExtra(4),AlgID(4)@20,AlgIDHash(4),KeySize(4).
            let alg = info
                .get(20..24)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
            let cipher = match alg {
                Some(0x0000_660E) => "AES-128",
                Some(0x0000_660F) => "AES-192",
                Some(0x0000_6610) => "AES-256",
                _ => "AES",
            };
            format!("ECMA-376 standard encryption: {cipher}, SHA-1")
        }
        (_, 3) => "ECMA-376 extensible encryption (not supported)".to_string(),
        (1, 1) => "Office binary RC4 encryption (not supported)".to_string(),
        _ => format!("unrecognized encryption (EncryptionInfo v{major}.{minor})"),
    })
}

/// Decrypt an encrypted OOXML package, returning the inner ZIP bytes.
pub fn decrypt(bytes: Vec<u8>, password: &str) -> Result<Vec<u8>> {
    let mut comp = cfb::CompoundFile::open(Cursor::new(bytes.as_slice()))
        .map_err(|e| Error::Cfb(format!("not a valid OLE2 container: {e}")))?;
    let info = read_stream(&mut comp, "/EncryptionInfo")?;
    let package = read_stream(&mut comp, "/EncryptedPackage")?;
    if info.len() < 4 {
        return Err(Error::Cfb("short EncryptionInfo".into()));
    }
    let major = u16::from_le_bytes([info[0], info[1]]);
    let minor = u16::from_le_bytes([info[2], info[3]]);

    let inner = match (major, minor) {
        (4, 4) => decrypt_agile(&info, &package, password)?,
        (2..=4, 2) => decrypt_standard(&info, &package, password)?,
        _ => {
            return Err(Error::Cfb(format!(
                "unsupported encryption (EncryptionInfo v{major}.{minor})"
            )));
        }
    };

    // A correct password yields a real ZIP; a wrong one yields garbage.
    if !super::looks_like_zip(inner.get(..4).unwrap_or(&[])) {
        return Err(Error::WrongPassword);
    }
    Ok(inner)
}

// ─── Agile encryption ────────────────────────────────────────────────────────

fn decrypt_agile(info: &[u8], package: &[u8], password: &str) -> Result<Vec<u8>> {
    let xml = String::from_utf8_lossy(&info[8.min(info.len())..]);

    // Parameters from the password key encryptor (<p:encryptedKey ...>).
    let pk = "encryptedKey";
    let pk_hash = HashAlgo::parse(req(&xml, pk, "hashAlgorithm")?)?;
    let pk_salt = b64(req(&xml, pk, "saltValue")?)?;
    let pk_keybytes = req(&xml, pk, "keyBits")?.parse::<usize>().unwrap_or(0) / 8;
    let pk_spin = req(&xml, pk, "spinCount")?.parse::<u32>().unwrap_or(0);
    let hash_size = req(&xml, pk, "hashSize")?.parse::<usize>().unwrap_or(0);
    let enc_verifier_input = b64(req(&xml, pk, "encryptedVerifierHashInput")?)?;
    let enc_verifier_value = b64(req(&xml, pk, "encryptedVerifierHashValue")?)?;
    let enc_key_value = b64(req(&xml, pk, "encryptedKeyValue")?)?;

    // Parameters of the data (package) cipher (<keyData ...>).
    let kd = "<keyData";
    let kd_hash = HashAlgo::parse(req(&xml, kd, "hashAlgorithm")?)?;
    let kd_salt = b64(req(&xml, kd, "saltValue")?)?;
    let kd_keybytes = req(&xml, kd, "keyBits")?.parse::<usize>().unwrap_or(0) / 8;
    let kd_block = req(&xml, kd, "blockSize")?.parse::<usize>().unwrap_or(16);

    let pw = utf16le(password);

    // Iterated password hash: H_n = hash(salt || pw), then spinCount rounds of
    // hash(LE32(i) || H).
    let mut h = pk_hash.digest(&[&pk_salt, &pw]);
    for i in 0..pk_spin {
        h = pk_hash.digest(&[&i.to_le_bytes(), &h]);
    }
    let gen_key = |block: &[u8], len: usize| fit(pk_hash.digest(&[&h, block]), len);

    // Verify the password before trusting the derived key.
    let vin_key = gen_key(&BLOCK_VERIFIER_INPUT, pk_keybytes);
    let vin = aes_cbc_decrypt(&vin_key, &pk_salt, &enc_verifier_input)?;
    let vval_key = gen_key(&BLOCK_VERIFIER_VALUE, pk_keybytes);
    let vval = aes_cbc_decrypt(&vval_key, &pk_salt, &enc_verifier_value)?;
    let expect = pk_hash.digest(&[&vin]);
    let n = hash_size.min(expect.len()).min(vval.len());
    if expect[..n] != vval[..n] {
        return Err(Error::WrongPassword);
    }

    // Recover the package secret key.
    let kv_key = gen_key(&BLOCK_KEY_VALUE, pk_keybytes);
    let mut secret = aes_cbc_decrypt(&kv_key, &pk_salt, &enc_key_value)?;
    secret.truncate(kd_keybytes);

    // Decrypt the package: 8-byte total length, then 4096-byte CBC segments,
    // each with IV = hash(keyData.salt || LE32(segmentIndex)).
    if package.len() < 8 {
        return Err(Error::Cfb("short EncryptedPackage".into()));
    }
    let total = u64::from_le_bytes(package[..8].try_into().unwrap()) as usize;
    let data = &package[8..];
    let mut out = Vec::with_capacity(total);
    for (i, seg) in data.chunks(4096).enumerate() {
        let mut iv = kd_hash.digest(&[&kd_salt, &(i as u32).to_le_bytes()]);
        iv.truncate(kd_block);
        out.extend_from_slice(&aes_cbc_decrypt(&secret, &iv, seg)?);
    }
    out.truncate(total);
    Ok(out)
}

// ─── Standard encryption ─────────────────────────────────────────────────────

fn decrypt_standard(info: &[u8], package: &[u8], password: &str) -> Result<Vec<u8>> {
    // version(4) + flags(4) + headerSize(4) + EncryptionHeader + EncryptionVerifier.
    let header_size =
        u32::from_le_bytes(info.get(8..12).ok_or_short()?.try_into().unwrap()) as usize;
    let key_bits = u32::from_le_bytes(info.get(28..32).ok_or_short()?.try_into().unwrap()) as usize;
    let key_len = (key_bits / 8).max(16);

    let mut p = 12 + header_size; // start of EncryptionVerifier
    let salt_size =
        u32::from_le_bytes(info.get(p..p + 4).ok_or_short()?.try_into().unwrap()) as usize;
    p += 4;
    let salt = info.get(p..p + salt_size).ok_or_short()?.to_vec();
    p += salt_size;
    let enc_verifier = info.get(p..p + 16).ok_or_short()?.to_vec();
    p += 16;
    let verifier_hash_size =
        u32::from_le_bytes(info.get(p..p + 4).ok_or_short()?.try_into().unwrap()) as usize;
    p += 4;
    // EncryptedVerifierHash is padded to a 32-byte block for AES.
    let enc_verifier_hash = info.get(p..p + 32).ok_or_short()?.to_vec();

    let pw = utf16le(password);
    let key = standard_key(&salt, &pw, key_len);

    // Verify password.
    let verifier = aes_ecb_decrypt(&key, &enc_verifier)?;
    let vhash = aes_ecb_decrypt(&key, &enc_verifier_hash)?;
    let expect = HashAlgo::Sha1.digest(&[&verifier]);
    let n = verifier_hash_size.min(expect.len()).min(vhash.len());
    if expect[..n] != vhash[..n] {
        return Err(Error::WrongPassword);
    }

    if package.len() < 8 {
        return Err(Error::Cfb("short EncryptedPackage".into()));
    }
    let total = u64::from_le_bytes(package[..8].try_into().unwrap()) as usize;
    let mut out = aes_ecb_decrypt(&key, &package[8..])?;
    out.truncate(total);
    Ok(out)
}

/// Standard-encryption key derivation (always SHA-1, 50000 iterations).
fn standard_key(salt: &[u8], pw: &[u8], key_len: usize) -> Vec<u8> {
    let h = HashAlgo::Sha1;
    let mut acc = h.digest(&[salt, pw]);
    for i in 0..50_000u32 {
        acc = h.digest(&[&i.to_le_bytes(), &acc]);
    }
    let final_hash = h.digest(&[&acc, &0u32.to_le_bytes()]);

    let derive = |pad: u8| {
        let mut buf = [pad; 64];
        for (b, f) in buf.iter_mut().zip(final_hash.iter()) {
            *b ^= *f;
        }
        h.digest(&[&buf])
    };
    let mut key = derive(0x36);
    key.extend(derive(0x5c));
    key.truncate(key_len);
    key
}

// ─── Primitives ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum HashAlgo {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgo {
    fn parse(name: &str) -> Result<HashAlgo> {
        match name.to_ascii_uppercase().replace(['-', '_'], "").as_str() {
            "SHA1" => Ok(HashAlgo::Sha1),
            "SHA256" | "SHA2256" => Ok(HashAlgo::Sha256),
            "SHA384" | "SHA2384" => Ok(HashAlgo::Sha384),
            "SHA512" | "SHA2512" => Ok(HashAlgo::Sha512),
            other => Err(Error::Cfb(format!("unsupported hash algorithm: {other}"))),
        }
    }

    fn digest(self, parts: &[&[u8]]) -> Vec<u8> {
        match self {
            HashAlgo::Sha1 => {
                use sha1::{Digest, Sha1};
                let mut d = Sha1::new();
                parts.iter().for_each(|p| d.update(p));
                d.finalize().to_vec()
            }
            HashAlgo::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut d = Sha256::new();
                parts.iter().for_each(|p| d.update(p));
                d.finalize().to_vec()
            }
            HashAlgo::Sha384 => {
                use sha2::{Digest, Sha384};
                let mut d = Sha384::new();
                parts.iter().for_each(|p| d.update(p));
                d.finalize().to_vec()
            }
            HashAlgo::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut d = Sha512::new();
                parts.iter().for_each(|p| d.update(p));
                d.finalize().to_vec()
            }
        }
    }
}

/// AES-CBC decrypt. `data` is processed in 16-byte blocks (zero-padded if a
/// trailing partial block somehow appears).
fn aes_cbc_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let cipher = AesKey::new(key)?;
    let mut prev = [0u8; 16];
    prev[..iv.len().min(16)].copy_from_slice(&iv[..iv.len().min(16)]);
    let mut out = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        let mut b = GenericArray::clone_from_slice(&block);
        cipher.decrypt_block(&mut b);
        for j in 0..16 {
            out.push(b[j] ^ prev[j]);
        }
        prev = block;
    }
    Ok(out)
}

/// AES-ECB decrypt (no chaining).
fn aes_ecb_decrypt(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let cipher = AesKey::new(key)?;
    let mut out = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        let mut b = GenericArray::clone_from_slice(&block);
        cipher.decrypt_block(&mut b);
        out.extend_from_slice(&b);
    }
    Ok(out)
}

/// An AES block cipher of the right key size (128/192/256).
enum AesKey {
    K128(Box<aes::Aes128>),
    K192(Box<aes::Aes192>),
    K256(Box<aes::Aes256>),
}

impl AesKey {
    fn new(key: &[u8]) -> Result<AesKey> {
        Ok(match key.len() {
            16 => AesKey::K128(Box::new(aes::Aes128::new_from_slice(key).unwrap())),
            24 => AesKey::K192(Box::new(aes::Aes192::new_from_slice(key).unwrap())),
            32 => AesKey::K256(Box::new(aes::Aes256::new_from_slice(key).unwrap())),
            n => return Err(Error::Cfb(format!("unsupported AES key length: {n} bytes"))),
        })
    }

    fn decrypt_block(&self, block: &mut GenericArray<u8, aes::cipher::consts::U16>) {
        match self {
            AesKey::K128(c) => c.decrypt_block(block),
            AesKey::K192(c) => c.decrypt_block(block),
            AesKey::K256(c) => c.decrypt_block(block),
        }
    }
}

/// UTF-16LE encode a password.
fn utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
}

/// Truncate or 0x36-pad a hash output to exactly `n` bytes (the agile key-fit
/// rule from [MS-OFFCRYPTO]).
fn fit(mut v: Vec<u8>, n: usize) -> Vec<u8> {
    if v.len() < n {
        v.resize(n, 0x36);
    } else {
        v.truncate(n);
    }
    v
}

/// Decode standard base64 (ignoring whitespace and `=` padding).
fn b64(s: &str) -> Result<Vec<u8>> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::new();
    let (mut buf, mut bits) = (0u32, 0u32);
    for &c in s.as_bytes() {
        if c == b'=' || c.is_ascii_whitespace() {
            continue;
        }
        let v = val(c).ok_or_else(|| Error::Cfb("invalid base64 in EncryptionInfo".into()))?;
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

/// Find an XML attribute `name="..."` that appears after the marker `after`
/// (e.g. the `<keyData` element). Returns the (unescaped) attribute value.
fn attr_in<'a>(xml: &'a str, after: &str, name: &str) -> Option<&'a str> {
    let start = xml.find(after)?;
    let sub = &xml[start..];
    let key = format!("{name}=\"");
    let i = sub.find(&key)? + key.len();
    let rest = &sub[i..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Like [`attr_in`] but returns a descriptive error when the attribute is
/// absent (used for required agile parameters).
fn req<'a>(xml: &'a str, after: &str, name: &str) -> Result<&'a str> {
    attr_in(xml, after, name)
        .ok_or_else(|| Error::Cfb(format!("EncryptionInfo missing {name} on {after}")))
}

/// Helper for indexing slices with a uniform "short EncryptionInfo" error.
trait OrShort<T> {
    fn ok_or_short(self) -> Result<T>;
}
impl<T> OrShort<T> for Option<T> {
    fn ok_or_short(self) -> Result<T> {
        self.ok_or_else(|| Error::Cfb("truncated EncryptionInfo".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip_known() {
        // "Man" -> "TWFu", "hello" -> "aGVsbG8="
        assert_eq!(b64("TWFu").unwrap(), b"Man");
        assert_eq!(b64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(b64("  aGVs bG8=  ").unwrap(), b"hello");
    }

    #[test]
    fn utf16le_encodes_ascii() {
        assert_eq!(utf16le("AB"), vec![0x41, 0x00, 0x42, 0x00]);
    }

    #[test]
    fn fit_truncates_and_pads() {
        assert_eq!(fit(vec![1, 2, 3, 4], 2), vec![1, 2]);
        assert_eq!(fit(vec![1, 2], 4), vec![1, 2, 0x36, 0x36]);
    }

    #[test]
    fn aes_cbc_then_known_vector() {
        // NIST SP800-38A AES-128-CBC first block.
        let key = hexs("2b7e151628aed2a6abf7158809cf4f3c");
        let iv = hexs("000102030405060708090a0b0c0d0e0f");
        let ct = hexs("7649abac8119b246cee98e9b12e9197d");
        let pt = aes_cbc_decrypt(&key, &iv, &ct).unwrap();
        assert_eq!(pt, hexs("6bc1bee22e409f96e93d7e117393172a"));
    }

    #[test]
    fn attr_scoping_picks_right_element() {
        let xml = r#"<keyData keyBits="128" saltValue="AAA"/><p:encryptedKey spinCount="100000" keyBits="128" saltValue="BBB"/>"#;
        assert_eq!(attr_in(xml, "<keyData", "saltValue"), Some("AAA"));
        assert_eq!(attr_in(xml, "encryptedKey", "saltValue"), Some("BBB"));
        assert_eq!(attr_in(xml, "encryptedKey", "spinCount"), Some("100000"));
    }

    fn hexs(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
}
