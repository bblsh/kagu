use tui::style::Color;

pub struct Settings {
    pub text_color: Option<Color>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings { text_color: None }
    }
}
