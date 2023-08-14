use crate::audio_manager::AudioManager;
use crate::message::{Message, MessageHeader, MessageType};
use crate::network_manager::{ConnectionCommand, NetworkManager, ServerOrClient};
use crate::realms::realm::ChannelType;
use crate::realms::realm_desc::RealmDescription;
use crate::types::{ChannelIdSize, RealmIdSize, UserIdSize};
use crate::user::User;
use quinn::{Connection, ConnectionError, Endpoint};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, error::TryRecvError, Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum ClientError {
    NotLoggedIn,
    FailedToJoinChannel,
}

pub enum AudioCommand {
    Start,
    Stop,
    Mute,
    Unmute,
}

#[derive(Debug)]
pub struct Client {
    _endpoint: Endpoint,
    connection: Connection,
    connection_sender: Arc<Mutex<Option<Sender<ConnectionCommand>>>>,
    messages: Arc<Mutex<VecDeque<Message>>>,

    // The current client, as a User
    // This is an option because it's possible this client isn't registered with the server yet
    user: Arc<Mutex<Option<User>>>,

    // This maps usernames to a user id
    user_id_to_username: Arc<Mutex<HashMap<UserIdSize, String>>>,
    username: String,

    // Our known Realms
    realms: Arc<Mutex<Vec<RealmDescription>>>,

    // Sender for audio commands
    audio_sender: Option<Sender<(UserIdSize, Vec<u8>)>>,

    // Audio manager to handle audio recording and playback
    audio_manager: Arc<Mutex<Option<AudioManager>>>,
}

impl Client {
    pub async fn new(server_address: String, username: String) -> Client {
        let endpoint = NetworkManager::new(server_address.clone(), ServerOrClient::Client).await;
        let address: std::net::SocketAddr = server_address.parse().unwrap();

        // Here "localhost" should match the server cert (but this is ignored right now)
        let connect = endpoint.connect(address, "localhost").unwrap();
        let connection = connect.await;

        let connection = match connection {
            Ok(conn) => conn,
            Err(ConnectionError::TimedOut) => {
                eprintln!("[client] Connection timed out. Is the server IP and port correct?");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("[client] Error while connecting: {}", e);
                std::process::exit(1);
            }
        };

        // Generate a sender and receiver for audio data
        let (audio_to_am_tx, audio_to_am_rx): (
            Sender<(UserIdSize, Vec<u8>)>,
            Receiver<(UserIdSize, Vec<u8>)>,
        ) = channel(1000);

        // Make our AudioManager and give it our client's endpoint
        let audio_manager = AudioManager::new()
            .endpoint(endpoint.clone())
            .connection(connection.clone())
            .audio_receiver(audio_to_am_rx);

        Client {
            _endpoint: endpoint,
            connection: connection,
            audio_sender: Some(audio_to_am_tx),
            connection_sender: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
            user: Arc::new(Mutex::new(None)),
            user_id_to_username: Arc::new(Mutex::new(HashMap::new())),
            username: username,
            realms: Arc::new(Mutex::new(Vec::new())),
            audio_manager: Arc::new(Mutex::new(Some(audio_manager))),
        }
    }

    pub async fn get_new_messages(self: &Self) -> Vec<Message> {
        let mut new_messages: Vec<Message> = Vec::new();

        let messages = self.messages.clone();
        let mut messages = messages.lock().await;

        while messages.len() > 0 {
            new_messages.push(messages.pop_front().unwrap());
        }

        new_messages
    }

    pub async fn run_client(self: &Self) {
        self.receive_data().await;
        self.login(self.username.clone()).await;
    }

    async fn login(self: &Self, username: String) {
        let login_message = Message::from(MessageType::LoginAttempt(username));

        Client::send(
            login_message.into_vec_u8().unwrap().as_slice(),
            self.connection.clone(),
        )
        .await;
    }

    pub async fn get_realms(self: &Self) -> Vec<RealmDescription> {
        let mut realms = self.realms.lock().await;

        let mut new_realms = Vec::new();

        while realms.len() > 0 {
            new_realms.push(realms.pop().unwrap());
        }

        new_realms
    }

    pub async fn get_user_id(self: &Self) -> Option<UserIdSize> {
        let guard = self.user.lock().await;
        match *guard {
            Some(ref user) => Some(user.get_id()),
            None => None,
        }
    }

    pub async fn get_username(self: &Self) -> Option<String> {
        let guard = self.user.lock().await;
        match *guard {
            Some(ref user) => Some(user.get_username().to_string()),
            None => None,
        }
    }

    async fn receive_data(self: &Self) {
        let connection = self.connection.clone();
        let messages = self.messages.clone();

        let (tx, mut rx): (Sender<ConnectionCommand>, Receiver<ConnectionCommand>) = channel(1000);

        let connection_sender = self.connection_sender.clone();
        {
            let mut connection_sender = connection_sender.lock().await;
            *connection_sender = Some(tx);
        }

        let user_handle = self.user.clone();
        let id_to_user = self.user_id_to_username.clone();

        let audio_sender = self.audio_sender.clone().unwrap();

        // Spawn a tokio thread to listen for data
        // Here we only need one thread, since there will only be one connection to the server
        tokio::spawn(async move {
            loop {
                // Listen for channel messages to stop listening on this channel
                match rx.try_recv() {
                    Ok(command) => match command {
                        ConnectionCommand::StopReceiving => {
                            break;
                        }
                    },
                    Err(TryRecvError::Empty) => (), // Do nothing here, nothing to receive yet
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("No sender available to receive from");
                        break;
                    }
                }

                let audio_sender = audio_sender.clone();

                let connection = connection.clone();
                let stream = connection.accept_bi().await;
                let _stream = match stream {
                    Ok((_send_stream, mut read_stream)) => {
                        let message = read_stream.read_to_end(12000000).await.unwrap();

                        let mut messages = messages.lock().await;

                        let msg_clone = message.clone();
                        let msg_clone = Message::from_vec_u8(msg_clone).unwrap();

                        // Handle login attempt
                        match msg_clone.get_message() {
                            MessageType::LoginSuccess(user) => {
                                let mut guard = user_handle.lock().await;
                                let id = user.get_id();
                                let username = String::from(user.get_username());
                                *guard = Some(user);

                                // Let's add ourselves to our User to UserIDs so we know who we are
                                let u_t_u_id = id_to_user.clone();
                                let mut u_t_u_id = u_t_u_id.lock().await;
                                u_t_u_id.insert(id.clone(), username);

                                // Now that we've logged in, let's request any realms we're part of
                                Client::send(
                                    Message::from(MessageType::GetRealms(id))
                                        .into_vec_u8()
                                        .unwrap()
                                        .as_slice(),
                                    connection.clone(),
                                )
                                .await;
                            }
                            MessageType::Audio(audio) => {
                                audio_sender.send((audio.0, audio.3)).await.unwrap();
                            }
                            _ => messages.push_back(Message::from_vec_u8(message).unwrap()),
                        }
                    }
                    Err(quinn::ConnectionError::ApplicationClosed(ac)) => {
                        println!(
                            "Connection closed. Code: {}, Reason: {}",
                            ac.error_code,
                            String::from_utf8(ac.reason.to_vec()).unwrap()
                        );
                        break;
                    }
                    Err(quinn::ConnectionError::LocallyClosed) => {
                        break;
                    }
                    _ => {
                        eprintln!("[client] unhandled stream error");
                        break;
                    }
                };
            }
        });
    }

    pub async fn send_text_message(&self, message: &str) -> Result<(), ClientError> {
        let guard = self.user.lock().await;
        match *guard {
            Some(ref user) => {
                let message = Message::new(user.get_id(), MessageType::Text(Vec::from(message)));
                let serialized = message.into_vec_u8().unwrap();
                Client::send(serialized.as_slice(), self.connection.clone()).await;
            }
            None => return Err(ClientError::NotLoggedIn),
        }

        Ok(())
    }

    pub async fn send_mention_message(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        message_chunks: Vec<(String, Option<UserIdSize>)>,
    ) {
        // Get our user id
        let guard = self.user.lock().await;
        if let Some(ref user) = *guard {
            // Send our mention message
            let message = Message::from(MessageType::TextMention((
                user.get_id(),
                realm_id,
                channel_id,
                message_chunks,
            )));
            let serialized = message.into_vec_u8().unwrap();
            Client::send(serialized.as_slice(), self.connection.clone()).await;
        }
    }

    pub async fn send_image(
        &self,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
        image: Vec<u8>,
    ) {
        // Get our user id
        let guard = self.user.lock().await;
        if let Some(ref user) = *guard {
            // Send our mention message
            let message = Message::from(MessageType::Image((
                MessageHeader {
                    user_id: user.get_id(),
                    realm_id: realm_id,
                    channel_id: channel_id,
                },
                image,
            )));
            let serialized = message.into_vec_u8().unwrap();
            Client::send(serialized.as_slice(), self.connection.clone()).await;
        }
    }

    pub async fn hang_up(&self, realm_id: &RealmIdSize, channel_id: &ChannelIdSize) {
        // Get our user id
        let guard = self.user.lock().await;
        if let Some(ref user) = *guard {
            // Send our join message
            let message = Message::from(MessageType::UserLeftVoiceChannel((
                user.get_id(),
                *realm_id,
                *channel_id,
            )));
            let serialized = message.into_vec_u8().unwrap();
            Client::send(serialized.as_slice(), self.connection.clone()).await;

            // If we have an AudioManager, tell it to stop
            let mut am = self.audio_manager.lock().await;
            match &mut *am {
                Some(manager) => {
                    manager.disconnect().await;
                }
                None => (),
            }
        }
    }

    pub async fn join_channel(
        &self,
        realm_id: RealmIdSize,
        channel_type: ChannelType,
        channel_id: ChannelIdSize,
    ) {
        match channel_type {
            // For, all text messages get sent to everyone
            ChannelType::TextChannel => {
                // Get our user id
                let guard = self.user.lock().await;
                if let Some(ref user) = *guard {
                    // Send our join message
                    let message = Message::from(MessageType::JoinChannel((
                        user.get_id(),
                        realm_id,
                        channel_type,
                        channel_id,
                    )));
                    let serialized = message.into_vec_u8().unwrap();
                    Client::send(serialized.as_slice(), self.connection.clone()).await;
                }
            }
            ChannelType::VoiceChannel => {
                // Get our user id
                let guard = self.user.lock().await;
                if let Some(ref user) = *guard {
                    // Send our join message
                    let message = Message::from(MessageType::UserJoinedVoiceChannel((
                        user.get_id(),
                        realm_id,
                        channel_id,
                    )));
                    let serialized = message.into_vec_u8().unwrap();
                    Client::send(serialized.as_slice(), self.connection.clone()).await;
                }
            }
        }
    }

    async fn send(buffer: &[u8], connection: Connection) {
        match connection.open_bi().await {
            Ok((mut send, _recv)) => {
                send.write_all(buffer).await.unwrap();
                send.finish().await.unwrap();
            }
            Err(_) => (),
        }
    }

    pub async fn disconnect(self: &mut Self) {
        // Tell the server we are disconnecting
        // Get our user id
        let guard = self.user.lock().await;
        if let Some(ref user) = *guard {
            // Send our disconnecting message
            let message = Message::from(MessageType::Disconnecting(user.get_id()));
            let serialized = message.into_vec_u8().unwrap();
            Client::send(serialized.as_slice(), self.connection.clone()).await;
        }

        // Send our QUIC disconnect
        self.connection.close(0u32.into(), b"done");
        self.connection.closed().await;

        // Tell our receiving thread to stop receiving data
        let connection_sender = self.connection_sender.clone();
        let mut conn_sender = connection_sender.lock().await;
        if let Some(conn_sender) = conn_sender.take() {
            match conn_sender.send(ConnectionCommand::StopReceiving).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    pub async fn connect_voice(&mut self, _realm_id: RealmIdSize, _channel_id: ChannelIdSize) {
        if let Some(user_id) = self.get_user_id().await {
            let mut am = self.audio_manager.lock().await;
            match *am {
                Some(ref mut manager) => {
                    // Set our user id before recording and broadcasting
                    manager.set_user_id(user_id);

                    // Start recording for broadcasting
                    manager.start_recording().await;
                    manager.start_listening().await;
                }
                None => (),
            }
        }
    }
}
