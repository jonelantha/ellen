use std::collections::HashMap;

use crate::word::Word;

use super::device::Ch22Device;
use super::io_device::Ch22IODevice;

pub struct Ch22IOSpace {
    next_device_id: u8,
    devices: HashMap<u8, Box<dyn Ch22IODevice>>,
    address_to_device_id: HashMap<Word, u8>,
}

impl Ch22IOSpace {
    pub fn new() -> Self {
        Ch22IOSpace {
            next_device_id: 0,
            devices: HashMap::new(),
            address_to_device_id: HashMap::new(),
        }
    }

    pub fn add_device(&mut self, addresses: &[u16], device: Box<dyn Ch22IODevice>) {
        let device_id = self.next_device_id;

        self.devices.insert(device_id, device);

        for address in addresses {
            self.address_to_device_id
                .insert((*address).into(), device_id);
        }

        self.next_device_id += 1;
    }

    fn get_device_mut(&mut self, address: Word) -> Option<&mut dyn Ch22IODevice> {
        if let Some(device_id) = self.address_to_device_id.get(&address) {
            Some(self.devices.get_mut(device_id).unwrap().as_mut())
        } else {
            None
        }
    }

    pub fn get_interrupt(&mut self, cycles: u32) -> u16 {
        self.devices.values_mut().fold(0, |accumulator, device| {
            accumulator | device.get_interrupt(cycles)
        })
    }
}

impl Ch22Device for Ch22IOSpace {
    fn read(&mut self, address: Word, cycles: &mut u32) -> u8 {
        let Some(device) = self.get_device_mut(address) else {
            return 0xff;
        };

        let is_slow = device.is_slow();

        if is_slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let value = device.read(address, *cycles);

        if is_slow {
            *cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8, cycles: &mut u32) -> bool {
        let Some(device) = self.get_device_mut(address) else {
            return false;
        };

        let is_slow = device.is_slow();

        if is_slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let needs_phase_2 = device.write(address, value, *cycles);

        if is_slow {
            *cycles += 1;
        }

        needs_phase_2
    }

    fn phase_2(&mut self, address: Word, cycles: u32) {
        if let Some(device) = self.get_device_mut(address) {
            device.phase_2(address, cycles);
        }
    }
}
