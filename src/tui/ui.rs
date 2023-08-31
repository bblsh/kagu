use tui::{backend::Backend, Frame};

use crate::tui::app::{App, Screen};
use crate::tui::screens;

// static RUSTCORD_LOGO: &str = r#"   ___     _   _     ___     _____     ___      ___      ___      ___
// | _ \   | | | |   / __|   |_   _|   / __|    / _ \    | _ \    |   \
// |   /   | |_| |   \__ \     | |    | (__    | (_) |   |   /    | |) |
// |_|_\    \___/    |___/    _|_|_    \___|    \___/    |_|_\    |___/
//  |"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""|
//  `-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "#;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    match app.current_screen {
        Screen::Main => screens::main::render(app, frame),
        Screen::Settings => screens::settings::render(app, frame),
        _ => screens::main::render(app, frame),
    }
}
