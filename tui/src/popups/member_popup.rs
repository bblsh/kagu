use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::KaguFormatting;
use crate::popups::popup_traits::PopupTraits;
use types::UserIdSize;

#[derive(Debug)]
pub enum MemberPopupInputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub enum MemberPopupActionsUiElements {
    AddRemoveFriend,
    Message,
    Call,
    Block,
    Kick,
    Ban,
}

#[derive(Debug)]
pub enum MemberPopupUiElement {
    Actions,
    Dm,
}

#[derive(Debug)]
pub enum MemberPopupUi {
    Info,
    Actions,
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
            current_ui: MemberPopupUi::Info,
            current_actions_ui_element: MemberPopupActionsUiElements::AddRemoveFriend,
            is_friend: false,
            is_request_pending: false,
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
    pub current_ui: MemberPopupUi,
    pub current_actions_ui_element: MemberPopupActionsUiElements,
    pub is_friend: bool,
    pub is_request_pending: bool,
}

impl PopupTraits for MemberPopup {
    fn reset(&mut self) {
        self.selected_index = 0;
        self.user_id = 0;
        self.username = String::new();
        self.input_mode = MemberPopupInputMode::Normal;
        self.current_ui_element = MemberPopupUiElement::Dm;
        self.dm_buffer = String::new();
        self.is_friend = false;
        self.is_request_pending = false;
        self.current_ui = MemberPopupUi::Info;
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

    pub fn render(&self, frame: &mut Frame<'_>) {
        // Clear out our space to draw in
        let cleared_area = self.build_member_popup(frame.size(), self.selected_index);

        let back_block = Block::default()
            .title(
                String::from(match self.current_ui {
                    MemberPopupUi::Info => "Member Info",
                    MemberPopupUi::Actions => "Member Actions",
                })
                .with_pre_post_spaces(),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);

        match self.current_ui {
            MemberPopupUi::Info => self.render_info(inner_content_area, frame),
            MemberPopupUi::Actions => self.render_actions(inner_content_area, frame),
        }
    }

    fn render_info(&self, inner_content_area: Rect, frame: &mut Frame<'_>) {
        let [name_area, id_area, filler_area, dm_area, actions_area] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(2),
                Constraint::Max(3),
                Constraint::Max(1),
            ])
            .margin(0)
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
        let filler_paragraph = Paragraph::new("");
        let dm_paragraph = Paragraph::new(self.dm_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(match self.current_ui_element {
                    MemberPopupUiElement::Dm => {
                        String::from("Send DM").with_focus().with_pre_post_spaces()
                    }
                    _ => String::from("Send DM").with_pre_post_spaces(),
                })
                .border_style(match self.input_mode {
                    MemberPopupInputMode::Normal => Style::default(),
                    MemberPopupInputMode::Editing => Style::default().fg(Color::Yellow),
                }),
        );

        let actions_paragraph = Paragraph::new(match self.current_ui_element {
            MemberPopupUiElement::Actions => String::from("Actions...")
                .with_focus()
                .with_pre_post_spaces(),
            _ => String::from("Actions...").with_pre_post_spaces(),
        });

        frame.render_widget(name_paragraph, name_area);
        frame.render_widget(id_paragraph, id_area);
        frame.render_widget(filler_paragraph, filler_area);
        frame.render_widget(dm_paragraph, dm_area);
        frame.render_widget(actions_paragraph, actions_area);

        if let MemberPopupInputMode::Editing = self.input_mode {
            frame.set_cursor(dm_area.x + self.dm_buffer.width() as u16 + 1, dm_area.y + 1)
        }
    }

    fn render_actions(&self, inner_content_area: Rect, frame: &mut Frame<'_>) {
        let [add_friend_area, message_area, call_area, block_area, kick_area, ban_area] =
            *Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Max(1),
                    Constraint::Max(1),
                ])
                .margin(0)
                .split(inner_content_area)
        else {
            return;
        };

        let mut add_remove_spans: Vec<Span> = Vec::new();
        if self.is_friend {
            add_remove_spans.push(Span::raw(match self.current_actions_ui_element {
                MemberPopupActionsUiElements::AddRemoveFriend => String::from("Remove Friend")
                    .with_focus()
                    .with_pre_post_spaces(),
                _ => String::from("Remove Friend").with_pre_post_spaces(),
            }));
        } else if self.is_request_pending {
            add_remove_spans.push(Span::styled(
                match self.current_actions_ui_element {
                    MemberPopupActionsUiElements::AddRemoveFriend => {
                        String::from("Request Pending").with_focus()
                    }
                    _ => String::from("Request Pending").with_pre_post_spaces(),
                },
                Style::default().fg(Color::Gray),
            ));
        } else {
            add_remove_spans.push(Span::raw(match self.current_actions_ui_element {
                MemberPopupActionsUiElements::AddRemoveFriend => String::from("Add Friend")
                    .with_focus()
                    .with_pre_post_spaces(),
                _ => String::from("Add Friend").with_pre_post_spaces(),
            }));
        }

        let add_remove_friend_paragraph = Paragraph::new(Line::from(add_remove_spans));

        let message_paragraph = Paragraph::new(match self.current_actions_ui_element {
            MemberPopupActionsUiElements::Message => {
                String::from("Message").with_focus().with_pre_post_spaces()
            }
            _ => String::from("Message").with_pre_post_spaces(),
        });

        let call_paragraph = Paragraph::new(match self.current_actions_ui_element {
            MemberPopupActionsUiElements::Call => {
                String::from("Call").with_focus().with_pre_post_spaces()
            }
            _ => String::from("Call").with_pre_post_spaces(),
        });

        let block_paragraph = Paragraph::new(match self.current_actions_ui_element {
            MemberPopupActionsUiElements::Block => {
                vec![Line::from(vec![
                    Span::styled(
                        String::from("Block").with_focus().with_pre_post_spaces(),
                        Style::default().fg(Color::Gray),
                    ),
                    // Span::styled("", Style::default()),
                ])]
            }
            _ => vec![Line::from(vec![Span::styled(
                String::from("Block").with_pre_post_spaces(),
                Style::default().fg(Color::Gray),
            )])],
        });

        let kick_paragraph = Paragraph::new(match self.current_actions_ui_element {
            MemberPopupActionsUiElements::Kick => {
                vec![Line::from(vec![Span::styled(
                    String::from("Kick").with_focus().with_pre_post_spaces(),
                    Style::default().fg(Color::Gray),
                )])]
            }
            _ => vec![Line::from(vec![Span::styled(
                String::from("Kick").with_pre_post_spaces(),
                Style::default().fg(Color::Gray),
            )])],
        });

        let ban_paragraph = Paragraph::new(match self.current_actions_ui_element {
            MemberPopupActionsUiElements::Ban => {
                vec![Line::from(vec![Span::styled(
                    String::from("Ban").with_focus().with_pre_post_spaces(),
                    Style::default().fg(Color::Gray),
                )])]
            }
            _ => vec![Line::from(vec![Span::styled(
                String::from("Ban").with_pre_post_spaces(),
                Style::default().fg(Color::Gray),
            )])],
        });

        frame.render_widget(add_remove_friend_paragraph, add_friend_area);
        frame.render_widget(message_paragraph, message_area);
        frame.render_widget(call_paragraph, call_area);
        frame.render_widget(block_paragraph, block_area);
        frame.render_widget(kick_paragraph, kick_area);
        frame.render_widget(ban_paragraph, ban_area);
    }
}
