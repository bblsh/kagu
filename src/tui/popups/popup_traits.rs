use tui::prelude::*;

pub trait PopupTraits {
    fn reset(&mut self);

    fn centered_popup(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    fn fixed_size_middle_popup(&self, width: u16, height: u16, r: Rect) -> Rect {
        let empty_height_space: u16 = (r.height - height) / 2;
        let empty_width_space: u16 = (r.width - width) / 2;

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Max(empty_height_space),
                    Constraint::Max(height),
                    Constraint::Max(empty_height_space),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Max(empty_width_space),
                    Constraint::Max(width),
                    Constraint::Max(empty_width_space),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}
