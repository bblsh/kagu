use crate::app::Pane;
use crate::{
    app::{App, AppResult, InputMode},
    popups::remove_realm_popup::RemoveRealmPopupInputMode,
};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match app.remove_realm_popup.input_mode {
        RemoveRealmPopupInputMode::Normal => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                app.dismiss_popup();
                app.input_mode = InputMode::Realms;
                app.current_pane = Pane::RealmsPane;
            }
            KeyCode::Enter => {
                app.remove_realm_popup.input_mode = RemoveRealmPopupInputMode::Editing
            }
            _ => (),
        },
        RemoveRealmPopupInputMode::Editing => match key_event.code {
            KeyCode::Char(c) => app.remove_realm_popup.confirm_buffer.push(c),
            KeyCode::Esc => app.remove_realm_popup.input_mode = RemoveRealmPopupInputMode::Normal,
            KeyCode::Enter => {
                if app.remove_realm_popup.confirm_buffer == app.remove_realm_popup.realm_name {
                    app.remove_realm(app.remove_realm_popup.realm_id);
                    app.dismiss_popup();
                    app.input_mode = InputMode::Realms;
                    app.current_pane = Pane::RealmsPane;
                }
            }
            KeyCode::Backspace => {
                app.remove_realm_popup.confirm_buffer.pop();
            }
            _ => (),
        },
    }

    Ok(())
}
