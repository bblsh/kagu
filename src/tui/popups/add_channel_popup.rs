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

#[derive(Debug, Default)]
pub enum AddChannelUiElement {
    #[default]
    TextOption,
    VoiceOption,
    ChannelName,
}

#[derive(Debug)]
pub enum AddChannelInputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub struct AddChannelPopup {
    pub current_ui_element: AddChannelUiElement,
    pub is_text_channel: bool, // Voice channel if false
    pub channel_name_buffer: String,
    pub input_mode: AddChannelInputMode,
}

impl Default for AddChannelPopup {
    fn default() -> Self {
        AddChannelPopup {
            current_ui_element: AddChannelUiElement::TextOption,
            is_text_channel: true,
            channel_name_buffer: String::new(),
            input_mode: AddChannelInputMode::Normal,
        }
    }
}

impl PopupTraits for AddChannelPopup {
    fn reset(&mut self) {
        self.current_ui_element = AddChannelUiElement::TextOption;
        self.is_text_channel = true;
        self.channel_name_buffer = String::new();
        self.input_mode = AddChannelInputMode::Normal;
    }
}

impl AddChannelPopup {
    pub fn reset_popup(&mut self) {
        self.reset();
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        // Clear out our space to
        let cleared_area = self.fixed_size_middle_popup(28, 10, frame.size());

        let back_block = Block::default()
            .title(String::from("Create Channel").with_pre_post_spaces())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        let [is_text, is_voice, _filler, channel_name] = *Layout::default()
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

        let mut text_status = match self.is_text_channel {
            true => vec![Line::from(vec![
                Span::styled(
                    "X",
                    Style::add_modifier(Style::default(), Modifier::UNDERLINED),
                ),
                Span::styled(" Text", Style::default()),
            ])],
            false => vec![Line::from(vec![
                Span::styled(
                    " ",
                    Style::add_modifier(Style::default(), Modifier::UNDERLINED),
                ),
                Span::styled(" Text", Style::default()),
            ])],
        };

        let mut voice_status = match self.is_text_channel {
            true => vec![Line::from(vec![
                Span::styled(
                    " ",
                    Style::add_modifier(Style::default(), Modifier::UNDERLINED),
                ),
                Span::styled(" Voice", Style::default()),
            ])],
            false => vec![Line::from(vec![
                Span::styled(
                    "X",
                    Style::add_modifier(Style::default(), Modifier::UNDERLINED),
                ),
                Span::styled(" Voice", Style::default()),
            ])],
        };

        match self.current_ui_element {
            AddChannelUiElement::TextOption => {
                text_status[0]
                    .spans
                    .insert(0, Span::styled(">", Style::default()));
                voice_status[0]
                    .spans
                    .insert(0, Span::styled(" ", Style::default()));
            }
            AddChannelUiElement::VoiceOption => {
                text_status[0]
                    .spans
                    .insert(0, Span::styled(" ", Style::default()));
                voice_status[0]
                    .spans
                    .insert(0, Span::styled(">", Style::default()));
            }
            AddChannelUiElement::ChannelName => {
                text_status[0]
                    .spans
                    .insert(0, Span::styled(" ", Style::default()));
                voice_status[0]
                    .spans
                    .insert(0, Span::styled(" ", Style::default()));
            }
        }

        let text_channel_selection = Paragraph::new(text_status);
        let voice_channel_selection = Paragraph::new(voice_status);
        let channel_name_paragraph = Paragraph::new(self.channel_name_buffer.clone()).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(match self.current_ui_element {
                    AddChannelUiElement::ChannelName => String::from("Channel Name")
                        .with_focus()
                        .with_pre_post_spaces(),
                    _ => String::from("Channel Name").with_pre_post_spaces(),
                })
                .border_style(match self.input_mode {
                    AddChannelInputMode::Normal => Style::default(),
                    AddChannelInputMode::Editing => Style::default().fg(Color::Yellow),
                }),
        );

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);
        frame.render_widget(text_channel_selection, is_text);
        frame.render_widget(voice_channel_selection, is_voice);
        frame.render_widget(channel_name_paragraph, channel_name);

        if let AddChannelInputMode::Editing = self.input_mode {
            frame.set_cursor(
                channel_name.x + self.channel_name_buffer.width() as u16 + 1,
                channel_name.y + 1,
            )
        }
    }
}
