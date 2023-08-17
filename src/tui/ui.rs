use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, InputMode, Pane, RustcordFormatting, UiElement};

// static RUSTCORD_LOGO: &str = r#"   ___     _   _     ___     _____     ___      ___      ___      ___
// | _ \   | | | |   / __|   |_   _|   / __|    / _ \    | _ \    |   \
// |   /   | |_| |   \__ \     | |    | (__    | (_) |   |   /    | |) |
// |_|_\    \___/    |___/    _|_|_    \___|    \___/    |_|_\    |___/
//  |"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""| _|"""""|
//  `-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "`-0-0-' "#;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let back_panel = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Max(10),
                Constraint::Max(frame.size().width - 10),
            ]
            .as_ref(),
        )
        .split(frame.size());

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
    frame.render_stateful_widget(realms, back_panel[0], &mut app.realms.state);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(frame.size().height - 3), Constraint::Max(3)].as_ref())
        .split(back_panel[1]);

    let top_blocks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(20),
            Constraint::Max({
                if frame.size().width < 45 {
                    45
                } else {
                    frame.size().width - 45
                }
            }),
            Constraint::Max(20),
        ])
        .split(chunks[0]);

    let channels_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Max(2),
            Constraint::Percentage(45),
            Constraint::Max(1),
            Constraint::Percentage(40),
            Constraint::Max(3),
        ])
        .split(top_blocks[0]);

    let text_channels_label = Paragraph::new(vec![
        match &app.input_mode {
            InputMode::ChannelType => match app.ui_element {
                UiElement::TextChannelLabel => Line::from(String::with_focus(String::from("Text"))),
                _ => Line::from("Text"),
            },
            _ => Line::from("Text"),
        },
        Line::from(""),
    ])
    .block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .title(match app.current_pane {
                Pane::ChannelsPane => Pane::to_str(&app.current_pane).with_focus(),
                _ => Pane::to_str(&Pane::ChannelsPane),
            }),
    );

    let mut text_channels_list: Vec<ListItem> = Vec::new();
    if let Some(realm_id) = app.current_realm_id {
        if let Some(realm) = app.realms_manager.get_realm(realm_id) {
            for channel in realm.get_text_channels() {
                text_channels_list.push(match channel.1.pending_mention {
                    true => ListItem::new(channel.1.get_name().clone().prepend_str("# "))
                        .style(Style::default().bg(Color::LightYellow).fg(Color::Black)),
                    false => ListItem::new(channel.1.get_name().clone().prepend_str("# "))
                        .style(Style::default()),
                });
            }
        }
    }

    let voice_channels_label = Paragraph::new(vec![
        match &app.input_mode {
            InputMode::ChannelType => match app.ui_element {
                UiElement::VoiceChannelLabel => {
                    Line::from(String::with_focus(String::from("Voice")))
                }
                _ => Line::from("Voice"),
            },
            _ => Line::from("Voice"),
        },
        Line::from(""),
        Line::from(""),
    ])
    .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));

    let voice_channels_list: Vec<ListItem> = app
        .voice_channels
        .items
        .iter()
        .map(|channel| {
            let mut lines = vec![Line::from(channel.1.clone().prepend_str("- "))];
            for id in &channel.2 {
                lines.push(Line::from(Span::styled(
                    app.get_username_from_id(*id).prepend_str("   "),
                    Style::default(),
                )));
            }
            ListItem::new(lines).style(Style::default())
        })
        .collect();

    let text_channels = List::new(text_channels_list)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    let voice_channels = List::new(voice_channels_list)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    let connected_label = Paragraph::new(match app.is_voice_connected {
        true => Span::styled("Voice connected", Style::default().fg(Color::LightGreen)),
        false => Span::styled("Voice off", Style::default()),
    })
    .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));

    // Render everything related to channels
    frame.render_widget(text_channels_label, channels_layout[0]);
    frame.render_stateful_widget(
        text_channels,
        channels_layout[1],
        &mut app.text_channels.state,
    );
    frame.render_widget(voice_channels_label, channels_layout[2]);
    frame.render_stateful_widget(
        voice_channels,
        channels_layout[3],
        &mut app.voice_channels.state,
    );
    frame.render_widget(connected_label, channels_layout[4]);

    let chat_paragraph = Paragraph::new(get_lines_from_text_channel(app))
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default(),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match &app.current_text_channel {
                    Some(channel) => match &app.current_pane {
                        Pane::ChatPane => channel.1.clone().with_focus(),
                        _ => channel.1.clone(),
                    },
                    None => match &app.current_pane {
                        Pane::ChatPane => Pane::to_str(&app.current_pane).with_focus(),
                        _ => Pane::to_str(&Pane::ChatPane),
                    },
                })
                .border_style(Style::default()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(chat_paragraph, top_blocks[1]);

    let members_list: Vec<ListItem> = app
        .users_online
        .items
        .iter()
        .map(|i| ListItem::new(i.1.clone()).style(Style::default()))
        .collect();
    let members = List::new(members_list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.current_pane {
                    Pane::MembersPane => Pane::to_str(&app.current_pane).with_focus(),
                    _ => Pane::to_str(&Pane::MembersPane),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    frame.render_stateful_widget(members, top_blocks[2], &mut app.users_online.state);

    // This Paragraph represents the text input, AND the border around it
    //let input = Paragraph::new(app.input.as_str())
    let input = Paragraph::new(app.input_buffer.get_input_line())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default(),
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.current_pane {
                    Pane::InputPane => Pane::to_str(&app.current_pane).with_focus(),
                    _ => Pane::to_str(&Pane::InputPane),
                })
                .border_style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input, chunks[1]);

    let input_width = app.input_buffer.get_input_width();
    if app.is_mentioning {
        let matched_members = &mut app.mention_list;
        for member in &app.users_online.items {
            if member.1.contains(&app.mention_buffer) {
                if !matched_members.items.contains(member) {
                    matched_members.items.push(member.clone());
                }
            }
        }

        let members_list: Vec<ListItem> = matched_members
            .items
            .iter()
            .map(|i| ListItem::new(i.1.clone()).style(Style::default()))
            .collect();
        let members = List::new(members_list)
            .block(Block::default().borders(Borders::ALL).title("Mention"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        let area =
            build_mention_command_popup(frame.size(), &input_width, matched_members.items.len());

        frame.render_widget(Clear, area); //this clears out the background
        frame.render_stateful_widget(members, area, &mut app.mention_list.state);
    } else if app.is_commanding {
        let matched_commands = &mut app.command_list;
        for command in &app.commands {
            if command.to_str().contains(&app.command_buffer) {
                if !matched_commands
                    .items
                    .contains(&(command.clone(), command.to_str()))
                {
                    matched_commands
                        .items
                        .push((command.clone(), command.to_str()));
                }
            }
        }

        let commands_list: Vec<ListItem> = matched_commands
            .items
            .iter()
            .map(|i| ListItem::new(i.1.clone()).style(Style::default()))
            .collect();
        let commands = List::new(commands_list)
            .block(Block::default().borders(Borders::ALL).title("Command"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        let area =
            build_mention_command_popup(frame.size(), &input_width, matched_commands.items.len());

        frame.render_widget(Clear, area); // This clears out the background
        frame.render_stateful_widget(commands, area, &mut app.command_list.state);
    }

    if app.is_popup_shown {
        let mut title = String::from(&*app.popup_title);
        title.push_str(" (Enter to dismiss)");

        let alert_block = Paragraph::new(&*app.popup_text.as_str())
            .block(Block::default().title(title).borders(Borders::ALL));
        let area = centered_popup(60, 20, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(alert_block, area);
    }

    // Move the cursor to the end of the text (if in edit mode)
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the
            // specified coordinates after rendering
            frame.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input_buffer.get_input_width() + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
        _ => (),
    }
}

fn build_mention_command_popup(r: Rect, input_length: &u16, num_items: usize) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Max(r.height - num_items as u16 - 4),
                Constraint::Max(num_items as u16 + 2),
                Constraint::Max(3),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Max(7 + input_length),
                Constraint::Max(30),
                Constraint::Max(r.width - 10 - input_length),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn centered_popup(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn get_lines_from_text_channel<'a>(app: &App) -> Vec<Line<'a>> {
    let mut lines: Vec<Line<'_>> = Vec::new();

    if let Some(realm_id) = app.current_realm_id {
        if let Some(channel) = &app.current_text_channel {
            // Now that we have a confirmed text channel, get it
            if let Some(realm) = app.realms_manager.get_realm(realm_id) {
                // Get this text channel
                if let Some(channel) = realm.get_text_channel(channel.0) {
                    // Add this message to our that channel's chat history
                    for message in &channel.chat_history {
                        // If there's optional image data
                        if let Some(_image) = &message.1 {
                            // Render the image here
                        } else {
                            let mut spans: Vec<Span> = vec![
                                Span::raw(app.get_username_from_id(message.0)),
                                Span::raw(": "),
                            ];
                            // This is a normal text/mention message
                            for chunk in &message.2 {
                                // If we have an ID, this is a mention chunk
                                if let Some(id) = chunk.1 {
                                    if let Some(our_id) = app.user_id {
                                        if id == our_id {
                                            spans.push(Span::styled(
                                                chunk.0.clone(),
                                                Style::default()
                                                    .bg(Color::LightYellow)
                                                    .fg(Color::Black),
                                            ));
                                        }
                                        // This mention chunk isn't mentioning us, so display it normally
                                        else {
                                            spans.push(Span::styled(
                                                chunk.0.clone(),
                                                Style::default().fg(Color::Yellow),
                                            ));
                                        }
                                    }
                                }
                                // This has no ID, so display it as normal text
                                else {
                                    spans.push(Span::styled(
                                        chunk.0.clone(),
                                        Style::default().fg(Color::DarkGray),
                                    ));
                                }
                            }

                            lines.push(Line::from(spans));
                        }
                    }
                }
            }
        }
    }

    lines
}
