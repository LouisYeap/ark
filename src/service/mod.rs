//! Business logic layer.

use uuid::Uuid;

use crate::domain::error::{Result, VaultError};
use crate::domain::{Account, Vault};
use crate::storage;

/// Validate master password requirements: 8–32 characters.
pub fn validate_master_password(password: &str) -> Result<()> {
    if !(8..=32).contains(&password.len()) {
        return Err(VaultError::WeakMasterPassword);
    }
    Ok(())
}

/// Check whether a vault already exists on disk.
pub fn vault_exists() -> bool {
    storage::vault_exists()
}

/// Create a new vault with the given master password.
pub fn create_vault(master_password: &str) -> Result<Vault> {
    validate_master_password(master_password)?;
    let vault = Vault::default();
    storage::save(&vault, master_password)?;
    Ok(vault)
}

/// Unlock an existing vault with the master password.
pub fn unlock_vault(master_password: &str) -> Result<Vault> {
    storage::load(master_password)
}

/// Lock the vault (save it back to disk).
pub fn lock_vault(vault: &Vault, master_password: &str) -> Result<()> {
    storage::save(vault, master_password)
}

/// Add an account to the vault and persist.
pub fn add_account(
    vault: &mut Vault,
    name: String,
    username: String,
    password: String,
    note: Option<String>,
    tags: Vec<String>,
    master_password: &str,
) -> Result<Uuid> {
    if name.trim().is_empty() {
        return Err(VaultError::InvalidInput("account name cannot be empty".to_string()));
    }
    let account = Account::new(name, username, password).with_options(note, tags);
    let id = vault.add_account(account);
    storage::save(vault, master_password)?;
    Ok(id)
}

/// Delete an account and persist.
pub fn delete_account(vault: &mut Vault, id: Uuid, master_password: &str) -> Result<()> {
    if vault.remove_account(id).is_none() {
        return Err(VaultError::AccountNotFound(id.to_string()));
    }
    storage::save(vault, master_password)?;
    Ok(())
}

/// Generate a random password with given parameters.
pub fn generate_password(
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut chars = String::new();
    if uppercase { chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ"); }
    if lowercase { chars.push_str("abcdefghijklmnopqrstuvwxyz"); }
    if numbers   { chars.push_str("0123456789"); }
    if symbols   { chars.push_str("!@#$%^&*()-_=+[]{}|;:,.<>?"); }

    if chars.is_empty() {
        chars.push_str("abcdefghijklmnopqrstuvwxyz");
    }

    let chars: Vec<char> = chars.chars().collect();
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// Copy text to system clipboard.
#[cfg(target_os = "macos")]
pub fn copy_to_clipboard(text: &str) -> std::result::Result<(), String> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    }
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn copy_to_clipboard(text: &str) -> std::result::Result<(), String> {
    use arboard::Clipboard;
    let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
pub fn copy_to_clipboard(text: &str) -> std::result::Result<(), String> {
    use std::process::{Command, Stdio};
    use std::io::Write;

    // Try xclip first
    if Command::new("which").arg("xclip").output().map(|o| o.status.success()).unwrap_or(false) {
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        child.wait().map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Try wl-paste
    if Command::new("which").arg("wl-paste").output().map(|o| o.status.success()).unwrap_or(false) {
        let mut child = Command::new("wl-paste")
            .args(["--type", "text/plain"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        child.wait().map_err(|e| e.to_string())?;
        return Ok(());
    }

    Err("no clipboard tool found (install xclip or wl-clipboard)".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn copy_to_clipboard(_text: &str) -> std::result::Result<(), String> {
    Err("clipboard not supported on this platform".to_string())
}
