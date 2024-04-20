use std::path::PathBuf;

use message::message::MessageHeader;
use types::FileTransferIdSize;

#[derive(Debug)]
pub enum ClientMessage {
    ConnectedToServer,

    // File transfer initiation
    BeginFileTransfer((FileTransferIdSize, PathBuf)),

    // Audio broadcasting control messages
    UpdateVoiceHeader(Option<MessageHeader>),
    BroadcastBuffer(Vec<Vec<u8>>),
    PauseBroadcasting,
    ResumeBroadcasting,
    StopBroadcasting,
}
