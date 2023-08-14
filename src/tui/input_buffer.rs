use crate::types::UserIdSize;
use tui::style::Style;
use tui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct InputBuffer {
    pub input: Vec<(String, Style, Option<UserIdSize>)>,
    pub is_mentioning: bool,
    pub is_commanding: bool,
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            input: vec![(String::new(), Style::default(), None)],
            is_mentioning: false,
            is_commanding: false,
        }
    }

    pub fn get_input_line(&self) -> Line<'_> {
        let mut spans = Vec::new();

        for span in &self.input {
            spans.push(Span::styled(span.0.clone(), span.1));
        }

        Line::from(spans)
    }

    pub fn get_input_width(&self) -> u16 {
        let mut length = 0;

        for span in &self.input {
            length = length + span.0.width() as u16;
        }

        length
    }

    pub fn get_input_string(&self) -> String {
        let mut input = String::new();

        for s in &self.input {
            input.push_str(s.0.as_str());
        }

        input
    }

    pub fn get_input_without_style(&self) -> Vec<(String, Option<UserIdSize>)> {
        //: Vec<(&String, Option<&UserIdSize>)> = Vec::new();
        let input: Vec<(String, Option<UserIdSize>)> = self
            .input
            .iter()
            .map(|message| (message.0.clone(), message.2))
            .collect();

        input
    }

    pub fn push(&mut self, text: String, style: Style, user_id: Option<UserIdSize>) {
        self.input.push((text, style, user_id));
    }
}
