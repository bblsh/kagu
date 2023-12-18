use crate::{
    app::{App, AppResult},
    popups::yes_no_popup::YesNoPopupUiElement,
};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.dismiss_popup();
        }
        KeyCode::Up => match app.yes_no_popup.current_ui_element {
            YesNoPopupUiElement::Yes => (),
            YesNoPopupUiElement::No => {
                app.yes_no_popup.current_ui_element = YesNoPopupUiElement::Yes
            }
        },
        KeyCode::Down => match app.yes_no_popup.current_ui_element {
            YesNoPopupUiElement::Yes => {
                app.yes_no_popup.current_ui_element = YesNoPopupUiElement::No
            }
            YesNoPopupUiElement::No => (),
        },
        KeyCode::Enter => {}
        _ => (),
    };

    Ok(())
}
