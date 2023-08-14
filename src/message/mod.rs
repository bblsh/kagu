use crate::{
    realms::{realm::ChannelType, realm_desc::RealmDescription, realms_manager::RealmsManager},
    types::{ChannelIdSize, RealmIdSize, UserIdSize},
    user::User,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MessageHeader {
    pub user_id: UserIdSize,
    pub realm_id: RealmIdSize,
    pub channel_id: ChannelIdSize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum MessageType {
    // User communications
    Text(Vec<u8>),
    Audio((UserIdSize, RealmIdSize, ChannelIdSize, Vec<u8>)),
    TextMention(
        (
            UserIdSize,
            RealmIdSize,
            ChannelIdSize,
            Vec<(String, Option<UserIdSize>)>,
        ),
    ),
    AudioConnection(UserIdSize),
    Image((MessageHeader, Vec<u8>)),

    // Logging in
    LoginAttempt(String),
    LoginSuccess(User),
    LoginFailed,

    // Users coming and going
    UserJoined(User),
    UserLeft(UserIdSize),
    JoinChannel((UserIdSize, RealmIdSize, ChannelType, ChannelIdSize)),
    LeaveChannel((UserIdSize, RealmIdSize, ChannelType, ChannelIdSize)),
    UserJoinedVoiceChannel((UserIdSize, RealmIdSize, ChannelIdSize)),
    UserLeftVoiceChannel((UserIdSize, RealmIdSize, ChannelIdSize)),
    Disconnecting(UserIdSize),

    // Users
    AllUsers(Vec<User>),

    // Realms
    RealmsManager(RealmsManager),
    Realms(Vec<RealmDescription>),
    GetRealms(UserIdSize),

    // User disconnects
    Disconnect,

    // Probing heartbeat
    Heartbeat,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub user_id: UserIdSize,
    pub realm_id: RealmIdSize,
    pub channel_id: ChannelIdSize,
    pub message: MessageType,
}

impl From<MessageType> for Message {
    fn from(message_type: MessageType) -> Message {
        match message_type {
            MessageType::TextMention(message) => Message::new(0, MessageType::TextMention(message)),
            MessageType::Audio(audio) => Message::new(0, MessageType::Audio(audio)),
            MessageType::AudioConnection(user_id) => {
                Message::new(0, MessageType::AudioConnection(user_id))
            }
            MessageType::Image(message) => Message::new(0, MessageType::Image(message)),
            MessageType::LoginAttempt(username) => {
                Message::new(0, MessageType::LoginAttempt(username))
            }
            MessageType::UserJoined(user) => {
                Message::new(user.get_id(), MessageType::UserJoined(user))
            }
            MessageType::UserLeft(user_id) => Message::new(user_id, MessageType::UserLeft(user_id)),
            MessageType::JoinChannel(join_info) => {
                Message::new(join_info.0, MessageType::JoinChannel(join_info))
            }
            MessageType::LeaveChannel(leave_info) => {
                Message::new(leave_info.0, MessageType::JoinChannel(leave_info))
            }
            MessageType::UserJoinedVoiceChannel(joined) => {
                Message::new(0, MessageType::UserJoinedVoiceChannel(joined))
            }
            MessageType::UserLeftVoiceChannel(left) => {
                Message::new(0, MessageType::UserLeftVoiceChannel(left))
            }
            MessageType::Disconnect => Message::new(0, MessageType::Disconnect),
            MessageType::Disconnecting(user_id) => {
                Message::new(0, MessageType::Disconnecting(user_id))
            }
            MessageType::AllUsers(users) => Message::new(0, MessageType::AllUsers(users)),
            MessageType::GetRealms(user_id) => {
                Message::new(user_id, MessageType::GetRealms(user_id))
            }
            MessageType::Realms(realms) => Message::new(0, MessageType::Realms(realms)),
            MessageType::RealmsManager(rm) => Message::new(0, MessageType::RealmsManager(rm)),
            _ => Message::new(0, MessageType::Heartbeat),
        }
    }
}

impl From<Vec<u8>> for Message {
    fn from(buffer: Vec<u8>) -> Message {
        bincode::deserialize(buffer.as_slice()).unwrap()
    }
}

impl Message {
    pub fn new(user_id: UserIdSize, message: MessageType) -> Message {
        Message {
            user_id: user_id,
            realm_id: 0,
            channel_id: 0,
            message: message,
        }
    }

    pub fn get_message(self: Self) -> MessageType {
        match self.message {
            MessageType::Text(text) => MessageType::Text(text),
            MessageType::TextMention(message) => MessageType::TextMention(message),
            MessageType::Audio(audio) => MessageType::Audio(audio),
            MessageType::Image(message) => MessageType::Image(message),
            MessageType::AudioConnection(user_id) => MessageType::AudioConnection(user_id),
            MessageType::LoginAttempt(username) => MessageType::LoginAttempt(username),
            MessageType::LoginSuccess(user) => MessageType::LoginSuccess(user),
            MessageType::LoginFailed => MessageType::LoginFailed,
            MessageType::UserJoined(user) => MessageType::UserJoined(user),
            MessageType::UserLeft(user) => MessageType::UserLeft(user),
            MessageType::JoinChannel(join_info) => MessageType::JoinChannel(join_info),
            MessageType::LeaveChannel(leave_info) => MessageType::LeaveChannel(leave_info),
            MessageType::UserJoinedVoiceChannel(join_info) => {
                MessageType::UserJoinedVoiceChannel(join_info)
            }
            MessageType::UserLeftVoiceChannel(leave_info) => {
                MessageType::UserLeftVoiceChannel(leave_info)
            }
            MessageType::AllUsers(users) => MessageType::AllUsers(users),
            MessageType::RealmsManager(rm) => MessageType::RealmsManager(rm),
            MessageType::Realms(realms) => MessageType::Realms(realms),
            MessageType::GetRealms(user_id) => MessageType::GetRealms(user_id),
            MessageType::Disconnect => MessageType::Disconnect,
            MessageType::Disconnecting(user_id) => MessageType::Disconnecting(user_id),
            MessageType::Heartbeat => MessageType::Heartbeat,
        }
    }

    pub fn get_user_id(self: Self) -> UserIdSize {
        self.user_id.clone()
    }

    pub fn into_vec_u8(self: &Self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn from_vec_u8(buffer: Vec<u8>) -> Result<Message, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer.as_slice())
    }
}
