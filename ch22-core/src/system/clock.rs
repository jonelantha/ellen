pub struct Clock<'a> {
    cycles: &'a mut u64,
}

impl<'a> Clock<'a> {
    pub fn new(cycles: &'a mut u64) -> Self {
        Clock { cycles }
    }

    pub fn get_cycles(&self) -> u64 {
        *self.cycles
    }

    pub fn one_mhz_sync(&mut self) {
        if *self.cycles & 1 != 0 {
            self.inc();
        }
    }

    pub fn inc(&mut self) {
        *self.cycles += 1;
    }
}
