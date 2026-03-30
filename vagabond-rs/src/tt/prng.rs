pub struct PRNG {
    state: u64,
}

impl PRNG {
    // Xorshift
    pub(crate) fn new(seed: u64) -> Self {
        assert!(seed != 0);
        PRNG { state: seed }
    }

    pub(crate) fn rand_64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}
