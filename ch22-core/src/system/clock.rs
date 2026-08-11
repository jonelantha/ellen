pub struct Clock {
    cycles: u64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Clock {
    pub fn new(cycles: u64) -> Self {
        Self { cycles }
    }

    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }

    pub fn one_mhz_sync(&mut self) {
        if self.cycles & 1 != 0 {
            self.inc();
        }
    }

    pub fn inc(&mut self) {
        self.cycles += 1;
    }
}
