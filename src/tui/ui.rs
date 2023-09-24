use tui::{backend::Backend, Frame};

use crate::tui::app::{App, Screen};
use crate::tui::screens;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    match app.current_screen {
        Screen::Main => screens::main::render(app, frame),
        Screen::Settings => screens::settings::render(app, frame),
        Screen::Personal => screens::personal::render(app, frame),
    }
}
