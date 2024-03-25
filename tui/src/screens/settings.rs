use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    let [settings_list_area, settings_view_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(10),
            Constraint::Max(frame.size().width - 10),
        ])
        .split(frame.size())
    else {
        return;
    };

    let settings_categories = vec![String::from("Audio")];
    let settings_list = List::new(settings_categories);
    frame.render_widget(settings_list, settings_list_area);

    let mut input_text = vec![String::from("Audio Inputs")];
    input_text.extend(app.client.get_audio_inputs());

    let inputs = List::new(input_text).block(
        Block::default()
            .border_style(Style::default().bg(Color::Green))
            .borders(Borders::ALL),
    );

    frame.render_widget(inputs, settings_view_area);
}
