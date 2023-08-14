use crate::types::UserIdSize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    id: UserIdSize,
    username: String,
}

impl User {
    pub fn new(id: UserIdSize, username: String) -> User {
        User {
            id: id,
            username: username,
        }
    }

    pub fn get_id(self: &Self) -> UserIdSize {
        self.id
    }

    pub fn get_username(self: &Self) -> &str {
        &self.username
    }
}
