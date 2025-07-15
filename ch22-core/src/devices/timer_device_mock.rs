use std::{cell::RefCell, rc::Rc};

use super::timer_device::TimerDevice;

#[derive(Default)]
pub struct TimerDeviceMock {
    sync_result: Option<u64>,
    accesses: Rc<RefCell<TimerDeviceAccessess>>,
}

impl TimerDeviceMock {
    pub fn new(sync_result: Option<u64>) -> Self {
        TimerDeviceMock {
            sync_result,
            ..TimerDeviceMock::default()
        }
    }

    pub fn get_accesses(&self) -> Rc<RefCell<TimerDeviceAccessess>> {
        self.accesses.clone()
    }
}

impl TimerDevice for TimerDeviceMock {
    fn sync(&mut self, cycles: u64) -> Option<u64> {
        self.accesses.borrow_mut().syncs.push(cycles);

        self.sync_result
    }
}

#[derive(Default)]
pub struct TimerDeviceAccessess {
    pub syncs: Vec<u64>,
}
