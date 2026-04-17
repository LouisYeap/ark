//! Keyboard input handling.

use crossterm::event::KeyEvent;

/// Disambiguation-free input enum for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppInput {
    Enter,
    Esc,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Char(char),
}

impl From<KeyEvent> for AppInput {
    fn from(event: KeyEvent) -> Self {
        use crossterm::event::KeyCode;
        match event.code {
            KeyCode::Enter | KeyCode::Null => AppInput::Enter,
            KeyCode::Esc => AppInput::Esc,
            KeyCode::Backspace => AppInput::Backspace,
            KeyCode::Tab => AppInput::Tab,
            KeyCode::Up | KeyCode::PageUp => AppInput::Up,
            KeyCode::Down | KeyCode::PageDown => AppInput::Down,
            KeyCode::Left => AppInput::Left,
            KeyCode::Right => AppInput::Right,
            KeyCode::Char(c) => AppInput::Char(c),
            _ => AppInput::Esc,
        }
    }
}

/// Read one key event from the terminal, blocking.
pub fn read_input() -> AppInput {
    use crossterm::event::read;
    let event = read().expect("failed to read terminal event");
    if let crossterm::event::Event::Key(key) = event {
        AppInput::from(key)
    } else {
        AppInput::Esc
    }
}
