use std::net::SocketAddr;
use std::time::Instant;

use crate::MAX_DATAGRAM_SIZE;
use crate::{delayed_send_packet::DelayedSendPacket, socket_manager::SocketManager};

pub struct ConnectionManager {
    pub id: u64,
    pub dcid_matcher: quiche::ConnectionId<'static>,
    pub connection: quiche::Connection,
    pub recv_info: quiche::RecvInfo,
    pub next_timeout_instant: Option<Instant>,
    pub connected_once: bool,
}

impl ConnectionManager {
    pub fn new(
        server_name: Option<&str>,
        id: u64,
        scid: quiche::ConnectionId<'static>,
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        config: &mut quiche::Config,
        writer_opt: Option<Box<std::fs::File>>,
    ) -> Result<Self, quiche::Error> {
        let recv_info = quiche::RecvInfo {
            from: local_addr,
            to: local_addr,
        };

        if server_name.is_some() {
            let connection =
                match quiche::connect(server_name, &scid, local_addr, peer_addr, config) {
                    Ok(conn) => conn,
                    Err(err) => {
                        return Err(err);
                    }
                };

            let next_timeout_instant = connection.timeout_instant();

            let conn_mgr = ConnectionManager {
                id,
                dcid_matcher: scid,
                connection,
                recv_info,
                next_timeout_instant,
                connected_once: false,
            };

            Ok(conn_mgr)
        } else {
            let connection = match quiche::accept(&scid, None, local_addr, peer_addr, config) {
                Ok(mut conn) => {
                    if let Some(writer) = writer_opt {
                        // called before recv
                        conn.set_keylog(writer);
                    }
                    conn
                }
                Err(err) => {
                    return Err(err);
                }
            };

            let next_timeout_instant = connection.timeout_instant();

            let conn_mgr = ConnectionManager {
                id,
                dcid_matcher: scid,
                connection,
                recv_info,
                next_timeout_instant,
                connected_once: false,
            };

            Ok(conn_mgr)
        }
    }

    pub fn send_data(&mut self, socket_mgr: &mut SocketManager) -> Result<u64, quiche::Error> {
        let mut num_sends = 0;
        loop {
            let mut next_send_packet = [0; MAX_DATAGRAM_SIZE];
            match self.connection.send(&mut next_send_packet) {
                Ok((write_len, send_info)) => {
                    if send_info.at <= Instant::now() {
                        match socket_mgr.send_data(&next_send_packet[..write_len], send_info.to) {
                            None => num_sends += 1,
                            Some(_err) => {
                                return Err(quiche::Error::Done); // Better Error Handling in Future
                            }
                        }
                    } else {
                        let delayed_send_packet = DelayedSendPacket {
                            data: next_send_packet,
                            data_len: write_len,
                            to: send_info.to,
                            instant: send_info.at,
                        };
                        socket_mgr.send_queue.push(delayed_send_packet);
                        num_sends += 1;
                    }
                }
                Err(quiche::Error::Done) => {
                    self.next_timeout_instant = self.connection.timeout_instant();
                    return Ok(num_sends);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    pub fn recv_data(
        &mut self,
        data: &mut [u8],
        addr_from: SocketAddr,
        socket_mgr: &mut SocketManager,
    ) -> Result<u64, quiche::Error> {
        self.recv_info.from = addr_from;

        // Does it handle potentially coalesced packets?
        match self.connection.recv(data, self.recv_info) {
            Ok(_read_size) => {
                self.send_data(socket_mgr) //to handle ACKs
            }
            Err(err) => Err(err),
        }
    }
}
