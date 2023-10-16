use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, InputMode, KaguFormatting, Pane, PopupType, UiElement};

use crate::tui::popups::popup_traits::PopupTraits;

#[derive(Debug, Default)]
pub enum AddChannelUiElement {
    #[default]
    TextOrVoice,
    ChannelName,
}

#[derive(Debug, Default)]
pub struct AddChannelPopup {
    pub current_ui_element: AddChannelUiElement,
    is_text_channel: bool, // Voice channel if false
    channel_name_buffer: String,
}

impl PopupTraits for AddChannelPopup {
    fn reset(&mut self) {
        self.current_ui_element = AddChannelUiElement::TextOrVoice;
        self.is_text_channel = true;
        self.channel_name_buffer = String::new();
    }
}

impl AddChannelPopup {
    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        // Clear out our space to
        let cleared_area = self.fixed_size_middle_popup(28, 10, frame.size());

        let back_block = Block::default()
            .title(" Create Channel ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let inner_content_area = back_block.inner(cleared_area);

        let [is_text, is_voice, channel_name] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(1), Constraint::Max(1), Constraint::Max(3)])
            .margin(1)
            .split(inner_content_area)
        else {
            return;
        };

        let text_channel_selection = Paragraph::new("__ text");
        let voice_channel_selection = Paragraph::new("__ voice");
        let channel_name_paragraph = Paragraph::new("NAME GOES HERE!!!").block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("Channel Name"),
        );

        frame.render_widget(Clear, cleared_area);
        frame.render_widget(back_block, cleared_area);
        frame.render_widget(text_channel_selection, is_text);
        frame.render_widget(voice_channel_selection, is_voice);
        frame.render_widget(channel_name_paragraph, channel_name);
    }
}
