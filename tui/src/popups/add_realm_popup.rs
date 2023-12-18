use crate::app::KaguFormatting;
use crate::popups::popup_traits::PopupTraits;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub enum AddRealmUiElement {
    #[default]
    RealmName,
    Invite,
}

#[derive(Debug)]
pub enum AddRealmInputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub struct AddRealmPopup {
    pub current_ui_element: AddRealmUiElement,
    pub invite_buffer: String,
    pub realm_name_buffer: String,
    pub input_mode: AddRealmInputMode,
}

impl Default for AddRealmPopup {
    fn default() -> Self {
        AddRealmPopup {
            current_ui_element: AddRealmUiElement::RealmName,
            invite_buffer: String::new(),
            realm_name_buffer: String::new(),
            input_mode: AddRealmInputMode::Normal,
        }
    }
}

impl PopupTraits for AddRealmPopup {
    fn reset(&mut self) {
        self.current_ui_element = AddRealmUiElement::RealmName;
        self.invite_buffer = String::new();
        self.realm_name_buffer = String::new();
        self.input_mode = AddRealmInputMode::Normal;
    }

    fn setup(&mut self, _title: Option<String>, _message: Option<String>) {
        self.reset();
    }
}

impl AddRealmPopup {
    pub fn render(&self, frame: &mut Frame<'_>) {
        // Clear out our space to draw in
        let cleared_area = self.fixed_size_middle_popup(28, 11, frame.size());

        let back_block = Block::default()
            .title(String::from("Create Realm").with_pre_post_spaces())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        let [realm_name_area, filler_area, invite_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(3), Constraint::Max(1), Constraint::Max(3)])
            .margin(1)
            .split(inner_content_area)
        else {
            return;
        };

        let realm_name_paragraph = Paragraph::new(self.realm_name_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(match self.current_ui_element {
                    AddRealmUiElement::RealmName => String::from("Realm Name")
                        .with_focus()
                        .with_pre_post_spaces(),
                    _ => String::from("Realm Name").with_pre_post_spaces(),
                })
                .border_style(match self.input_mode {
                    AddRealmInputMode::Normal => Style::default(),
                    AddRealmInputMode::Editing => match self.current_ui_element {
                        AddRealmUiElement::RealmName => Style::default().fg(Color::Yellow),
                        AddRealmUiElement::Invite => Style::default(),
                    },
                }),
        );

        let invite_paragraph = Paragraph::new(self.invite_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(match self.current_ui_element {
                    AddRealmUiElement::Invite => String::from("Invite Code")
                        .with_focus()
                        .with_pre_post_spaces(),
                    _ => String::from("Invite Code").with_pre_post_spaces(),
                })
                .border_style(match self.input_mode {
                    AddRealmInputMode::Normal => Style::default(),
                    AddRealmInputMode::Editing => match self.current_ui_element {
                        AddRealmUiElement::RealmName => Style::default(),
                        AddRealmUiElement::Invite => Style::default().fg(Color::Yellow),
                    },
                }),
        );

        let filler_paragraph = Paragraph::new(String::from(""));

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);
        frame.render_widget(realm_name_paragraph, realm_name_area);
        frame.render_widget(filler_paragraph, filler_area);
        frame.render_widget(invite_paragraph, invite_area);

        if let AddRealmInputMode::Editing = self.input_mode {
            match self.current_ui_element {
                AddRealmUiElement::RealmName => frame.set_cursor(
                    realm_name_area.x + self.realm_name_buffer.width() as u16 + 1,
                    realm_name_area.y + 1,
                ),
                AddRealmUiElement::Invite => frame.set_cursor(
                    invite_area.x + self.invite_buffer.width() as u16 + 1,
                    invite_area.y + 1,
                ),
            }
        }
    }
}
