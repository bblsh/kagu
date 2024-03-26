use crate::app::App;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{List, Paragraph},
    Frame,
};

pub fn render(app: &mut App, setting_area: Rect, frame: &mut Frame<'_>) {
    let [audio_inputs_label_area, audio_inputs_list_area, spacer_1_area, audio_ouputs_label_area, audio_outputs_list_area] =
        *Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Max(1), // Audio inputs label
                Constraint::Max(3), // Audio inputs
                Constraint::Max(1), // Spacer
                Constraint::Max(1), // Audio outputs label
                Constraint::Max(3), // Audio outputs
            ])
            .split(setting_area)
    else {
        return;
    };

    let inputs_label = Paragraph::new(String::from("Audio Inputs")).style(Style::default().bold());
    let inputs = app.client.get_audio_inputs();
    let inputs_list = List::new(inputs);

    let outputs_label =
        Paragraph::new(String::from("Audio Outputs")).style(Style::default().bold());
    let outputs = app.client.get_audio_outputs();
    let outputs_list = List::new(outputs);

    let spacer_1_paragraph = Paragraph::new(String::from(""));

    frame.render_widget(inputs_label, audio_inputs_label_area);
    frame.render_widget(inputs_list, audio_inputs_list_area);
    frame.render_widget(spacer_1_paragraph, spacer_1_area);
    frame.render_widget(outputs_label, audio_ouputs_label_area);
    frame.render_widget(outputs_list, audio_outputs_list_area);
}
