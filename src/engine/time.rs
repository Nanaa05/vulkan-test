use std::time::Instant;

pub struct Time {
    last: Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32();
        self.last = now;
        dt
    }
}
