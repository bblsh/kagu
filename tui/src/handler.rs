use crate::app::{App, AppResult};
use crate::app::{PopupType, Screen};
use crate::handlers;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        // Regardless of mode or screen, exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
                return Ok(());
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.hang_up();
                return Ok(());
            }
        }
        _ => (),
    }

    match key_event.code {
        KeyCode::Char('q') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.current_screen = Screen::Main;
                return Ok(());
            }
        }
        KeyCode::Char('s') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.current_screen = Screen::Settings;
                return Ok(());
            }
        }
        KeyCode::Char('p') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.current_screen = Screen::Personal;
                return Ok(());
            }
        }
        _ => (),
    }

    // If there's a popup shown, send input to that popup
    if app.is_popup_shown {
        match app.popup_type {
            PopupType::General => {
                return handlers::popups::general::handle_key_events(key_event, app)
            }
            PopupType::YesNo => return handlers::popups::yes_no::handle_key_events(key_event, app),
            PopupType::AddChannel => {
                return handlers::popups::add_channel::handle_key_events(key_event, app)
            }
            PopupType::RemoveChannel => {
                return handlers::popups::remove_channel::handle_key_events(key_event, app)
            }
            PopupType::Member => {
                return handlers::popups::member::handle_key_events(key_event, app)
            }
            PopupType::AddRealm => {
                return handlers::popups::add_realm::handle_key_events(key_event, app)
            }
            PopupType::RemoveRealm => {
                return handlers::popups::remove_realm::handle_key_events(key_event, app)
            }
        }
    }

    // Send each key event to that screen's handler
    match app.current_screen {
        Screen::Main => handlers::screens::main::handle_key_events(key_event, app).unwrap(),
        Screen::Personal => handlers::screens::personal::handle_key_events(key_event, app).unwrap(),
        Screen::Settings => handlers::screens::settings::handle_key_events(key_event, app).unwrap(),
    }

    Ok(())
}
