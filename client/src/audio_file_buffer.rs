use std::collections::VecDeque;

pub struct AudioFileBuffer {
    audio_files: VecDeque<VecDeque<Vec<u8>>>,
    current_file: VecDeque<Vec<u8>>,
}

impl AudioFileBuffer {
    pub fn new() -> AudioFileBuffer {
        AudioFileBuffer {
            audio_files: VecDeque::new(),
            current_file: VecDeque::new(),
        }
    }

    pub fn queue_audio(&mut self, audio: Vec<Vec<u8>>) {
        // If nothing is currently playing, queue the song
        if self.current_file.is_empty() {
            self.current_file = VecDeque::from(audio);
        } else {
            let audio_file = VecDeque::from(audio);
            self.audio_files.push_back(audio_file);
        }
    }

    pub fn get_next_frame(&mut self) -> Option<Vec<u8>> {
        // Pop the front frame (if there is one)
        let frame = self.current_file.pop_front();

        // If there's nothing left in the current buffer, load the next buffer
        if self.current_file.is_empty() {
            if let Some(buffer) = self.audio_files.pop_front() {
                self.current_file = buffer
            }
        }

        frame
    }

    pub fn clear_buffers(&mut self) {
        self.audio_files.clear();
    }
}
