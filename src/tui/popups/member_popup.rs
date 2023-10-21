use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::tui::app::{App, InputMode, KaguFormatting, Pane, PopupType, UiElement};

use crate::tui::popups::popup_traits::PopupTraits;
use crate::types::UserIdSize;

#[derive(Debug)]
pub enum MemberPopupInputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub enum MemberPopupUiElement {
    Dm,
}

impl Default for MemberPopup {
    fn default() -> Self {
        MemberPopup {
            selected_index: 0,
            user_id: 0,
            username: String::new(),
            input_mode: MemberPopupInputMode::Normal,
            dm_buffer: String::new(),
            current_ui_element: MemberPopupUiElement::Dm,
        }
    }
}

#[derive(Debug)]
pub struct MemberPopup {
    pub selected_index: usize,
    pub user_id: UserIdSize,
    pub username: String,
    pub input_mode: MemberPopupInputMode,
    pub current_ui_element: MemberPopupUiElement,
    pub dm_buffer: String,
}

impl PopupTraits for MemberPopup {
    fn reset(&mut self) {
        self.selected_index = 0;
        self.user_id = 0;
        self.username = String::new();
        self.input_mode = MemberPopupInputMode::Normal;
        self.current_ui_element = MemberPopupUiElement::Dm;
        self.dm_buffer = String::new();
    }

    fn setup(&mut self, _title: Option<String>, _message: Option<String>) {
        self.reset();
    }
}

impl MemberPopup {
    fn build_member_popup(&self, r: Rect, selected_index: usize) -> Rect {
        let member_popup = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Max(3 + selected_index as u16),
                    Constraint::Max(10),
                    Constraint::Max(r.height - 3 + selected_index as u16 - 10),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Max(r.width - 20 - 10),
                    Constraint::Max(20),
                    Constraint::Max(10),
                ]
                .as_ref(),
            )
            .split(member_popup[1])[1]
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        // Clear out our space to draw in
        //let cleared_area = self.fixed_size_middle_popup(28, 10, frame.size());
        let cleared_area = self.build_member_popup(frame.size(), self.selected_index);

        let back_block = Block::default()
            .title(String::from("Member Info").with_pre_post_spaces())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        let [name_area, id_area, _filler_area, dm_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(3),
            ])
            .margin(1)
            .split(inner_content_area)
        else {
            return;
        };

        let name = vec![Line::from(vec![
            Span::styled(
                self.username.clone(),
                Style::add_modifier(Style::default(), Modifier::BOLD),
            ),
            // Span::styled("", Style::default()),
        ])];

        let id = vec![Line::from(vec![
            Span::styled(self.user_id.to_string().add_hashtag(), Style::default()),
            // Span::styled("", Style::default()),
        ])];

        let name_paragraph = Paragraph::new(name);
        let id_paragraph = Paragraph::new(id);
        let dm_paragraph = Paragraph::new(self.dm_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(match self.current_ui_element {
                    MemberPopupUiElement::Dm => {
                        String::from("Send DM").with_focus().with_pre_post_spaces()
                    }
                })
                .border_style(match self.input_mode {
                    MemberPopupInputMode::Normal => Style::default(),
                    MemberPopupInputMode::Editing => Style::default().fg(Color::Yellow),
                }),
        );

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);
        frame.render_widget(name_paragraph, name_area);
        frame.render_widget(id_paragraph, id_area);
        frame.render_widget(dm_paragraph, dm_area);

        if let MemberPopupInputMode::Editing = self.input_mode {
            frame.set_cursor(dm_area.x + self.dm_buffer.width() as u16 + 1, dm_area.y + 1)
        }
    }
}
