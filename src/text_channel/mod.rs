use crate::types::{ChannelIdSize, MessageIdSize, TextMessageChunks, UserIdSize};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TextChannel {
    id: ChannelIdSize,
    name: String,
    pub num_messages: MessageIdSize,
    pub pending_mention: bool,
    pub chat_history: Vec<(
        // User ID
        UserIdSize,
        // Time sent
        Option<DateTime<Utc>>,
        // Optional Vec<u8> to hold images
        Option<Vec<u8>>,
        // Vec to hold chunks of strings with id mentions
        TextMessageChunks,
        // id of this message
        Option<MessageIdSize>,
    )>,
}

impl TextChannel {
    pub fn new(id: ChannelIdSize, name: String) -> TextChannel {
        TextChannel {
            id,
            name,
            num_messages: 0,
            pending_mention: false,
            chat_history: Vec::new(),
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
