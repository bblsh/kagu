use crate::tui::app::{App, AppResult};
use crossterm::event::KeyEvent;

pub async fn handle_key_events(_key_event: KeyEvent, _app: &mut App<'_>) -> AppResult<()> {
    Ok(())
}
