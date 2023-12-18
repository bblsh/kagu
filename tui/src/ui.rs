use ratatui::Frame;

use crate::app::{App, Screen};
use crate::screens;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    match app.current_screen {
        Screen::Main => screens::main::render(app, frame),
        Screen::Settings => screens::settings::render(app, frame),
        Screen::Personal => screens::personal::render(app, frame),
    }
}
