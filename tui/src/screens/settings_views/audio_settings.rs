use crate::app::App;
use ratatui::{
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders, List},
    Frame,
};

pub fn render(app: &mut App, setting_area: Rect, frame: &mut Frame<'_>) {
    let mut input_text = vec![String::from("Audio Inputs")];
    input_text.extend(app.client.get_audio_inputs());

    let inputs = List::new(input_text).block(
        Block::default()
            .border_style(Style::default())
            .borders(Borders::TOP),
    );
    frame.render_widget(inputs, setting_area);
}
