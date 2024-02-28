// /// Max size of a datagram after IPv6 and UDP headers.
// pub const MAX_DATAGRAM_SIZE: usize = 1232;

// /// Server/domain name that should matching the server cert.
// pub const SERVER_NAME: &str = "localhost";

// /// Application-Layer Protocol Negotiation name used to define the QUIC protocol used in this application.
// const ALPN_NAME: &[u8] = b"kagu";

pub const BUFFER_SIZE_PER_CONNECTION: usize = 65536;
pub const MESSAGE_HEADER_SIZE: usize = 2;
pub const ALPN_NAME: &[u8] = b"kagu";
