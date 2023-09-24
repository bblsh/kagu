use std::rc::Rc;

use crate::tui::app::App;
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

    let kagu_button = Paragraph::new("K").block(Block::default().borders(Borders::ALL));
    let realms_list =
        Paragraph::new("Realms go here").block(Block::default().borders(Borders::ALL));

    frame.render_widget(kagu_button, left_panel[0]);
    frame.render_widget(realms_list, left_panel[1]);

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
