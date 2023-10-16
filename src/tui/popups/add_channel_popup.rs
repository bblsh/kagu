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
}

impl PopupTraits for AddChannelPopup {
    fn reset(&mut self) {
        self.current_ui_element = AddChannelUiElement::TextOrVoice;
    }
}

impl AddChannelPopup {
    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let back_block = Block::default()
            .title(" Create Channel ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let area = self.fixed_size_middle_popup(28, 16, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(back_block, area);
    }
}
