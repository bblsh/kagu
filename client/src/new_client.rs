use message::message::Message;
use realms::realm::ChannelType;
use types::*;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;

use crossbeam::channel::{Receiver, Sender};

#[derive(Debug)]
pub struct NewClient {
    server_address: SocketAddr,
    username: String,
    cert_dir: PathBuf,
    ui_receiver: Option<Receiver<Message>>,
    message_sender: Option<Sender<Message>>,
}

impl NewClient {
    pub fn new(server_address: SocketAddr, username: String, cert_dir: PathBuf) -> NewClient {
        NewClient {
            server_address,
            username,
            cert_dir,
            ui_receiver: None,
            message_sender: None,
        }
    }

    pub fn run_client(&self) {
        let bind_address = match self.server_address.is_ipv6() {
            true => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)),
            false => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
        };
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
