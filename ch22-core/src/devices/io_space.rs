use std::collections::HashMap;

use crate::interrupt_type::InterruptType;
use crate::word::Word;

use super::addressable_device::AddressableDevice;
use super::io_device::IODevice;

pub type DeviceID = usize;

pub struct IOSpace {
    devices: DeviceList,
}

impl IOSpace {
    pub fn new() -> Self {
        IOSpace {
            devices: DeviceList::default(),
        }
    }

    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        slow: bool,
    ) -> DeviceID {
        self.devices
            .add_device(addresses, device, interrupt_type, slow)
    }

    pub fn get_interrupt(&mut self, interrupt_type: InterruptType, cycles: u64) -> bool {
        self.devices
            .get_by_interrupt_type(interrupt_type)
            .any(|device| device.get_interrupt(cycles))
    }

    pub fn set_interrupt(&mut self, device_id: DeviceID, iterrupt: bool) {
        self.devices.get_by_id(device_id).set_interrupt(iterrupt);
    }
}

impl AddressableDevice for IOSpace {
    fn read(&mut self, address: Word, cycles: &mut u64) -> u8 {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return 0xff;
        };

        if config.slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let value = device.read(address, *cycles);

        if config.slow {
            *cycles += 1;
        }

        value
    }

    fn write(&mut self, address: Word, value: u8, cycles: &mut u64) -> bool {
        let Some((device, config)) = self.devices.get_with_config_by_address(address) else {
            return false;
        };

        if config.slow && *cycles & 1 != 0 {
            *cycles += 1;
        }

        let needs_phase_2 = device.write(address, value, *cycles);

        if config.slow {
            *cycles += 1;
        }

        needs_phase_2
    }

    fn phase_2(&mut self, address: Word, cycles: u64) {
        if let Some(device) = self.devices.get_by_address(address) {
            device.phase_2(cycles);
        }
    }
}

#[derive(Default)]
struct DeviceList {
    device_list: Vec<Box<dyn IODevice>>,
    address_to_device_id: HashMap<Word, DeviceID>,
    config_list: Vec<Config>,
}

impl DeviceList {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn IODevice>,
        interrupt_type: Option<InterruptType>,
        slow: bool,
    ) -> DeviceID {
        self.device_list.push(device);

        self.config_list.push(Config {
            interrupt_type,
            slow,
        });

        // assumes devices will not be removed
        let device_id = self.device_list.len() - 1;

        for address in addresses {
            self.address_to_device_id
                .insert((*address).into(), device_id);
        }

        device_id
    }

    fn get_by_id(&mut self, device_id: DeviceID) -> &mut dyn IODevice {
        self.device_list[device_id].as_mut()
    }

    fn get_with_config_by_id(&mut self, device_id: DeviceID) -> (&mut dyn IODevice, &Config) {
        let device = self.device_list[device_id].as_mut();
        let config = &self.config_list[device_id];

        (device, config)
    }

    fn get_by_address(&mut self, address: Word) -> Option<&mut dyn IODevice> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_by_id(*device_id))
    }

    fn get_with_config_by_address(
        &mut self,
        address: Word,
    ) -> Option<(&mut dyn IODevice, &Config)> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_with_config_by_id(*device_id))
    }

    fn get_by_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> impl Iterator<Item = &mut Box<dyn IODevice>> {
        let config_list = &self.config_list;

        self.device_list
            .iter_mut()
            .enumerate()
            .filter(move |(device_id, _)| {
                config_list[*device_id].interrupt_type == Some(interrupt_type)
            })
            .map(|(_, device)| device)
    }
}

struct Config {
    interrupt_type: Option<InterruptType>,
    slow: bool,
}
