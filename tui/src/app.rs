use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::error;
use std::io;
use std::path::Path;

use ratatui::{backend::CrosstermBackend, Terminal};
use tui_widget_list::widget_list::stateful_widget_list::StatefulWidgetList;

use crate::command::Command;
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    stateful_list::StatefulList,
    tui::Tui,
};
use client::Client;
use message::MessageType;
use realms::channels::text_channel::TextChannelMessage;
use realms::realm::ChannelType;
use realms::realms_manager::RealmsManager;
use types::MessageIdSize;
use types::{ChannelIdSize, RealmIdSize, UserIdSize};

use super::input_buffer::InputBuffer;
use super::popups::popup_traits::PopupTraits;
use crate::popups::{
    add_channel_popup::AddChannelPopup, add_realm_popup::AddRealmPopup,
    general_popup::GeneralPopup, member_popup::MemberPopup,
    remove_channel_popup::RemoveChannelPopup, remove_realm_popup::RemoveRealmPopup,
    yes_no_popup::YesNoPopup,
};

use chrono::Local;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum PopupType {
    General,
    YesNo,
    AddChannel,
    RemoveChannel,
    Member,
    AddRealm,
    RemoveRealm,
}

#[derive(Debug)]
pub enum Screen {
    Main,
    Settings,
    Personal,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    ChannelType,
    TextChannel,
    VoiceChannel,
    Members,
    Realms,
    Loading,
    Popup,
    Chat,
}

#[derive(Debug)]
pub enum UiElement {
    TextChannelLabel,
    VoiceChannelLabel,
    None,
}

#[derive(Debug)]
pub enum Pane {
    RealmsPane,
    ChannelsPane,
    ChatPane,
    MembersPane,
    InputPane,
    None,
}

impl Pane {
    pub fn to_str(&self) -> String {
        match self {
            Pane::RealmsPane => String::from("Realms"),
            Pane::ChannelsPane => String::from("Channels"),
            Pane::ChatPane => String::from("Chat"),
            Pane::MembersPane => String::from("Members"),
            Pane::InputPane => String::from("Input"),
            _ => String::new(),
        }
    }
}

pub trait KaguFormatting {
    fn with_focus(self) -> Self;
    fn add_hashtag_with_space(self) -> Self;
    fn add_hashtag(self) -> Self;
    fn prepend_str(self, text: &str) -> Self;
    fn with_pre_post_spaces(self) -> Self;
}

impl KaguFormatting for String {
    fn with_focus(mut self) -> Self {
        self.insert(0, '[');
        self.push(']');
        self
    }

    fn with_pre_post_spaces(mut self) -> Self {
        self.insert(0, ' ');
        self.push(' ');
        self
    }

    fn add_hashtag_with_space(mut self) -> Self {
        self.insert_str(0, "# ");
        self
    }

    fn add_hashtag(mut self) -> Self {
        self.insert(0, '#');
        self
    }

    fn prepend_str(mut self, text: &str) -> Self {
        self.insert_str(0, text);
        self
    }
}

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    pub user_id: Option<UserIdSize>,
    /// Current input mode
    pub input_mode: InputMode,
    /// Current UI element selected
    pub ui_element: UiElement,
    /// Current pane in focus
    pub current_pane: Pane,
    /// Current screen type to draw
    pub current_screen: Screen,
    /// Is the application running?
    pub running: bool,
    /// Client to handle all interactions with the server
    pub client: Client,
    /// User ID to usernames
    pub user_id_to_username: HashMap<UserIdSize, String>,
    /// Realms manager to manage our realms and channels
    pub realms_manager: RealmsManager,
    /// Current users online
    pub users_online: StatefulList<(UserIdSize, String)>,
    /// Realms
    pub realms: StatefulList<(RealmIdSize, String)>,
    /// Text channels to display
    pub text_channels: StatefulList<(ChannelIdSize, String)>,
    /// Voice channels to display
    pub voice_channels: StatefulList<(ChannelIdSize, String, Vec<UserIdSize>)>,
    /// Status indicating if we are connected via voice
    pub is_voice_connected: bool,
    /// Current Realm we are in
    pub current_realm_id: Option<RealmIdSize>,
    /// Current text channel we're in
    pub current_text_channel: Option<(ChannelIdSize, String)>,
    /// Current voice channel we're in
    pub current_voice_channel: Option<ChannelIdSize>,
    /// State for showing text commands
    pub is_commanding: bool,
    /// Vec of available user commands
    pub commands: Vec<Command>,
    /// Capture command text to match againt commands
    pub command_buffer: String,
    /// List to select a command
    pub command_list: StatefulList<(Command, String)>,
    /// Command the user is currently commanding
    pub current_command: Option<Command>,
    /// State for showing names to @mention
    pub is_mentioning: bool,
    /// Capture mention text to match against members
    pub mention_buffer: String,
    /// List to select a member while mentioning
    pub mention_list: StatefulList<(UserIdSize, String)>,
    /// Users mentioned in a TextMention
    pub users_mentioned: Vec<UserIdSize>,
    /// Struct to hold our input
    pub input_buffer: InputBuffer,
    /// If we are showing a popup to the user
    pub is_popup_shown: bool,
    /// Type of popup to render
    pub popup_type: PopupType,
    /// Text shown in popup
    pub popup_text: String,
    /// Title for the popup window
    pub popup_title: String,
    /// General popup
    pub general_popup: GeneralPopup,
    /// Add channel popup
    pub add_channel_popup: AddChannelPopup,
    /// Member info popup
    pub member_popup: MemberPopup,
    // Yes / No confirmation popup
    pub yes_no_popup: YesNoPopup,
    /// Remove channel popup
    pub remove_channel_popup: RemoveChannelPopup,
    /// Add realm popup
    pub add_realm_popup: AddRealmPopup,
    /// Remove realm popup
    pub remove_realm_popup: RemoveRealmPopup,
    /// Incoming friend requests
    pub friend_requests: Vec<UserIdSize>,
    /// Pending friend requests
    pub pending_friend_requests: Vec<UserIdSize>,
    /// Friends list
    pub friends: Vec<UserIdSize>,
    /// Timestamp for when we started typing
    pub time_started_typing: Option<DateTime<Utc>>,
    /// Stateful widget list for chat history and replies
    pub chat_history: StatefulWidgetList<Option<MessageIdSize>>,
    /// What message id we are replying to
    pub reply_target_message_id: Option<MessageIdSize>,
    pub _not_used: &'a bool,
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new(client: Client) -> Self {
        // There's likely a better way to populate these commands
        let mut commands_list = StatefulList::default();
        commands_list
            .items
            .push((Command::Image, Command::Image.to_str()));

        Self {
            user_id: None,
            input_mode: InputMode::Normal,
            current_screen: Screen::Main,
            ui_element: UiElement::None,
            current_pane: Pane::ChatPane,
            running: true,
            client,
            user_id_to_username: HashMap::new(),
            realms_manager: RealmsManager::default(),
            users_online: StatefulList::default(),
            realms: StatefulList::default(),
            text_channels: StatefulList::default(),
            voice_channels: StatefulList::default(),
            is_voice_connected: false,
            current_realm_id: None,
            current_text_channel: None,
            current_voice_channel: None,
            is_commanding: false,
            commands: Command::get_commands(),
            command_buffer: String::new(),
            command_list: commands_list,
            current_command: None,
            is_mentioning: false,
            mention_buffer: String::new(),
            mention_list: StatefulList::default(),
            users_mentioned: Vec::new(),
            input_buffer: InputBuffer::default(),
            is_popup_shown: false,
            popup_type: PopupType::General,
            popup_text: String::new(),
            popup_title: String::new(),
            general_popup: GeneralPopup::default(),
            add_channel_popup: AddChannelPopup::default(),
            member_popup: MemberPopup::default(),
            yes_no_popup: YesNoPopup::default(),
            remove_channel_popup: RemoveChannelPopup::default(),
            add_realm_popup: AddRealmPopup::default(),
            remove_realm_popup: RemoveRealmPopup::default(),
            friend_requests: Vec::new(),
            pending_friend_requests: Vec::new(),
            friends: Vec::new(),
            time_started_typing: None,
            chat_history: StatefulWidgetList::default(),
            reply_target_message_id: None,
            _not_used: &false,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub async fn quit(&mut self) {
        self.hang_up().await;
        self.client.disconnect().await;
        self.running = false;
    }

    pub async fn run_app(&mut self) -> AppResult<()> {
        // Initialize the terminal user interface.
        let backend = CrosstermBackend::new(io::stderr());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(250);
        let mut tui = Tui::new(terminal, events);
        tui.init()?;

        let user_id = self.client.get_user_id().await;
        match user_id {
            Some(id) => self.user_id = Some(id),
            None => {
                eprintln!("Failed to get User ID");
                self.quit().await;
            }
        };

        let username = self.client.get_username().await;
        match username {
            Some(_) => (),
            None => {
                eprintln!("Failed to get Username");
                self.quit().await;
            }
        };

        // We should be logged in and have our own User, so use this to show our name
        self.user_id_to_username
            .insert(self.user_id.unwrap(), username.unwrap());

        // Start the main loop.
        while self.running {
            // Update any new messages received by the Client
            for message in self.client.get_new_messages().await {
                match message.message {
                    MessageType::UserJoined(user) => {
                        // We should already know we're online, so ignore anything about us
                        if let Some(id) = self.client.get_user_id().await {
                            if id == user.get_id() {
                                continue;
                            }
                        }

                        // Add this user to a map to know who is who
                        self.user_id_to_username
                            .insert(user.get_id(), String::from(user.get_username()));

                        // Now add them to our list of users currently online
                        self.users_online
                            .items
                            .push((user.get_id(), String::from(user.get_username())));
                    }
                    MessageType::UserLeft(user_id) => {
                        //self.user_id_to_username.remove(&user_id);
                        let index = self
                            .users_online
                            .items
                            .iter()
                            .position(|x| x.0 == user_id)
                            .unwrap();
                        self.users_online.items.remove(index);
                    }
                    MessageType::UserJoinedVoiceChannel(join) => {
                        // Add this user to that channel's connected_users
                        self.realms_manager.add_user_to_voice_channel(
                            join.user_id,
                            join.realm_id,
                            join.channel_id,
                        );

                        // For now let's update the voice_channels list with
                        // what we already have saved elsewhere
                        for channel in &mut self.voice_channels.items {
                            if channel.0 == join.channel_id {
                                channel.2.push(join.user_id);
                            }
                        }

                        // If this is us, let us know we've been connected via voice
                        if let Some(id) = self.client.get_user_id().await {
                            if id == join.user_id {
                                self.is_voice_connected = true;
                                // Update our current voice channel ID
                                self.current_voice_channel = Some(join.channel_id);
                            }
                        }
                    }
                    MessageType::UserLeftVoiceChannel(left) => {
                        self.realms_manager.remove_user_from_voice_channel(
                            left.user_id,
                            left.realm_id,
                            left.channel_id,
                        );

                        // For now let's update the voice_channels list with
                        // what we already have saved elsewhere
                        for channel in &mut self.voice_channels.items {
                            if let Some(index) = channel.2.iter().position(|x| x == &left.user_id) {
                                channel.2.remove(index);
                            }
                        }
                    }
                    MessageType::AllUsers(users) => {
                        for user in users {
                            self.user_id_to_username
                                .insert(user.get_id(), String::from(user.get_username()));
                            self.users_online
                                .items
                                .push((user.get_id(), String::from(user.get_username())));
                        }
                    }
                    MessageType::Text(message) => {
                        // Add this message to its respective channel's history
                        // Get our realm
                        if let Some(realm) = self.realms_manager.get_realm_mut(message.0.realm_id) {
                            // Get this text channel
                            if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id)
                            {
                                // Add this message to our that channel's chat history
                                channel.chat_history.push(TextChannelMessage {
                                    message_id: message.0.message_id,
                                    user_id: message.0.user_id,
                                    target_reply_message_id: None,
                                    time_sent: message.0.datetime,
                                    image: None,
                                    message_chunks: message.1.clone(),
                                });

                                // See if we've been mentioned
                                for chunk in message.1 {
                                    if let Some(id) = chunk.1 {
                                        if id == self.user_id.unwrap() {
                                            // If we are currently in this channel, don't mark a pending mention
                                            if let Some(current_channel) =
                                                &self.current_text_channel
                                            {
                                                if current_channel.0 != *channel.get_id() {
                                                    channel.pending_mention = true;
                                                }
                                            }
                                        }
                                    }
                                }

                                // If this user appeared to be typing, they shouldn't be anymore
                                // since a message was just sent. So remove them from the typing list
                                let index = channel
                                    .users_typing
                                    .iter()
                                    .position(|&u| u.0 == message.0.user_id);

                                // Remove the old entry if there is one
                                if let Some(i) = index {
                                    channel.users_typing.remove(i);
                                }
                            }

                            // Add this to the chat history if we're in that channel
                            if let Some(current_channel) = &self.current_text_channel {
                                if let Some(current_realm) = &self.current_realm_id {
                                    if current_channel.0 == message.0.channel_id
                                        && current_realm == &message.0.realm_id
                                    {
                                        self.chat_history.items.push(message.0.message_id);

                                        // If we aren't scrolling through messages,
                                        // move the offset down to the end
                                        if self.input_mode != InputMode::Chat {
                                            self.chat_history.select_last();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    MessageType::Reply(message) => {
                        // Add this message to its respective channel's history
                        // Get our realm
                        if let Some(realm) = self.realms_manager.get_realm_mut(message.0.realm_id) {
                            // Get this text channel
                            if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id)
                            {
                                // Add this message to our that channel's chat history
                                channel.chat_history.push(TextChannelMessage {
                                    message_id: message.0.message_id,
                                    user_id: message.0.user_id,
                                    target_reply_message_id: Some(message.1),
                                    time_sent: message.0.datetime,
                                    image: None,
                                    message_chunks: message.2.clone(),
                                });

                                // See if we've been mentioned
                                for chunk in message.2 {
                                    if let Some(id) = chunk.1 {
                                        if id == self.user_id.unwrap() {
                                            // If we are currently in this channel, don't mark a pending mention
                                            if let Some(current_channel) =
                                                &self.current_text_channel
                                            {
                                                if current_channel.0 != *channel.get_id() {
                                                    channel.pending_mention = true;
                                                }
                                            }
                                        }
                                    }
                                }

                                // If this user appeared to be typing, they shouldn't be anymore
                                // since a message was just sent. So remove them from the typing list
                                let index = channel
                                    .users_typing
                                    .iter()
                                    .position(|&u| u.0 == message.0.user_id);

                                // Remove the old entry if there is one
                                if let Some(i) = index {
                                    channel.users_typing.remove(i);
                                }
                            }

                            // Add this to the chat history if we're in that channel
                            if let Some(current_channel) = &self.current_text_channel {
                                if let Some(current_realm) = &self.current_realm_id {
                                    if current_channel.0 == message.0.channel_id
                                        && current_realm == &message.0.realm_id
                                    {
                                        self.chat_history.items.push(message.0.message_id);

                                        // If we aren't scrolling through messages,
                                        // move the offset down to the end
                                        if self.input_mode != InputMode::Chat {
                                            self.chat_history.select_last();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    MessageType::Image(_message) => {
                        // For now, don't add the message we've received
                        // Better logic is needed on how to save and display an image

                        // Add this message to its respective channel's history
                        // Get our realm
                        // if let Some(realm) = self.realms_manager.get_realm_mut(message.0.realm_id) {
                        //     // Get this text channel
                        //     if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id)
                        //     {
                        //         // Add this message to our that channel's chat history
                        //         channel.push_image(message.0.user_id, message.1);
                        //     }
                        // }
                    }
                    MessageType::Realms(realms) => {
                        //First clear our realms manager
                        self.realms_manager.clear();

                        // Clear our current list of realms and channels
                        self.realms = StatefulList::default();

                        for realm in &realms {
                            // Save realm id and names
                            self.realms.items.push((realm.id, realm.name.clone()));

                            // Save text channel id and names
                            self.text_channels = StatefulList::default();
                            let text_channels = realm.get_text_channels();
                            for (id, mut name) in text_channels {
                                let hashtag = String::from("# ");
                                name.insert_str(0, &hashtag);
                                self.text_channels.items.push((id, name.clone()));
                            }

                            // Save voice channel id and names
                            self.voice_channels = StatefulList::default();
                            let voice_channels = realm.get_voice_channels();
                            for (id, name) in voice_channels {
                                self.voice_channels.items.push((id, name, Vec::new()));
                            }

                            // Auto-join the first available text channel if one wasn't already joined
                            if self.current_text_channel.is_none()
                                && !self.text_channels.items.is_empty()
                            {
                                self.join_channel(
                                    realm.id,
                                    ChannelType::TextChannel,
                                    self.text_channels.items[0].0,
                                )
                                .await;
                            }
                        }
                    }
                    MessageType::RealmsManager(rm) => {
                        // First clear everything that we know
                        self.realms.items.clear();
                        self.text_channels.items.clear();
                        self.voice_channels.items.clear();

                        // Now move this new RealmsManager into our app
                        self.realms_manager = rm;

                        // Now that we have all realms and channels,
                        // let's update references to them to be displayed
                        for realm in self.realms_manager.get_realms() {
                            // Update our Realms list
                            self.realms.items.push((*realm.0, realm.1.clone()));
                        }

                        // For now, let's initally join the first text channel of the first realm
                        if !self.realms.items.is_empty() {
                            self.current_realm_id = Some(self.realms.items[0].0);
                            self.enter_realm(self.current_realm_id.unwrap()).await;
                        }
                    }
                    MessageType::RealmAdded(ra) => {
                        // Add this realm to our list of realms
                        self.realms_manager.add_realm_with_id(ra.0, ra.1);
                        self.refresh_realms_list();
                    }
                    MessageType::RealmRemoved(rr) => {
                        // If we are in this realm, stop viewing it
                        if let Some(realm_id) = &self.current_realm_id {
                            if realm_id == &rr {
                                self.current_realm_id = None;
                                self.current_text_channel = None;
                                if self.is_voice_connected {
                                    self.hang_up().await;
                                }
                                self.current_voice_channel = None;

                                self.chat_history.items.clear();
                                self.chat_history.unselect();
                                self.forget_text_channels();
                                self.forget_voice_channels();

                                if self.input_mode == InputMode::Editing {
                                    self.input_mode = InputMode::Normal;
                                }
                            }
                        }

                        // Now we can remove this realm from our realms
                        self.realms_manager.remove_realm(rr);

                        self.refresh_realms_list();
                    }
                    MessageType::ChannelAdded(ca) => {
                        // Add this new channel to the proper realm
                        self.realms_manager.add_channel_with_id(
                            ca.0,
                            ca.2,
                            ca.1.clone(),
                            ca.3.clone(),
                        );

                        // Refresh this realm if we're in it
                        // Otherwise the realm will be refreshed when it is joined again
                        if let Some(realm_id) = self.current_realm_id {
                            if realm_id == ca.0 {
                                self.refresh_realm(realm_id).await;
                            }
                        }
                    }
                    MessageType::ChannelRemoved(cr) => {
                        // Need to account for when we're in a voice channel and that channel is removed
                        // here

                        // Add this new channel to the proper realm
                        self.realms_manager.remove_channel(cr.0, cr.1, cr.2);

                        // Refresh this realm if we're in it
                        // Otherwise the realm will be refreshed when it is joined again
                        if let Some(realm_id) = self.current_realm_id {
                            if let Some(channel) = &self.current_text_channel {
                                if channel.0 == cr.2 {
                                    self.current_text_channel = None;
                                }
                            }

                            if realm_id == cr.0 {
                                self.refresh_realm(realm_id).await;
                            }
                        }
                    }
                    MessageType::NewFriendRequest(nfr) => {
                        // Add this user id to our list of requests
                        // Don't add it twice (need to prevent repeated friend requests)
                        if !self.friend_requests.contains(&nfr.0.user_id) {
                            self.friend_requests.push(nfr.0.user_id);
                        }
                    }
                    MessageType::FriendshipEnded(fe) => {
                        // Remove this old friend from our list of friends
                        let index = self.friends.iter().position(|id| *id == fe.user_id);

                        if let Some(index) = index {
                            self.friends.remove(index);
                        }
                    }
                    MessageType::Typing(typing) => {
                        // Add this to our list of users typing
                        if let Some(realm) = self.realms_manager.get_realm_mut(typing.realm_id) {
                            // Get this text channel
                            if let Some(channel) = realm.get_text_channel_mut(typing.channel_id) {
                                let index = channel
                                    .users_typing
                                    .iter()
                                    .position(|&u| u.0 == typing.user_id);

                                // Remove the old entry if there is one
                                if let Some(i) = index {
                                    channel.users_typing.remove(i);
                                }

                                channel.users_typing.push((typing.user_id, Utc::now()));
                            }
                        }
                    }
                    MessageType::Disconnect => {
                        self.quit().await;
                    }
                    _ => (),
                };
            }

            // Render the user interface
            tui.draw(self)?;

            // Handle events
            match tui.events.next()? {
                Event::Tick => self.tick(),
                Event::Key(key_event) => handle_key_events(key_event, self).await?,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        // Exit the user interface
        tui.exit()?;

        Ok(())
    }

    pub fn get_username_from_id(&self, user_id: UserIdSize) -> String {
        match self.user_id_to_username.get(&user_id) {
            Some(username) => username.to_string(),
            None => user_id.to_string(),
        }
    }

    // Join the client to a text or voice channel
    pub async fn join_channel(
        &mut self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
        match channel_type {
            ChannelType::TextChannel => {
                self.client
                    .join_channel(realm_id, channel_type, channel_id)
                    .await;

                // Update our current text channel
                for channel in &self.text_channels.items {
                    if channel.0 == channel_id {
                        self.current_text_channel = Some((channel_id, channel.1.clone()));

                        // Now that we're in this text channel, unmark any pending mentions
                        if let Some(realm) = self.realms_manager.get_realm_mut(realm_id) {
                            if let Some(channel) = realm.get_text_channel_mut(channel_id) {
                                channel.pending_mention = false;

                                self.chat_history.items.clear();
                                self.chat_history.unselect();

                                // Populate our chat history with chat messages
                                for message in &channel.chat_history {
                                    self.chat_history.items.push(message.message_id);
                                }
                            }
                        }

                        break;
                    }
                }
            }
            ChannelType::VoiceChannel => {
                self.client
                    .join_channel(realm_id, channel_type, channel_id)
                    .await;

                // Let the voices be heard
                self.connect_voice(realm_id, channel_id).await;
            }
        }
    }

    pub async fn refresh_realm(&mut self, realm_id: RealmIdSize) {
        if let Some(realm) = self.realms_manager.get_realm(realm_id) {
            // Update our text channels list
            self.text_channels.items.clear();
            for text_channel in realm.get_text_channels() {
                self.text_channels.items.push((
                    *text_channel.0,
                    text_channel
                        .1
                        .get_name()
                        .to_string()
                        .add_hashtag_with_space(),
                ));
            }

            // Update our voice channels list
            self.voice_channels.items.clear();
            for voice_channel in realm.get_voice_channels() {
                self.voice_channels.items.push((
                    *voice_channel.0,
                    voice_channel.1.get_name().to_string(),
                    voice_channel.1.get_connected_users().clone(),
                ));
            }

            // If the channel we're viewing was removed, stop viewing it
            if let Some(text_channel) = &self.current_text_channel {
                if !self
                    .text_channels
                    .items
                    .iter()
                    .any(|c| c.0 == text_channel.0)
                {
                    if !self.text_channels.items.is_empty() {
                        self.current_text_channel = Some((
                            self.text_channels.items[0].0,
                            self.text_channels.items[0].1.clone(),
                        ));
                    } else {
                        self.current_text_channel = None;
                    }
                }
            }

            if let Some(voice_channel) = &self.current_voice_channel {
                if !self
                    .voice_channels
                    .items
                    .iter()
                    .any(|c| &c.0 == voice_channel)
                {
                    if !self.voice_channels.items.is_empty() {
                        self.current_voice_channel = Some(self.voice_channels.items[0].0);
                    } else {
                        self.hang_up().await;
                        self.current_text_channel = None;
                    }
                }
            }
        }
    }

    pub fn refresh_realms_list(&mut self) {
        // First clear current realms
        self.realms.items.clear();

        for realm in self.realms_manager.get_realms() {
            // Update our Realms list
            self.realms.items.push((*realm.0, realm.1.clone()));
        }

        // If we were previously in a realm, stay in that realm
        if let Some(realm_id) = &self.current_realm_id {
            if !self.realms.items.iter().any(|c| &c.0 == realm_id) {
                if !self.realms.items.is_empty() {
                    self.current_realm_id = Some(self.current_realm_id.unwrap());
                } else {
                    self.current_realm_id = None;
                }
            }
        }
    }

    pub async fn enter_realm(&mut self, realm_id: RealmIdSize) {
        if let Some(realm) = self.realms_manager.get_realm(realm_id) {
            // Update our text channels list
            self.text_channels.items.clear();
            for text_channel in realm.get_text_channels() {
                self.text_channels.items.push((
                    *text_channel.0,
                    text_channel
                        .1
                        .get_name()
                        .to_string()
                        .add_hashtag_with_space(),
                ));
            }

            // Update our voice channels list
            self.voice_channels.items.clear();
            for voice_channel in realm.get_voice_channels() {
                self.voice_channels.items.push((
                    *voice_channel.0,
                    voice_channel.1.get_name().to_string(),
                    voice_channel.1.get_connected_users().clone(),
                ));
            }

            // Join the first saved text channel
            if !self.text_channels.items.is_empty() {
                self.join_channel(
                    realm_id,
                    ChannelType::TextChannel,
                    self.text_channels.items[0].0,
                )
                .await;
            } else {
                self.current_text_channel = None;
            }

            self.current_realm_id = Some(realm_id);
        }
    }

    pub async fn connect_voice(&mut self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {
        self.client.connect_voice(realm_id, channel_id).await;
    }

    pub async fn hang_up(&mut self) {
        if let Some(channel) = self.current_voice_channel {
            self.client
                .hang_up(self.current_realm_id.as_ref().unwrap(), &channel)
                .await;

            self.is_voice_connected = false;
            self.current_voice_channel = None;
        }
    }

    pub async fn handle_input(&mut self) {
        // First check to see if this is a command message
        match self.current_command {
            Some(command) => match command {
                Command::Image => {
                    self.send_image().await;
                }
            },
            None => {
                if self.reply_target_message_id.is_some() {
                    self.client
                        .send_reply_message(
                            self.current_realm_id.unwrap(),
                            self.current_text_channel.as_ref().unwrap().0,
                            self.reply_target_message_id.unwrap(),
                            self.input_buffer.get_input_without_style(),
                        )
                        .await;
                } else {
                    self.client
                        .send_mention_message(
                            self.current_realm_id.unwrap(),
                            self.current_text_channel.as_ref().unwrap().0,
                            self.input_buffer.get_input_without_style(),
                        )
                        .await;
                }
            }
        }

        self.current_command = None;
    }

    pub async fn send_image(&mut self) {
        // First check to see if the image exists
        if let Some(input) = self.input_buffer.input.last() {
            let path = input.0.as_str();
            if Path::new(path).exists() {
                // Check the size of the file. Don't send it if it's more than 10MB
                let metadata = std::fs::metadata(path).unwrap();
                if metadata.len() > 10000000 {
                    self.general_popup.setup(
                        Some(String::from("Image Error")),
                        Some(String::from("File size exceeds 10MB")),
                    );
                    self.show_popup(PopupType::General);
                } else {
                    let image = std::fs::read(path);
                    match image {
                        Ok(img) => {
                            self.client
                                .send_image(
                                    self.current_realm_id.unwrap(),
                                    self.current_text_channel.as_ref().unwrap().0,
                                    img,
                                )
                                .await;
                        }
                        Err(_) => {
                            self.general_popup.setup(
                                Some(String::from("Image Error")),
                                Some(String::from("Failed to load file")),
                            );
                            self.show_popup(PopupType::General);
                        }
                    }
                }
            } else {
                self.general_popup.setup(
                    Some(String::from("Image Error")),
                    Some(format!("{} does not exist", path)),
                );
                self.show_popup(PopupType::General);
            }
        }
    }

    pub fn show_popup(&mut self, popup_type: PopupType) {
        self.popup_type = popup_type;
        self.input_mode = InputMode::Popup;
        self.is_popup_shown = true;
    }

    pub fn dismiss_popup(&mut self) {
        self.is_popup_shown = false;
        self.input_mode = InputMode::Normal;
        self.current_pane = Pane::ChatPane;
        self.popup_type = PopupType::General;
        self.popup_title = String::new();
        self.popup_text = String::new();
    }

    pub fn show_yes_no_popup(&mut self, title: String, message: String) {
        self.yes_no_popup.setup(Some(title), Some(message));
        self.show_popup(PopupType::YesNo);
    }

    pub fn show_add_channel_popup(&mut self) {
        self.add_channel_popup.setup(None, None);
        self.show_popup(PopupType::AddChannel);
    }

    pub fn show_add_realm_popup(&mut self) {
        self.add_realm_popup.setup(None, None);
        self.show_popup(PopupType::AddRealm);
    }

    pub fn show_remove_realm_popup(&mut self, realm_id: RealmIdSize, realm_name: String) {
        self.remove_realm_popup.setup(None, None);
        self.remove_realm_popup.realm_id = realm_id;
        self.remove_realm_popup.realm_name = realm_name;
        self.show_popup(PopupType::RemoveRealm);
    }

    pub fn show_remove_channel_popup(
        &mut self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
        channel_name: String,
    ) {
        self.remove_channel_popup.setup(None, None);
        self.remove_channel_popup.realm_id = realm_id;
        self.remove_channel_popup.channel_type = channel_type;
        self.remove_channel_popup.channel_id = channel_id;
        self.remove_channel_popup.channel_name = channel_name;
        self.show_popup(PopupType::RemoveChannel)
    }

    pub fn show_member_popup(
        &mut self,
        user_id: UserIdSize,
        username: String,
        selected_index: usize,
    ) {
        self.member_popup.setup(None, None);
        self.member_popup.selected_index = selected_index;
        self.member_popup.user_id = user_id;
        self.member_popup.username = username;

        // First check to see if we're friends with this user
        // or if we have any pending requests for them
        self.member_popup.is_friend = self.friends.contains(&user_id);
        self.member_popup.is_request_pending = self.pending_friend_requests.contains(&user_id);

        // Now show the popup
        self.show_popup(PopupType::Member);
    }

    pub fn get_current_time_string(&self) -> String {
        let local_time = Local::now();
        local_time.format("%H:%M").to_string()
    }

    pub async fn add_channel(&mut self, channel_type: ChannelType, channel_name: String) {
        self.client
            .add_channel(self.current_realm_id.unwrap(), channel_type, channel_name)
            .await;
    }

    pub async fn remove_channel(&mut self, channel_type: ChannelType, channel_id: ChannelIdSize) {
        self.client
            .remove_channel(self.current_realm_id.unwrap(), channel_type, channel_id)
            .await;
    }

    pub async fn add_realm(&mut self, realm_name: String) {
        self.client.add_realm(realm_name).await;
    }

    pub async fn remove_realm(&mut self, realm_id: RealmIdSize) {
        self.client.remove_realm(realm_id).await;
    }

    pub async fn add_friend(&mut self, friend_id: UserIdSize) {
        self.client.add_friend(friend_id).await;

        self.pending_friend_requests.push(friend_id);
    }

    pub async fn remove_friend(&mut self, friend_id: UserIdSize) {
        // Remove this old friend from our list of friends
        let index = self.friends.iter().position(|id| *id == friend_id);

        if let Some(index) = index {
            self.client.remove_friend(friend_id).await;
            self.friends.remove(index);
        }
    }

    pub async fn send_typing(&mut self) {
        let mut send = false;

        // If we don't have a time set, set it
        match self.time_started_typing {
            Some(time) => {
                // Check the last time we sent a typing message
                // Don't send it if it's been more than five seconds
                let seconds_difference = Utc::now().signed_duration_since(time).num_seconds();
                if seconds_difference > 4 {
                    send = true;
                }
            }
            None => {
                send = true;
            }
        }

        if send {
            if let Some(realm_id) = self.current_realm_id {
                if let Some(channel) = &self.current_text_channel {
                    self.client.send_typing(realm_id, channel.0).await;
                    self.time_started_typing = Some(Utc::now());
                }
            }
        }
    }

    pub fn forget_text_channels(&mut self) {
        self.text_channels.items.clear();
        self.text_channels.unselect();
    }

    pub fn forget_voice_channels(&mut self) {
        self.voice_channels.items.clear();
        self.voice_channels.unselect();
    }

    pub fn begin_editing(&mut self) {
        if self.current_text_channel.is_some() {
            self.voice_channels.unselect();
            self.text_channels.unselect();
            self.input_mode = InputMode::Editing;
            self.current_pane = Pane::InputPane;
            self.ui_element = UiElement::None;
        }
    }
}
