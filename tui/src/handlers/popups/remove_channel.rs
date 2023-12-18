use crate::app::Pane;
use crate::{
    app::{App, AppResult, InputMode},
    popups::remove_channel_popup::RemoveChannelPopupUiElement,
};
use crossterm::event::{KeyCode, KeyEvent};
use realms::realm::ChannelType;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App<'_>) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.dismiss_popup();
            match app.remove_channel_popup.channel_type {
                ChannelType::TextChannel => {
                    app.input_mode = InputMode::TextChannel;
                    app.current_pane = Pane::ChannelsPane;
                }
                ChannelType::VoiceChannel => {
                    app.input_mode = InputMode::VoiceChannel;
                    app.current_pane = Pane::ChannelsPane;
                }
            }
        }
        KeyCode::Up => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => (),
            RemoveChannelPopupUiElement::No => {
                app.remove_channel_popup.current_ui_element = RemoveChannelPopupUiElement::Yes
            }
        },
        KeyCode::Down => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => {
                app.remove_channel_popup.current_ui_element = RemoveChannelPopupUiElement::No
            }
            RemoveChannelPopupUiElement::No => (),
        },
        KeyCode::Enter => match app.remove_channel_popup.current_ui_element {
            RemoveChannelPopupUiElement::Yes => {
                app.remove_channel(
                    app.remove_channel_popup.channel_type.clone(),
                    app.remove_channel_popup.channel_id,
                )
                .await;

                match app.remove_channel_popup.channel_type {
                    ChannelType::TextChannel => {
                        app.text_channels.unselect();
                        app.input_mode = InputMode::ChannelType
                    }
                    ChannelType::VoiceChannel => {
                        app.voice_channels.unselect();
                        app.input_mode = InputMode::ChannelType
                    }
                }

                app.dismiss_popup();
            }
            RemoveChannelPopupUiElement::No => {
                app.dismiss_popup();
                match app.remove_channel_popup.channel_type {
                    ChannelType::TextChannel => {
                        app.input_mode = InputMode::TextChannel;
                        app.current_pane = Pane::ChannelsPane;
                    }
                    ChannelType::VoiceChannel => {
                        app.input_mode = InputMode::VoiceChannel;
                        app.current_pane = Pane::ChannelsPane;
                    }
                }
            }
        },
        _ => (),
    };

    Ok(())
}
