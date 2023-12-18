use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use types::{ChannelIdSize, MessageIdSize, TextMessageChunks, UserIdSize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TextChannelMessage {
    /// ID of this message
    pub message_id: Option<MessageIdSize>,
    /// ID of the user who sent the message
    pub user_id: UserIdSize,
    /// ID of the message this may be a reply to
    pub target_reply_message_id: Option<MessageIdSize>,
    /// Time this message was sent, in UTC DateTime format
    pub time_sent: Option<DateTime<Utc>>,
    /// Image data, if this message has an image
    pub image: Option<Vec<u8>>,
    /// Chunks of the message
    pub message_chunks: TextMessageChunks,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TextChannel {
    id: ChannelIdSize,
    name: String,
    pub num_messages: MessageIdSize,
    pub pending_mention: bool,
    pub chat_history: Vec<TextChannelMessage>,
    pub users_typing: Vec<(UserIdSize, DateTime<Utc>)>,
}

impl TextChannel {
    pub fn new(id: ChannelIdSize, name: String) -> TextChannel {
        TextChannel {
            id,
            name,
            num_messages: 0,
            pending_mention: false,
            chat_history: Vec::new(),
            users_typing: Vec::new(),
        }
    }

    pub fn get_id(&self) -> &ChannelIdSize {
        &self.id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    // When an id is generated, increment the number for next time
    pub fn generate_message_id(&mut self) -> MessageIdSize {
        let id = self.num_messages;
        self.num_messages += 1;
        id
    }

    // pub fn push_image(&mut self, user_id: UserIdSize, image: Vec<u8>) {
    //     self.chat_history.push((user_id, Some(image), Vec::new()));
    // }
}
