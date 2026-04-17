//! File storage — reads/writes the encrypted vault file.

use std::fs;
use std::path::PathBuf;

use crate::domain::error::{VaultError, Result};
use crate::domain::Vault;
use crate::crypto::{encrypt_vault, decrypt_vault};
use serde::{Deserialize, Serialize};

/// On-disk encrypted vault format.
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedVault {
    pub version: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

/// Resolve vault file path: platform-specific config directory.
fn vault_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or(VaultError::VaultCorrupted)?;
    let vault_dir = dir.join("ark");
    fs::create_dir_all(&vault_dir)
        .map_err(|_| VaultError::IoError)?;
    Ok(vault_dir.join("vault.ark"))
}

/// Check whether the vault file exists.
pub fn vault_exists() -> bool {
    vault_path().map(|p| p.exists()).unwrap_or(false)
}

/// Save vault to disk, encrypted with master password.
pub fn save(vault: &Vault, master_password: &str) -> Result<()> {
    let path = vault_path()?;
    let json = serde_json::to_string(vault)
        .map_err(|_| VaultError::VaultCorrupted)?;

    let (ciphertext, nonce, salt) = encrypt_vault(&json, master_password)?;

    let encrypted = EncryptedVault {
        version: vault.version.clone(),
        salt,
        nonce,
        ciphertext,
    };

    let data = serde_json::to_string_pretty(&encrypted)
        .map_err(|_| VaultError::VaultCorrupted)?;

    fs::write(&path, data).map_err(|_| VaultError::IoError)?;
    Ok(())
}

/// Load vault from disk, decrypted with master password.
pub fn load(master_password: &str) -> Result<Vault> {
    let path = vault_path()?;
    let data = fs::read_to_string(&path).map_err(|_| VaultError::VaultCorrupted)?;
    let encrypted: EncryptedVault = serde_json::from_str(&data)
        .map_err(|_| VaultError::VaultCorrupted)?;

    let plaintext = decrypt_vault(
        &encrypted.ciphertext,
        &encrypted.nonce,
        &encrypted.salt,
        master_password,
    )?;

    serde_json::from_slice(&plaintext)
        .map_err(|_| VaultError::VaultCorrupted)
}
