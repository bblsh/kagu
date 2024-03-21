pub const BUFFER_SIZE_PER_CONNECTION: usize = 65536;
pub const MESSAGE_HEADER_SIZE: usize = 2;

/// Application-Layer Protocol Negotiation name used to define the QUIC protocol used in this application.
pub const ALPN_NAME: &[u8] = b"kagu";
