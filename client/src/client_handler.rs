use crate::audio_broadcaster::AudioBroadcaster;
use crate::client_message::ClientMessage;
use crate::ping_counter::PingCounter;
use message::message::{Message, MessageType};
use network_manager::*;
use user::User;

use crossbeam::channel::{Receiver, Sender};
use swiftlet_quic::endpoint::{ConnectionEndReason, ConnectionId, Endpoint};
use swiftlet_quic::EndpointEventCallbacks;

use std::sync::{Arc, Mutex};

pub struct ClientHandler {
    connected: bool,
    user: Option<User>,
    connection_id: Option<ConnectionId>,
    outgoing_receiver: Receiver<Message>,
    incoming_sender: Sender<Message>,
    audio_in_sender: Sender<Message>,
    el_to_client_sender: Sender<ClientMessage>,
    client_to_el_receiver: Receiver<ClientMessage>,
    ping_counter: PingCounter,
    audio_broadcaster: AudioBroadcaster,
    broadcast_audio: bool,
    send_audio: bool,
    send_idx: u8,
    is_broadcasting: Arc<Mutex<bool>>,
    is_preparing_audio: Arc<Mutex<bool>>,
}

impl ClientHandler {
    pub fn new(
        outgoing_receiver: Receiver<Message>,
        incoming_sender: Sender<Message>,
        audio_in_sender: Sender<Message>,
        el_to_client_sender: Sender<ClientMessage>,
        client_to_el_receiver: Receiver<ClientMessage>,
        is_broadcasting: Arc<Mutex<bool>>,
        is_preparing_audio: Arc<Mutex<bool>>,
    ) -> Self {
        ClientHandler {
            connected: false,
            user: None,
            connection_id: None,
            outgoing_receiver,
            incoming_sender,
            audio_in_sender,
            el_to_client_sender,
            client_to_el_receiver,
            ping_counter: PingCounter::new(),
            audio_broadcaster: AudioBroadcaster::new(),
            broadcast_audio: true,
            send_audio: true,
            send_idx: 0,
            is_broadcasting,
            is_preparing_audio,
        }
    }

    fn process_message(&mut self, _cid: &ConnectionId, message: Message, _endpoint: &mut Endpoint) {
        match message.message {
            MessageType::Audio(_) => {
                // Lazy fix to prevent blocking
                // todo: implement a way to not receive audio
                if !self.audio_in_sender.is_full() {
                    self.audio_in_sender.send(message).unwrap();
                }
            }
            MessageType::PingReply(_) => {
                let duration = self.ping_counter.get_rtt_latency();
                let message = Message::from(MessageType::PingLatency(duration));
                let _ = self.incoming_sender.try_send(message);
            }
            MessageType::LoginSuccess(ref user) => {
                // Save our user in the event loop
                self.user = Some(user.clone());
                self.incoming_sender.send(message).unwrap();
            }
            _ => self.incoming_sender.send(message).unwrap(),
        }
    }

    #[inline]
    fn get_message_size(&self, read_data: &[u8]) -> usize {
        usize::from_ne_bytes([read_data[0], read_data[1], 0, 0, 0, 0, 0, 0])
    }

    fn send_message(&self, realtime: bool, endpoint: &mut Endpoint, message: Message) {
        if let Some(connection_id) = &self.connection_id {
            let message_buffer = message.into_vec_u8().unwrap();
            let mut send_buffer = Vec::new();

            if !realtime {
                send_buffer.extend(u16::try_from(message_buffer.len()).unwrap().to_ne_bytes());
            }

            send_buffer.extend(message_buffer);

            match realtime {
                true => {
                    let _ = endpoint.rt_stream_send(connection_id, Some(send_buffer), true);
                }
                false => {
                    let _ = endpoint.main_stream_send(connection_id, send_buffer);
                }
            }
        }
    }

    fn send_ping(&mut self, endpoint: &mut Endpoint) {
        if let Some(user) = &self.user {
            let ping_id = self.ping_counter.generate_id();
            let mut message = Message::from(MessageType::Ping(ping_id));
            message.user_id = user.get_id();
            self.send_message(false, endpoint, message);
        }
    }

    // keep track of transfer id and if we should be transferring
    fn _start_file_transfer(&mut self) {
        //
    }
}

impl EndpointEventCallbacks for ClientHandler {
    fn connection_started(&mut self, _endpoint: &mut Endpoint, cid: &ConnectionId) {
        self.connected = true;
        self.connection_id = Some(*cid);

        let _ = self
            .el_to_client_sender
            .send(ClientMessage::ConnectedToServer);
    }

    fn connection_ended(
        &mut self,
        _endpoint: &mut Endpoint,
        _cid: &ConnectionId,
        _reason: ConnectionEndReason,
        _remaining_connections: usize,
    ) -> bool {
        // Deal with multiple servers later
        let _ = self
            .incoming_sender
            .send(Message::from(MessageType::ServerShutdown));

        // if let Some(my_conn_id) = &self.connection_id {
        //     if *my_conn_id == *cid {
        //         self.connection_id = None;
        //     }
        // }

        false
    }

    fn tick(&mut self, endpoint: &mut Endpoint) -> bool {
        let mut exit = false;

        if let Some(time) = self.ping_counter.last_ping() {
            let now = std::time::Instant::now();
            let diff = now - time;
            // Send a ping every 5 seconds
            if diff.as_secs() > 5 {
                self.send_ping(endpoint);
            }
        } else {
            // We haven't sent a ping, so send one now
            self.send_ping(endpoint);
        }

        // Check to see if there's anything to send
        while let Ok(message) = self.outgoing_receiver.try_recv() {
            // If we're sending a Disconnecting message, we know to exit after sending it
            match message.message {
                MessageType::Disconnecting(_) => {
                    exit = true;
                    self.send_message(false, endpoint, message);
                }
                MessageType::Audio(_) => {
                    self.send_message(true, endpoint, message);
                }
                MessageType::Ping(_) => self.send_message(true, endpoint, message),
                _ => self.send_message(false, endpoint, message),
            }

            if exit {
                let _ = endpoint.close_connection(&self.connection_id.unwrap(), 0);
            }
        }

        // Check for any audio to broadcast
        if self.broadcast_audio {
            match self.send_audio {
                true => {
                    if let Some(message) = self.audio_broadcaster.get_next_message() {
                        self.send_message(true, endpoint, message);

                        self.send_idx = 0;
                        self.send_audio = false;
                    }
                    // If we don't get any samples to broadcast, verify if audio is being prepared
                    else if let Ok(preparing) = self.is_preparing_audio.lock() {
                        // Audio isn't being prepared, so we aren't broadcasting anything anymore
                        if !*preparing {
                            let mut guard = self.is_broadcasting.lock().unwrap();
                            *guard = false;
                        }
                    }
                }
                false => {
                    // Because we have the event loop set to 5ms, send audio every 10ms
                    self.send_idx += 1;
                    if self.send_idx == 1 {
                        self.send_audio = true;
                    }
                }
            }
        }

        // Check for messages from the external client
        while let Ok(message) = self.client_to_el_receiver.try_recv() {
            match message {
                ClientMessage::BeginFileTransfer(_transfer) => {
                    //
                }
                ClientMessage::UpdateVoiceHeader(header) => {
                    self.audio_broadcaster.set_header(header)
                }
                ClientMessage::BroadcastBuffer(buffer) => {
                    self.audio_broadcaster.queue_audio(buffer);

                    // We got an audio buffer, so we are no longer preparing audio
                    let mut guard = self.is_preparing_audio.lock().unwrap();
                    *guard = false;
                }
                ClientMessage::PauseBroadcasting => self.broadcast_audio = false,
                ClientMessage::ResumeBroadcasting => self.broadcast_audio = true,
                ClientMessage::StopBroadcasting => self.audio_broadcaster.clear_buffers(),
                _ => (),
            }
        }

        exit
    }

    fn main_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        read_data: &[u8],
    ) -> Option<usize> {
        if read_data.len() == MESSAGE_HEADER_SIZE {
            Some(self.get_message_size(read_data))
        } else {
            // We know this is (likely) a message
            let message_buffer = read_data.to_vec();
            let message = Message::from_vec_u8(message_buffer).unwrap();

            self.process_message(cid, message, endpoint);

            // Tell swiftlet to read another message header
            Some(MESSAGE_HEADER_SIZE)
        }
    }

    fn rt_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        read_data: &[u8],
        _rt_id: u64,
    ) -> usize {
        // We know this is (likely) a message
        let message_buffer = read_data.to_vec();
        let message = Message::from_vec_u8(message_buffer).unwrap();

        self.process_message(cid, message, endpoint);

        0
    }
}
