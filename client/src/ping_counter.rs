use types::PingIdSize;

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PingCounter {
    ping_id: Option<PingIdSize>,
    ping_time: Instant,
}

impl Default for PingCounter {
    fn default() -> PingCounter {
        PingCounter {
            ping_id: None,
            ping_time: Instant::now(),
        }
    }
}

impl PingCounter {
    pub fn new() -> PingCounter {
        PingCounter::default()
    }

    pub fn generate_id(&mut self) -> PingIdSize {
        let id = self.ping_id.unwrap_or(0);
        let id = id + 1;
        self.ping_id = Some(id);
        self.ping_time = Instant::now();

        id
    }

    pub fn get_rtt_latency(&self) -> Duration {
        Instant::now() - self.ping_time
    }

    pub fn last_ping(&self) -> Option<Instant> {
        self.ping_id.map(|_| self.ping_time)
    }
}
