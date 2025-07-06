use super::timer_device::TimerDevice;

type TimerDeviceID = usize;

#[derive(Default)]
pub struct TimerDeviceList {
    device_list: Vec<Box<dyn TimerDevice>>,
}

impl TimerDeviceList {
    pub fn sync(&mut self, cycles: u64) {
        for device in self.device_list.iter_mut() {
            device.sync(cycles);
        }
    }

    pub fn add_device(&mut self, device: Box<dyn TimerDevice>) -> TimerDeviceID {
        self.device_list.push(device);

        // assumes devices will not be removed
        let device_id = self.device_list.len() - 1;

        device_id
    }

    pub fn set_device_trigger(&mut self, device_id: TimerDeviceID, trigger: Option<u64>) {
        self.device_list[device_id].set_trigger(trigger);
    }
}
