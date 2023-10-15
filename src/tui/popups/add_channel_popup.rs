use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, InputMode, KaguFormatting, Pane, PopupType, UiElement};

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

impl AddChannelPopup {
    pub fn render<B: Backend>(frame: &mut Frame<'_, B>) {}
}
