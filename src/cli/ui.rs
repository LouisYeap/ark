//! UI rendering — ratatui 0.26.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::cli::{App, AppState};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const WHITE: Color = Color::White;
const ERROR: Color = Color::Red;
const SUCCESS: Color = Color::Green;
const MUTED: Color = Color::Blue;

// ─────────────────────────────────────────────────────────────────────────────

pub fn render(app: &App, frame: &mut Frame) {
    match &app.state {
        AppState::Splash => render_splash(frame),
        AppState::EnterPassword { .. } => render_enter_password(app, frame),
        AppState::MainMenu => render_main_menu(frame),
        AppState::AccountList => render_account_list(app, frame),
        AppState::AccountDetail { .. } => render_account_detail(app, frame),
        AppState::ConfirmDelete { .. } => render_confirm_delete(app, frame),
        AppState::AddAccount { .. } => render_add_account(app, frame),
        AppState::Search { .. } => render_search(app, frame),
        AppState::PasswordGenerator => render_password_generator(app, frame),
        AppState::Quit => {}
    }

    if let Some(ref msg) = app.clipboard_msg {
        render_clipboard_notif(frame, msg);
    }
}

// ─────────────────────────────────────────────────────────────────────────────

fn centered_rect(heights: &[u16], size: Rect, v_margin: u16) -> Vec<Rect> {
    let total: u16 = heights.iter().sum();
    let available = size.height.saturating_sub(total).saturating_sub(v_margin * 2);
    let top = size.y + v_margin + available / 2;
    let mut out = vec![];
    let mut cur = top;
    for &h in heights {
        out.push(Rect { x: size.x, y: cur, width: size.width, height: h });
        cur += h;
    }
    out
}

/// Styled paragraph — takes owned String for lifetime simplicity.
fn para(text: String, fg: Color, align: Alignment) -> Paragraph<'static> {
    Paragraph::new(Text::from(text)).style(Style::default().fg(fg)).alignment(align)
}

/// Unstyled raw span.
fn sr(text: String) -> Span<'static> {
    Span::raw(text)
}

/// Styled span with foreground color.
fn ss(text: String, color: Color) -> Span<'static> {
    Span::styled(text, Style::default().fg(color))
}

/// Styled span with foreground color + bold.
fn ssb(text: String, color: Color) -> Span<'static> {
    Span::styled(text, Style::default().fg(color).add_modifier(Modifier::BOLD))
}

// ─────────────────────────────────────────────────────────────────────────────
// Splash
// ─────────────────────────────────────────────────────────────────────────────

fn render_splash(frame: &mut Frame) {
    let size = frame.size();
    let chunks = centered_rect(&[3, 1, 3], size, 0);
    frame.render_widget(para("🔐  A R K".to_string(), ACCENT, Alignment::Center), chunks[0]);
    frame.render_widget(para("Secure Password Vault".to_string(), DIM, Alignment::Center), chunks[1]);
    frame.render_widget(para("[ Enter ] Unlock or create vault    [ Q ] Quit".to_string(), DIM, Alignment::Center), chunks[2]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Master password entry
// ─────────────────────────────────────────────────────────────────────────────

fn render_enter_password(app: &App, frame: &mut Frame) {
    let (is_new, password, confirm_password, error_msg) = match &app.state {
        AppState::EnterPassword { is_new, password, confirm_password, error_msg } => {
            (is_new, password, confirm_password, error_msg)
        }
        _ => return,
    };

    let size = frame.size();
    let heading = if *is_new { "🔑  Create Master Password" } else { "🔑  Enter Master Password" };

    let hint_str: String = if let Some(ref err) = error_msg {
        format!("❗ {}", err)
    } else if *is_new {
        if confirm_password.is_some() {
            "[ Enter ] Confirm    [ Esc ] Back".to_string()
        } else {
            "[ Enter ] Next    [ ← ] Delete    [ Esc ] Back".to_string()
        }
    } else {
        format!(
            "[ Enter ] Unlock    [ ← ] Delete    [ Esc ] Back ({}/{})",
            app.attempts,
            crate::cli::MAX_PASSWORD_ATTEMPTS
        )
    };

    let hint_color = if error_msg.is_some() { ERROR } else { DIM };
    let dots = "●".repeat(password.len());

    if *is_new && confirm_password.is_some() {
        let chunks = centered_rect(&[3, 3, 3, 3], size, 0);
        frame.render_widget(para(heading.to_string(), ACCENT, Alignment::Center), chunks[0]);
        frame.render_widget(para(dots, WHITE, Alignment::Center), chunks[1]);
        frame.render_widget(para("●".repeat(confirm_password.as_ref().unwrap().len()), DIM, Alignment::Center), chunks[2]);
        frame.render_widget(para(hint_str, hint_color, Alignment::Center), chunks[3]);
    } else {
        let chunks = centered_rect(&[3, 3, 3], size, 0);
        frame.render_widget(para(heading.to_string(), ACCENT, Alignment::Center), chunks[0]);
        frame.render_widget(para(dots, WHITE, Alignment::Center), chunks[1]);
        frame.render_widget(para(hint_str, hint_color, Alignment::Center), chunks[2]);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main menu
// ─────────────────────────────────────────────────────────────────────────────

fn render_main_menu(frame: &mut Frame) {
    let size = frame.size();
    let block = Block::default()
        .title(Line::from("  Main Menu  "))
        .borders(Borders::ALL)
        .style(Style::default().fg(WHITE));

    let inner = block.inner(size);

    let items: [(&str, &str, &str); 5] = [
        ("1", "📋  Account List", "View and manage accounts"),
        ("2", "➕  Add Account", "Store a new login"),
        ("3", "🔍  Search", "Quick search by name or username"),
        ("4", "🎲  Password Generator", "Create a strong password"),
        ("Q", "🔒  Lock & Quit", "Lock vault and exit"),
    ];

    let lines: Vec<Line<'_>> = items
        .iter()
        .flat_map(|(key, title, desc)| {
            let k = format!("[{}] ", key);
            let d = format!("   {}", desc);
            vec![
                Line::from(vec![ssb(k, ACCENT), sr(title.to_string()), ss(d, DIM)]),
                Line::from(""),
            ]
        })
        .collect();

    frame.render_widget(block, size);
    frame.render_widget(Paragraph::new(lines), inner);

    let hint = Paragraph::new(Line::from("🔒 Vault is unlocked · Press key to navigate"))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center);
    frame.render_widget(
        hint,
        Rect { x: 0, y: size.height.saturating_sub(1), width: size.width, height: 1 },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Account list
// ─────────────────────────────────────────────────────────────────────────────

fn render_account_list(app: &App, frame: &mut Frame) {
    let vault = app.unlocked.as_ref().expect("vault not unlocked").vault.sorted_accounts();
    let total = vault.len();
    let offset = app.list_offset;
    let size = frame.size();

    let title_str = format!("  Account List  —  {} account(s)  ", total);
    let block = Block::default()
        .title(Line::from(title_str.as_str()))
        .borders(Borders::ALL)
        .style(Style::default().fg(WHITE));

    let page_size = (size.height as usize).saturating_sub(6);
    let page: Vec<ListItem<'_>> = vault
        .iter()
        .skip(offset)
        .take(page_size)
        .enumerate()
        .map(|(i, account)| {
            let idx = offset + i;
            let sel = if idx == offset { "▶" } else { " " };
            let tags_str = if account.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", account.tags.join(", "))
            };
            let name = account.name.clone();
            let username = account.username.clone();
            let tags = tags_str;
            ListItem::new(Line::from(vec![
                ss(sel.to_string(), ACCENT),
                sr("  ".to_string()),
                ssb(name, WHITE),
                sr("  ·  ".to_string()),
                ss(username, MUTED),
                ss(tags, DIM),
            ]))
        })
        .collect();

    let list_items: Vec<ListItem<'_>> = if page.is_empty() {
        vec![ListItem::new(ss("  No accounts yet. Press [2] to add one.".to_string(), DIM))]
    } else {
        page
    };

    frame.render_widget(List::new(list_items).block(block), size);

    let nav = Paragraph::new(Line::from("[ ↑/↓ ] Move   [Enter] View   [C] Copy   [D] Delete   [Esc] Back"))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center);
    frame.render_widget(
        nav,
        Rect { x: 0, y: size.height.saturating_sub(1), width: size.width, height: 1 },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Account detail
// ─────────────────────────────────────────────────────────────────────────────

fn render_account_detail(app: &App, frame: &mut Frame) {
    let account = match &app.state {
        AppState::AccountDetail { account } => account,
        _ => return,
    };

    let size = frame.size();
    let title_str = format!("  {}  ", account.name);
    let block = Block::default()
        .title(Line::from(title_str.as_str()))
        .borders(Borders::ALL)
        .style(Style::default().fg(WHITE));

    let inner = block.inner(size);

    let note_str = account.note.as_deref().unwrap_or("—");
    let tags_str = if account.tags.is_empty() { "—" } else { &account.tags.join(", ") };
    let created_str = account.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated_str = account.updated_at.format("%Y-%m-%d %H:%M").to_string();

    let name = account.name.clone();
    let username = account.username.clone();
    let note = note_str.to_string();
    let tags = tags_str.to_string();
    let created = created_str.clone();
    let updated = updated_str.clone();

    let rows: Vec<Line<'_>> = vec![
        Line::from(vec![ss("   Name: ".to_string(), DIM), ss(name, WHITE)]),
        Line::from(vec![ss("   Username: ".to_string(), DIM), ss(username, WHITE)]),
        Line::from(vec![ss("   Password: ".to_string(), DIM), ss("••••••••••••".to_string(), WHITE)]),
        Line::from(vec![ss("   Note: ".to_string(), DIM), ss(note, WHITE)]),
        Line::from(vec![ss("   Tags: ".to_string(), DIM), ss(tags, WHITE)]),
        Line::from(vec![ss("   Created: ".to_string(), DIM), ss(created, WHITE)]),
        Line::from(vec![ss("   Updated: ".to_string(), DIM), ss(updated, WHITE)]),
    ];

    frame.render_widget(block, size);
    frame.render_widget(Paragraph::new(rows).wrap(Wrap { trim: true }), inner);

    let hint = Paragraph::new(Line::from(" [C] Copy password   [Esc] Back"))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center);
    frame.render_widget(
        hint,
        Rect { x: 0, y: size.height.saturating_sub(1), width: size.width, height: 1 },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Confirm delete
// ─────────────────────────────────────────────────────────────────────────────

fn render_confirm_delete(app: &App, frame: &mut Frame) {
    let account = match &app.state {
        AppState::ConfirmDelete { account } => account,
        _ => return,
    };

    let size = frame.size();
    let chunks = centered_rect(&[3, 5, 3], size, 0);

    frame.render_widget(para("⚠  Confirm Delete".to_string(), ERROR, Alignment::Center), chunks[0]);

    let name = account.name.clone();
    let username = account.username.clone();
    let lines: Vec<Line<'_>> = vec![
        Line::from(ss(name, WHITE)),
        Line::from(ss(username, MUTED)),
        Line::from(""),
        Line::from(ss("This action cannot be undone.".to_string(), DIM)),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), chunks[1]);

    frame.render_widget(
        para("[ Enter ] Delete permanently   [ Esc ] Cancel".to_string(), DIM, Alignment::Center),
        chunks[2],
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Add account
// ─────────────────────────────────────────────────────────────────────────────

fn render_add_account(app: &App, frame: &mut Frame) {
    let (name, username, password, note, tags, active_field) = match &app.state {
        AppState::AddAccount { name, username, password, note, tags, active_field } => {
            (name.clone(), username.clone(), password.clone(), note.clone(), tags.clone(), *active_field)
        }
        _ => return,
    };

    let size = frame.size();
    let block = Block::default()
        .title(Line::from("  Add Account  "))
        .borders(Borders::ALL)
        .style(Style::default().fg(WHITE));

    frame.render_widget(block, size);

    let inner = Rect {
        x: 2,
        y: 2,
        width: size.width.saturating_sub(4),
        height: size.height.saturating_sub(4),
    };

    let field_names = ["Name", "Username", "Password", "Note", "Tags"];
    let field_values: Vec<String> = vec![
        name.clone(),
        username.clone(),
        "•".repeat(password.len()),
        note.clone(),
        tags.clone(),
    ];

    let mut lines = vec![Line::from("")];
    for (i, fname) in field_names.iter().enumerate() {
        let is_active = active_field == i;
        let (fg, prefix, cursor) = if is_active { (ACCENT, "▶ ", " █") } else { (DIM, "  ", "") };
        let display = if field_values[i].is_empty() {
            if is_active { String::new() } else { "—".to_string() }
        } else {
            field_values[i].clone()
        };
        let prefix_str = format!("{}{}: ", prefix, fname);
        lines.push(Line::from(vec![
            ss(prefix_str, fg),
            ss(display, WHITE),
            ss(cursor.to_string(), ACCENT),
        ]));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);

    let hint = Paragraph::new(Line::from("[ Tab ] Switch field   [ Enter ] Save   [ Esc ] Cancel"))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center);
    frame.render_widget(
        hint,
        Rect { x: 0, y: size.height.saturating_sub(1), width: size.width, height: 1 },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Search
// ─────────────────────────────────────────────────────────────────────────────

fn render_search(app: &App, frame: &mut Frame) {
    let (query, selected) = match &app.state {
        AppState::Search { query, selected } => (query.clone(), *selected),
        _ => return,
    };

    let size = frame.size();
    let search_height = 3u16;

    let search_text = format!("🔍  {}", query);
    frame.render_widget(
        Paragraph::new(Line::from(search_text.as_str()))
            .block(
                Block::default()
                    .title(Line::from("  Search  "))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(WHITE)),
            )
            .style(Style::default().fg(ACCENT)),
        Rect { x: 0, y: 0, width: size.width, height: search_height },
    );

    let results_area = Rect {
        x: 0,
        y: search_height,
        width: size.width,
        height: size.height.saturating_sub(search_height).saturating_sub(2),
    };

    let results = app.unlocked.as_ref().map(|u| u.vault.search(&query)).unwrap_or_default();

    let list_items: Vec<ListItem<'_>> = if results.is_empty() && !query.is_empty() {
        vec![ListItem::new(ss("  No results found.".to_string(), DIM))]
    } else {
        results
            .iter()
            .enumerate()
            .map(|(i, account)| {
                let sel = if i == selected { "▶" } else { " " };
                let name = account.name.clone();
                let username = account.username.clone();
                ListItem::new(Line::from(vec![
                    ss(sel.to_string(), ACCENT),
                    sr("  ".to_string()),
                    ssb(name, WHITE),
                    sr("  ·  ".to_string()),
                    ss(username, MUTED),
                ]))
            })
            .collect()
    };

    frame.render_widget(
        List::new(list_items).block(Block::default().borders(Borders::ALL)).style(Style::default().fg(WHITE)),
        results_area,
    );

    let hint = Paragraph::new(Line::from("[ ↑/↓ ] Navigate   [ Enter ] View   [ Esc ] Back"))
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center);
    frame.render_widget(
        hint,
        Rect { x: 0, y: size.height.saturating_sub(1), width: size.width, height: 1 },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Password generator
// ─────────────────────────────────────────────────────────────────────────────

fn render_password_generator(app: &App, frame: &mut Frame) {
    let size = frame.size();
    let block = Block::default()
        .title(Line::from("  Password Generator  "))
        .borders(Borders::ALL)
        .style(Style::default().fg(WHITE));

    frame.render_widget(block, size);

    let inner = Rect {
        x: 2,
        y: 2,
        width: size.width.saturating_sub(4),
        height: size.height.saturating_sub(4),
    };

    let (length, upper, lower, number, symbol, result) = (
        app.pwgen_length,
        app.pwgen_upper,
        app.pwgen_lower,
        app.pwgen_number,
        app.pwgen_symbol,
        app.pwgen_result.as_deref().unwrap_or("").to_string(),
    );

    let lines: Vec<Line<'_>> = vec![
        Line::from(""),
        Line::from(vec![
            ss("  Length: ".to_string(), DIM),
            ssb(format!("[{}]  ", length), ACCENT),
            ss("[+]".to_string(), DIM),
            ss("  ".to_string(), DIM),
            ss("[-]".to_string(), DIM),
            ss("   (8–64)".to_string(), DIM),
        ]),
        Line::from(""),
        Line::from(vec![
            ss(format!("  [A] "), if upper { SUCCESS } else { DIM }),
            ss("Uppercase  ".to_string(), WHITE),
            ss(format!("[B] "), if lower { SUCCESS } else { DIM }),
            ss("Lowercase".to_string(), WHITE),
        ]),
        Line::from(vec![
            ss(format!("  [N] "), if number { SUCCESS } else { DIM }),
            ss("Numbers    ".to_string(), WHITE),
            ss(format!("[S] "), if symbol { SUCCESS } else { DIM }),
            ss("Symbols".to_string(), WHITE),
        ]),
        Line::from(""),
        Line::from(ssb(format!("  Generated:  {}", result), ACCENT)),
        Line::from(""),
        Line::from(ss("  [ Enter ] Regenerate   [ C ] Copy   [ Esc ] Back".to_string(), DIM)),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

// ─────────────────────────────────────────────────────────────────────────────
// Clipboard notification
// ─────────────────────────────────────────────────────────────────────────────

fn render_clipboard_notif(frame: &mut Frame, msg: &str) {
    let size = frame.size();
    let w = 50.min(size.width.saturating_sub(4));
    let h = 3;
    let x = (size.width.saturating_sub(w)) / 2;
    let y = size.height.saturating_sub(4);

    let box_area = Rect { x, y, width: w, height: h };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SUCCESS)),
        box_area,
    );
    frame.render_widget(
        Paragraph::new(Line::from(msg))
            .style(Style::default().fg(SUCCESS))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        box_area,
    );
}
