use timer_device_list::TimerDeviceList;

pub mod timer_device_list;

#[derive(Default)]
pub struct Clock {
    cycles: u64,
    pub timer_devices: TimerDeviceList,
}

impl Clock {
    #[cfg(test)]
    pub fn new(cycles: u64) -> Self {
        Clock {
            cycles,
            ..Default::default()
        }
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

        self.timer_devices.sync(self.cycles);
    }
}
