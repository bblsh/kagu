use std::rc::Rc;

use crate::tui::app::{App, KaguFormatting, Pane};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
    Frame,
};

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let back_panel = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Max(10),
                Constraint::Max(20),
                Constraint::Max(frame.size().width - 10 - 20),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let left_panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(3), Constraint::Max(frame.size().height - 4)].as_ref())
        .split(back_panel[0]);

    let kagu_button = Paragraph::new("Kagu");
    frame.render_widget(kagu_button, left_panel[0]);

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
    frame.render_stateful_widget(realms, left_panel[1], &mut app.realms.state);

    let middle_panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(3), Constraint::Max(frame.size().height - 3)].as_ref())
        .split(back_panel[1]);

    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(3), Constraint::Max(frame.size().height - 3)].as_ref())
        .split(back_panel[2]);

    let test_top_bar =
        Paragraph::new("Friends or friend_name").block(Block::default().borders(Borders::ALL));
    let test_dm_chat =
        Paragraph::new("DM history here").block(Block::default().borders(Borders::ALL));

    frame.render_widget(test_top_bar, right_panel[0]);
    frame.render_widget(test_dm_chat, right_panel[1]);

    let test_friends_button =
        Paragraph::new("Friends btn").block(Block::default().borders(Borders::ALL));
    let test_friends_list = Paragraph::new("DMs go here").block(
        Block::default()
            .title("Direct Messages")
            .borders(Borders::ALL),
    );

    frame.render_widget(test_friends_button, middle_panel[0]);
    frame.render_widget(test_friends_list, middle_panel[1]);
}
