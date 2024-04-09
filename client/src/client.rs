use crate::client_handler::ClientHandler;
use crate::client_message::ClientMessage;
use audio::audio_manager::AudioManager;
use message::message::{Message, MessageHeader, MessageType};
use network_manager::*;
use realms::realm::ChannelType;
use types::*;
use user::User;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use crossbeam::channel::{Receiver, Sender};
use swiftlet_quic::endpoint::{Config, Endpoint};
use swiftlet_quic::EndpointHandler;

#[derive(Debug)]
pub struct Client {
    server_address: SocketAddr,
    username: String,
    user: Option<User>,
    cert_dir: PathBuf,
    audio_manager: AudioManager,
    incoming_sender: Sender<Message>,
    incoming_receiver: Receiver<Message>,
    outgoing_sender: Sender<Message>,
    outgoing_receiver: Receiver<Message>,
    audio_in_sender: Sender<Message>,

    // Channel used to send messages from the inner event loop to this client
    el_to_client_sender: Sender<ClientMessage>,
    el_to_client_receiver: Receiver<ClientMessage>,
    client_to_el_receiver: Receiver<ClientMessage>,
    client_to_el_sender: Sender<ClientMessage>,

    is_connected: bool,

    // Handle to the event loop thread
    event_loop_handle: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new(server_address: SocketAddr, username: String, cert_dir: PathBuf) -> Client {
        let (outgoing_sender, outgoing_receiver): (Sender<Message>, Receiver<Message>) =
            crossbeam::channel::bounded(10);

        let (incoming_sender, incoming_receiver): (Sender<Message>, Receiver<Message>) =
            crossbeam::channel::bounded(10);

        let (audio_in_sender, audio_in_receiver): (Sender<Message>, Receiver<Message>) =
            crossbeam::channel::bounded(10);

        let (el_to_client_sender, el_to_client_receiver): (
            Sender<ClientMessage>,
            Receiver<ClientMessage>,
        ) = crossbeam::channel::bounded(1);

        let (client_to_el_sender, client_to_el_receiver): (
            Sender<ClientMessage>,
            Receiver<ClientMessage>,
        ) = crossbeam::channel::bounded(1);

        Client {
            server_address,
            username,
            user: None,
            cert_dir,

            audio_manager: AudioManager::new(
                // Use our outgoing sender for all messages as the audio sender
                outgoing_sender.clone(),
                audio_in_receiver,
                // Set a dummy MessageHeader for now
                MessageHeader::new(0, 0, 0),
            ),

            incoming_sender,
            incoming_receiver,
            outgoing_sender,
            outgoing_receiver,
            audio_in_sender,
            el_to_client_sender,
            el_to_client_receiver,
            is_connected: false,
            event_loop_handle: None,
            client_to_el_receiver,
            client_to_el_sender,
        }
    }

    pub fn run_client(&mut self) {
        let bind_address = match self.server_address.is_ipv6() {
            true => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)),
            false => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
        };

        let server_address = self.server_address;
        let outgoing_receiver = self.outgoing_receiver.clone();
        let incoming_sender = self.incoming_sender.clone();
        let audio_in_sender = self.audio_in_sender.clone();
        let el_to_client_sender = self.el_to_client_sender.clone();
        let client_to_el_receiver = self.client_to_el_receiver.clone();

        let config = Config {
            idle_timeout_in_ms: 5000,
            reliable_stream_buffer: 65536,
            unreliable_stream_buffer: 65536,
            keep_alive_timeout: Some(std::time::Duration::from_millis(2000)),
            initial_main_recv_size: BUFFER_SIZE_PER_CONNECTION,
            main_recv_first_bytes: MESSAGE_HEADER_SIZE,
            initial_background_recv_size: BUFFER_SIZE_PER_CONNECTION,
            background_recv_first_bytes: MESSAGE_HEADER_SIZE,
            initial_rt_recv_size: 65536,
            rt_recv_first_bytes: 0,
        };

        let (cert, _pkey) = self.get_pem_paths(&self.cert_dir);

        let client_thread = std::thread::spawn(move || {
            let mut client_endpoint = match Endpoint::new_client_with_first_connection(
                bind_address,
                b"kagu",
                cert.as_str(),
                server_address,
                "localhost",
                config,
            ) {
                Ok(endpoint) => endpoint,
                Err(_) => {
                    eprintln!("Failed to create client endpoint");
                    return;
                }
            };

            let mut client_handler = ClientHandler::new(
                outgoing_receiver,
                incoming_sender,
                audio_in_sender,
                el_to_client_sender,
                client_to_el_receiver,
            );
            let mut rtc_handler = EndpointHandler::new(&mut client_endpoint, &mut client_handler);

            match rtc_handler.run_event_loop(std::time::Duration::from_millis(5)) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error running event loop: {:?}", e)
                }
            }
        });

        self.event_loop_handle = Some(client_thread);
    }

    fn get_pem_paths(&self, cert_dir: &Path) -> (String, String) {
        let mut cert = cert_dir.to_str().unwrap().to_string();
        cert.push_str("/cert.pem");

        let mut pkey = cert_dir.to_str().unwrap().to_string();
        pkey.push_str("/pkey.pem");

        (cert, pkey)
    }

    fn send(&self, message: Message) {
        self.outgoing_sender.send(message).unwrap();
    }

    pub fn is_connected(&self) -> bool {
        match self.is_connected {
            true => true,
            false => self.el_to_client_receiver.try_recv().is_ok(),
        }
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
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

    pub fn disconnect(&mut self) {
        // todo: can't disconnect if we are connected but not logged in
        if let Some(user) = &self.user {
            let message = Message::from(MessageType::Disconnecting(user.get_id()));
            self.send(message);
        }

        // Wait for the event loop thread to exit
        // At this point the event loop should have closed our connection
        if let Some(handle) = self.event_loop_handle.take() {
            let _ = handle.join(); // todo: handle errors?
        }
    }

    pub fn log_in(&self) {
        let message = Message::from(MessageType::LoginAttempt(self.username.clone()));
        self.send(message);
    }

    pub fn send_mention_message(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        message_chunks: Vec<(String, Option<UserIdSize>)>,
    ) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);
            let message = Message::from(MessageType::Text((header, message_chunks)));
            self.send(message);
        }
    }

    pub fn send_reply_message(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        message_id: MessageIdSize,
        message_chunks: Vec<(String, Option<UserIdSize>)>,
    ) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);
            let message = Message::from(MessageType::Reply((header, message_id, message_chunks)));
            self.send(message);
        }
    }

    pub fn send_image(&self, _realm_id: RealmIdSize, _channel_id: ChannelIdSize, _image: Vec<u8>) {}

    pub fn join_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);

            match channel_type {
                ChannelType::TextChannel => (), // Do nothing for now
                ChannelType::VoiceChannel => {
                    let message = Message::from(MessageType::UserJoinedVoiceChannel(header));
                    self.send(message);
                }
            }
        }
    }

    pub fn connect_voice(&mut self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {
        if let Some(user) = &self.user {
            // Set our header for what channel we're connecting to
            self.audio_manager
                .set_header(MessageHeader::new(user.get_id(), realm_id, channel_id));

            // Start recording and sending
            match self.audio_manager.start_recording() {
                Ok(_) => (),
                Err(_) => println!("FAILED TO START LISTENING"),
            }

            // Let the voices be heard
            match self.audio_manager.start_listening() {
                Ok(_) => (),
                Err(_) => println!("FAILED TO START RECORDING"),
            }
        }
    }

    pub fn play_audio_file(&self, file: String) {
        let mut audio = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        audio.push("../assets/audio/");
        audio.push(file.as_str());

        self.audio_manager
            .play_audio_file(String::from(audio.to_str().unwrap()));
    }

    pub fn add_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_name: String,
    ) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, 0);
            let message = Message::from(MessageType::AddChannel((
                header,
                channel_type,
                channel_name,
            )));
            self.send(message);
        }
    }

    pub fn remove_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);
            let message = Message::from(MessageType::RemoveChannel((header, channel_type)));
            self.send(message);
        }
    }

    pub fn add_realm(&self, realm_name: String) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), 0, 0);
            let message = Message::from(MessageType::AddRealm((header, realm_name)));
            self.send(message);
        }
    }

    pub fn remove_realm(&self, realm_id: RealmIdSize) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), 0, 0);
            let message = Message::from(MessageType::RemoveRealm((header, realm_id)));
            self.send(message);
        }
    }

    pub fn add_friend(&self, friend_id: UserIdSize) {
        if let Some(our_user) = &self.user {
            let header = MessageHeader::new(our_user.get_id(), 0, 0);
            let message = Message::from(MessageType::NewFriendRequest((header, friend_id)));
            self.send(message);
        }
    }

    pub fn remove_friend(&self, friend_id: UserIdSize) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), 0, 0);
            let message = Message::from(MessageType::RemoveFriend((header, friend_id)));
            self.send(message);
        }
    }

    pub fn send_typing(&self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);
            let message = Message::from(MessageType::Typing(header));
            self.send(message);
        }
    }

    pub fn hang_up(&mut self, realm_id: RealmIdSize, channel_id: ChannelIdSize) {
        self.audio_manager.stop_recording();
        self.audio_manager.stop_listening();

        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), realm_id, channel_id);
            let message = Message::from(MessageType::UserLeftVoiceChannel(header));
            self.send(message);
        }
    }

    pub fn get_realms(&self) {
        if let Some(user) = &self.user {
            let message = Message::from(MessageType::GetRealms(user.get_id()));
            self.send(message);
        }
    }

    pub fn get_all_users(&self) {
        if let Some(user) = &self.user {
            let message = Message::from(MessageType::GetAllUsers(MessageHeader::new(
                user.get_id(),
                0,
                0,
            )));
            self.send(message);
        }
    }

    pub fn get_audio_inputs(&self) -> Vec<String> {
        self.audio_manager.get_audio_inputs()
    }

    pub fn get_audio_outputs(&self) -> Vec<String> {
        self.audio_manager.get_audio_outputs()
    }

    pub fn set_audio_input(&mut self, input_name: String) {
        self.audio_manager.set_audio_input(input_name)
    }

    pub fn set_audio_output(&mut self, output_name: String) {
        self.audio_manager.set_audio_output(output_name);
    }

    /// This is to be called prior to transferring a file
    pub fn request_file_upload(&self) {
        if let Some(user) = &self.user {
            let header = MessageHeader::new(user.get_id(), 0, 0);
            let message = Message::from(MessageType::FileTransferRequest(header));
            self.send(message);
        }
    }

    /// To upload a file, `request_file_upload()` must first be called.
    /// Once a FileTransferApproved message is received this may be called.
    pub fn upload_file(&self, transfer_id: FileTransferIdSize, file_path: PathBuf) {}

    /// Audio sent using this should be sampled at 48000Hz and in 10ms chunks
    pub fn send_audio_frame(&self, mut header: MessageHeader, audio: Vec<u8>) {
        if let Some(_user) = &self.user {
            header.datetime = Some(chrono::Utc::now());
            self.send(Message::from(MessageType::Audio((header, audio))));
        }
    }
}
