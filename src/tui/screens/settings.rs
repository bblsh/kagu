use crate::tui::app::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let back_panel = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Max(10),
                Constraint::Max(frame.size().width - 10),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let test_paragraph = Paragraph::new("Hello from the Settings page").block(
        Block::default()
            .border_style(Style::default().bg(Color::Green))
            .borders(Borders::ALL),
    );

    frame.render_widget(test_paragraph, back_panel[1]);
}
