use types::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FileTransfer {
    pub id: FileTransferIdSize,
    pub data: Vec<u8>,
}

impl FileTransfer {
    pub fn new(id: FileTransferIdSize, data: Vec<u8>) -> FileTransfer {
        FileTransfer { id, data }
    }
}
