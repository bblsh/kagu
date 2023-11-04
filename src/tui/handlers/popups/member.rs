use crate::tui::{
    app::{App, AppResult, InputMode, Pane},
    popups::member_popup::{
        MemberPopupActionsUiElements, MemberPopupInputMode, MemberPopupUi, MemberPopupUiElement,
    },
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
        MemberPopupInputMode::Normal => match app.member_popup.current_ui {
            MemberPopupUi::Info => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    // Dismiss the popup
                    app.dismiss_popup();

                    // Set the current pane to be the Members pane
                    app.current_pane = Pane::MembersPane;
                    app.input_mode = InputMode::Members;
                }
                KeyCode::Up => match app.member_popup.current_ui_element {
                    MemberPopupUiElement::Dm => (),
                    MemberPopupUiElement::Actions => {
                        app.member_popup.current_ui_element = MemberPopupUiElement::Dm
                    }
                },
                KeyCode::Down => match app.member_popup.current_ui_element {
                    MemberPopupUiElement::Dm => {
                        app.member_popup.current_ui_element = MemberPopupUiElement::Actions
                    }
                    MemberPopupUiElement::Actions => (),
                },
                KeyCode::Enter => match app.member_popup.current_ui_element {
                    MemberPopupUiElement::Dm => {
                        app.member_popup.input_mode = MemberPopupInputMode::Editing;
                    }
                    MemberPopupUiElement::Actions => {
                        app.member_popup.current_ui = MemberPopupUi::Actions;
                    }
                },
                _ => (),
            },
            MemberPopupUi::Actions => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    // Change back to the Member Info UI
                    app.member_popup.current_ui = MemberPopupUi::Info;
                }
                KeyCode::Up => match app.member_popup.current_actions_ui_element {
                    MemberPopupActionsUiElements::AddFriend => (),
                    MemberPopupActionsUiElements::Message => {
                        app.member_popup.current_actions_ui_element =
                            MemberPopupActionsUiElements::AddFriend
                    }
                    MemberPopupActionsUiElements::Call => {
                        app.member_popup.current_actions_ui_element =
                            MemberPopupActionsUiElements::Message
                    }
                    _ => (), // Ignore others for now until those messages are supported
                },
                KeyCode::Down => match app.member_popup.current_actions_ui_element {
                    MemberPopupActionsUiElements::AddFriend => {
                        app.member_popup.current_actions_ui_element =
                            MemberPopupActionsUiElements::Message
                    }
                    MemberPopupActionsUiElements::Message => {
                        app.member_popup.current_actions_ui_element =
                            MemberPopupActionsUiElements::Call
                    }
                    MemberPopupActionsUiElements::Call => (),
                    _ => (), // Ignore others for now until those messages are supported
                },
                KeyCode::Enter => match app.member_popup.current_actions_ui_element {
                    MemberPopupActionsUiElements::AddFriend => {
                        app.add_friend(app.member_popup.user_id).await;
                    }
                    MemberPopupActionsUiElements::Message => (),
                    MemberPopupActionsUiElements::Call => (),
                    _ => (), // Ignore others for now until those messages are supported
                },
                _ => (),
            },
        },
    };

    Ok(())
}
