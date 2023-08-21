use crate::types::{ChannelIdSize, UserIdSize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TextChannel {
    id: ChannelIdSize,
    name: String,
    pub pending_mention: bool,
    pub chat_history: Vec<(
        // User ID
        UserIdSize,
        // Optional Vec<u8> to hold images
        Option<Vec<u8>>,
        // Vec to hold chunks of strings with id mentions
        Vec<(String, Option<UserIdSize>)>,
    )>,
}

impl TextChannel {
    pub fn new(id: ChannelIdSize, name: String) -> TextChannel {
        TextChannel {
            id,
            name,
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

    pub fn push_image(&mut self, user_id: UserIdSize, image: Vec<u8>) {
        self.chat_history.push((user_id, Some(image), Vec::new()));
    }
}
