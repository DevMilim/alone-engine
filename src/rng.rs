use std::{
    ops::Range,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Random {
    seed: u64,
}

impl Random {
    pub fn time_seed() -> Self {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let seed = duration.as_secs() ^ duration.subsec_nanos() as u64;
        Self { seed }
    }
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);

        self.seed
    }

    pub fn range(&mut self, range: Range<i32>) -> i32 {
        range.start + (self.next_u64() % (range.end - range.start) as u64) as i32
    }
}
