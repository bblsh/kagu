use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::tui::app::KaguFormatting;
use crate::tui::popups::popup_traits::PopupTraits;
use crate::types::RealmIdSize;

use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct RemoveRealmPopup {
    pub title: String,
    pub message: String,
    pub realm_name: String,
    pub realm_id: RealmIdSize,
    pub confirm_buffer: String,
    pub current_ui_element: RemoveRealmPopupUiElement,
    pub input_mode: RemoveRealmPopupInputMode,
}

#[derive(Debug)]
pub enum RemoveRealmPopupUiElement {
    Yes,
    No,
}

#[derive(Debug)]
pub enum RemoveRealmPopupInputMode {
    Normal,
    Editing,
}

impl PopupTraits for RemoveRealmPopup {
    fn reset(&mut self) {
        self.title = String::new();
        self.message = String::new();
        self.realm_name = String::new();
        self.confirm_buffer = String::new();
        self.current_ui_element = RemoveRealmPopupUiElement::No;
        self.realm_id = 0;
        self.input_mode = RemoveRealmPopupInputMode::Normal;
    }

    fn setup(&mut self, title: Option<String>, message: Option<String>) {
        self.reset();
        self.title = title.unwrap_or(String::from("Remove Realm"));
        self.message = message.unwrap_or(String::from("Remove realm?"));
    }
}

impl Default for RemoveRealmPopup {
    fn default() -> Self {
        RemoveRealmPopup {
            title: String::new(),
            message: String::new(),
            realm_name: String::new(),
            realm_id: 0,
            confirm_buffer: String::new(),
            current_ui_element: RemoveRealmPopupUiElement::No,
            input_mode: RemoveRealmPopupInputMode::Normal,
        }
    }
}

impl RemoveRealmPopup {
    pub fn render(&self, frame: &mut Frame<'_>) {
        // Clear out our space to draw in
        let cleared_area = self.fixed_size_middle_popup(35, 10, frame.size());

        let back_block = Block::default()
            .title(String::from("Remove Realm").with_pre_post_spaces())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        let [message_area, _filler, confirm_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(1), Constraint::Max(1), Constraint::Max(3)])
            .margin(1)
            .split(inner_content_area)
        else {
            return;
        };

        // Display "Remove [name]?"
        let mut message = String::from("Remove ");
        message.push_str(self.realm_name.as_str());
        message.push('?');

        let confirm_paragraph = Paragraph::new(self.confirm_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(
                    String::from("Type realm name to Confirm")
                        .with_focus()
                        .with_pre_post_spaces(),
                )
                .border_style(match self.input_mode {
                    RemoveRealmPopupInputMode::Normal => Style::default(),
                    RemoveRealmPopupInputMode::Editing => {
                        if self.confirm_buffer == self.realm_name {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().fg(Color::Red)
                        }
                    }
                }),
        );

        let message_paragraph = Paragraph::new(message);

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);
        frame.render_widget(message_paragraph, message_area);
        frame.render_widget(confirm_paragraph, confirm_area);

        if let RemoveRealmPopupInputMode::Editing = self.input_mode {
            frame.set_cursor(
                confirm_area.x + self.confirm_buffer.width() as u16 + 1,
                confirm_area.y + 1,
            )
        }
    }
}
