use crate::text_channel::TextChannel;
use crate::types::{ChannelIdSize, RealmIdSize, UserIdSize};
use crate::user::User;
use crate::voice_channel::VoiceChannel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ChannelTypes is an enum to describe a type of channel within a Realm
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ChannelType {
    TextChannel,
    VoiceChannel,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Realm {
    pub id: RealmIdSize,
    pub name: String,
    pub users: HashMap<UserIdSize, User>,
    pub text_channels: HashMap<ChannelIdSize, TextChannel>,
    pub voice_channels: HashMap<ChannelIdSize, VoiceChannel>,
}

impl Realm {
    pub fn new(id: RealmIdSize, realm_name: String) -> Realm {
        Realm {
            id: id,
            name: realm_name,
            users: HashMap::new(),
            text_channels: HashMap::new(),
            voice_channels: HashMap::new(),
        }
    }

    pub fn add_channel(&mut self, channel_type: ChannelType, name: String) {
        match channel_type {
            ChannelType::TextChannel => {
                self.text_channels.insert(
                    self.text_channels.len() as ChannelIdSize,
                    TextChannel::new(self.text_channels.len() as ChannelIdSize, name),
                );
            }
            ChannelType::VoiceChannel => {
                self.voice_channels.insert(
                    self.voice_channels.len() as ChannelIdSize,
                    VoiceChannel::new(self.voice_channels.len() as ChannelIdSize, name),
                );
            }
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> &RealmIdSize {
        &self.id
    }

    pub fn add_user(&mut self, user_id: UserIdSize, user: User) {
        self.users.insert(user_id, user);
    }

    pub fn remove_user(&mut self, user_id: UserIdSize) {
        self.users.remove(&user_id);
    }

    pub fn get_text_channels(&self) -> &HashMap<ChannelIdSize, TextChannel> {
        &self.text_channels
    }

    pub fn get_text_channels_mut(&mut self) -> &mut HashMap<ChannelIdSize, TextChannel> {
        &mut self.text_channels
    }

    pub fn get_text_channel(&self, channel_id: ChannelIdSize) -> Option<&TextChannel> {
        self.text_channels.get(&channel_id)
    }

    pub fn get_text_channel_mut(&mut self, channel_id: ChannelIdSize) -> Option<&mut TextChannel> {
        self.text_channels.get_mut(&channel_id)
    }

    pub fn get_voice_channel(&self, channel_id: ChannelIdSize) -> Option<&VoiceChannel> {
        self.voice_channels.get(&channel_id)
    }

    pub fn get_voice_channel_mut(
        &mut self,
        channel_id: ChannelIdSize,
    ) -> Option<&mut VoiceChannel> {
        self.voice_channels.get_mut(&channel_id)
    }

    pub fn get_voice_channels(&self) -> &HashMap<ChannelIdSize, VoiceChannel> {
        &self.voice_channels
    }

    pub fn get_voice_channels_mut(&mut self) -> &mut HashMap<ChannelIdSize, VoiceChannel> {
        &mut self.voice_channels
    }

    pub fn add_user_to_voice_channel(&mut self, user_id: UserIdSize, channel_id: ChannelIdSize) {
        if let Some(channel) = self.voice_channels.get_mut(&channel_id) {
            channel.connected_users.push(user_id);
        }
    }

    pub fn remove_user_from_voice_channel(
        &mut self,
        user_id: UserIdSize,
        channel_id: ChannelIdSize,
    ) {
        if let Some(channel) = self.voice_channels.get_mut(&channel_id) {
            let index = channel
                .connected_users
                .iter()
                .position(|x| x == &user_id)
                .unwrap();
            channel.connected_users.remove(index);
        }
    }
}
