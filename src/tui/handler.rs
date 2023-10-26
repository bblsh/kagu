use crate::tui::app::{App, AppResult};
use crate::tui::app::{PopupType, Screen};
use crate::tui::handlers;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        // Regardless of mode or screen, exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit().await;
                return Ok(());
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.hang_up().await;
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
                return handlers::popups::add_channel::handle_key_events(key_event, app).await
            }
            PopupType::RemoveChannel => {
                return handlers::popups::remove_channel::handle_key_events(key_event, app).await
            }
            PopupType::Member => {
                return handlers::popups::member::handle_key_events(key_event, app).await
            }
            PopupType::AddRealm => {
                return handlers::popups::add_realm::handle_key_events(key_event, app).await
            }
            PopupType::RemoveRealm => {
                return handlers::popups::remove_realm::handle_key_events(key_event, app).await
            }
        }
    }

    // Send each key event to that screen's handler
    match app.current_screen {
        Screen::Main => handlers::screens::main::handle_key_events(key_event, app)
            .await
            .unwrap(),
        Screen::Personal => handlers::screens::personal::handle_key_events(key_event, app)
            .await
            .unwrap(),
        Screen::Settings => handlers::screens::settings::handle_key_events(key_event, app)
            .await
            .unwrap(),
    }

    Ok(())
}
