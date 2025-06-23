use std::collections::HashMap;
use std::collections::hash_map::ValuesMut;

use crate::word::Word;

use super::device::Ch22Device;
use super::io_device::Ch22IODevice;

pub struct Ch22IOSpace {
    interrupt: u8,
    devices: DeviceList,
}

impl Ch22IOSpace {
    pub fn new() -> Self {
        Ch22IOSpace {
            interrupt: 0,
            devices: DeviceList::default(),
        }
    }

    pub fn add_device(&mut self, addresses: &[u16], device: Box<dyn Ch22IODevice>) -> u8 {
        self.devices.add_device(addresses, device)
    }

    pub fn get_interrupt(&mut self, cycles: u32) -> u8 {
        self.devices
            .get_all_mut()
            .for_each(|device| device.sync(cycles, &mut self.interrupt));

        self.interrupt
    }

    pub fn set_interrupt(&mut self, mask: u8, interrupt_flags: u8) {
        self.interrupt = (self.interrupt & !mask) | (interrupt_flags & mask);
    }

    pub fn set_device_trigger(&mut self, device_id: u8, trigger: Option<u32>) {
        self.devices
            .get_device_by_id(device_id)
            .set_trigger(trigger);
    }
}

impl Ch22Device for Ch22IOSpace {
    fn read(&mut self, address: Word, cycles: &mut u32) -> u8 {
        let Some(device) = self.devices.get_device_mut(address) else {
            return 0xff;
        };

        let is_slow = device.is_slow();

        if is_slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let value = device.read(address, *cycles, &mut self.interrupt);

        if is_slow {
            *cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8, cycles: &mut u32) -> bool {
        let Some(device) = self.devices.get_device_mut(address) else {
            return false;
        };

        let is_slow = device.is_slow();

        if is_slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let needs_phase_2 = device.write(address, value, *cycles, &mut self.interrupt);

        if is_slow {
            *cycles += 1;
        }

        needs_phase_2
    }

    fn phase_2(&mut self, address: Word, cycles: u32) {
        if let Some(device) = self.devices.get_device_mut(address) {
            device.phase_2(cycles, &mut self.interrupt);
        }
    }
}

#[derive(Default)]
struct DeviceList {
    next_device_id: u8,
    devices: HashMap<u8, Box<dyn Ch22IODevice>>,
    address_to_device_id: HashMap<Word, u8>,
}

impl DeviceList {
    pub fn add_device(&mut self, addresses: &[u16], device: Box<dyn Ch22IODevice>) -> u8 {
        let device_id = self.next_device_id;

        self.devices.insert(device_id, device);

        for address in addresses {
            self.address_to_device_id
                .insert((*address).into(), device_id);
        }

        self.next_device_id += 1;

        device_id
    }

    fn get_device_by_id(&mut self, device_id: u8) -> &mut dyn Ch22IODevice {
        self.devices.get_mut(&device_id).unwrap().as_mut()
    }

    fn get_device_mut(&mut self, address: Word) -> Option<&mut dyn Ch22IODevice> {
        if let Some(device_id) = self.address_to_device_id.get(&address) {
            Some(self.devices.get_mut(device_id).unwrap().as_mut())
        } else {
            None
        }
    }

    fn get_all_mut(&mut self) -> ValuesMut<u8, Box<dyn Ch22IODevice>> {
        self.devices.values_mut()
    }
}
