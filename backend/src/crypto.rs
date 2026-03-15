use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;

fn get_encryption_key() -> [u8; 32] {
    let key_str =
        std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
            // Default key for development only
            STANDARD.encode([0u8; 32])
        });
    let key_bytes = STANDARD.decode(&key_str).expect("Invalid base64 ENCRYPTION_KEY");
    let mut key = [0u8; 32];
    let len = key_bytes.len().min(32);
    key[..len].copy_from_slice(&key_bytes[..len]);
    key
}

pub fn encrypt(plaintext: &str) -> String {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("Encryption failed");

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    STANDARD.encode(&combined)
}

pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Invalid key: {}", e))?;

    let combined = STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    if combined.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
}

pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    let last4 = &key[key.len() - 4..];
    if key.starts_with("sk-") {
        format!("sk-****{}", last4)
    } else {
        format!("****{}", last4)
    }
}
