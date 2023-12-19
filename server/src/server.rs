use message::message::{Message, MessageType};
use network_manager::network_manager::{NetworkManager, ServerOrClient};
use realms::realm::ChannelType;
use realms::realms_manager::RealmsManager;
use types::UserIdSize;
use user::User;

use chrono::Utc;
use quinn::{Connection, Endpoint};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

// Not used at the moment
pub enum ServerMessageType {
    RemoveConnection(Connection),
    SuccessfulLogin((String, Connection)),
    FailedLogin(Connection),
    Message((Connection, Message)),
}

// Not used at the moment
pub struct ServerMessage {
    message: ServerMessageType,
}

impl ServerMessage {
    pub fn new(message: ServerMessageType) -> ServerMessage {
        ServerMessage { message }
    }
}

pub struct Server {
    endpoint: Endpoint,
}

impl Server {
    pub async fn new(server_address: String, server_port: u16) -> Server {
        let endpoint =
            NetworkManager::connect_endpoint(server_address, server_port, ServerOrClient::Server)
                .await;

        Server { endpoint }
    }

    pub async fn run_server(&self) {
        // The recevier receives data, and sends this to the process thread to handle
        let (data_tx, data_rx): (Sender<ServerMessage>, Receiver<ServerMessage>) = channel(5000);

        // The processing thread receives messsages and sends messages to the send thread to send
        let (message_tx, message_rx): (Sender<ServerMessage>, Receiver<ServerMessage>) =
            channel(5000);

        self.start_receive_thread(data_tx).await;
        self.start_process_thread(data_rx, message_tx).await;
        self.start_send_thread(message_rx).await;

        println!("[server] server ready");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
    }

    fn make_test_realms() -> RealmsManager {
        let mut realms_manager = RealmsManager::default();
        let id = realms_manager.add_realm(String::from("MshKngdm"));

        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Peach's Castle"));
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Yoshi Land"));
        realms_manager.add_channel(
            id,
            ChannelType::TextChannel,
            String::from("Jolly Roger Bay"),
        );
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Hazy Maze Cave"));
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Rainbow Ride"));
        realms_manager.add_channel(
            id,
            ChannelType::VoiceChannel,
            String::from("Bowser's Castle"),
        );
        realms_manager.add_channel(
            id,
            ChannelType::VoiceChannel,
            String::from("Goombas Galore"),
        );
        realms_manager.add_channel(
            id,
            ChannelType::VoiceChannel,
            String::from("Tick Tock Clock"),
        );

        // Make another realm
        let id = realms_manager.add_realm(String::from("GrtysLr"));
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Spiral Mtn"));
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Frzzy Peak"));
        realms_manager.add_channel(id, ChannelType::TextChannel, String::from("Mumbo's Mtn"));
        realms_manager.add_channel(id, ChannelType::VoiceChannel, String::from("Gobis Valley"));

        realms_manager
    }

    pub async fn start_receive_thread(&self, tx: Sender<ServerMessage>) {
        let endpoint = self.endpoint.clone();

        tokio::spawn(async move {
            // Listen for any connections
            while let Some(conn) = endpoint.accept().await {
                let connection = conn.await.unwrap();

                let tx = tx.clone();

                println!(
                    "[server] incoming connection: addr={}",
                    connection.remote_address()
                );

                // Spawn a tokio thread to listen for data from that connection
                tokio::spawn(async move {
                    // To make sure the user is logged in prior to interacting
                    let mut is_user_logged_in = false;

                    loop {
                        let stream = connection.accept_bi().await;
                        match stream {
                            Ok((_send_stream, mut read_stream)) => {
                                let buffer = read_stream.read_to_end(12000000).await.unwrap();

                                if !is_user_logged_in {
                                    match Message::from_vec_u8(buffer) {
                                        // We have a message, check to see if this is a
                                        Ok(message) => {
                                            match message.get_message() {
                                                MessageType::LoginAttempt(username) => {
                                                    println!("[server] logging in {}", &username);

                                                    // "Authenticate" the user
                                                    is_user_logged_in = true;

                                                    // Send a successful login message
                                                    let sm = ServerMessage::new(
                                                        ServerMessageType::SuccessfulLogin((
                                                            username,
                                                            connection.clone(),
                                                        )),
                                                    );

                                                    match tx.send(sm).await {
                                                        Ok(_) => (),
                                                        Err(_) => eprintln!(
                                                            "[server] failed to send to a channel"
                                                        ),
                                                    }
                                                }
                                                _ => {
                                                    eprintln!("[server] unauthenticated user message. removing connection");
                                                    // Send a failed login message
                                                    let sm = ServerMessage::new(
                                                        ServerMessageType::FailedLogin(
                                                            connection.clone(),
                                                        ),
                                                    );

                                                    match tx.send(sm).await {
                                                        Ok(_) => (),
                                                        Err(_) => eprintln!(
                                                            "[server] failed to send to a channel"
                                                        ),
                                                    }

                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            eprintln!("[server] failed to authenticate user. disconnecting connection {}", connection.remote_address());
                                            break;
                                        }
                                    }
                                } else {
                                    // This connection/user is logged in, so handle this message normally
                                    match Message::from_vec_u8(buffer) {
                                        // The message was deserialized, so let's send it to the send thread
                                        Ok(message) => {
                                            // Send this message to be handled
                                            let sm =
                                                ServerMessage::new(ServerMessageType::Message((
                                                    connection.clone(),
                                                    message,
                                                )));

                                            match tx.send(sm).await {
                                                Ok(_) => (),
                                                Err(_) => {
                                                    eprintln!(
                                                        "[server] failed to send to a channel"
                                                    )
                                                }
                                            }
                                        }
                                        Err(_) => eprintln!("[server] failed to parse message"),
                                    }
                                }
                            }
                            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                                println!("[server] connection closed");
                                // The user sent a disconnect message by now, so no need to send anything here
                                break;
                            }
                            Err(e) => {
                                println!("[server] stream error ({}) removing connection", e);
                                // Send message to remove connection
                                let sm = ServerMessage::new(ServerMessageType::RemoveConnection(
                                    connection.clone(),
                                ));

                                match tx.send(sm).await {
                                    Ok(_) => (),
                                    Err(_) => {
                                        eprintln!("[server] failed to send to a channel")
                                    }
                                }

                                break;
                            }
                        }
                    }
                });
            }
        });
    }

    async fn start_process_thread(
        &self,
        mut rx: Receiver<ServerMessage>,
        tx: Sender<ServerMessage>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            // --- !!! TEST SETUP HERE !!! ---
            // This needs to be replaced with a database at some point in time
            let mut realms_manager = Server::make_test_realms();
            // END SETUP

            let mut users: Vec<User> = Vec::new();
            let mut num_users: UserIdSize = 0;
            let mut connections: HashMap<UserIdSize, Connection> = HashMap::new();

            loop {
                if let Some(message) = rx.recv().await {
                    match message.message {
                        // User successfully logged in
                        ServerMessageType::SuccessfulLogin((username, connection)) => {
                            println!("[server] registering {}", &username);

                            // Super lazy user id generation here
                            let id = num_users;

                            // Make the new user
                            let user = User::new(id, username.clone());

                            // Add this to our map of connections
                            connections.insert(id, connection);

                            // For generating new user ids
                            num_users += 1;

                            // Build a ServerMessage to send to the send thread
                            let message = Message::from(MessageType::LoginSuccess(user.clone()));
                            Server::send_to_id(&connections, id, message, tx.clone()).await;

                            // There may be users in the channel that the new user doesn't
                            // know about yet, so let's send them an update with everyone
                            users.push(user);
                            let message = Message::from(MessageType::AllUsers(users.clone()));
                            Server::send_to_id(&connections, id, message, tx.clone()).await;

                            // Send the UserJoined message to everyone
                            Server::send_to_everyone(
                                &connections,
                                Message::from(MessageType::UserJoined(User::new(id, username))),
                                tx.clone(),
                            )
                            .await;
                        }
                        // The user failed to log in or sent a message without first logging in
                        ServerMessageType::FailedLogin(connection) => {
                            // Build our ServerMessage to send to the send thread
                            let message = Message::from(MessageType::LoginFailed);
                            let server_message = ServerMessage::new(ServerMessageType::Message((
                                connection, message,
                            )));

                            // Send the ServerMessage to the send thread
                            match tx.send(server_message).await {
                                Ok(_) => (),
                                Err(_) => eprintln!("[server] error sending in process thread"),
                            }

                            // Don't need to remove a connection here since we've never added this one
                        }
                        // The user disconnected or the connection was lost, so remove this connection
                        ServerMessageType::RemoveConnection(connection) => {
                            let id = connections.iter_mut().find_map(|(key, val)| {
                                if val.stable_id() == connection.stable_id() {
                                    Some(*key)
                                } else {
                                    None
                                }
                            });

                            // This connection was saved, so remove it and tell everyone the user left
                            if let Some(user_id) = id {
                                connections
                                    .retain(|_, conn| conn.stable_id() != connection.stable_id());

                                // Now remove this user from our list of users
                                users.retain(|user| user.get_id() != user_id);

                                // Send the UserLeft message to everyone
                                Server::send_to_everyone(
                                    &connections,
                                    Message::from(MessageType::UserLeft(user_id)),
                                    tx.clone(),
                                )
                                .await;
                            }
                        }
                        // Process a normal message from a user
                        ServerMessageType::Message(message) => {
                            Server::handle_message(
                                message.1,
                                &mut connections,
                                tx.clone(),
                                &mut realms_manager,
                                &mut users,
                            )
                            .await;
                        }
                    }
                }
            }
        })
    }

    async fn handle_message(
        message: Message,
        connections: &mut HashMap<UserIdSize, Connection>,
        message_sender: Sender<ServerMessage>,
        realms_manager: &mut RealmsManager,
        users: &mut Vec<User>,
    ) {
        // Debug print
        println!("[server] processing message {:?}", message);

        match message.get_message() {
            MessageType::GetRealms(user_id) => {
                //println!("[server] got request for realms");
                Server::send_realms_to_id(connections, user_id, realms_manager, message_sender)
                    .await;
            }
            MessageType::Text(mut message) => {
                // Before sending, we need to generate an id for this message
                if let Some(realm) = realms_manager.get_realm_mut(message.0.realm_id) {
                    if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id) {
                        let id = channel.generate_message_id();

                        // Set the message id
                        message.0.message_id = Some(id);

                        // Set the time the message was sent
                        message.0.datetime = Some(Utc::now());

                        Server::send_to_everyone(
                            connections,
                            Message::from(MessageType::Text(message)),
                            message_sender,
                        )
                        .await;
                    }
                }

                // If we couldn't find the realm or channel, don't send it
            }
            MessageType::Reply(mut message) => {
                // Before sending, we need to generate an id for this message
                if let Some(realm) = realms_manager.get_realm_mut(message.0.realm_id) {
                    if let Some(channel) = realm.get_text_channel_mut(message.0.channel_id) {
                        let id = channel.generate_message_id();

                        // Set the message id
                        message.0.message_id = Some(id);

                        // Set the time the message was sent
                        message.0.datetime = Some(Utc::now());

                        Server::send_to_everyone(
                            connections,
                            Message::from(MessageType::Reply(message)),
                            message_sender,
                        )
                        .await;
                    }
                }

                // If we couldn't find the realm or channel, don't send it
            }
            MessageType::Audio(message) => {
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::Audio(message)),
                    message_sender,
                )
                .await;
            }
            MessageType::UserJoinedVoiceChannel(message) => {
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::UserJoinedVoiceChannel(message)),
                    message_sender,
                )
                .await;
            }
            MessageType::UserLeftVoiceChannel(message) => {
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::UserLeftVoiceChannel(message)),
                    message_sender,
                )
                .await;
            }
            MessageType::AddChannel(message) => {
                // Add the channel to our realms manager
                let channel =
                    realms_manager.add_channel(message.0.realm_id, message.1.clone(), message.2);

                // Send the new channel to everyone
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::ChannelAdded((
                        message.0.realm_id,
                        message.1,
                        channel.0,
                        channel.1,
                    ))),
                    message_sender,
                )
                .await;
            }
            MessageType::RemoveChannel(message) => {
                // Remove this channel from our Realms Manager
                realms_manager.remove_channel(
                    message.0.realm_id,
                    message.1.clone(),
                    message.0.channel_id,
                );

                // Send the new channel to everyone
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::ChannelRemoved((
                        message.0.realm_id,
                        message.1,
                        message.0.channel_id,
                    ))),
                    message_sender,
                )
                .await;
            }
            MessageType::AddRealm(message) => {
                // Add this realm to our Realms Manager
                let realm_id = realms_manager.add_realm(message.1.clone());

                // Send the new realm to everyone
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::RealmAdded((realm_id, message.1))),
                    message_sender,
                )
                .await;
            }
            MessageType::RemoveRealm(message) => {
                // Remove this realm from our Realms Manager
                realms_manager.remove_realm(message.1);

                // Send the removed realm to everyone
                Server::send_to_everyone(
                    connections,
                    Message::from(MessageType::RealmRemoved(message.1)),
                    message_sender,
                )
                .await;
            }
            MessageType::NewFriendRequest(request) => {
                // Send a friend request to this user
                Server::send_to_id(
                    connections,
                    request.1,
                    Message::from(MessageType::NewFriendRequest(request)),
                    message_sender,
                )
                .await;
            }
            MessageType::RemoveFriend(rf) => {
                // Break the bad news to this now former friend
                Server::send_to_id(
                    connections,
                    rf.1,
                    Message::from(MessageType::FriendshipEnded(rf.0)),
                    message_sender,
                )
                .await;
            }
            MessageType::Typing(typing) => {
                Server::send_to_everyone_except_id(
                    typing.user_id,
                    connections,
                    Message::from(MessageType::Typing(typing)),
                    message_sender,
                )
                .await;
            }
            MessageType::Disconnecting(user_id) => {
                // Remove this user from our list of users
                users.retain(|user| user.get_id() != user_id);

                // Remove this from our list of connections
                connections.retain(|id, _| id != &user_id);

                Server::send_to_everyone_except_id(
                    user_id,
                    connections,
                    Message::from(MessageType::UserLeft(user_id)),
                    message_sender,
                )
                .await;
            }
            _ => (),
        }
    }

    async fn start_send_thread(&self, mut rx: Receiver<ServerMessage>) {
        tokio::spawn(async move {
            loop {
                if let Some(message) = rx.recv().await {
                    if let ServerMessageType::Message((connection, message)) = message.message {
                        match connection.open_bi().await {
                            // Try sending the message
                            Ok((mut send, _recv)) => match send
                                .write_all(message.into_vec_u8().unwrap().as_slice())
                                .await
                            {
                                Ok(_) => match send.finish().await {
                                    Ok(_) => (),
                                    Err(e) => {
                                        eprintln!("[server] error on send.finish() {}", e);
                                    }
                                },
                                Err(e) => eprintln!("[server] error on send.write_all() {}", e),
                            },
                            Err(e) => {
                                eprintln!("[server] error sending to connection? {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    async fn send_realms_to_id(
        connections: &mut HashMap<UserIdSize, Connection>,
        user_id: UserIdSize,
        realms_manager: &RealmsManager,
        sender: Sender<ServerMessage>,
    ) {
        // Make our RealmsDescription message
        let message = Message::from(MessageType::RealmsManager(realms_manager.clone()));

        Server::send_to_id(connections, user_id, message, sender).await;
    }

    async fn send_to_id(
        connections: &HashMap<UserIdSize, Connection>,
        user_id: UserIdSize,
        message: Message,
        sender: Sender<ServerMessage>,
    ) {
        if let Some(conn) = connections.get(&user_id) {
            Server::send(message, conn.clone(), sender).await;
        } else {
            eprintln!("[server] error sending to connection id {}", user_id);
        }
    }

    async fn send(message: Message, connection: Connection, sender: Sender<ServerMessage>) {
        match sender
            .send(ServerMessage::new(ServerMessageType::Message((
                connection, message,
            ))))
            .await
        {
            Ok(_) => (),
            Err(_) => {
                eprintln!("[server] failed to send");
            }
        }
    }

    async fn send_to_everyone(
        connections: &HashMap<UserIdSize, Connection>,
        message: Message,
        sender: Sender<ServerMessage>,
    ) {
        for (_id, connection) in connections.iter() {
            Server::send(message.clone(), connection.clone(), sender.clone()).await;
        }
    }

    async fn send_to_everyone_except_id(
        user_id: UserIdSize,
        connections: &HashMap<UserIdSize, Connection>,
        message: Message,
        sender: Sender<ServerMessage>,
    ) {
        for (id, connection) in connections.iter() {
            if id != &user_id {
                Server::send(message.clone(), connection.clone(), sender.clone()).await;
            }
        }
    }
}
