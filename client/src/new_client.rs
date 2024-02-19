use crate::client_handler::ClientHandler;
use message::message::{Message, MessageType};
use network_manager::*;
use realms::realm::ChannelType;
use types::*;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;
use std::thread;

use crossbeam::channel::{Receiver, Sender};
use swiftlet_quic::endpoint::{Config, Endpoint};
use swiftlet_quic::Handler;

#[derive(Debug)]
pub struct NewClient {
    server_address: SocketAddr,
    username: String,
    cert_dir: PathBuf,
    incoming_sender: Sender<Message>,
    incoming_receiver: Receiver<Message>,
    outgoing_sender: Sender<Message>,
    outgoing_receiver: Receiver<Message>,
}

impl NewClient {
    pub fn new(server_address: SocketAddr, username: String, cert_dir: PathBuf) -> NewClient {
        let (outgoing_sender, outgoing_receiver): (Sender<Message>, Receiver<Message>) =
            crossbeam::channel::bounded(10);

        let (incoming_sender, incoming_receiver): (Sender<Message>, Receiver<Message>) =
            crossbeam::channel::bounded(10);

        NewClient {
            server_address,
            username,
            cert_dir,
            incoming_sender,
            incoming_receiver,
            outgoing_sender,
            outgoing_receiver,
        }
    }

    pub fn run_client(&self) {
        let bind_address = match self.server_address.is_ipv6() {
            true => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)),
            false => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
        };

        let server_address = self.server_address;
        let cert_path = self.cert_dir.clone();
        let outgoing_receiver = self.outgoing_receiver.clone();
        let incoming_sender = self.incoming_sender.clone();

        let config = Config {
            idle_timeout_in_ms: 5000,
            reliable_stream_buffer: 65536,
            unreliable_stream_buffer: 65536,
            keep_alive_timeout: Some(std::time::Duration::from_millis(2000)),
            initial_main_recv_size: BUFFER_SIZE_PER_CONNECTION,
            main_recv_first_bytes: MESSAGE_HEADER_SIZE,
            initial_background_recv_size: BUFFER_SIZE_PER_CONNECTION,
            background_recv_first_bytes: MESSAGE_HEADER_SIZE,
        };

        let _client_thread = thread::spawn(move || {
            let client_endpoint = match Endpoint::new_client_with_first_connection(
                bind_address,
                b"kagu",
                cert_path.to_str().unwrap(),
                server_address,
                "localhost",
                config,
            ) {
                Ok(endpoint) => endpoint,
                Err(_) => {
                    eprintln!("Failed to create client endpoint");
                    // Can add more detailed print here later
                    return;
                }
            };

            let mut client_handler = ClientHandler::new(outgoing_receiver, incoming_sender);
            let mut rtc_handler = Handler::new(client_endpoint, &mut client_handler);

            match rtc_handler.run_event_loop(std::time::Duration::from_millis(5)) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Error running event loop?")
                }
            }
        });
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_new_messages(&self) -> Vec<Message> {
        match self.incoming_receiver.is_empty() {
            true => Vec::new(),
            false => {
                let mut messages = Vec::new();

                while let Ok(message) = self.incoming_receiver.try_recv() {
                    messages.push(message);
                }

                messages
            }
        }
    }

    pub fn get_user_id(&self) -> Option<UserIdSize> {
        None
    }

    pub fn disconnect(&mut self) {}

    pub fn log_in(&self, username: String) {
        let message = Message::from(MessageType::LoginAttempt(username));
        let _ = self.outgoing_sender.send(message);
    }

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

    pub fn get_realms(&self) {}

    pub fn request_all_users(&self) {}
}
