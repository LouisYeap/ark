//! Ark — Secure offline password vault.

mod cli;
mod domain;
mod crypto;
mod storage;
mod service;

use cli::{run_tui, App};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for vault existence to show correct splash screen
    let app = App::new();
    run_tui(app)?;
    Ok(())
}
