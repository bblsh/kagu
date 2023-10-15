use crate::tui::app::{KaguFormatting, Pane, Screen};
use crate::{
    realms::realm::ChannelType,
    tui::app::{App, AppResult, InputMode, UiElement},
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui::style::{Color, Style};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    Ok(())
}
