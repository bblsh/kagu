use std::collections::BTreeMap;

use message::message::{Message, MessageType};
use network_manager::MESSAGE_HEADER_SIZE;
use realms::realms_manager::RealmsManager;
use types::UserIdSize;
use user::User;

use chrono::Utc;
use swiftlet_quic::endpoint::{ConnectionEndReason, ConnectionId, Endpoint};
use swiftlet_quic::EndpointEventCallbacks;

enum SendTo {
    Everyone,
    EveryoneExceptUserID(UserIdSize),
    SingleUser(UserIdSize),
    Users(Vec<UserIdSize>),
}

type DisconnectReasonSize = u64;
#[derive(Debug)]
#[repr(u64)]
enum DisconnectReason {
    ServerShutdown = 0,
    NotLoggedIn,
}

pub struct ServerState {
    _name: String,
    clients: BTreeMap<ConnectionId, User>,
    client_count: UserIdSize,
    realms_manager: RealmsManager,
    disconnect_queue: Vec<(ConnectionId, DisconnectReasonSize)>,
    exiting: bool,
}

impl ServerState {
    pub fn new(server_name: String) -> ServerState {
        ServerState {
            _name: server_name,
            clients: BTreeMap::new(),
            client_count: 0,
            realms_manager: RealmsManager::default(),
            disconnect_queue: Vec::new(),
            exiting: false,
        }
    }

    #[inline]
    fn get_message_size(&self, read_data: &[u8]) -> usize {
        usize::from_ne_bytes([read_data[0], read_data[1], 0, 0, 0, 0, 0, 0])
    }

    fn process_message(&mut self, cid: &ConnectionId, message: Message, endpoint: &mut Endpoint) {
        //println!("{:?}", message);

        // If the user hasn't been logged in, disconnect
        // unless the user is trying to log in
        if !self.clients.contains_key(cid) {
            match message.message {
                MessageType::LoginAttempt(username) => {
                    // "Authenticate" this user
                    let user = self.authenticate_user(cid, username);
                    let user_id = user.get_id();

                    // Notify the user of a successful login
                    let message = Message::from(MessageType::LoginSuccess(user.clone()));
                    self.send(SendTo::SingleUser(user_id), message, endpoint);

                    // Announce the new user to everyone
                    let message = Message::from(MessageType::UserJoined(user));
                    self.send(SendTo::EveryoneExceptUserID(user_id), message, endpoint);
                }
                _ => self
                    .disconnect_queue
                    .push((*cid, DisconnectReason::NotLoggedIn as u64)),
            }
        } else {
            match message.message {
                MessageType::Disconnecting(user_id) => {
                    self.clients.retain(|_, u| u.get_id() != user_id);

                    let message = Message::from(MessageType::UserLeft(user_id));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::GetAllUsers(gau) => {
                    let mut users = Vec::new();
                    for connection in &self.clients {
                        users.push(connection.1.clone());
                    }

                    let message = Message::from(MessageType::AllUsers(users));
                    self.send(SendTo::SingleUser(gau.user_id), message, endpoint);
                }
                MessageType::GetRealms(user_id) => {
                    let rm = self.realms_manager.clone();
                    let message = Message::from(MessageType::RealmsManager(rm));
                    self.send(SendTo::SingleUser(user_id), message, endpoint);
                }
                MessageType::AddRealm(ar) => {
                    let realm_id = self.realms_manager.add_realm(ar.1.clone());
                    let message = Message::from(MessageType::RealmAdded((realm_id, ar.1)));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::AddChannel(ac) => {
                    let channel =
                        self.realms_manager
                            .add_channel(ac.0.realm_id, ac.1.clone(), ac.2);
                    let message = Message::from(MessageType::ChannelAdded((
                        ac.0.realm_id,
                        ac.1,
                        channel.0,
                        channel.1,
                    )));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::Text(mut message) => {
                    // Before sending, we need to generate an id for this message
                    if let Some(realm) = self.realms_manager.get_realm_mut(message.0.realm_id) {
                        if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id) {
                            let id = channel.generate_message_id();

                            // Set the message id
                            message.0.message_id = Some(id);

                            // Set the time the message was sent
                            message.0.datetime = Some(Utc::now());

                            let text = Message::from(MessageType::Text(message));
                            self.send(SendTo::Everyone, text, endpoint);
                        }
                    }

                    // If we couldn't find the realm or channel, don't send it
                }
                MessageType::Typing(message) => {
                    let message = Message::from(MessageType::Typing(message));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::UserJoinedVoiceChannel(message) => {
                    let message = Message::from(MessageType::UserJoinedVoiceChannel(message));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::UserLeftVoiceChannel(message) => {
                    let message = Message::from(MessageType::UserLeftVoiceChannel(message));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                MessageType::NewFriendRequest((header, requested_id)) => {
                    let message =
                        Message::from(MessageType::NewFriendRequest((header, requested_id)));
                    self.send(SendTo::SingleUser(requested_id), message, endpoint);
                }
                MessageType::RemoveFriend((header, old_friend_id)) => {
                    // Break the bad news to this now former friend
                    let message = Message::from(MessageType::FriendshipEnded(header));
                    self.send(SendTo::SingleUser(old_friend_id), message, endpoint);
                }
                MessageType::FriendRequestAccepted((header, new_friend_id)) => {
                    let message =
                        Message::from(MessageType::FriendRequestAccepted((header, new_friend_id)));
                    self.send(SendTo::SingleUser(new_friend_id), message, endpoint);
                }
                MessageType::FriendRequestRejected((header, rejected_id)) => {
                    let message =
                        Message::from(MessageType::FriendRequestRejected((header, rejected_id)));
                    self.send(SendTo::SingleUser(rejected_id), message, endpoint);
                }
                MessageType::Audio((header, audio)) => {
                    let message = Message::from(MessageType::Audio((header, audio)));
                    self.send(SendTo::Everyone, message, endpoint);
                }
                _ => println!("Not implemented: {:?}", message),
            }
        }
    }

    fn authenticate_user(&mut self, cid: &ConnectionId, username: String) -> User {
        // Generate a user id for this user
        let user_id = self.client_count;
        self.client_count += 1;

        // Add this user to our list of clients
        let user = User::new(user_id, username);
        self.clients.insert(*cid, user.clone());

        user
    }

    fn disconnect_users(&mut self, endpoint: &mut Endpoint) {
        // Check to see if a user should be disconnected
        while let Some(disconnect) = self.disconnect_queue.pop() {
            let _ = endpoint.close_connection(&disconnect.0, disconnect.1);
        }
    }

    fn _terminate_server(&mut self, endpoint: &mut Endpoint) {
        for connection in self.clients.iter() {
            let _ =
                endpoint.close_connection(connection.0, DisconnectReason::ServerShutdown as u64);
        }
        self.exiting = true;
    }

    fn send(&self, send_to: SendTo, message: Message, endpoint: &mut Endpoint) {
        println!("{:?}", message);
        let message_buffer = message.into_vec_u8().unwrap();
        let mut send_buffer = Vec::new();
        send_buffer.extend(u16::try_from(message_buffer.len()).unwrap().to_ne_bytes());
        send_buffer.extend(message_buffer);

        for connection in &self.clients {
            println!("conn id: {}", connection.0);
        }

        match send_to {
            SendTo::Everyone => {
                for connection in &self.clients {
                    let _ = endpoint.main_stream_send(connection.0, send_buffer.clone());
                }
            }
            SendTo::EveryoneExceptUserID(user_id) => {
                for connection in &self.clients {
                    if connection.1.get_id() != user_id {
                        let _ = endpoint.main_stream_send(connection.0, send_buffer.clone());
                    }
                }
            }
            SendTo::SingleUser(user_id) => {
                for connection in &self.clients {
                    if connection.1.get_id() == user_id {
                        let _ = endpoint.main_stream_send(connection.0, send_buffer.clone());
                    }
                }
            }
            SendTo::Users(user_ids) => {
                for connection in &self.clients {
                    if user_ids.contains(&connection.1.get_id()) {
                        let _ = endpoint.main_stream_send(connection.0, send_buffer.clone());
                    }
                }
            }
        }
    }
}

impl EndpointEventCallbacks for ServerState {
    fn tick(&mut self, endpoint: &mut Endpoint) -> bool {
        // Handle disconnect of users to be disconnected
        self.disconnect_users(endpoint);

        false
    }

    fn connection_started(&mut self, _endpoint: &mut Endpoint, _cid: &ConnectionId) {
        println!("[server] client connected");
    }

    fn connection_ended(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        reason: ConnectionEndReason,
        remaining_connections: usize,
    ) -> bool {
        false
    }

    fn main_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        read_data: &[u8],
    ) -> Option<usize> {
        if read_data.len() == MESSAGE_HEADER_SIZE {
            Some(self.get_message_size(read_data))
        } else {
            // We know this is (likely) a message
            let message_buffer = read_data.to_vec();
            let message = Message::from_vec_u8(message_buffer).unwrap();

            self.process_message(cid, message, endpoint);

            // Tell swiftlet to read another message header
            Some(MESSAGE_HEADER_SIZE)
        }
    }
}
