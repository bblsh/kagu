use serde::{Deserialize, Serialize};
use types::UserIdSize;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    id: UserIdSize,
    username: String,
    about: String,
}

impl User {
    pub fn new(id: UserIdSize, username: String) -> User {
        User {
            id,
            username,
            about: String::new(),
        }
    }

    pub fn get_id(&self) -> UserIdSize {
        self.id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}
