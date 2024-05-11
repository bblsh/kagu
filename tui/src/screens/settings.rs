use super::settings_views;
use crate::app::{App, SettingsCategory};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Color, Line, Modifier, Span},
    style::Style,
    symbols,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    let [kagu_bar_area, main_settings_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(1), Constraint::Max(frame.size().width - 1)])
        .split(frame.size())
    else {
        return;
    };

    let kagu_bar = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(10),                                // Kagu logo
            Constraint::Max(20),                                // Voice status
            Constraint::Max(frame.size().width - 10 - 20 - 15), // Blank space
            Constraint::Max(15),                                // Current time
        ])
        .split(kagu_bar_area);

    let mut kagu_spans: Vec<Span> = vec![Span::raw("Kagu")];

    if !app.friend_requests.is_empty() {
        kagu_spans.push(Span::styled(" (", Style::default().fg(Color::LightRed)));
        kagu_spans.push(Span::styled(
            app.friend_requests.len().to_string(),
            Style::default().fg(Color::LightRed),
        ));
        kagu_spans.push(Span::styled(")", Style::default().fg(Color::LightRed)));
    }

    let kagu_text = vec![Line::from(kagu_spans)];
    let kagu_logo = Paragraph::new(kagu_text);
    let time = Paragraph::new(app.get_current_time_string()).alignment(Alignment::Right);
    let connected_label = Paragraph::new(match app.is_voice_connected {
        true => Span::styled("Voice connected", Style::default().fg(Color::LightGreen)),
        false => Span::styled("Voice off", Style::default()),
    });
    frame.render_widget(kagu_logo, kagu_bar[0]);
    frame.render_widget(connected_label, kagu_bar[1]);
    frame.render_widget(time, kagu_bar[3]);

    let [settings_list_area, settings_view_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(10),
            Constraint::Max(frame.size().width - 10),
        ])
        .split(main_settings_area)
    else {
        return;
    };

    let settings_categories: Vec<ListItem> = app
        .settings_category_list
        .items
        .iter()
        .map(|i| ListItem::new(i.to_string()).style(Style::default()))
        .collect();

    let settings_list = List::new(settings_categories)
        .block(
            Block::default()
                .border_style(Style::default())
                .borders(Borders::RIGHT | Borders::TOP)
                .border_set(symbols::border::Set {
                    top_right: symbols::line::HORIZONTAL_DOWN,
                    ..symbols::border::PLAIN
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    frame.render_stateful_widget(
        settings_list,
        settings_list_area,
        &mut app.settings_category_list.state,
    );

    // Get the inner area to render the current settings view
    let settings_block = Block::default().borders(Borders::TOP);
    let inner_content_area = settings_block.inner(settings_view_area);
    frame.render_widget(settings_block, settings_view_area);

    match app.current_settings_category {
        SettingsCategory::Audio => {
            settings_views::audio_settings::render(app, inner_content_area, frame)
        }
        SettingsCategory::Colors => (),
        SettingsCategory::FriendRequests => (),
    }
}
