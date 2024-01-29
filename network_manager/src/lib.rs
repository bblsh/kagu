pub mod connection_manager;
//pub mod network_manager;
mod delayed_send_packet;
pub mod quiche_config;
pub mod socket_manager;

/// Max size of a datagram after IPv6 and UDP headers.
pub const MAX_DATAGRAM_SIZE: usize = 1232;

/// Server/domain name that should matching the server cert.
pub const SERVER_NAME: &str = "localhost";

/// Application-Layer Protocol Negotiation name used to define the QUIC protocol used in this application.
pub const ALPN_NAME: &[u8] = b"kagu";

pub enum NextInstantType {
    NextTick,
    DelayedSend,
    ConnectionTimeout(usize), // usize index always valid...? double check logic later
}

pub enum UpdateEvent {
    NoUpdate,
    NextTick,
    ReceivedData,
    PotentiallyReceivedData,
    FinishedReceiving,
    SocketManagerError,
    ConnectionManagerError,
    NewConnectionStarted(u64),
    FinishedConnectingOnce(u64),
    ConnectionClosed(u64),
    StreamReceivedData(StreamRecvData),
    StreamReceivedError,
}

pub struct StreamRecvData {
    pub conn_id: u64,
    pub stream_id: u64,
    pub data_size: usize,
    pub is_finished: bool,
}
