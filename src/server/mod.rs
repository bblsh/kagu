use crate::message::{Message, MessageType};
use crate::network_manager::{NetworkManager, ServerOrClient};
use crate::realms::realm::ChannelType;
use crate::realms::realms_manager::RealmsManager;
use crate::types::{ConnectionIdSize, UserIdSize};
use crate::user::User;

use quinn::{Connection, Endpoint};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::{mpsc::Receiver, mpsc::Sender, Mutex};
use tokio::task::JoinHandle;

// Not used at the moment
enum _ServerMessageType {
    NewConnection(Connection),
    RemoveConnection(ConnectionIdSize),
    Message((ConnectionIdSize, Message)),
}

// Not used at the moment
struct _ServerMessage {
    message: _ServerMessageType,
}

impl _ServerMessage {
    pub fn _new(message: _ServerMessageType) -> _ServerMessage {
        _ServerMessage { message: message }
    }
}

pub struct Server {
    endpoint: Endpoint,
    num_connections: Arc<Mutex<ConnectionIdSize>>,
    connections: Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
    users: Arc<Mutex<Vec<User>>>,
    num_users: Arc<Mutex<UserIdSize>>,
    realms_manager: Arc<Mutex<RealmsManager>>,
}

impl Server {
    pub async fn new(server_address: String) -> Server {
        let endpoint = NetworkManager::new(server_address, ServerOrClient::Server).await;

        Server {
            endpoint: endpoint,
            num_connections: Arc::new(Mutex::new(0)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(Vec::new())),
            num_users: Arc::new(Mutex::new(0)),
            realms_manager: Arc::new(Mutex::new(RealmsManager::new())),
        }
    }

    pub async fn run_server(self: &Self) {
        // --- !!! TEST SETUP HERE !!! ---
        // This needs to be replaced with a database at some point in time
        self.make_test_realm().await;
        // END SETUP

        let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(5000);
        let connections_handle = self.connections.clone();

        let receive_handle = self.start_receive_thread(sender).await;
        let send_handle = Server::start_send_thread(connections_handle, receiver).await;

        println!("[server] server ready");

        receive_handle.await.unwrap();
        send_handle.await.unwrap();
    }

    async fn make_test_realm(self: &Self) {
        let mut realms_manager = self.realms_manager.lock().await;
        let id = realms_manager.add_realm(String::from("MshKngdm"));
        realms_manager.add_channel(
            id.clone(),
            ChannelType::TextChannel,
            String::from("Peach's Castle"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::TextChannel,
            String::from("Yoshi Land"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::TextChannel,
            String::from("Jolly Roger Bay"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::TextChannel,
            String::from("Hazy Maze Cave"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::TextChannel,
            String::from("Rainbow Ride"),
        );
        realms_manager.add_channel(
            id,
            ChannelType::VoiceChannel,
            String::from("Bowser's Castle"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::VoiceChannel,
            String::from("Goombas Galore"),
        );
        realms_manager.add_channel(
            id.clone(),
            ChannelType::VoiceChannel,
            String::from("Tick Tock Clock"),
        );
        drop(realms_manager);
    }

    pub async fn start_receive_thread(
        self: &Self,
        message_sender: mpsc::Sender<Vec<u8>>,
    ) -> JoinHandle<()> {
        let endpoint = self.endpoint.clone();
        let connections_handle = self.connections.clone();

        let message_sender = message_sender.clone();

        let users = self.users.clone();

        let realms_manager_handle = self.realms_manager.clone();

        let num_connections_handle = self.num_connections.clone();

        let num_users_handle = self.num_users.clone();

        let receive_handle: tokio::task::JoinHandle<_> = tokio::spawn(async move {
            // Listen for any connections
            let connections_handle = connections_handle.clone();
            while let Some(conn) = endpoint.accept().await {
                let connection = conn.await.unwrap();

                println!(
                    "[server] incoming connection: addr={}",
                    connection.remote_address()
                );

                let connections_handle = connections_handle.clone();

                let connection_id =
                    Server::generate_connection_id(num_connections_handle.clone()).await;
                {
                    let mut connections = connections_handle.lock().await;
                    connections.insert(connection_id, connection.clone());
                }

                let _connections = connections_handle.clone();

                let connection_id_clone = connection_id.clone();

                let message_sender = message_sender.clone();

                let connections_recv_handle = connections_handle.clone();
                let users = users.clone();
                let realms_manager = realms_manager_handle.clone();
                let num_users_handle = num_users_handle.clone();

                // Spawn a tokio thread to listen for data
                tokio::spawn(async move {
                    loop {
                        let stream = connection.accept_bi().await;
                        let _stream = match stream {
                            Ok((_send_stream, mut read_stream)) => {
                                let message = read_stream.read_to_end(12000000).await.unwrap();

                                Server::handle_message(
                                    connection_id_clone,
                                    connections_recv_handle.clone(),
                                    message,
                                    message_sender.clone(),
                                    users.clone(),
                                    realms_manager.clone(),
                                    num_users_handle.clone(),
                                )
                                .await;
                            }
                            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                                println!("Connection closed");
                                // The user disconnected, so remove this from our list of connections
                                let conns = connections_recv_handle.clone();
                                let mut conns = conns.lock().await;
                                conns.remove(&connection_id_clone);
                                break;
                            }
                            Err(e) => {
                                println!("[server] stream error ({}) Removing connection", e);
                                let conns = connections_recv_handle.clone();
                                let mut conns = conns.lock().await;
                                conns.remove(&connection_id_clone);
                                break;
                            }
                        };
                    }
                });
            }
        });

        receive_handle
    }

    async fn handle_message(
        connection_id: ConnectionIdSize,
        connections_handle: Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
        message: Vec<u8>,
        message_sender: Sender<Vec<u8>>,
        users: Arc<Mutex<Vec<User>>>,
        realms_manager: Arc<Mutex<RealmsManager>>,
        num_users: Arc<Mutex<UserIdSize>>,
    ) {
        let deserialized: Message = bincode::deserialize(message.as_slice()).unwrap();
        match deserialized.get_message() {
            MessageType::LoginAttempt(username) => {
                println!("Authenticating user...");
                Server::authenticate_user(
                    &connections_handle,
                    connection_id,
                    username.clone(),
                    users,
                    num_users,
                )
                .await;
                println!("[server] client {} logged in", username);
            }
            MessageType::Text(text) => {
                println!(
                    "[server] Received message from client: {}",
                    String::from_utf8(text.clone()).unwrap()
                );
                message_sender.send(message).await.unwrap();
            }
            MessageType::TextMention(message) => {
                println!("[server] got a mention message");
                Server::send_to_everyone(
                    Message::from(MessageType::TextMention(message)),
                    &connections_handle,
                )
                .await;
            }
            MessageType::Audio(audio) => {
                Server::send_to_everyone(
                    Message::from(MessageType::Audio(audio)),
                    &connections_handle,
                )
                .await;
            }
            MessageType::Image(image) => {
                println!("[server] someone sent an image");
                Server::send_to_everyone(
                    Message::from(MessageType::Image(image)),
                    &connections_handle,
                )
                .await;
            }
            MessageType::AudioConnection(_user_id) => {
                println!("[server] audio connected opened");
            }
            MessageType::GetRealms(_user_id) => {
                println!("[server] got request for all realms");
                Server::send_realms_to_connection_id(
                    &connections_handle,
                    connection_id,
                    &realms_manager,
                )
                .await;
            }
            MessageType::JoinChannel(_join_msg) => {
                println!("Someone joined a channel...");
            }
            MessageType::UserJoinedVoiceChannel(join) => {
                println!("[server] user joined a voice channel...");
                Server::send_to_everyone(
                    Message::from(MessageType::UserJoinedVoiceChannel(join)),
                    &connections_handle,
                )
                .await;
            }
            MessageType::UserLeftVoiceChannel(left) => {
                println!("[server] user left a voice channel...");
                Server::send_to_everyone(
                    Message::from(MessageType::UserLeftVoiceChannel(left)),
                    &connections_handle,
                )
                .await;
            }
            MessageType::Disconnecting(user_id) => {
                // Remove this user from our users
                let mut users = users.lock().await;
                users.retain(|user| user.get_id() != user_id);
                drop(users);

                Server::send_to_everyone_except_id(
                    Message::from(MessageType::UserLeft(user_id)),
                    connection_id,
                    &connections_handle,
                )
                .await;
            }
            MessageType::Disconnect => {
                Server::disconnect_user(connection_id, &connections_handle).await;
            }
            MessageType::Heartbeat => (),
            _ => (),
        };
    }

    async fn disconnect_user(
        conn_id: ConnectionIdSize,
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
    ) {
        let conns = connections.clone();
        let mut conns = conns.lock().await;
        if let Some(connection) = conns.get(&conn_id) {
            connection.close(0u32.into(), b"done");
        }
        conns.remove(&conn_id);
    }

    // Lazily generate a connection ID
    async fn generate_connection_id(num_conns: Arc<Mutex<ConnectionIdSize>>) -> ConnectionIdSize {
        let mut num_conns = num_conns.lock().await;
        let id = *num_conns;
        *num_conns = *num_conns + 1;
        id
    }

    // Lazily generate a user ID
    async fn generate_user_id(num_users: Arc<Mutex<UserIdSize>>) -> UserIdSize {
        let mut num_users = num_users.lock().await;
        let id = *num_users;
        *num_users = *num_users + 1;
        id
    }

    async fn start_send_thread(
        connections: Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
        mut message_rx: Receiver<Vec<u8>>,
    ) -> JoinHandle<()> {
        let send_handle = tokio::spawn(async move {
            let connections = connections.clone();

            // Continually listen for Messages from our Messages recevier channel
            loop {
                match message_rx.recv().await {
                    Some(buffer) => match Message::from_vec_u8(buffer) {
                        Ok(message) => Server::send_to_everyone(message, &connections).await,
                        Err(_) => eprintln!("[server] failed to deserialize message"),
                    },
                    None => (),
                }
            }
        });

        send_handle
    }

    async fn send_to_everyone(
        message: Message,
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
    ) {
        let mut conns = connections.lock().await;
        for (_id, connection) in conns.iter_mut() {
            Server::send(message.into_vec_u8().unwrap().as_slice(), connection).await;
        }
    }

    async fn send_to_everyone_except_id(
        message: Message,
        conn_id: ConnectionIdSize,
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
    ) {
        let mut conns = connections.lock().await;
        for (id, connection) in conns.iter_mut() {
            if id != &conn_id {
                Server::send(message.into_vec_u8().unwrap().as_slice(), connection).await;
            }
        }
    }

    async fn send(buffer: &[u8], connection: &mut Connection) {
        match connection.open_bi().await {
            Ok((mut send, _recv)) => match send.write_all(buffer).await {
                Ok(_) => match send.finish().await {
                    Ok(_) => (),
                    Err(_) => (),
                },
                Err(e) => eprintln!("[server] error on send.finish() {}", e),
            },
            Err(e) => {
                eprintln!("Error sending to connection? {}", e);
            }
        }
    }

    async fn send_to_connection_id(
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
        connection_id: ConnectionIdSize,
        buffer: &[u8],
    ) {
        let mut conns = connections.lock().await;
        if let Some(conn) = conns.get_mut(&connection_id) {
            Server::send(buffer, conn).await;
        } else {
            eprintln!("[server] error sending to connection id {}", connection_id);
        }
    }

    async fn authenticate_user(
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
        connection_id: ConnectionIdSize,
        username: String,
        users: Arc<Mutex<Vec<User>>>,
        num_users: Arc<Mutex<UserIdSize>>,
    ) {
        // For now, authenticate everyone
        //let conns = connections.lock().await;
        let id = Server::generate_user_id(num_users).await;

        // Create a new user and add it to our vec of users
        let new_user = User::new(id, username.clone());
        let users_handle = users.clone();
        let mut users_handle = users_handle.lock().await;
        users_handle.push(new_user.clone());
        drop(users_handle);

        // Respond to the user that the login was successful
        let message = Message::new(id, MessageType::LoginSuccess(new_user));
        Server::send_to_connection_id(
            connections,
            connection_id,
            message.into_vec_u8().unwrap().as_slice(),
        )
        .await;

        // There may be users in the channel that the new user doesn't
        // know about yet, so let's send them an update with everyone
        let users = users.lock().await;
        let all_users = users.clone();
        let message = Message::from(MessageType::AllUsers(all_users));
        Server::send_to_connection_id(
            connections,
            connection_id,
            message.into_vec_u8().unwrap().as_slice(),
        )
        .await;

        // Send the UserJoined message to everyone
        Server::send_to_everyone(
            Message::from(MessageType::UserJoined(User::new(id, username))),
            connections,
        )
        .await;
    }

    async fn send_realms_to_connection_id(
        connections: &Arc<Mutex<HashMap<ConnectionIdSize, Connection>>>,
        connection_id: ConnectionIdSize,
        realms_manager: &Arc<Mutex<RealmsManager>>,
    ) {
        // Make our RealmsDescription message
        let rm = realms_manager.lock().await;

        let message = Message::from(MessageType::RealmsManager(rm.clone()));

        Server::send_to_connection_id(
            connections,
            connection_id,
            message.into_vec_u8().unwrap().as_slice(),
        )
        .await;
    }
}
