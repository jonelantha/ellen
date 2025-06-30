use std::collections::HashMap;

use crate::interrupt_type::InterruptType;
use crate::word::Word;

use super::device::Ch22Device;
use super::io_device::Ch22IODevice;

pub struct Ch22IOSpace {
    devices: DeviceList,
}

impl Ch22IOSpace {
    pub fn new() -> Self {
        Ch22IOSpace {
            devices: DeviceList::default(),
        }
    }

    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn Ch22IODevice>,
        interrupt_type: Option<InterruptType>,
        slow: bool,
    ) -> usize {
        self.devices
            .add_device(addresses, device, interrupt_type, slow)
    }

    pub fn get_interrupt(&mut self, interrupt_type: InterruptType, cycles: u32) -> bool {
        self.devices
            .get_by_interrupt_type(interrupt_type)
            .any(|device| device.get_interrupt(cycles))
    }

    pub fn sync(&mut self, cycles: u32) {
        for device in self.devices.get_all() {
            device.sync(cycles);
        }
    }

    pub fn set_interrupt(&mut self, device_id: usize, iterrupt: bool) {
        self.devices.get_by_id(device_id).set_interrupt(iterrupt);
    }

    pub fn set_device_trigger(&mut self, device_id: usize, trigger: Option<u32>) {
        self.devices.get_by_id(device_id).set_trigger(trigger);
    }

    pub fn wrap_triggers(&mut self, wrap: u32) {
        for device in self.devices.get_all() {
            device.wrap_trigger(wrap);
        }
    }
}

impl Ch22Device for Ch22IOSpace {
    fn read(&mut self, address: Word, cycles: &mut u32) -> u8 {
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

    fn write(&mut self, address: Word, value: u8, cycles: &mut u32) -> bool {
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

    fn phase_2(&mut self, address: Word, cycles: u32) {
        if let Some(device) = self.devices.get_by_address(address) {
            device.phase_2(cycles);
        }
    }
}

#[derive(Default)]
struct DeviceList {
    device_list: Vec<Box<dyn Ch22IODevice>>,
    address_to_device_id: HashMap<Word, usize>,
    config_list: Vec<Config>,
}

impl DeviceList {
    pub fn add_device(
        &mut self,
        addresses: &[u16],
        device: Box<dyn Ch22IODevice>,
        interrupt_type: Option<InterruptType>,
        slow: bool,
    ) -> usize {
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

    fn get_by_id(&mut self, device_id: usize) -> &mut dyn Ch22IODevice {
        self.device_list[device_id as usize].as_mut()
    }

    fn get_with_config_by_id(&mut self, device_id: usize) -> (&mut dyn Ch22IODevice, &Config) {
        let device = self.device_list[device_id as usize].as_mut();
        let config = &self.config_list[device_id as usize];

        (device, config)
    }

    fn get_by_address(&mut self, address: Word) -> Option<&mut dyn Ch22IODevice> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_by_id(*device_id))
    }

    fn get_with_config_by_address(
        &mut self,
        address: Word,
    ) -> Option<(&mut dyn Ch22IODevice, &Config)> {
        let device_id = self.address_to_device_id.get(&address)?;

        Some(self.get_with_config_by_id(*device_id))
    }

    fn get_all(&mut self) -> impl Iterator<Item = &mut Box<dyn Ch22IODevice>> {
        self.device_list.iter_mut()
    }

    fn get_by_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> impl Iterator<Item = &mut Box<dyn Ch22IODevice>> {
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
