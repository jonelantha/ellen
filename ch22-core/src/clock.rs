use crate::devices::timer_device_list::TimerDeviceList;

pub struct Clock<'a> {
    cycles: &'a mut u64,
    timer_devices: &'a mut TimerDeviceList,
}

impl<'a> Clock<'a> {
    pub fn new(cycles: &'a mut u64, timer_devices: &'a mut TimerDeviceList) -> Self {
        Clock {
            cycles,
            timer_devices,
        }
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

        self.timer_devices.sync(*self.cycles);
    }
}
