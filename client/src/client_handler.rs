use crossbeam::channel::{Receiver, Sender};
use message::message::{Message, MessageType};
use user::User;

use swiftlet_quic::endpoint::{ConnectionId, Endpoint};
use swiftlet_quic::Events;

const MESSAGE_HEADER_SIZE: usize = 2;

pub struct ClientHandler {
    connected: bool,
    user: Option<User>,
    connection_id: Option<ConnectionId>,
    probable_index: usize,
    msg_type_recv: Option<Message>,
    reading_header: bool,
    outgoing_receiver: Receiver<Message>,
    incoming_sender: Sender<Message>,
}

impl ClientHandler {
    pub fn new(outgoing_receiver: Receiver<Message>, incoming_sender: Sender<Message>) -> Self {
        ClientHandler {
            connected: false,
            user: None,
            connection_id: None,
            probable_index: 0,
            msg_type_recv: None,

            // Set to true because we tell our endpoint how many bytes to expect at first
            reading_header: true,

            outgoing_receiver,
            incoming_sender,
        }
    }

    fn handle_stream_msg(
        &mut self,
        _endpoint: &mut Endpoint,
        conn_id: &ConnectionId,
        read_data: &[u8],
    ) -> bool {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(read_data);
        let message = Message::from_vec_u8(buffer).unwrap();

        match message.message {
            MessageType::LoginSuccess(User) => {
                println!("Successful login?");
            }
            _ => return false,
        };

        true
    }

    #[inline]
    fn get_stream_msg_size(&self, read_data: &[u8]) -> usize {
        usize::from_ne_bytes([read_data[1], read_data[2], 0, 0, 0, 0, 0, 0])
    }

    fn send_message(&self, endpoint: &mut Endpoint, message: Message) {
        if let Some(connection_id) = &self.connection_id {
            let message_buffer = message.into_vec_u8().unwrap();
            let mut send_buffer = Vec::new();
            send_buffer.extend(u16::try_from(message_buffer.len()).unwrap().to_le_bytes());
            send_buffer.extend(message_buffer);

            let _ = endpoint.main_stream_send(connection_id, send_buffer);
        }
    }
}

impl Events for ClientHandler {
    fn connection_started(&mut self, endpoint: &mut Endpoint, cid: &ConnectionId) {
        self.connected = true;
    }

    fn connection_ending_warning(&mut self, _endpoint: &mut Endpoint, cid: &ConnectionId) {
        if let Some(my_conn_id) = &self.connection_id {
            if *my_conn_id == *cid {
                self.connection_id = None;
                self.msg_type_recv = None;
            }
        }
    }

    fn connection_ended(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        remaining_connections: usize,
    ) -> bool {
        if let Some(my_conn_id) = &self.connection_id {
            if *my_conn_id == *cid {
                self.connection_id = None;
                self.probable_index = 0;
                self.msg_type_recv = None;
            }
        }

        false
    }

    fn tick(&mut self, endpoint: &mut Endpoint) -> bool {
        // Check to see if there's anything to send
        while let Ok(message) = self.outgoing_receiver.try_recv() {
            self.send_message(endpoint, message);
        }

        false
    }

    fn debug_text(&mut self, text: &'static str) {}

    fn main_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        conn_id: &ConnectionId,
        read_data: &[u8],
    ) -> Option<usize> {
        if let Some(connection_id) = &mut self.connection_id {
            if *connection_id == *conn_id {
                connection_id.update(conn_id);

                if self.reading_header {
                    self.reading_header = false;
                    Some(self.get_stream_msg_size(read_data))
                } else if self.handle_stream_msg(endpoint, conn_id, read_data) {
                    self.reading_header = true;
                    Some(MESSAGE_HEADER_SIZE)
                } else {
                    None
                }
            } else {
                None
            }
        } else if read_data.len() == MESSAGE_HEADER_SIZE {
            // This is from a new connection (new server?) so handle this header
            self.reading_header = false;
            Some(self.get_stream_msg_size(read_data))
        } else {
            None
        }
    }

    fn background_stream_recv(
        &mut self,
        endpoint: &mut Endpoint,
        cid: &ConnectionId,
        read_data: &[u8],
    ) -> Option<usize> {
        None
    }
}
