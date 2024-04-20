use crate::audio_file_buffer::AudioFileBuffer;
use message::message::{Message, MessageHeader, MessageType};

use chrono::Utc;

pub struct AudioBroadcaster {
    // Current header to send audio over
    voice_header: Option<MessageHeader>,

    // Place to hold our audio buffers to broadcast
    audio_file_buffer: AudioFileBuffer,
}

impl AudioBroadcaster {
    pub fn new() -> AudioBroadcaster {
        AudioBroadcaster {
            voice_header: None,
            audio_file_buffer: AudioFileBuffer::new(),
        }
    }

    pub fn set_header(&mut self, header: Option<MessageHeader>) {
        self.voice_header = header;
    }

    pub fn queue_audio(&mut self, audio: Vec<Vec<u8>>) {
        self.audio_file_buffer.queue_audio(audio);
    }

    pub fn clear_buffers(&mut self) {
        self.audio_file_buffer.clear_buffers();
    }

    pub fn get_next_message(&mut self) -> Option<Message> {
        match self.voice_header {
            Some(mut header) => match self.audio_file_buffer.get_next_frame() {
                Some(audio) => {
                    header.datetime = Some(Utc::now());
                    Some(Message::from(MessageType::Audio((header, audio))))
                }
                None => None,
            },
            None => None,
        }
    }
}
