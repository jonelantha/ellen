use crate::devices_lib::timer_device_list::TimerDeviceList;

#[derive(Default)]
pub struct Clock {
    cycles: u64,
    pub timer_devices: TimerDeviceList,
}

impl Clock {
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
