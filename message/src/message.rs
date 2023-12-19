
use chrono::{DateTime, Utc};
use realms::{realm::ChannelType, realm_desc::RealmDescription, realms_manager::RealmsManager};
use serde::{Deserialize, Serialize};
use types::{ChannelIdSize, MessageIdSize, RealmIdSize, TextMessageChunks, UserIdSize};
use user::User;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct MessageHeader {
    pub user_id: UserIdSize,
    pub realm_id: RealmIdSize,
    pub channel_id: ChannelIdSize,
    pub datetime: Option<DateTime<Utc>>,
    pub message_id: Option<MessageIdSize>,
}

impl MessageHeader {
    pub fn new(
        user_id: UserIdSize,
        realm_id: RealmIdSize,
        channel_id: ChannelIdSize,
    ) -> MessageHeader {
        MessageHeader {
            user_id,
            realm_id,
            channel_id,
            datetime: Some(Utc::now()),
            message_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum MessageType {
    // User communications
    Audio((MessageHeader, Vec<u8>)),
    Text((MessageHeader, TextMessageChunks)),
    Reply((MessageHeader, MessageIdSize, TextMessageChunks)),
    AudioConnection(UserIdSize),
    Image((MessageHeader, Vec<u8>)),
    Typing(MessageHeader),

    // Logging in
    LoginAttempt(String),
    LoginSuccess(User),
    LoginFailed,

    // Users coming and going
    UserJoined(User),
    UserLeft(UserIdSize),
    JoinChannel((MessageHeader, ChannelType)),
    LeaveChannel((MessageHeader, ChannelType)),
    UserJoinedVoiceChannel(MessageHeader),
    UserLeftVoiceChannel(MessageHeader),
    Disconnecting(UserIdSize),

    // Users
    AllUsers(Vec<User>),

    // Friend actions
    NewFriendRequest((MessageHeader, UserIdSize)),
    FriendRequestAccepted((MessageHeader, UserIdSize)),
    FriendRequestRejected((MessageHeader, UserIdSize)),
    RemoveFriend((MessageHeader, UserIdSize)),
    FriendshipEnded(MessageHeader),

    // Realms
    RealmsManager(RealmsManager),
    Realms(Vec<RealmDescription>),
    GetRealms(UserIdSize),
    AddRealm((MessageHeader, String)),
    RemoveRealm((MessageHeader, RealmIdSize)),
    RealmAdded((RealmIdSize, String)),
    RealmRemoved(RealmIdSize),

    // Channels
    AddChannel((MessageHeader, ChannelType, String)),
    RemoveChannel((MessageHeader, ChannelType)),
    RenameChannel((MessageHeader, ChannelType)),
    ChannelAdded((RealmIdSize, ChannelType, ChannelIdSize, String)),
    ChannelRemoved((RealmIdSize, ChannelType, ChannelIdSize)),

    // User disconnects
    Disconnect,

    // Probing heartbeat
    Heartbeat,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Message {
    pub user_id: UserIdSize,
    pub realm_id: RealmIdSize,
    pub channel_id: ChannelIdSize,
    pub message: MessageType,
}

impl From<MessageType> for Message {
    fn from(message_type: MessageType) -> Message {
        match message_type {
            MessageType::Text(message) => Message::new(0, MessageType::Text(message)),
            MessageType::Reply(message) => Message::new(0, MessageType::Reply(message)),
            MessageType::Audio(audio) => Message::new(0, MessageType::Audio(audio)),
            MessageType::AudioConnection(user_id) => {
                Message::new(0, MessageType::AudioConnection(user_id))
            }
            MessageType::Image(message) => Message::new(0, MessageType::Image(message)),
            MessageType::Typing(typing) => Message::new(0, MessageType::Typing(typing)),
            MessageType::LoginAttempt(username) => {
                Message::new(0, MessageType::LoginAttempt(username))
            }
            MessageType::LoginSuccess(user) => Message::new(0, MessageType::LoginSuccess(user)),
            MessageType::UserJoined(user) => {
                Message::new(user.get_id(), MessageType::UserJoined(user))
            }
            MessageType::UserLeft(user_id) => Message::new(user_id, MessageType::UserLeft(user_id)),
            MessageType::JoinChannel(join_info) => {
                Message::new(join_info.0.user_id, MessageType::JoinChannel(join_info))
            }
            MessageType::LeaveChannel(leave_info) => {
                Message::new(leave_info.0.user_id, MessageType::LeaveChannel(leave_info))
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
            MessageType::NewFriendRequest(request) => {
                Message::new(0, MessageType::NewFriendRequest(request))
            }
            MessageType::FriendRequestAccepted(accepted) => {
                Message::new(0, MessageType::FriendRequestAccepted(accepted))
            }
            MessageType::FriendRequestRejected(rejected) => {
                Message::new(0, MessageType::FriendRequestRejected(rejected))
            }
            MessageType::RemoveFriend(rf) => Message::new(0, MessageType::RemoveFriend(rf)),
            MessageType::FriendshipEnded(fe) => Message::new(0, MessageType::FriendshipEnded(fe)),
            MessageType::GetRealms(user_id) => {
                Message::new(user_id, MessageType::GetRealms(user_id))
            }
            MessageType::Realms(realms) => Message::new(0, MessageType::Realms(realms)),
            MessageType::RealmsManager(rm) => Message::new(0, MessageType::RealmsManager(rm)),
            MessageType::AddRealm(ar) => Message::new(0, MessageType::AddRealm(ar)),
            MessageType::RemoveRealm(rr) => Message::new(0, MessageType::RemoveRealm(rr)),
            MessageType::RealmAdded(ra) => Message::new(0, MessageType::RealmAdded(ra)),
            MessageType::RealmRemoved(rr) => Message::new(0, MessageType::RealmRemoved(rr)),
            MessageType::AddChannel(ac) => Message::new(0, MessageType::AddChannel(ac)),
            MessageType::RemoveChannel(rc) => Message::new(0, MessageType::RemoveChannel(rc)),
            MessageType::RenameChannel(rc) => Message::new(0, MessageType::RenameChannel(rc)),
            MessageType::ChannelAdded(ca) => Message::new(0, MessageType::ChannelAdded(ca)),
            MessageType::ChannelRemoved(cr) => Message::new(0, MessageType::ChannelRemoved(cr)),
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
            user_id,
            realm_id: 0,
            channel_id: 0,
            message,
        }
    }

    pub fn get_message(self) -> MessageType {
        match self.message {
            MessageType::Text(message) => MessageType::Text(message),
            MessageType::Reply(reply) => MessageType::Reply(reply),
            MessageType::Audio(audio) => MessageType::Audio(audio),
            MessageType::Image(message) => MessageType::Image(message),
            MessageType::Typing(typing) => MessageType::Typing(typing),
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
            MessageType::NewFriendRequest(request) => MessageType::NewFriendRequest(request),
            MessageType::FriendRequestAccepted(accepted) => {
                MessageType::FriendRequestAccepted(accepted)
            }
            MessageType::FriendRequestRejected(rejected) => {
                MessageType::FriendRequestRejected(rejected)
            }
            MessageType::RemoveFriend(rf) => MessageType::RemoveFriend(rf),
            MessageType::FriendshipEnded(fe) => MessageType::FriendshipEnded(fe),
            MessageType::RealmsManager(rm) => MessageType::RealmsManager(rm),
            MessageType::Realms(realms) => MessageType::Realms(realms),
            MessageType::GetRealms(user_id) => MessageType::GetRealms(user_id),
            MessageType::AddRealm(ar) => MessageType::AddRealm(ar),
            MessageType::RemoveRealm(rr) => MessageType::RemoveRealm(rr),
            MessageType::RealmAdded(ra) => MessageType::RealmAdded(ra),
            MessageType::RealmRemoved(rr) => MessageType::RealmRemoved(rr),
            MessageType::AddChannel(ac) => MessageType::AddChannel(ac),
            MessageType::RemoveChannel(rc) => MessageType::RemoveChannel(rc),
            MessageType::RenameChannel(rc) => MessageType::RenameChannel(rc),
            MessageType::ChannelAdded(ca) => MessageType::ChannelAdded(ca),
            MessageType::ChannelRemoved(cr) => MessageType::ChannelRemoved(cr),
            MessageType::Disconnect => MessageType::Disconnect,
            MessageType::Disconnecting(user_id) => MessageType::Disconnecting(user_id),
            MessageType::Heartbeat => MessageType::Heartbeat,
        }
    }

    pub fn get_user_id(self) -> UserIdSize {
        self.user_id
    }

    pub fn into_vec_u8(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn from_vec_u8(buffer: Vec<u8>) -> Result<Message, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer.as_slice())
    }
}
