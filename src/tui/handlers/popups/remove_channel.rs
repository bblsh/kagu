use crate::tui::{
    app::{App, AppResult},
    popups::remove_channel_popup::RemoveChannelPopupUiElement,
};
use crossterm::event::{KeyCode, KeyEvent};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.dismiss_popup();
        }
        KeyCode::Up => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => (),
            RemoveChannelPopupUiElement::No => {
                app.remove_channel_popup.current_ui_element = RemoveChannelPopupUiElement::Yes
            }
        },
        KeyCode::Down => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => {
                app.remove_channel_popup.current_ui_element = RemoveChannelPopupUiElement::No
            }
            RemoveChannelPopupUiElement::No => (),
        },
        KeyCode::Enter => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => {
                // app.remove_channel
                app.dismiss_popup();
            }
            RemoveChannelPopupUiElement::No => app.dismiss_popup(),
        },
        _ => (),
    };

    Ok(())
}
