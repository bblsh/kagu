use crate::tui::{
    app::{App, AppResult, InputMode, Pane},
    popups::member_popup::{MemberPopupInputMode, MemberPopupUiElement},
};
use crossterm::event::{KeyCode, KeyEvent};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match app.member_popup.input_mode {
        MemberPopupInputMode::Editing => match key_event.code {
            KeyCode::Char(c) => app.member_popup.dm_buffer.push(c),
            KeyCode::Esc => app.member_popup.input_mode = MemberPopupInputMode::Normal,
            KeyCode::Backspace => {
                app.member_popup.dm_buffer.pop();
            }
            KeyCode::Enter => {
                // Slide into their DM here
            }
            _ => (),
        },
        MemberPopupInputMode::Normal => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                // Dismiss the popup
                app.dismiss_popup();

                // Set the current pane to be the Members pane
                app.current_pane = Pane::MembersPane;
                app.input_mode = InputMode::Members;
            }
            KeyCode::Enter => match app.member_popup.current_ui_element {
                MemberPopupUiElement::Dm => {
                    app.member_popup.input_mode = MemberPopupInputMode::Editing;
                }
                _ => (),
            },
            _ => (),
        },
    };

    Ok(())
}
