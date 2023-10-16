use crate::tui::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        //
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            // First check to see if we're entering text

            app.dismiss_popup();
        }
        KeyCode::Char(' ') => {
            // Select element?
        }
        KeyCode::Enter => {
            // Select or confirm?
        }
        _ => (),
    }

    Ok(())
}
