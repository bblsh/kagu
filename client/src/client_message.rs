use std::path::PathBuf;

use types::FileTransferIdSize;

#[derive(Debug)]
pub enum ClientMessage {
    ConnectedToServer,
    BeginFileTransfer((FileTransferIdSize, PathBuf)),
}
