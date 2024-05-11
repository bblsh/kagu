use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        KeyCode::Down => app.settings_category_list.next(),
        KeyCode::Up => app.settings_category_list.previous(),
        KeyCode::Enter => {
            let selected_category_index = app.settings_category_list.state.selected().unwrap();
            app.current_settings_category =
                app.settings_category_list.items[selected_category_index];
        }
        _ => (),
    }
    Ok(())
}
