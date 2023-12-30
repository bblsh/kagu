use crate::app::{App, KaguFormatting, Pane};
use ratatui::prelude::*;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    let [kagu_bar_area, bottom_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(1), Constraint::Max(frame.size().width - 1)])
        .split(frame.size())
    else {
        return;
    };

    let [kagu_logo_area, kagu_voice_status_area, _kagu_spacer_area, kagu_time_area] =
        *Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([
                Constraint::Max(10),                                // Kagu logo
                Constraint::Max(20),                                // Voice status
                Constraint::Max(frame.size().width - 10 - 20 - 15), // Blank space
                Constraint::Max(15),                                // Current time
            ])
            .split(kagu_bar_area)
    else {
        return;
    };

    let kagu_logo = Paragraph::new("Kagu");
    let time = Paragraph::new(app.get_current_time_string()).alignment(Alignment::Right);
    let connected_label = Paragraph::new(match app.is_voice_connected {
        true => Span::styled("Voice connected", Style::default().fg(Color::LightGreen)),
        false => Span::styled("Voice off", Style::default()),
    });
    frame.render_widget(kagu_logo, kagu_logo_area);
    frame.render_widget(connected_label, kagu_voice_status_area);
    frame.render_widget(time, kagu_time_area);

    let [realms_area, middle_panel_area, right_panel_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(10),
            Constraint::Max(20),
            Constraint::Max(frame.size().width - 10 - 20),
        ])
        .split(bottom_area)
    else {
        return;
    };

    let realms_list: Vec<ListItem> = app
        .realms
        .items
        .iter()
        .map(|i| ListItem::new(i.1.clone()).style(Style::default().fg(Color::LightBlue)))
        .collect();
    let realms = List::new(realms_list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.current_pane {
                    Pane::RealmsPane => Pane::to_str(&app.current_pane).with_focus(),
                    _ => Pane::to_str(&Pane::RealmsPane),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    frame.render_stateful_widget(realms, realms_area, &mut app.realms.state);

    let [friends_btn_area, dm_list_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(3), Constraint::Max(frame.size().height - 3)])
        .split(middle_panel_area)
    else {
        return;
    };

    let [friends_name_area, dm_history_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(3), Constraint::Max(frame.size().height - 3)])
        .split(right_panel_area)
    else {
        return;
    };

    let test_top_bar =
        Paragraph::new("Friends or friend_name").block(Block::default().borders(Borders::ALL));
    let test_dm_chat =
        Paragraph::new("DM history here").block(Block::default().borders(Borders::ALL));

    frame.render_widget(test_top_bar, friends_name_area);
    frame.render_widget(test_dm_chat, dm_history_area);

    let test_friends_button =
        Paragraph::new("Friends btn").block(Block::default().borders(Borders::ALL));
    let test_friends_list = Paragraph::new("DMs go here").block(
        Block::default()
            .title("Direct Messages")
            .borders(Borders::ALL),
    );

    frame.render_widget(test_friends_button, friends_btn_area);
    frame.render_widget(test_friends_list, dm_list_area);
}
