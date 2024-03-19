use std::collections::{BTreeMap, VecDeque};

use types::UserIdSize;

pub struct AudioBuffer {
    buffer: VecDeque<[f32; 480]>,
    //start: std::time::Instant,
}

impl AudioBuffer {
    pub fn new() -> AudioBuffer {
        AudioBuffer {
            buffer: VecDeque::new(),
            //start: std::time::Instant::now(),
        }
    }

    pub fn is_buffered(&self) -> bool {
        self.buffer.len() > 1
    }

    pub fn push_back(&mut self, data: [f32; 480]) {
        self.buffer.push_back(data);
    }

    pub fn pop_front(&mut self) -> [f32; 480] {
        self.buffer.pop_front().unwrap_or([0.0; 480])
    }
}
