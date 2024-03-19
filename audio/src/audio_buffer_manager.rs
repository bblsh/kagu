use std::collections::BTreeMap;

use crate::audio_buffer::AudioBuffer;
use types::UserIdSize;

pub struct AudioBufferManager {
    buffers: BTreeMap<UserIdSize, AudioBuffer>,
}

impl AudioBufferManager {
    pub fn new() -> AudioBufferManager {
        AudioBufferManager {
            buffers: BTreeMap::new(),
        }
    }

    pub fn buffer_data(&mut self, user_id: UserIdSize, data: [f32; 480]) {
        if let Some(buffer) = self.buffers.get_mut(&user_id) {
            buffer.push_back(data);
        } else {
            let mut buffer = AudioBuffer::new();
            buffer.push_back(data);

            self.buffers.insert(user_id, buffer);
        }
    }

    pub fn get_output_data(&mut self) -> [f32; 480] {
        let mut output_buffer: [f32; 480] = [0.0; 480];

        for buffer in self.buffers.values_mut() {
            if buffer.is_buffered() {
                let user_audio = buffer.pop_front();
                for i in 0..480 {
                    output_buffer[i] += user_audio[i];
                }
            }
        }

        output_buffer
    }
}
