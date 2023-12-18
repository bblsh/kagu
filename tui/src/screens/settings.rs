use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(_app: &mut App, frame: &mut Frame<'_>) {
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
