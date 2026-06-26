//! 凭据掩码与 AES-GCM 加解密（F-OP-02）。

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// 展示用掩码：保留末 4 位。
pub fn mask_secret(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "（未配置）".into();
    }
    if trimmed.len() <= 4 {
        return "****".into();
    }
    format!("****{}", &trimmed[trimmed.len() - 4..])
}

/// 校验 ENCRYPTION_KEY 是否已设置（32+ 字符推荐）。
pub fn encryption_ready(key: &str) -> bool {
    key.trim().len() >= 16
}

fn derive_key(raw: &str) -> [u8; 32] {
    let digest = Sha256::digest(raw.trim().as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// 加密明文，返回 base64(nonce || ciphertext)。
pub fn encrypt_value(plaintext: &str, encryption_key: &str) -> Result<String, String> {
    if !encryption_ready(encryption_key) {
        return Err("ENCRYPTION_KEY not configured".into());
    }
    let key = derive_key(encryption_key);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut packed = nonce_bytes.to_vec();
    packed.extend(ciphertext);
    Ok(STANDARD.encode(packed))
}

/// 解密 `encrypt_value` 输出。
pub fn decrypt_value(encoded: &str, encryption_key: &str) -> Result<String, String> {
    if !encryption_ready(encryption_key) {
        return Err("ENCRYPTION_KEY not configured".into());
    }
    let packed = STANDARD.decode(encoded).map_err(|e| e.to_string())?;
    if packed.len() < 13 {
        return Err("invalid ciphertext".into());
    }
    let (nonce_bytes, ciphertext) = packed.split_at(12);
    let key = derive_key(encryption_key);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;
    String::from_utf8(plain).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_roundtrip() {
        let key = "test-encryption-key-32chars!!";
        let enc = encrypt_value("sk-secret-api-key", key).unwrap();
        let dec = decrypt_value(&enc, key).unwrap();
        assert_eq!(dec, "sk-secret-api-key");
    }
}
