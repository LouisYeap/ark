//! Domain errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("wrong master password")]
    WrongMasterPassword,

    #[error("vault file not found or corrupted")]
    VaultCorrupted,

    #[error("account not found: {0}")]
    AccountNotFound(String),

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("I/O error")]
    IoError,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("master password does not meet requirements (8-32 characters)")]
    WeakMasterPassword,
}

pub type Result<T> = std::result::Result<T, VaultError>;

/// Error codes for programmatic error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ErrorCode {
    WrongMasterPassword,
    VaultCorrupted,
    AccountNotFound,
    EncryptionFailed,
    DecryptionFailed,
    IoError,
    InvalidInput,
    WeakMasterPassword,
}
