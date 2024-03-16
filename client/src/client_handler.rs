use message::message::{Message, MessageType};
use network_manager::*;
use user::User;

use crossbeam::channel::{Receiver, Sender};
use swiftlet_quic::endpoint::{ConnectionEndReason, ConnectionId, Endpoint};
use swiftlet_quic::EndpointEventCallbacks;

pub struct ClientHandler {
    connected: bool,
    _user: Option<User>, // not used yet?
    connection_id: Option<ConnectionId>,
    outgoing_receiver: Receiver<Message>,
    incoming_sender: Sender<Message>,
    audio_in_sender: Sender<Message>,
}

impl ClientHandler {
    pub fn new(
        outgoing_receiver: Receiver<Message>,
        incoming_sender: Sender<Message>,
        audio_in_sender: Sender<Message>,
    ) -> Self {
        ClientHandler {
            connected: false,
            _user: None,
            connection_id: None,
            outgoing_receiver,
            incoming_sender,
            audio_in_sender,
        }
    }

    fn process_message(&mut self, _cid: &ConnectionId, message: Message, _endpoint: &mut Endpoint) {
        match message.message {
            MessageType::Audio(_) => self.audio_in_sender.send(message).unwrap(),
            _ => self.incoming_sender.send(message).unwrap(),
        }
    }

    #[inline]
    fn get_message_size(&self, read_data: &[u8]) -> usize {
        usize::from_ne_bytes([read_data[0], read_data[1], 0, 0, 0, 0, 0, 0])
    }

    fn send_message(&self, endpoint: &mut Endpoint, message: Message) {
        if let Some(connection_id) = &self.connection_id {
            let message_buffer = message.into_vec_u8().unwrap();
            let mut send_buffer = Vec::new();
            send_buffer.extend(u16::try_from(message_buffer.len()).unwrap().to_ne_bytes());
            send_buffer.extend(message_buffer);

            let _ = endpoint.main_stream_send(connection_id, send_buffer);
        }
    }
}

impl EndpointEventCallbacks for ClientHandler {
    fn connection_started(&mut self, _endpoint: &mut Endpoint, cid: &ConnectionId) {
        self.connected = true;
        self.connection_id = Some(*cid);
    }

    fn connection_ended(
        &mut self,
        _endpoint: &mut Endpoint,
        cid: &ConnectionId,
        _reason: ConnectionEndReason,
        _remaining_connections: usize,
    ) -> bool {
        if let Some(my_conn_id) = &self.connection_id {
            if *my_conn_id == *cid {
                self.connection_id = None;
            }
        }

        false
    }

    fn tick(&mut self, endpoint: &mut Endpoint) -> bool {
        let mut exit = false;

        // Check to see if there's anything to send
        while let Ok(message) = self.outgoing_receiver.try_recv() {
            // If we're sending a Disconnecting message, we know to exit after sending it
            if let MessageType::Disconnecting(_) = message.message {
                exit = true;
            }
            self.send_message(endpoint, message);

            if exit {
                let _ = endpoint.close_connection(&self.connection_id.unwrap(), 0);
            }
        }

        exit
    }

    fn main_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        _cid: &ConnectionId,
        read_data: &[u8],
    ) -> Option<usize> {
        if read_data.len() == MESSAGE_HEADER_SIZE {
            Some(self.get_message_size(read_data))
        } else {
            // We know this is (likely) a message
            let message_buffer = read_data.to_vec();
            let message = Message::from_vec_u8(message_buffer).unwrap();

            self.process_message(_cid, message, endpoint);

            // Tell swiftlet to read another message header
            Some(MESSAGE_HEADER_SIZE)
        }
    }
}
