use crate::MAX_DATAGRAM_SIZE;

use std::net::SocketAddr;
use std::time::Instant;

pub struct DelayedSendPacket {
    pub data: [u8; MAX_DATAGRAM_SIZE],
    pub data_len: usize,
    pub to: SocketAddr,
    pub instant: Instant,
}

impl Ord for DelayedSendPacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        #[allow(clippy::comparison_chain)]
        // Clippy was saying to use match with a cmp here instead... lol THIS is the definition of cmp
        if self.instant > other.instant {
            std::cmp::Ordering::Less
        } else if self.instant < other.instant {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for DelayedSendPacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DelayedSendPacket {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl Eq for DelayedSendPacket {}
