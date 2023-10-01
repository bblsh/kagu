use crate::tui::app::Screen;
use crate::tui::app::{App, AppResult};
use crate::tui::handlers;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
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

    // Send each key event to that screen's handler
    match app.current_screen {
        Screen::Main => handlers::main::handle_key_events(key_event, app)
            .await
            .unwrap(),
        Screen::Personal => handlers::personal::handle_key_events(key_event, app)
            .await
            .unwrap(),
        Screen::Settings => handlers::settings::handle_key_events(key_event, app)
            .await
            .unwrap(),
    }

    Ok(())
}
