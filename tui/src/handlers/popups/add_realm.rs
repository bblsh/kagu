use crate::{
    app::{App, AppResult, InputMode, Pane},
    popups::add_realm_popup::{AddRealmInputMode, AddRealmUiElement},
};
use crossterm::event::{KeyCode, KeyEvent};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match app.add_realm_popup.input_mode {
        AddRealmInputMode::Normal => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                app.dismiss_popup();
                app.input_mode = InputMode::Realms;
                app.current_pane = Pane::RealmsPane;
            }
            KeyCode::Up => match app.add_realm_popup.current_ui_element {
                AddRealmUiElement::RealmName => (),
                AddRealmUiElement::Invite => {
                    app.add_realm_popup.current_ui_element = AddRealmUiElement::RealmName;
                }
            },
            KeyCode::Down => match app.add_realm_popup.current_ui_element {
                AddRealmUiElement::RealmName => {
                    app.add_realm_popup.current_ui_element = AddRealmUiElement::Invite;
                }
                AddRealmUiElement::Invite => (),
            },
            KeyCode::Enter => app.add_realm_popup.input_mode = AddRealmInputMode::Editing,
            _ => (),
        },
        AddRealmInputMode::Editing => match key_event.code {
            KeyCode::Char(c) => match app.add_realm_popup.current_ui_element {
                AddRealmUiElement::RealmName => app.add_realm_popup.realm_name_buffer.push(c),
                AddRealmUiElement::Invite => app.add_realm_popup.invite_buffer.push(c),
            },
            KeyCode::Esc => app.add_realm_popup.input_mode = AddRealmInputMode::Normal,
            KeyCode::Backspace => match app.add_realm_popup.current_ui_element {
                AddRealmUiElement::RealmName => {
                    app.add_realm_popup.realm_name_buffer.pop();
                }
                AddRealmUiElement::Invite => {
                    app.add_realm_popup.invite_buffer.pop();
                }
            },
            KeyCode::Enter => {
                match app.add_realm_popup.current_ui_element {
                    AddRealmUiElement::RealmName => {
                        app.add_realm(app.add_realm_popup.realm_name_buffer.clone())
                            .await;
                    }
                    AddRealmUiElement::Invite => {
                        // Join by invite code
                    }
                };
                app.dismiss_popup();
                app.realms.unselect();
                app.input_mode = InputMode::Normal;
                app.current_pane = Pane::RealmsPane;
            }
            _ => (),
        },
    };

    Ok(())
}
