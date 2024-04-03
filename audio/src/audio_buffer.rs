use std::collections::VecDeque;

pub struct AudioBuffer {
    buffer: VecDeque<[f32; 960]>,
}

impl AudioBuffer {
    pub fn new() -> AudioBuffer {
        AudioBuffer {
            buffer: VecDeque::new(),
        }
    }

    pub fn is_buffered(&self) -> bool {
        self.buffer.len() > 1
    }

    pub fn push_back(&mut self, data: [f32; 960]) {
        self.buffer.push_back(data);
    }

    pub fn pop_front(&mut self) -> [f32; 960] {
        self.buffer.pop_front().unwrap_or([0.0; 960])
    }
}
