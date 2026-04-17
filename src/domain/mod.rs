//! Domain models for the password vault.

pub mod error;


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single account (website/app) stored in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub password: String,
    pub note: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, username: String, password: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            username,
            password,
            note: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_options(mut self, note: Option<String>, tags: Vec<String>) -> Self {
        self.note = note;
        self.tags = tags;
        self.updated_at = Utc::now();
        self
    }
}

/// In-memory vault — holds all accounts in plaintext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub version: String,
    pub accounts: Vec<Account>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Vault {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            accounts: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl Vault {
    pub fn add_account(&mut self, account: Account) -> Uuid {
        let id = account.id;
        self.accounts.push(account);
        self.updated_at = Utc::now();
        id
    }

    pub fn remove_account(&mut self, id: Uuid) -> Option<Account> {
        if let Some(pos) = self.accounts.iter().position(|a| a.id == id) {
            self.updated_at = Utc::now();
            Some(self.accounts.remove(pos))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn update_account(&mut self, account: Account) {
        if let Some(existing) = self.accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = account;
            self.updated_at = Utc::now();
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Account> {
        let q = query.to_lowercase();
        self.accounts
            .iter()
            .filter(|a| {
                a.name.to_lowercase().contains(&q)
                    || a.username.to_lowercase().contains(&q)
                    || a.note.as_ref().map_or(false, |n| n.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn sorted_accounts(&self) -> Vec<&Account> {
        let mut accounts: Vec<&Account> = self.accounts.iter().collect();
        accounts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        accounts
    }
}
