//! Cryptographic operations: Argon2id key derivation + AES-256-GCM encryption.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::{Rng, rngs::OsRng};

use crate::domain::error::{VaultError, Result};

/// Derive a 256-bit key from a master password using Argon2id.
/// Returns (key, base64_salt).
pub fn derive_key(password: &str) -> Result<(Vec<u8>, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

    let hash_str = hash.hash.ok_or(VaultError::EncryptionFailed("no hash output".to_string()))?;
    let key = hash_str.as_bytes();

    if key.len() < 32 {
        return Err(VaultError::EncryptionFailed("key too short".to_string()));
    }

    Ok((key[..32].to_vec(), salt.to_string()))
}

/// Derive a key from an existing password + salt (for decryption).
pub fn derive_key_with_salt(password: &str, salt_b64: &str) -> Result<Vec<u8>> {
    let salt = SaltString::from_b64(salt_b64)
        .map_err(|_| VaultError::DecryptionFailed("invalid salt".to_string()))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| VaultError::DecryptionFailed(e.to_string()))?;

    let hash_str = hash.hash.ok_or(VaultError::DecryptionFailed("no hash output".to_string()))?;
    let key = hash_str.as_bytes();

    if key.len() < 32 {
        return Err(VaultError::DecryptionFailed("key too short".to_string()));
    }

    Ok(key[..32].to_vec())
}

/// Encrypt plaintext with AES-256-GCM.
/// Returns (ciphertext, nonce) — both as raw bytes.
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

    // 96-bit (12-byte) random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypt ciphertext with AES-256-GCM.
pub fn decrypt(ciphertext: &[u8], key: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::DecryptionFailed(e.to_string()))?;

    if nonce_bytes.len() != 12 {
        return Err(VaultError::DecryptionFailed("invalid nonce length".to_string()));
    }

    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| VaultError::WrongMasterPassword)
}

/// Encrypt a vault JSON string. Returns (ciphertext_b64, nonce_b64, salt_b64).
pub fn encrypt_vault(vault_json: &str, master_password: &str) -> Result<(String, String, String)> {
    let (key, salt_b64) = derive_key(master_password)?;
    let (ciphertext, nonce) = encrypt(vault_json.as_bytes(), &key)?;
    use base64::Engine;
    Ok((
        base64::engine::general_purpose::STANDARD.encode(&ciphertext),
        base64::engine::general_purpose::STANDARD.encode(&nonce),
        salt_b64,
    ))
}

/// Decrypt vault ciphertext. Returns plaintext JSON bytes.
pub fn decrypt_vault(
    ciphertext_b64: &str,
    nonce_b64: &str,
    salt_b64: &str,
    master_password: &str,
) -> Result<Vec<u8>> {
    use base64::Engine;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|_| VaultError::VaultCorrupted)?;
    let nonce = base64::engine::general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|_| VaultError::VaultCorrupted)?;

    let key = derive_key_with_salt(master_password, salt_b64)?;
    decrypt(&ciphertext, &key, &nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "test_master_password_123";
        let plaintext = b"Hello, World!";

        let (ciphertext, nonce, salt) = encrypt_vault(std::str::from_utf8(plaintext).unwrap(), password).unwrap();
        let decrypted = decrypt_vault(&ciphertext, &nonce, &salt, password).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let (ciphertext, nonce, salt) = encrypt_vault("secret data", "correct_password").unwrap();
        let result = decrypt_vault(&ciphertext, &nonce, &salt, "wrong_password");
        assert!(result.is_err());
    }
}
