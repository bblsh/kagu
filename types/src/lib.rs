/// Type for the ID for a `Realm`.
pub type RealmIdSize = u32;

/// Type for the ID for a `Channel`.
pub type ChannelIdSize = u8;

/// Type for the ID of a `User`.
pub type UserIdSize = u32;

/// Type to describe the number of `Realml`s.
pub type NumRealmsSize = u32;

/// Type for the ID of a Connection.
pub type ConnectionIdSize = u64;

/// Type for vec of data to describe what user may be tagged in parts of a message.
pub type TextMessageChunks = Vec<(String, Option<UserIdSize>)>;

/// Type for the ID for a `Message`.
pub type MessageIdSize = u32;

/// Type for the ID of a Ping and PingReply
pub type PingIdSize = u64;

/// Type for the ID of a file transfer
pub type FileTransferIdSize = u64;

/// Internal ID for a song to broadcast
pub type AudioFileIdSize = u64;
