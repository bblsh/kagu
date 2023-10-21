use tui::{
    backend::Backend,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::popups::popup_traits::PopupTraits;

#[derive(Debug, Default)]
pub struct GeneralPopup {
    pub title: String,
    pub message: String,
}

impl PopupTraits for GeneralPopup {
    fn reset(&mut self) {
        self.title = String::new();
        self.message = String::new();
    }

    fn setup(&mut self, title: Option<String>, message: Option<String>) {
        self.reset();
        self.title = title.unwrap_or(String::from(""));
        self.message = message.unwrap_or(String::from(""));
    }
}

impl GeneralPopup {
    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let mut title = self.title.clone();
        title.push_str(" (Enter to dismiss)");

        let alert_block = Paragraph::new(self.message.clone())
            .block(Block::default().title(title).borders(Borders::ALL));
        let area = self.centered_popup(60, 20, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(alert_block, area);
    }
}
