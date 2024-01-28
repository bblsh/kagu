use message::message::Message;
use realms::realm::ChannelType;
use types::*;

use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug)]
pub struct NewClient {
    server_address: SocketAddr,
    username: String,
    cert_dir: PathBuf,
}

impl NewClient {
    pub fn new(server_address: SocketAddr, username: String, cert_dir: PathBuf) -> NewClient {
        NewClient {
            server_address,
            username,
            cert_dir,
        }
    }

    pub fn run_client(&self) {
        //
    }

    pub fn get_username(&self) -> Option<String> {
        None
    }

    pub fn get_new_messages(&self) -> Vec<Message> {
        Vec::new()
    }

    pub fn get_user_id(&self) -> Option<UserIdSize> {
        None
    }

    pub fn disconnect(&mut self) {}

    pub fn send_mention_message(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        message_chunks: Vec<(String, Option<UserIdSize>)>,
    ) {
    }

    pub fn send_reply_message(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        message_id: MessageIdSize,
        message_chunks: Vec<(String, Option<UserIdSize>)>,
    ) {
    }

    pub fn send_image(&self, realm_id: RealmIdSize, channel_id: ChannelIdSize, image: Vec<u8>) {}

    pub fn join_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
    }

    pub fn connect_voice(&mut self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {}

    pub fn add_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_name: String,
    ) {
    }

    pub fn remove_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
    }

    pub fn add_realm(&self, realm_name: String) {}

    pub fn remove_realm(&self, realm_id: RealmIdSize) {}

    pub fn add_friend(&self, friend_id: UserIdSize) {}

    pub fn remove_friend(&self, friend_id: UserIdSize) {}

    pub fn send_typing(&self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {}

    pub fn hang_up(&self, realm_id: &RealmIdSize, channel_id: &ChannelIdSize) {}
}
