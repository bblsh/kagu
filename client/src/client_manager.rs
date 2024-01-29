use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::time::Instant;

use network_manager::connection_manager::ConnectionManager;
use network_manager::quiche_config::QuicheConfig;
use network_manager::socket_manager::{SocketError, SocketManager};
use network_manager::NextInstantType;
use network_manager::UpdateEvent;
use network_manager::{StreamRecvData, MAX_DATAGRAM_SIZE};

use ring::rand::{SecureRandom, SystemRandom};

pub struct ClientManager {
    user_name: String,
    socket_mgr: SocketManager,
    read_data: [u8; 65536],
    rand: SystemRandom,
    config: quiche::Config,
    conn_mgr: ConnectionManager,
    next_tick_instant: Instant,
}

impl ClientManager {
    pub fn new(
        user_name: String,
        bind_addr: SocketAddr,
        peer_addr: SocketAddr,
        alpn: &[u8],
        cert_path: &str,
        server_name: &str,
    ) -> Result<Self, quiche::Error> {
        let (mut socket_mgr, local_addr) = SocketManager::new(bind_addr);

        let rand = SystemRandom::new();

        let mut config = match QuicheConfig::create_quiche_config(alpn, cert_path, None, Some(3)) {
            Ok(cfg) => cfg,
            Err(err) => {
                return Err(err);
            }
        };

        let mut scid_ref = [0; quiche::MAX_CONN_ID_LEN];
        rand.fill(&mut scid_ref[..]).unwrap();
        let scid = quiche::ConnectionId::from_ref(&scid_ref);

        let mut conn_mgr = match ConnectionManager::new(
            Some(server_name),
            1,
            scid.into_owned(),
            local_addr,
            peer_addr,
            &mut config,
            None,
        ) {
            Ok(cm) => cm,
            Err(err) => {
                return Err(err);
            }
        };

        conn_mgr.send_data(&mut socket_mgr);

        let client_state = ClientManager {
            user_name,
            socket_mgr,
            read_data: [0; 65536],
            rand,
            config,
            conn_mgr,
            next_tick_instant: Instant::now(),
        };

        Ok(client_state)
    }

    pub fn close_connection(&mut self, error_code: u64) {
        self.conn_mgr.connection.close(false, error_code, b"reason");
        self.conn_mgr.send_data(&mut self.socket_mgr);
    }

    pub fn is_connection_closed(&self) -> bool {
        self.conn_mgr.connection.is_closed()
    }

    pub fn new_connection(
        &mut self,
        peer_addr: SocketAddr,
        alpn: &[u8],
        cert_path: &str,
        server_name: &str,
    ) -> bool {
        if self.conn_mgr.connection.is_closed() {
            let mut scid_ref = [0; quiche::MAX_CONN_ID_LEN];
            self.rand.fill(&mut scid_ref[..]).unwrap();
            let scid = quiche::ConnectionId::from_ref(&scid_ref);

            let local_addr = self.conn_mgr.recv_info.to;
            let mut conn_mgr = match ConnectionManager::new(
                Some(server_name),
                1,
                scid.into_owned(),
                local_addr,
                peer_addr,
                &mut self.config,
                None,
            ) {
                Ok(cm) => cm,
                Err(err) => {
                    return false;
                }
            };
            conn_mgr.send_data(&mut self.socket_mgr);

            self.conn_mgr = conn_mgr;

            true
        } else {
            false
        }
    }

    fn get_next_instant(&self) -> (Instant, NextInstantType) {
        let mut next_instant = self.next_tick_instant;
        let mut next_instant_type = NextInstantType::NextTick;

        if let Some(delayed_send_packet) = self.socket_mgr.send_queue.peek() {
            if delayed_send_packet.instant < next_instant {
                next_instant = delayed_send_packet.instant;
                next_instant_type = NextInstantType::DelayedSend;
            }
        }

        if let Some(conn_timeout) = self.conn_mgr.connection.timeout_instant() {
            if conn_timeout < next_instant {
                next_instant = conn_timeout;
                next_instant_type = NextInstantType::ConnectionTimeout(1);
            }
        }

        (next_instant, next_instant_type)
    }

    fn send_check(&mut self) {
        while let Some(delayed_send_packet) = self.socket_mgr.send_queue.peek() {
            if delayed_send_packet.instant <= Instant::now() {
                let sp = self.socket_mgr.send_queue.pop();
                match sp {
                    Some(send_packet) => {
                        self.socket_mgr
                            .send_data(&send_packet.data[..send_packet.data_len], send_packet.to);
                    }
                    None => {
                        break; // How...?
                    }
                }
            } else {
                break;
            }
        }
    }

    fn handle_connection_timeout(&mut self) -> bool {
        if let Some(current_connection_timeout) = self.conn_mgr.connection.timeout_instant() {
            if current_connection_timeout <= Instant::now() {
                self.conn_mgr.connection.on_timeout();
                self.conn_mgr.send_data(&mut self.socket_mgr); // Resets timeout instance internally
                self.conn_mgr.connection.is_closed()
            } else {
                self.conn_mgr.next_timeout_instant = Some(current_connection_timeout);
                false
            }
        } else {
            self.conn_mgr.next_timeout_instant = None; // Changed to another value after a send (or recv?)
            false
        }
    }

    pub fn update(&mut self) -> UpdateEvent {
        let (mut next_instant, mut ni_type) = self.get_next_instant();
        while next_instant <= Instant::now() {
            match ni_type {
                NextInstantType::NextTick => {
                    return UpdateEvent::NextTick;
                }
                NextInstantType::DelayedSend => {
                    self.send_check();
                }
                NextInstantType::ConnectionTimeout(_) => {
                    if self.handle_connection_timeout() {
                        return UpdateEvent::ConnectionClosed(1);
                    }
                }
            }
            (next_instant, ni_type) = self.get_next_instant();
        }

        let sleep_duration = next_instant.duration_since(Instant::now());
        if sleep_duration.as_millis() > 0 && self.socket_mgr.sleep_till_recv_data(sleep_duration) {
            return UpdateEvent::ReceivedData;
        }

        UpdateEvent::PotentiallyReceivedData
    }

    pub fn set_next_tick_instant(&mut self, next_tick_instant: Instant) {
        self.next_tick_instant = next_tick_instant;
    }

    pub fn recv_data(&mut self, stream_recv_data: &mut [u8]) -> UpdateEvent {
        match self.socket_mgr.recv_data(&mut self.read_data) {
            Ok((recv_size, addr_from)) => {
                if recv_size <= MAX_DATAGRAM_SIZE {
                    // Only look at datagram if it is less than or equal to the max
                    match self.conn_mgr.recv_data(
                        &mut self.read_data[..recv_size],
                        addr_from,
                        &mut self.socket_mgr,
                    ) {
                        Ok(num_sends) => {
                            if self.conn_mgr.connected_once {
                                if self.conn_mgr.connection.is_closed() {
                                    return UpdateEvent::ConnectionClosed(1);
                                } else if let Some(next_readable_stream) =
                                    self.conn_mgr.connection.stream_readable_next()
                                {
                                    match self
                                        .conn_mgr
                                        .connection
                                        .stream_recv(next_readable_stream, stream_recv_data)
                                    {
                                        Ok((data_size, is_finished)) => {
                                            self.conn_mgr.send_data(&mut self.socket_mgr);

                                            let recv_data = StreamRecvData {
                                                conn_id: 1,
                                                stream_id: next_readable_stream,
                                                data_size,
                                                is_finished,
                                            };

                                            return UpdateEvent::StreamReceivedData(recv_data);
                                        }
                                        Err(err) => {
                                            return UpdateEvent::StreamReceivedError;
                                        }
                                    }
                                }
                            } else if self.conn_mgr.connection.is_established() {
                                self.conn_mgr.connected_once = true;
                                return UpdateEvent::FinishedConnectingOnce(1);
                            } else if self.conn_mgr.connection.is_closed() {
                                return UpdateEvent::ConnectionClosed(1);
                            }
                            UpdateEvent::NoUpdate
                        }
                        Err(_) => UpdateEvent::ConnectionManagerError,
                    }
                } else {
                    UpdateEvent::NoUpdate
                }
            }
            Err(SocketError::RecvBlocked) => UpdateEvent::FinishedReceiving,
            Err(_) => UpdateEvent::SocketManagerError,
        }
    }

    pub fn create_stream(&mut self, stream_id: u64, priority: u8) -> Result<(), quiche::Error> {
        self.conn_mgr
            .connection
            .stream_priority(stream_id, priority, true)
    }

    pub fn send_stream_data(
        &mut self,
        stream_id: u64,
        data: &[u8],
        is_final: bool,
    ) -> Result<u64, quiche::Error> {
        // Do connection checking here in future
        match self
            .conn_mgr
            .connection
            .stream_send(stream_id, data, is_final)
        {
            Ok(bytes_written) => {
                if bytes_written == data.len() {
                    match self.conn_mgr.send_data(&mut self.socket_mgr) {
                        Ok(num_sends) => Ok(num_sends),
                        Err(err) => Err(err),
                    }
                } else {
                    Err(quiche::Error::BufferTooShort)
                }
            }
            Err(err) => Err(err),
        }
    }
}
