use serde::{Deserialize, Serialize};
use types::{ChannelIdSize, UserIdSize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VoiceChannel {
    id: ChannelIdSize,
    name: String,
    connected_users: Vec<UserIdSize>,
}

impl VoiceChannel {
    pub fn new(id: ChannelIdSize, name: String) -> VoiceChannel {
        VoiceChannel {
            id,
            name,
            connected_users: Vec::new(),
        }
    }

    pub fn get_id(&self) -> &ChannelIdSize {
        &self.id
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_connected_users(&self) -> &Vec<UserIdSize> {
        &self.connected_users
    }

    pub fn get_connected_users_mut(&mut self) -> &mut Vec<UserIdSize> {
        &mut self.connected_users
    }
}
