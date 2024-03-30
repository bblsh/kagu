use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_widget_list::widget_list::{
    widget_list::WidgetList,
    widget_list_item::{WidgetListItem, WidgetListItemType},
};

use crate::app::{App, InputMode, KaguFormatting, Pane, PopupType, UiElement};
use chrono::Utc;

pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    let top_and_bottom_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(1), Constraint::Max(frame.size().width - 1)])
        .split(frame.size());

    let [kagu_logo_area, kagu_voice_status_area, kagu_latency_area, kagu_blank_area, kagu_time_area] =
        *Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([
                Constraint::Max(10),                               // Kagu logo
                Constraint::Max(16),                               // Voice status
                Constraint::Max(7),                                // Latency
                Constraint::Max(frame.size().width - 7 - 26 - 15), // Blank space
                Constraint::Max(15),                               // Current time
            ])
            .split(top_and_bottom_layout[0])
    else {
        return;
    };

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
    let latency = match app.is_voice_connected {
        true => match app.ping_latency {
            Some(dur) => {
                let mut formatted = dur.as_millis().to_string();
                formatted.push_str(" ms");
                formatted
            }
            None => String::from(""),
        },
        false => String::from(""),
    };
    let latency_paragraph = Paragraph::new(latency);
    frame.render_widget(kagu_logo, kagu_logo_area);
    frame.render_widget(connected_label, kagu_voice_status_area);
    frame.render_widget(latency_paragraph, kagu_latency_area);
    frame.render_widget(time, kagu_time_area);

    let back_panel = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Max(10),
            Constraint::Max(frame.size().width - 10),
        ])
        .split(top_and_bottom_layout[1]);

    let left_panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(frame.size().height - 4)])
        .split(back_panel[0]);

    let realms_list: Vec<ListItem> = app
        .realms
        .items
        .iter()
        .map(|i| {
            ListItem::new(i.1.clone()).style(
                if let Some(realm) = app.realms_manager.get_realm(i.0) {
                    let mut notification = false;
                    // Check to see if we have a pending mention in any channels
                    for channel in &realm.text_channels {
                        // Check if we have a pending mention
                        if channel.1.pending_mention {
                            notification = true;
                        }
                    }

                    // We have been mentioned, so show this to the user
                    if notification {
                        Style::default().fg(Color::Black).bg(Color::LightYellow)
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                },
            )
        })
        .collect();
    let realms = List::new(realms_list)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::RIGHT)
                .title(match app.current_pane {
                    Pane::RealmsPane => Pane::to_str(&app.current_pane).with_focus(),
                    _ => Pane::to_str(&Pane::RealmsPane),
                })
                .border_set(symbols::border::Set {
                    top_right: symbols::line::HORIZONTAL_DOWN,
                    ..symbols::border::PLAIN
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    frame.render_stateful_widget(realms, left_panel[0], &mut app.realms.state);

    let [top_area, input_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Max(frame.size().height - 4), Constraint::Max(3)])
        .split(back_panel[1])
    else {
        return;
    };

    let [channels_block_area, chat_block_area, members_block_area] = *Layout::default()
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
        .split(top_area)
    else {
        return;
    };

    let channels_outer_block = Block::default()
        .borders(Borders::TOP | Borders::RIGHT)
        .border_set(symbols::border::Set {
            top_right: symbols::line::HORIZONTAL_DOWN,
            ..symbols::border::PLAIN
        })
        .title(match &app.current_pane {
            Pane::ChannelsPane => Pane::to_str(&app.current_pane)
                .with_focus()
                .with_pre_post_spaces(),
            _ => Pane::to_str(&Pane::ChannelsPane).with_pre_post_spaces(),
        })
        .border_style(Style::default());
    let inner_channels_area = channels_outer_block.inner(channels_block_area);
    frame.render_widget(channels_outer_block, channels_block_area);

    let [text_channel_label_area, text_channels_list_area, voice_channel_label_area, voice_channels_list_area] =
        *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(1),
                Constraint::Percentage(45),
                Constraint::Max(1),
                Constraint::Percentage(40),
            ])
            .margin(0)
            .split(inner_channels_area)
    else {
        return;
    };

    let text_channels_label = Paragraph::new(vec![
        match &app.input_mode {
            InputMode::ChannelType => match app.ui_element {
                UiElement::TextChannelLabel => Line::from(String::with_focus(String::from("Text"))),
                _ => Line::from("Text"),
            },
            _ => Line::from("Text"),
        },
        Line::from(""),
    ]);

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
    ]);

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
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    let voice_channels = List::new(voice_channels_list)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");

    // Render everything related to channels
    frame.render_widget(text_channels_label, text_channel_label_area);
    frame.render_stateful_widget(
        text_channels,
        text_channels_list_area,
        &mut app.text_channels.state,
    );
    frame.render_widget(voice_channels_label, voice_channel_label_area);
    frame.render_stateful_widget(
        voice_channels,
        voice_channels_list_area,
        &mut app.voice_channels.state,
    );

    let mut users_typing: Vec<String> = Vec::new();
    let mut users_typing_string = String::new();
    if let Some(realm_id) = app.current_realm_id {
        if let Some(realm) = app.realms_manager.get_realm(realm_id) {
            if let Some(channel_id) = &app.current_text_channel {
                if let Some(channel) = realm.get_text_channel(channel_id.0) {
                    for user in &channel.users_typing {
                        let seconds_difference =
                            Utc::now().signed_duration_since(user.1).num_seconds();
                        if seconds_difference < 5 {
                            users_typing.push(app.get_username_from_id(user.0));
                        }
                    }

                    if users_typing.len() == 1 {
                        users_typing_string.push(' ');
                        users_typing_string.push_str(users_typing[0].as_str());
                        users_typing_string.push_str(" is typing... ");
                    } else if users_typing.len() == 2 {
                        users_typing_string.push(' ');
                        users_typing_string.push_str(users_typing[0].as_str());
                        users_typing_string.push_str(" and ");
                        users_typing_string.push_str(users_typing[1].as_str());
                        users_typing_string.push_str(" are typing... ");
                    } else if users_typing.len() > 2 {
                        users_typing_string.push_str(" Multiple users are typing... ");
                    }
                }
            }
        }
    }

    // Split the area for chat into two areas
    // One area will hold chat history, and the other a typing indicator
    let chat_history_outer_block = Block::default()
        .borders(Borders::TOP | Borders::RIGHT)
        .border_set(symbols::border::Set {
            top_right: symbols::line::HORIZONTAL_DOWN,
            ..symbols::border::PLAIN
        })
        .title(match &app.current_text_channel {
            Some(channel) => match &app.current_pane {
                Pane::ChatPane => channel.1.clone().with_focus().with_pre_post_spaces(),
                _ => channel.1.clone().with_pre_post_spaces(),
            },
            None => match &app.current_pane {
                Pane::ChatPane => Pane::to_str(&app.current_pane)
                    .with_focus()
                    .with_pre_post_spaces(),
                _ => Pane::to_str(&Pane::ChatPane).with_pre_post_spaces(),
            },
        })
        .border_style(Style::default());
    let inner_chat_area = chat_history_outer_block.inner(chat_block_area);
    frame.render_widget(chat_history_outer_block, chat_block_area);

    let [messages_area, typing_indicator_area] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(inner_chat_area.height - 1),
            Constraint::Max(1),
        ])
        .margin(0)
        .split(inner_chat_area)
    else {
        return;
    };

    let typing_indicator_paragraph = Paragraph::new(users_typing_string);
    frame.render_widget(typing_indicator_paragraph, typing_indicator_area);

    let chat_list = get_paragraphs_from_text_channel(app, chat_block_area.width as usize - 1)
        .highlight_symbol(if app.reply_target_message_id.is_some() {
            ">"
        } else {
            match app.input_mode {
                InputMode::Chat => ">",
                _ => "",
            }
        })
        .highlight_style(if app.reply_target_message_id.is_some() {
            Style::default().on_gray()
        } else {
            match app.input_mode {
                InputMode::Chat => Style::default().on_gray(),
                _ => Style::default(),
            }
        });
    frame.render_stateful_widget(chat_list, messages_area, &mut app.chat_history.state);

    let members_list: Vec<ListItem> = app
        .users_online
        .items
        .iter()
        .map(|i| ListItem::new(i.1.clone()).style(Style::default()))
        .collect();
    let members = List::new(members_list)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(match app.current_pane {
                    Pane::MembersPane => Pane::to_str(&app.current_pane)
                        .with_focus()
                        .with_pre_post_spaces(),
                    _ => Pane::to_str(&Pane::MembersPane).with_pre_post_spaces(),
                }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    frame.render_stateful_widget(members, members_block_area, &mut app.users_online.state);

    let mut reply_string = String::from("Replying to ");
    if app.reply_target_message_id.is_some() {
        if let Some(realm_id) = app.current_realm_id {
            if let Some(realm) = app.realms_manager.get_realm(realm_id) {
                if let Some(channel_id) = &app.current_text_channel {
                    if let Some(channel) = realm.get_text_channel(channel_id.0) {
                        let index = channel
                            .chat_history
                            .iter()
                            .position(|m| m.message_id == app.reply_target_message_id)
                            .unwrap();

                        let name = app.get_username_from_id(channel.chat_history[index].user_id);

                        reply_string.push_str(name.as_str());
                    }
                }
            }
        }
    }

    let input_paragraph = Paragraph::new(app.input_buffer.get_input_line())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default(),
            _ => Style::default(),
        })
        .block(if app.current_text_channel.is_some() {
            Block::default()
                .borders(Borders::TOP)
                .title(if app.reply_target_message_id.is_some() {
                    reply_string.with_focus().with_pre_post_spaces().on_gray()
                } else {
                    Span::styled(
                        match app.current_pane {
                            Pane::InputPane => Pane::to_str(&app.current_pane)
                                .with_focus()
                                .with_pre_post_spaces(),
                            _ => Pane::to_str(&Pane::InputPane).with_pre_post_spaces(),
                        },
                        Style::default(),
                    )
                })
                .border_style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(match app.current_pane {
                    Pane::InputPane => String::from("No channel to send to")
                        .with_focus()
                        .with_pre_post_spaces(),
                    _ => String::from("No channel to send to").with_pre_post_spaces(),
                })
                .border_style(Style::default().fg(Color::Gray))
        })
        .wrap(Wrap { trim: false });
    frame.render_widget(input_paragraph, input_area);

    let input_width = app.input_buffer.get_input_width();
    if app.is_mentioning {
        let matched_members = &mut app.mention_list;
        for member in &app.users_online.items {
            if member.1.contains(&app.mention_buffer) && !matched_members.items.contains(member) {
                matched_members.items.push(member.clone());
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
            if command.to_str().contains(&app.command_buffer)
                && !matched_commands
                    .items
                    .contains(&(*command, command.to_str()))
            {
                matched_commands.items.push((*command, command.to_str()));
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

    // Draw any popups
    if app.is_popup_shown {
        match app.popup_type {
            PopupType::General => app.general_popup.render(frame),
            PopupType::YesNo => app.yes_no_popup.render(frame),
            PopupType::AddChannel => app.add_channel_popup.render(frame),
            PopupType::RemoveChannel => app.remove_channel_popup.render(frame),
            PopupType::Member => app.member_popup.render(frame),
            PopupType::AddRealm => app.add_realm_popup.render(frame),
            PopupType::RemoveRealm => app.remove_realm_popup.render(frame),
        }
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
                input_area.x + app.input_buffer.get_input_width(),
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )
        }
        _ => (),
    }
}

fn build_mention_command_popup(r: Rect, input_length: &u16, num_items: usize) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(r.height - num_items as u16 - 4),
            Constraint::Max(num_items as u16 + 2),
            Constraint::Max(3),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(7 + input_length),
            Constraint::Max(30),
            Constraint::Max(r.width - 10 - input_length),
        ])
        .split(popup_layout[1])[1]
}

fn get_paragraphs_from_text_channel<'a>(app: &App, width: usize) -> WidgetList<'a> {
    let mut widgets: Vec<WidgetListItem<'a>> = Vec::new();

    if let Some(realm_id) = app.current_realm_id {
        if let Some(channel) = &app.current_text_channel {
            // Now that we have a confirmed text channel, get it
            if let Some(realm) = app.realms_manager.get_realm(realm_id) {
                // Get this text channel
                if let Some(channel) = realm.get_text_channel(channel.0) {
                    // Add this message to our that channel's chat history
                    for message_id in &app.chat_history.items {
                        let mut lines: Vec<Line<'_>> = Vec::new();
                        let mut num_lines = 0;

                        if let Some(message_index) = channel
                            .chat_history
                            .iter()
                            .position(|m| *message_id == m.message_id)
                        {
                            let message = &channel.chat_history[message_index];

                            if message.target_reply_message_id.is_some() {
                                // Get this message that is being replied to
                                let target_id = message.target_reply_message_id;

                                let target_message_index = channel
                                    .chat_history
                                    .iter()
                                    .position(|m| m.message_id == target_id);

                                let target_message =
                                    &channel.chat_history[target_message_index.unwrap()];

                                let mut target_message_str = String::new();
                                for chunk in &target_message.message_chunks {
                                    target_message_str.push_str(chunk.0.as_str());
                                }

                                let mut name_chunk = String::from("@");
                                name_chunk.push_str(
                                    app.get_username_from_id(target_message.user_id).as_str(),
                                );

                                let mut length = target_message_str.len();
                                if width < name_chunk.len() + 8 + length {
                                    length = width - name_chunk.len() - 8;
                                }

                                target_message_str = target_message_str[0..length].to_string();

                                let spans: Vec<Span> = vec![
                                    Span::raw(" ┌── "),
                                    Span::styled(name_chunk, Style::default().fg(Color::Yellow)),
                                    Span::raw(" "),
                                    Span::styled(
                                        target_message_str,
                                        Style::default()
                                            .add_modifier(Modifier::ITALIC)
                                            .fg(Color::Gray),
                                    ),
                                ];
                                lines.push(Line::from(spans));
                                num_lines += 1;
                            }

                            let spans: Vec<Span> = vec![
                                Span::styled(
                                    app.get_username_from_id(message.user_id),
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" "),
                                Span::styled(
                                    match message.time_sent {
                                        Some(time) => {
                                            time.with_timezone(&Local).format("%H:%M").to_string()
                                        }
                                        None => Local::now().format("%H:%M").to_string(),
                                    },
                                    Style::default().add_modifier(Modifier::ITALIC),
                                ),
                            ];
                            lines.push(Line::from(spans));
                            num_lines += 1;

                            let mut spans: Vec<Span> = Vec::new();

                            // For text wrapping
                            let mut complete_message = String::new();

                            // This is a normal text/mention message
                            for chunk in &message.message_chunks {
                                complete_message.push_str(chunk.0.as_str());

                                // If we have an ID, this is a mention chunk
                                if let Some(id) = chunk.1 {
                                    if let Some(user) = &app.user {
                                        if id == user.get_id() {
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
                            //num_lines += 1;

                            let wrapped_height = textwrap::wrap(&complete_message, width).len();
                            let height = num_lines + wrapped_height;

                            widgets.push(WidgetListItem::new(
                                WidgetListItemType::Paragraph(
                                    Paragraph::new(lines).wrap(Wrap { trim: false }),
                                ),
                                width,
                                height,
                            ));
                        }
                    }
                }
            }
        }
    }

    WidgetList::from(widgets)
}
