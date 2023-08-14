#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Command {
    Image,
}

impl Command {
    pub fn to_str(&self) -> String {
        match self {
            Command::Image => String::from("image"),
        }
    }

    pub fn get_commands() -> Vec<Command> {
        vec![Command::Image]
    }
}
