use std::collections::BinaryHeap;
use std::net::SocketAddr;

use crate::delayed_send_packet::DelayedSendPacket;

pub enum SocketError {
    SendSizeWrong,
    SendBlocked,
    SendOtherIssue,
    RecvBlocked,
    RecvOtherIssue,
}

pub struct SocketManager {
    poll: mio::Poll,
    events: mio::Events,
    socket: mio::net::UdpSocket,
    pub send_queue: BinaryHeap<DelayedSendPacket>,
}

impl SocketManager {
    pub fn new(bind_addr: SocketAddr) -> (Self, SocketAddr) {
        let socket = mio::net::UdpSocket::bind(bind_addr).unwrap();
        let local_addr = socket.local_addr().unwrap();
        let mut socket_state = SocketManager {
            poll: mio::Poll::new().unwrap(),
            events: mio::Events::with_capacity(1024),
            socket,
            send_queue: BinaryHeap::new(),
        };
        socket_state
            .poll
            .registry()
            .register(
                &mut socket_state.socket,
                mio::Token(0),
                mio::Interest::READABLE,
            )
            .unwrap();

        (socket_state, local_addr)
    }

    pub fn sleep_till_recv_data(&mut self, timeout: std::time::Duration) -> bool {
        self.poll.poll(&mut self.events, Some(timeout)).unwrap();
        !self.events.is_empty()
    }

    #[inline]
    pub fn send_data(&mut self, data: &[u8], to_addr: SocketAddr) -> Option<SocketError> {
        // Drops packet before it enters network stack if it would block
        // Uncertain if it will partially fill socket (could even be OS dependent)
        match self.socket.send_to(data, to_addr) {
            Ok(send_size) => {
                if send_size != data.len() {
                    Some(SocketError::SendSizeWrong)
                } else {
                    None
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    Some(SocketError::SendBlocked)
                } else {
                    Some(SocketError::SendOtherIssue)
                }
            }
        }
    }

    #[inline]
    pub fn recv_data(&mut self, data: &mut [u8]) -> Result<(usize, SocketAddr), SocketError> {
        match self.socket.recv_from(data) {
            Ok((recv_size, addr_from)) => Ok((recv_size, addr_from)),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    Err(SocketError::RecvBlocked)
                } else {
                    Err(SocketError::RecvOtherIssue)
                }
            }
        }
    }
}
