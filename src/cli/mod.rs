//! TUI application — state machine and event loop.

pub mod input;
pub mod ui;

use ratatui::Terminal;
use std::io::{self, Write};

use crate::domain::error::{Result, VaultError};
use crate::domain::Account;
use crate::service;

pub use input::AppInput;

/// Application state for the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Splash,
    EnterPassword {
        is_new: bool,
        password: String,
        confirm_password: Option<String>,
        error_msg: Option<String>,
    },
    MainMenu,
    AccountList,
    AccountDetail { account: crate::domain::Account },
    ConfirmDelete { account: crate::domain::Account },
    AddAccount {
        name: String,
        username: String,
        password: String,
        note: String,
        tags: String,
        active_field: usize,
    },
    Search {
        query: String,
        selected: usize,
    },
    PasswordGenerator,
    Quit,
}

const MAX_PASSWORD_ATTEMPTS: u8 = 3;
const CLIPBOARD_CLEAR_SECS: u64 = 10;

/// Unlocked vault state.
pub struct UnlockedVault {
    pub vault: crate::domain::Vault,
    pub master_password: String,
}

impl UnlockedVault {
    fn new(vault: crate::domain::Vault, master_password: String) -> Self {
        Self { vault, master_password }
    }
}

/// The main TUI application.
pub struct App {
    pub state: AppState,
    pub unlocked: Option<UnlockedVault>,
    pub attempts: u8,
    pub clipboard_msg: Option<String>,
    pub pwgen_length: usize,
    pub pwgen_upper: bool,
    pub pwgen_lower: bool,
    pub pwgen_number: bool,
    pub pwgen_symbol: bool,
    pub pwgen_result: Option<String>,
    pub list_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Splash,
            unlocked: None,
            attempts: 0,
            clipboard_msg: None,
            pwgen_length: 16,
            pwgen_upper: true,
            pwgen_lower: true,
            pwgen_number: true,
            pwgen_symbol: true,
            pwgen_result: None,
            list_offset: 0,
        }
    }

    fn create_vault(&mut self, password: &str) -> Result<()> {
        service::validate_master_password(password)?;
        let vault = service::create_vault(password)?;
        self.unlocked = Some(UnlockedVault::new(vault, password.to_string()));
        self.state = AppState::MainMenu;
        Ok(())
    }

    fn do_unlock(&mut self, password: &str) -> Result<()> {
        let vault = service::unlock_vault(password)?;
        self.unlocked = Some(UnlockedVault::new(vault, password.to_string()));
        self.state = AppState::MainMenu;
        Ok(())
    }

    fn lock(&mut self) {
        if let Some(ref unlocked) = self.unlocked {
            let _ = service::lock_vault(&unlocked.vault, &unlocked.master_password);
        }
        self.unlocked = None;
        self.state = AppState::Splash;
        self.attempts = 0;
    }

    pub fn handle_key(&mut self, input: AppInput) {
        match (&self.state, input) {
            // ── Splash ──────────────────────────────────────────────
            (AppState::Splash, AppInput::Enter) => {
                self.state = AppState::EnterPassword {
                    is_new: !service::vault_exists(),
                    password: String::new(),
                    confirm_password: None,
                    error_msg: None,
                };
            }
            (AppState::Splash, AppInput::Esc | AppInput::Char('q') | AppInput::Char('Q')) => {
                self.state = AppState::Quit;
            }

            // ── Enter Password ──────────────────────────────────────
            (AppState::EnterPassword { .. }, AppInput::Esc) => {
                self.state = AppState::Splash;
            }

            // Char input for password or confirm field
            (AppState::EnterPassword { .. }, AppInput::Char(c)) => {
                if let AppState::EnterPassword { password, confirm_password: None, .. } = &mut self.state {
                    if password.len() < 32 {
                        password.push(c);
                    }
                } else if let AppState::EnterPassword { confirm_password: Some(ref mut cp), .. } = &mut self.state {
                    if cp.len() < 32 {
                        cp.push(c);
                    }
                }
            }
            (AppState::EnterPassword { .. }, AppInput::Backspace) => {
                if let AppState::EnterPassword { password, confirm_password: None, .. } = &mut self.state {
                    password.pop();
                } else if let AppState::EnterPassword { confirm_password: Some(ref mut cp), .. } = &mut self.state {
                    cp.pop();
                }
            }

            // Enter on password field (existing vault → unlock)
            (
                AppState::EnterPassword {
                    is_new: false,
                    password: ref pw,
                    confirm_password: None,
                    error_msg: _,
                },
                AppInput::Enter,
            ) => {
                match self.do_unlock(&pw.clone()) {
                    Ok(_) => {}
                    Err(VaultError::WrongMasterPassword) => {
                        self.attempts += 1;
                        if self.attempts >= MAX_PASSWORD_ATTEMPTS {
                            self.state = AppState::Quit;
                            return;
                        }
                        self.state = AppState::EnterPassword {
                            is_new: false,
                            password: String::new(),
                            confirm_password: None,
                            error_msg: Some(format!("Wrong password ({}/{})", self.attempts, MAX_PASSWORD_ATTEMPTS)),
                        };
                    }
                    Err(e) => {
                        self.state = AppState::EnterPassword {
                            is_new: false,
                            password: String::new(),
                            confirm_password: None,
                            error_msg: Some(e.to_string()),
                        };
                    }
                }
            }

            // Enter on password field (new vault → validate & go to confirm)
            (
                AppState::EnterPassword {
                    is_new: true,
                    password: ref pw,
                    confirm_password: None,
                    error_msg: _,
                },
                AppInput::Enter,
            ) => {
                if pw.len() < 8 {
                    self.state = AppState::EnterPassword {
                        is_new: true,
                        password: String::new(),
                        confirm_password: None,
                        error_msg: Some("Password must be at least 8 characters".to_string()),
                    };
                } else {
                    let pwd = pw.clone();
                    self.state = AppState::EnterPassword {
                        is_new: true,
                        password: pwd,
                        confirm_password: Some(String::new()),
                        error_msg: None,
                    };
                }
            }

            // Enter on confirm field
            (
                AppState::EnterPassword {
                    is_new: true,
                    password: ref pwd,
                    confirm_password: Some(ref cp),
                    error_msg: _,
                    ..
                },
                AppInput::Enter,
            ) => {
                if cp != pwd {
                    self.state = AppState::EnterPassword {
                        is_new: true,
                        password: String::new(),
                        confirm_password: None,
                        error_msg: Some("Passwords do not match".to_string()),
                    };
                } else {
                    // Clone to get owned data before the borrow ends
                    let pwd_owned = pwd.clone();
                    if let Err(e) = self.create_vault(&pwd_owned) {
                        self.state = AppState::EnterPassword {
                            is_new: true,
                            password: String::new(),
                            confirm_password: None,
                            error_msg: Some(e.to_string()),
                        };
                    }
                }
            }

            // Any key on error state → retry
            (AppState::EnterPassword { error_msg: Some(_), .. }, _) => {
                let is_new = matches!(self.state, AppState::EnterPassword { is_new: true, .. });
                self.state = AppState::EnterPassword {
                    is_new,
                    password: String::new(),
                    confirm_password: None,
                    error_msg: None,
                };
            }

            // ── Main Menu ────────────────────────────────────────────
            (AppState::MainMenu, AppInput::Char('1')) => {
                self.list_offset = 0;
                self.state = AppState::AccountList;
            }
            (AppState::MainMenu, AppInput::Char('2')) => {
                self.state = AppState::AddAccount {
                    name: String::new(),
                    username: String::new(),
                    password: String::new(),
                    note: String::new(),
                    tags: String::new(),
                    active_field: 0,
                };
            }
            (AppState::MainMenu, AppInput::Char('3')) => {
                self.state = AppState::Search {
                    query: String::new(),
                    selected: 0,
                };
            }
            (AppState::MainMenu, AppInput::Char('4')) => {
                self.regenerate_pwgen();
                self.state = AppState::PasswordGenerator;
            }
            (AppState::MainMenu, AppInput::Esc | AppInput::Char('q') | AppInput::Char('Q')) => {
                self.lock();
            }

            // ── Account List ─────────────────────────────────────────
            (AppState::AccountList, AppInput::Esc) => {
                self.state = AppState::MainMenu;
            }
            (AppState::AccountList, AppInput::Down) => {
                if let Some(ref unlocked) = self.unlocked {
                    let total = unlocked.vault.accounts.len();
                    if self.list_offset + 1 < total {
                        self.list_offset += 1;
                    }
                }
            }
            (AppState::AccountList, AppInput::Up) => {
                if self.list_offset > 0 {
                    self.list_offset -= 1;
                }
            }
            (AppState::AccountList, AppInput::Enter) => {
                if let Some(account) = self.get_selected_account() {
                    self.state = AppState::AccountDetail { account };
                }
            }
            (AppState::AccountList, AppInput::Char('d') | AppInput::Char('D')) => {
                if let Some(account) = self.get_selected_account() {
                    self.state = AppState::ConfirmDelete { account };
                }
            }
            (AppState::AccountList, AppInput::Char('c') | AppInput::Char('C')) => {
                self.copy_selected_password();
            }

            // ── Account Detail ───────────────────────────────────────
            (AppState::AccountDetail { .. }, AppInput::Esc) => {
                self.list_offset = 0;
                self.state = AppState::AccountList;
            }
            (AppState::AccountDetail { account, .. }, AppInput::Char('c') | AppInput::Char('C')) => {
                if let Err(e) = service::copy_to_clipboard(&account.password) {
                    self.clipboard_msg = Some(format!("Copy failed: {}", e));
                } else {
                    self.clipboard_msg = Some(format!("Copied! (clears in {}s)", CLIPBOARD_CLEAR_SECS));
                }
                let _ = std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(CLIPBOARD_CLEAR_SECS));
                    let _ = service::copy_to_clipboard("");
                });
            }

            // ── Confirm Delete ───────────────────────────────────────
            (AppState::ConfirmDelete { .. }, AppInput::Esc) => {
                self.state = AppState::AccountList;
            }
            (AppState::ConfirmDelete { account }, AppInput::Enter) => {
                if let Some(ref mut unlocked) = self.unlocked {
                    let _ = service::delete_account(&mut unlocked.vault, account.id, &unlocked.master_password);
                }
                self.state = AppState::AccountList;
            }

            // ── Add Account ───────────────────────────────────────────
            (AppState::AddAccount { .. }, AppInput::Esc) => {
                self.state = AppState::MainMenu;
            }
            (AppState::AddAccount { .. }, AppInput::Enter) => {
                self.commit_add_account();
            }
            (AppState::AddAccount { .. }, AppInput::Tab) => {
                if let AppState::AddAccount { active_field, .. } = &mut self.state {
                    *active_field = (*active_field + 1) % 5;
                }
            }
            (AppState::AddAccount { .. }, AppInput::Char(c)) => {
                if let AppState::AddAccount { name, username, password, note, tags, active_field, .. } = &mut self.state {
                    match *active_field {
                        0 => name.push(c),
                        1 => username.push(c),
                        2 => password.push(c),
                        3 => note.push(c),
                        4 => tags.push(c),
                        _ => {}
                    }
                }
            }
            (AppState::AddAccount { .. }, AppInput::Backspace) => {
                if let AppState::AddAccount { name, username, password, note, tags, active_field, .. } = &mut self.state {
                    match *active_field {
                        0 => { name.pop(); }
                        1 => { username.pop(); }
                        2 => { password.pop(); }
                        3 => { note.pop(); }
                        4 => { tags.pop(); }
                        _ => {}
                    }
                }
            }

            // ── Search ───────────────────────────────────────────────
            (AppState::Search { .. }, AppInput::Esc) => {
                self.state = AppState::MainMenu;
            }
            (AppState::Search { .. }, AppInput::Char(c)) => {
                if let AppState::Search { query, selected, .. } = &mut self.state {
                    query.push(c);
                    *selected = 0;
                }
            }
            (AppState::Search { .. }, AppInput::Backspace) => {
                if let AppState::Search { query, selected, .. } = &mut self.state {
                    query.pop();
                    *selected = selected.saturating_sub(1);
                }
            }
            (AppState::Search { .. }, AppInput::Down) => {
                let results = self.get_search_results();
                if let AppState::Search { selected, .. } = &mut self.state {
                    if !results.is_empty() {
                        *selected = (*selected + 1).min(results.len() - 1);
                    }
                }
            }
            (AppState::Search { .. }, AppInput::Up) => {
                if let AppState::Search { selected, .. } = &mut self.state {
                    *selected = selected.saturating_sub(1);
                }
            }
            (AppState::Search { .. }, AppInput::Enter) => {
                let results = self.get_search_results();
                if let AppState::Search { selected, .. } = &mut self.state {
                    if !results.is_empty() && *selected < results.len() {
                        self.state = AppState::AccountDetail { account: results[*selected].clone() };
                    }
                }
            }

            // ── Password Generator ──────────────────────────────────
            (AppState::PasswordGenerator, AppInput::Esc) => {
                self.state = AppState::MainMenu;
            }
            (AppState::PasswordGenerator, AppInput::Enter) => {
                self.regenerate_pwgen();
            }
            (AppState::PasswordGenerator, AppInput::Char('c') | AppInput::Char('C')) => {
                if let Some(ref pw) = self.pwgen_result {
                    if let Err(e) = service::copy_to_clipboard(pw) {
                        self.clipboard_msg = Some(format!("Copy failed: {}", e));
                    } else {
                        self.clipboard_msg = Some(format!("Copied! (clears in {}s)", CLIPBOARD_CLEAR_SECS));
                    }
                    let _ = std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(CLIPBOARD_CLEAR_SECS));
                        let _ = service::copy_to_clipboard("");
                    });
                }
            }
            (AppState::PasswordGenerator, AppInput::Char('+')) => {
                if self.pwgen_length < 64 {
                    self.pwgen_length += 1;
                    self.regenerate_pwgen();
                }
            }
            (AppState::PasswordGenerator, AppInput::Char('-')) => {
                if self.pwgen_length > 8 {
                    self.pwgen_length -= 1;
                    self.regenerate_pwgen();
                }
            }
            (AppState::PasswordGenerator, AppInput::Char('a')) => {
                self.pwgen_upper = !self.pwgen_upper;
                self.regenerate_pwgen();
            }
            (AppState::PasswordGenerator, AppInput::Char('b')) => {
                self.pwgen_lower = !self.pwgen_lower;
                self.regenerate_pwgen();
            }
            (AppState::PasswordGenerator, AppInput::Char('n')) => {
                self.pwgen_number = !self.pwgen_number;
                self.regenerate_pwgen();
            }
            (AppState::PasswordGenerator, AppInput::Char('s')) => {
                self.pwgen_symbol = !self.pwgen_symbol;
                self.regenerate_pwgen();
            }

            _ => {}
        }
    }

    fn get_selected_account(&self) -> Option<Account> {
        let vault = self.unlocked.as_ref()?.vault.sorted_accounts();
        vault.get(self.list_offset).map(|a| (*a).clone())
    }

    fn get_search_results(&self) -> Vec<Account> {
        let AppState::Search { query, .. } = &self.state else {
            return Vec::new();
        };
        if query.is_empty() {
            return Vec::new();
        }
        self.unlocked.as_ref()
            .map(|u| u.vault.search(query).into_iter().map(|a| (*a).clone()).collect())
            .unwrap_or_default()
    }

    fn copy_selected_password(&mut self) {
        if let Some(account) = self.get_selected_account() {
            if let Err(e) = service::copy_to_clipboard(&account.password) {
                self.clipboard_msg = Some(format!("Copy failed: {}", e));
            } else {
                self.clipboard_msg = Some(format!("Copied! (clears in {}s)", CLIPBOARD_CLEAR_SECS));
            }
            let _ = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(CLIPBOARD_CLEAR_SECS));
                let _ = service::copy_to_clipboard("");
            });
        }
    }

    fn commit_add_account(&mut self) {
        let AppState::AddAccount { name, username, password, note, tags, .. } = &mut self.state else {
            return;
        };
        if name.trim().is_empty() || username.trim().is_empty() || password.is_empty() {
            return;
        }
        let note_str = if note.trim().is_empty() {
            None
        } else {
            Some(note.trim().to_string())
        };
        let tags_vec: Vec<String> = tags.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(ref mut unlocked) = self.unlocked {
            if service::add_account(
                &mut unlocked.vault,
                name.trim().to_string(),
                username.trim().to_string(),
                password.clone(),
                note_str,
                tags_vec,
                &unlocked.master_password,
            ).is_ok() {
                *name = String::new();
                *username = String::new();
                *password = String::new();
                *note = String::new();
                *tags = String::new();
                self.state = AppState::MainMenu;
            }
        }
    }

    fn regenerate_pwgen(&mut self) {
        self.pwgen_result = Some(service::generate_password(
            self.pwgen_length,
            self.pwgen_upper,
            self.pwgen_lower,
            self.pwgen_number,
            self.pwgen_symbol,
        ));
    }
}

/// Run the TUI event loop.
pub fn run_tui(mut app: App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use ratatui::backend::CrosstermBackend;
    use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};

    // Enable raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        if app.state == AppState::Quit {
            break;
        }

        terminal.draw(|f| {
            ui::render(&app, f);
        })?;

        let input = input::read_input();
        app.handle_key(input);
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    io::stdout().flush()?;
    Ok(())
}
