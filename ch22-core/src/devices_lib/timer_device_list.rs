use super::timer_device::TimerDevice;

pub type TimerDeviceID = usize;

#[derive(Default)]
pub struct TimerDeviceList {
    devices_and_triggers: Vec<(Box<dyn TimerDevice>, Option<u64>)>,
    next_sync: Option<u64>,
}

impl TimerDeviceList {
    pub fn add_device(&mut self, device: Box<dyn TimerDevice>) -> TimerDeviceID {
        self.devices_and_triggers.push((device, None));

        // assumes devices will not be removed
        self.devices_and_triggers.len() - 1
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.devices_and_triggers[device_id].1 = trigger;

        self.update_next_sync();
    }

    pub fn needs_sync(&mut self, cycles: u64) -> bool {
        self.next_sync.is_some_and(|next_sync| next_sync == cycles)
    }

    pub fn sync(&mut self, cycles: u64) {
        if !self.needs_sync(cycles) {
            return;
        }

        self.devices_and_triggers
            .iter_mut()
            .filter(|(_, trigger)| *trigger == Some(cycles))
            .for_each(|(device, trigger)| *trigger = device.sync(cycles));

        self.update_next_sync();
    }

    fn update_next_sync(&mut self) {
        self.next_sync = self
            .devices_and_triggers
            .iter()
            .filter_map(|(_, trigger)| *trigger)
            .min();
    }
}
