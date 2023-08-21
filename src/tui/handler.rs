use crate::tui::app::{Pane, RustcordFormatting};
use crate::{
    realms::realm::ChannelType,
    tui::app::{App, AppResult, InputMode, UiElement},
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui::style::{Color, Style};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        // Regardless of mode, exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit().await;
                return Ok(());
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.hang_up().await;
                return Ok(());
            }
        }
        _ => (),
    }

    match app.input_mode {
        InputMode::Normal => match key_event.code {
            KeyCode::Char('Q') | KeyCode::Char('q') | KeyCode::Esc => {
                app.quit().await;
                return Ok(());
            }
            KeyCode::Char('i') => {
                app.input_mode = InputMode::Editing;
                app.current_pane = Pane::InputPane;
            }
            KeyCode::Up => match app.current_pane {
                Pane::InputPane => app.current_pane = Pane::ChatPane,
                _ => (),
            },
            KeyCode::Down => match app.current_pane {
                Pane::ChannelsPane | Pane::ChatPane | Pane::MembersPane | Pane::RealmsPane => {
                    app.current_pane = Pane::InputPane;
                }
                _ => (),
            },
            KeyCode::Left => match app.current_pane {
                Pane::ChannelsPane => app.current_pane = Pane::RealmsPane,
                Pane::ChatPane => app.current_pane = Pane::ChannelsPane,
                Pane::MembersPane => app.current_pane = Pane::ChatPane,
                Pane::InputPane => app.current_pane = Pane::RealmsPane,
                _ => (),
            },
            KeyCode::Right => match app.current_pane {
                Pane::ChannelsPane => app.current_pane = Pane::ChatPane,
                Pane::ChatPane => app.current_pane = Pane::MembersPane,
                Pane::RealmsPane => app.current_pane = Pane::ChannelsPane,
                _ => (),
            },
            KeyCode::Enter => match app.current_pane {
                Pane::InputPane => {
                    app.input_mode = InputMode::Editing;
                }
                Pane::ChannelsPane => {
                    app.input_mode = InputMode::ChannelType;
                    app.ui_element = UiElement::TextChannelLabel;
                }
                Pane::MembersPane => {
                    app.input_mode = InputMode::Members;
                    app.users_online.next();
                }
                Pane::RealmsPane => {
                    app.input_mode = InputMode::Realms;
                    app.realms.next();
                }
                _ => (),
            },
            _ => (),
        },
        InputMode::Popup => match key_event.code {
            KeyCode::Enter => app.dismiss_popup(),
            _ => (),
        },
        InputMode::ChannelType => match key_event.code {
            KeyCode::Enter => match app.ui_element {
                UiElement::TextChannelLabel => {
                    app.input_mode = InputMode::TextChannel;
                    app.text_channels.next();
                }
                UiElement::VoiceChannelLabel => {
                    app.input_mode = InputMode::VoiceChannel;
                    app.voice_channels.next();
                }
                _ => (),
            },
            KeyCode::Down => match app.ui_element {
                UiElement::TextChannelLabel => app.ui_element = UiElement::VoiceChannelLabel,
                UiElement::VoiceChannelLabel => (),
                _ => (),
            },
            KeyCode::Up => match app.ui_element {
                UiElement::TextChannelLabel => (),
                UiElement::VoiceChannelLabel => app.ui_element = UiElement::TextChannelLabel,
                _ => (),
            },
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui_element = UiElement::None;
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char('i') => {
                app.ui_element = UiElement::None;
                app.input_mode = InputMode::Editing;
                app.current_pane = Pane::InputPane;
            }
            _ => (),
        },
        InputMode::TextChannel => match key_event.code {
            KeyCode::Char('i') => {
                app.ui_element = UiElement::None;
                app.input_mode = InputMode::Editing;
                app.text_channels.unselect();
                app.current_pane = Pane::InputPane;
            }
            KeyCode::Up => app.text_channels.previous(),
            KeyCode::Down => app.text_channels.next(),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.input_mode = InputMode::ChannelType;
                app.ui_element = UiElement::TextChannelLabel;
                app.text_channels.unselect();
            }
            KeyCode::Enter => {
                // Get the index of the currently selected channel
                let selected_id = app.text_channels.state.selected().unwrap();

                // Join the selected text channel
                app.join_channel(
                    app.current_realm_id.unwrap(),
                    ChannelType::TextChannel,
                    app.text_channels.items.get(selected_id).unwrap().0,
                )
                .await;
            }
            _ => (),
        },
        InputMode::VoiceChannel => match key_event.code {
            KeyCode::Char('i') => {
                app.ui_element = UiElement::None;
                app.input_mode = InputMode::Editing;
                app.voice_channels.unselect();
                app.current_pane = Pane::InputPane;
            }
            KeyCode::Up => app.voice_channels.previous(),
            KeyCode::Down => app.voice_channels.next(),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.input_mode = InputMode::ChannelType;
                app.ui_element = UiElement::VoiceChannelLabel;
                app.voice_channels.unselect();
            }
            KeyCode::Enter => {
                // Get the index of the currently selected channel
                let selected_id = app.voice_channels.state.selected().unwrap();
                let channel_id = app.voice_channels.items.get(selected_id).unwrap().0;

                if let Some(current_channel) = app.current_voice_channel {
                    if channel_id != current_channel {
                        // Leave a channel if we're in one already
                        app.hang_up().await;

                        // Join the selected voice channel
                        app.join_channel(
                            app.current_realm_id.unwrap(),
                            ChannelType::VoiceChannel,
                            channel_id,
                        )
                        .await;
                    }
                } else {
                    // Join the selected voice channel
                    app.join_channel(
                        app.current_realm_id.unwrap(),
                        ChannelType::VoiceChannel,
                        channel_id,
                    )
                    .await;
                }
            }
            _ => (),
        },
        InputMode::Editing if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Enter => {
                if &app.input_buffer.input.len() > &0 {
                    if app.is_mentioning {
                        if app.mention_list.items.len() > 0 {
                            let selected_id = app.mention_list.state.selected().unwrap();
                            let user_id = app.mention_list.items.get(selected_id).unwrap().0;
                            let user_name =
                                app.mention_list.items.get(selected_id).unwrap().1.clone();

                            if !app.users_mentioned.contains(&user_id) {
                                app.users_mentioned.push(user_id);
                            }

                            // Push the remainder of this user's name to the input buffer
                            if let Some(input) = app.input_buffer.input.last_mut() {
                                for _ in 0..app.mention_buffer.len() {
                                    input.0.pop();
                                }

                                // Remove @ character
                                input.0.pop();
                            }

                            app.input_buffer.input.push((
                                user_name.prepend_str("@"),
                                Style::default().fg(Color::Yellow),
                                Some(user_id),
                            ));
                            app.input_buffer.is_mentioning = true;
                        }
                    }

                    app.handle_input().await;

                    app.input_buffer.input.clear();
                    app.input_buffer
                        .input
                        .push((String::new(), Style::default(), None));

                    app.input_buffer.is_mentioning = false;
                    app.is_mentioning = false;
                    app.mention_buffer.clear();
                    app.mention_list.items.clear();
                    app.mention_list.unselect();
                    app.users_mentioned.clear();

                    app.current_command = None;
                }
            }
            KeyCode::Char('@') => {
                if app.is_commanding {
                    if let Some(input) = app.input_buffer.input.last_mut() {
                        input.0.push('@');
                    }
                } else {
                    app.is_mentioning = true;
                    if let Some(input) = app.input_buffer.input.last_mut() {
                        input.0.push('@');
                    }
                    app.mention_list.next();
                }
            }
            KeyCode::Char('/') => {
                // Only start commanding if this is the first character
                if app.input_buffer.get_input_string().len() == 0 {
                    app.is_commanding = true;
                    if let Some(input) = app.input_buffer.input.last_mut() {
                        input.0.push('/');
                    }
                    app.command_list.next();
                } else {
                    // Copied from Char(c) case below this
                    if let Some(input) = app.input_buffer.input.last_mut() {
                        input.0.push('/');
                    }

                    if app.is_mentioning {
                        app.mention_buffer.push('/');

                        // Reset our mention list
                        app.mention_list.items.clear();
                        app.mention_list.unselect();
                        app.mention_list.next();
                    } else if app.is_commanding {
                        app.command_buffer.push('/');

                        // Reset our mention list
                        app.command_list.items.clear();
                        app.command_list.unselect();
                        app.command_list.next();
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Some(input) = app.input_buffer.input.last_mut() {
                    input.0.push(c);
                }

                if app.is_mentioning {
                    app.mention_buffer.push(c);

                    // Reset our mention list
                    app.mention_list.items.clear();
                    app.mention_list.unselect();
                    app.mention_list.next();
                } else if app.is_commanding {
                    app.command_buffer.push(c);

                    // Reset our mention list
                    app.command_list.items.clear();
                    app.command_list.unselect();
                    app.command_list.next();
                }
            }
            KeyCode::Backspace => {
                let num_chunks = app.input_buffer.input.len();
                if let Some(input) = app.input_buffer.input.last_mut() {
                    if input.0.len() == 0 {
                        if app.input_buffer.is_commanding {
                            app.input_buffer.input.pop();
                            app.input_buffer.input.pop();
                        }
                        app.input_buffer.is_commanding = false;
                        app.current_command = None;

                        // Don't pop anything if this is our first text chunk, leave it empty
                        if num_chunks > 1 {
                            app.input_buffer.input.pop();

                            // Check the style. If it isn't default, we know this is a mention
                            // and can be completely removed
                            if let Some(next_in) = app.input_buffer.input.last() {
                                if next_in.1 != Style::default() {
                                    app.input_buffer.input.pop();
                                }
                            }
                        }
                    } else {
                        input.0.pop();
                    }
                }

                if app.is_mentioning {
                    if app.mention_buffer.len() == 0 {
                        app.is_mentioning = false;
                        // Clear our mention list
                        app.mention_list.items.clear();
                        app.mention_list.unselect();
                    } else {
                        app.mention_buffer.pop();
                    }
                } else if app.is_commanding {
                    if app.command_buffer.len() == 0 {
                        app.is_commanding = false;
                        // Clear our command list
                        app.command_list.items.clear();
                        app.command_list.unselect();
                    } else {
                        app.command_buffer.pop();
                    }
                }
            }
            KeyCode::Esc => {
                if app.is_mentioning {
                    app.is_mentioning = false;
                    app.mention_buffer.clear();
                    app.mention_list.items.clear();
                    app.mention_list.unselect();
                    app.users_mentioned.clear();
                } else if app.is_commanding {
                    app.is_commanding = false;
                    app.command_buffer.clear();
                    app.command_list.items.clear();
                    app.command_list.unselect();
                } else {
                    app.input_mode = InputMode::Normal
                }
            }
            KeyCode::Down => {
                if app.is_mentioning {
                    app.mention_list.next();
                } else if app.is_commanding {
                    app.command_list.next();
                }
            }
            KeyCode::Up => {
                if app.is_mentioning {
                    app.mention_list.previous();
                } else if app.is_commanding {
                    app.command_list.previous();
                }
            }
            KeyCode::Tab => {
                if app.is_mentioning {
                    if app.mention_list.items.len() > 0 {
                        let selected_id = app.mention_list.state.selected().unwrap();
                        let user_id = app.mention_list.items.get(selected_id).unwrap().0;
                        let user_name = app.mention_list.items.get(selected_id).unwrap().1.clone();

                        if !app.users_mentioned.contains(&user_id) {
                            app.users_mentioned.push(user_id);
                        }

                        // Push the remainder of this user's name to the input buffer
                        //app.input
                        //    .push_str(&user_name.get(app.mention_buffer.len()..).unwrap());
                        // Remove current chars from buffer
                        if let Some(input) = app.input_buffer.input.last_mut() {
                            for _ in 0..app.mention_buffer.len() {
                                input.0.pop();
                            }

                            // Remove @ character
                            input.0.pop();
                        }

                        app.input_buffer.input.push((
                            user_name.prepend_str("@"),
                            Style::default().fg(Color::Yellow),
                            Some(user_id),
                        ));

                        app.input_buffer.is_mentioning = true;

                        // Add a new element to represent a new span after the span with a mention
                        app.input_buffer
                            .input
                            .push((String::new(), Style::default(), None));

                        app.mention_buffer.clear();
                        app.mention_list.items.clear();
                        app.mention_list.unselect();
                        app.users_mentioned.clear();

                        app.is_mentioning = false;
                    }
                } else if app.is_commanding {
                    if app.command_list.items.len() > 0 {
                        let selected_id = app.command_list.state.selected().unwrap();
                        let command = app.command_list.items.get(selected_id).unwrap().0;

                        app.current_command = Some(command.clone());

                        // Push the remainder of this command's name to the input buffer
                        // Remove current chars from buffer
                        if let Some(input) = app.input_buffer.input.last_mut() {
                            for _ in 0..app.command_buffer.len() {
                                input.0.pop();
                            }

                            // Remove @ character
                            input.0.pop();
                        }

                        // Insert /image in blue text
                        app.input_buffer.input.push((
                            command.to_str().prepend_str("/"),
                            Style::default().fg(Color::LightBlue),
                            None,
                        ));

                        // Insert gray text with "file path" to tell the user what to enter
                        app.input_buffer.input.push((
                            String::from(" file path: "),
                            Style::default().fg(Color::Gray),
                            None,
                        ));

                        app.input_buffer.is_commanding = true;

                        // Add a new element to represent a new span after the span with a mention
                        app.input_buffer
                            .input
                            .push((String::new(), Style::default(), None));

                        app.command_buffer.clear();
                        app.command_list.items.clear();
                        app.command_list.unselect();

                        app.is_commanding = false;
                    }
                }
            }
            _ => (),
        },
        InputMode::Members if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                // Stop inspecting this member
                if app.is_viewing_member {
                    app.is_viewing_member = false;
                }
                // We're not looking at a member, so exit member selection
                else {
                    app.users_online.unselect();
                    app.input_mode = InputMode::Normal;
                }
            }
            KeyCode::Up => {
                app.users_online.previous();
            }
            KeyCode::Down => {
                app.users_online.next();
            }
            KeyCode::Enter => {
                // Display user information
                app.is_viewing_member = true;
            }
            _ => (),
        },
        InputMode::Realms if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                app.realms.unselect();
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Up => {
                app.realms.previous();
            }
            KeyCode::Down => {
                app.realms.next();
            }
            KeyCode::Enter => {
                // Enter this realm
            }
            _ => (),
        },
        _ => (),
    }

    Ok(())
}
