use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    if key_event.code == KeyCode::Enter {
        app.dismiss_popup();
    }

    Ok(())
}
